use std::collections::HashSet;

use super::{Graph, MultiEdge, NodeId, Rule};
use crate::{utils, verifier::event::Pattern};
#[cfg(test)]
use std::any::Any;
#[derive(Debug)]
pub(crate) struct Output {
    name: String,
    input: Pattern,
    allowed_outputs: Vec<Pattern>,
}

impl Output {
    // check if the forbidden pattern is present
    fn allowed_event(&self, event: &str) -> bool {
        let e: Vec<&str> = event.split("/").collect();
        let input = e[0].trim();
        let output = e[1].trim();
        if self.input.check(input) {
            let mut res: bool = false;
            for allowed_outputs in &self.allowed_outputs {
                res = res || allowed_outputs.check(output);
            }
            return res;
        } else {
            return true;
        }
    }

    fn inner_apply<'a>(&'a self, graph: &'a Graph) -> (HashSet<NodeId>, HashSet<(&'a MultiEdge, Vec<usize>)>) {
        let sink_nodes = graph.get_sink_state_set();
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();
        for edge in graph.iter_edges() {
            if sink_nodes.contains(edge.get_source()) || sink_nodes.contains(edge.get_dest()) {
                continue;
            }
            let mut indexes = Vec::new();
            for (index, label) in edge.get_labels().iter().enumerate() {
                if !self.allowed_event(&label) {
                    indexes.push(index);
                }
            }
            if !indexes.is_empty() {
                nodes.insert(edge.get_source().clone());
                nodes.insert(edge.get_dest().clone());
                edges.insert((edge, indexes));
            }
        }
        (nodes, edges)
    }
}

impl Rule for Output {
    fn from_reader(reader: &mut dyn crate::utils::reader::Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut input = None;
        let mut allowed_outputs: Vec<String> = Vec::new();
        while let Some(line) = reader.read_line() {
            let line = line.trim_start();
            if line.starts_with("I:") {
                let e: Vec<_> = line.split(":").collect();
                input = Some(e[1].trim().to_string());
            } else if line.starts_with("O:") {
                let e: Vec<_> = line.split(":").collect();
                allowed_outputs.push(e[1].trim().to_string());
            } else if line.starts_with(":OR") {
                break;
            } else {
                panic!("failed to parse Message Reject Rule:{}", line);
            }
        }
        let allowed_outputs = allowed_outputs.iter().map(|e| Pattern::from_str(e)).collect();
        Ok(Output {
            input: Pattern::from_str(&input.as_ref().unwrap()),
            allowed_outputs,
            name,
        })
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn apply(
        &mut self,
        graph: &super::Graph,
        output_folder: &mut std::path::PathBuf,
    ) {
        let (nodes, edges) = self.inner_apply(graph);

        output_folder.push(self.get_name());
        // println!("graph: {}",graph.get_name());
        if let Err(e) = std::fs::create_dir_all(&output_folder) {
            panic!("failed to create output directory due to :{}", e.to_string());
        }

        let ret = utils::output::write_files(graph, nodes, edges, output_folder);
        if let Err(e) = ret {
            panic!(
                "unable to output the result of rule {} due to {}",
                self.get_name(),
                e.to_string()
            );
        }
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
    use crate::{
        graph::{edgemap::EdgeMap, nodeid::NodeId, Graph},
        verifier::rules::parse_rule_from_str,
    };

    use super::Output;

    #[test]
    fn test_output_rule() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "3" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "0" -> "1" [fontsize=5, label="0/1", color=black];
            "0" -> "1" [fontsize=5, label="0/osef", color=black];
            "1" -> "0" [fontsize=5, label="1/0", color=black];
            "2" -> "0" [fontsize=5, label="2/0", color=black];
            "1" -> "2" [fontsize=5, label="1/2", color=black];
            "1" -> "2" [fontsize=5, label="1/osef", color=black];
            "1" -> "3" [fontsize=5, label="1/3", color=black];
            "3" -> "0" [fontsize=5, label="3/0", color=black];
        }"#;
        //         ┌─────────────────────────┐  3/0
        //   ┌───▶ │            0            │ ◀────────┐
        //   │     └─────────────────────────┘          │
        //   │       │       │          ▲               │
        //   │       │ 0/1   │ 0/osef   │ 1/0           │
        //   │       ▼       ▼          │               │
        //   │     ┌─────────────────────────┐  1/3   ┌───┐
        //   │ 2/0 │            1            │ ─────▶ │ 3 │
        //   │     └─────────────────────────┘        └───┘
        //   │       │       │
        //   │       │ 1/2   │ 1/osef
        //   │       ▼       │
        //   │     ┌──────┐  │
        //   └──── │  2   │ ◀┘
        //         └──────┘
        let graph: Graph = Graph::new(input, false);
        let rule_text = r#"OR:test
        I:0
        O:1
        :OR"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<Output>()
            .expect("expect Sink target rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&NodeId::new("0")));
        assert!(nodes.contains(&NodeId::new("1")));
        let edges_expected = r#"
        "0"->"1" [fontsize=5,label="0/osef"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert_eq!(expected_edge_map, edges);
    }
}
