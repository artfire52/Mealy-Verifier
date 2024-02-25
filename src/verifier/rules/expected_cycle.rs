use std::{
    collections::HashMap,
    fs::{self},
};

use super::{Cycle, Rule};
use crate::{graph::Graph, utils::reader::Reader, verifier::event::Events};
#[cfg(test)]
use std::any::Any;

#[derive(Debug)]
pub(crate) struct ExpectedCycle {
    pub(crate) cycle: Events,
    pub(crate) name: String,
}

impl ExpectedCycle {
    fn check(&self, cycle: &Cycle) -> bool {
        let mut ret;
        let labels = cycle.get_labels();
        if labels.len() != self.cycle.len() {
            return false;
        }

        for (index, edge_labels) in labels.into_iter().enumerate() {
            ret = false;
            for label in edge_labels {
                if self.cycle.check(index, &label) {
                    ret = true;
                }
            }
            if !ret {
                return false;
            }
        }
        true
    }

    fn inner_apply(&self, cycles: &HashMap<super::NodeId, Vec<super::Cycle>>) -> bool {
        //check cycle
        for cycle_ in cycles.values() {
            for cycle in cycle_ {
                if self.check(cycle) {
                    return true;
                }
            }
        }
        return false;
    }
}

impl Rule for ExpectedCycle {
    fn from_reader(reader: &mut dyn Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut cycle: Option<Events> = None;
        while let Some(line) = reader.read_line() {
            if line.starts_with(":EC") {
                break;
            } else if line.contains('/') {
                cycle = Some(Events::from_str(line));
            } else {
                panic!("failed to parse expected cycle rule");
            }
        }
        let cycle = match cycle {
            Some(c) => c,
            None => panic!("failed to parse expected cycle rule"),
        };
        Ok(ExpectedCycle { cycle, name })
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn apply(
        &mut self,
        _graph: &Graph,
        cycles: &HashMap<super::NodeId, Vec<super::Cycle>>,
        output_folder: &mut std::path::PathBuf,
    ) {
        output_folder.push(self.get_name());
        if let Err(e) = fs::create_dir_all(&output_folder) {
            panic!("failed to create output directory due to :{}", e.to_string());
        }
        if self.inner_apply(cycles) {
            output_folder.push("true".to_string());
        } else {
            output_folder.push("false".to_string());
        }

        let _file = match fs::OpenOptions::new().write(true).truncate(true).open(&output_folder) {
            Ok(f) => f,
            Err(_) => fs::File::create(&output_folder)
                .expect(&format!("failse to create output of rule :{}", self.get_name())),
        };
        output_folder.pop();
        output_folder.pop();
    }
    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    //https://dot-to-ascii.ggerganov.com/
    use crate::algorithm::cycle::Cycle;
    use crate::graph::Graph;

    use super::ExpectedCycle;

    #[test]
    fn test_expected_cycle() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "3" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "0" -> "1" [fontsize=5, label="0/1", color=black];
            "1" -> "0" [fontsize=5, label="1/0", color=black];
            "2" -> "0" [fontsize=5, label="2/0", color=black];
            "1" -> "2" [fontsize=5, label="1/2", color=black];
            "1" -> "3" [fontsize=5, label="1/3", color=black];
            "3" -> "0" [fontsize=5, label="3/0", color=black];
            
        }"#;
        //                       3/0
        //                ┌───────────────┐
        //                ▼               │
        //              ┌──────┐          │
        //        ┌───▶ │  0   │ ◀┐       │
        //        │     └──────┘  │       │
        //        │       │       │       │
        //        │ 2/0   │ 0/1   │ 1/0   │
        //        │       ▼       │       │
        //        │     ┌──────┐  │       │
        //   ┌────┼──── │  1   │ ─┘       │
        //   │    │     └──────┘          │
        //   │    │       │               │
        //   │    │       │ 1/2           │
        //   │    │       ▼               │
        //   │    │     ┌──────┐          │
        //   │    └──── │  2   │          │
        //   │          └──────┘          │
        //   │   1/3    ┌──────┐          │
        //   └────────▶ │  3   │ ─────────┘
        //              └──────┘
        let graph: Graph = Graph::new(input, false);

        let cycles = Cycle::all_simple_cycles_with_edges(&graph);
        let rule_text = r#"EC:test
        0/1;1/2;2/0
        :EC"#;
        let rule = crate::verifier::rules::parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<ExpectedCycle>()
            .expect("expect ExpectedCycle rule");

        assert!(rule.inner_apply(&cycles));

        let rule_text = r#"EC:test
        0/1;1/2;2/d
        :EC"#;
        let rule = crate::verifier::rules::parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<ExpectedCycle>()
            .expect("expect ExpectedCycle rule");

        assert!(!rule.inner_apply(&cycles));
    }
}
