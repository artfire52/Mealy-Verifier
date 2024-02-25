use std::collections::HashSet;

use super::{MultiEdge, NodeId, Rule};
use crate::{graph::Graph, utils, verifier::event::Events};
#[cfg(test)]
use std::any::Any;
#[derive(Debug)]
pub(crate) struct SinkTarget {
    triggers: Events,
    sink_description: Events,
    name: String,
}
impl SinkTarget {
    //Test if a sink state match one or several description made and add the corresponding description to a table.
    fn match_description(&self, graph: &Graph, sink_id: &NodeId) -> bool {
        let mut match_one_description;
        let outgoing_labels = graph.get_outgoing_labels(sink_id);
        for label in outgoing_labels.iter() {
            match_one_description = self.sink_description.check_all(&label);
            if !match_one_description {
                return false;
            }
        }
        true
    }

    fn matching_sink_state(&self, graph: &Graph, sink_ids: &HashSet<&NodeId>) -> HashSet<NodeId> {
        let mut result = HashSet::new();
        for sink_id in sink_ids {
            if self.match_description(graph, sink_id) {
                result.insert((*sink_id).clone());
            }
        }
        result
    }

    fn inner_apply<'a>(&'a self, graph: &'a Graph) -> (HashSet<NodeId>, HashSet<(&'a MultiEdge, Vec<usize>)>) {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();
        let sinks = graph.get_sink_state_set();
        let matching_sink_ids = self.matching_sink_state(graph, &sinks);
        for edge in graph.iter_edges() {
            if matching_sink_ids.contains(edge.get_source()) {
                continue;
            }

            let mut indexes = Vec::new();
            let dest = edge.get_dest();
            for (index, label) in edge.get_labels().iter().enumerate() {
                if self.triggers.check_all(&label) && !matching_sink_ids.contains(dest) {
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

impl Rule for SinkTarget {
    fn from_reader(reader: &mut dyn crate::utils::reader::Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut sink_description: Option<Events> = None;
        let mut triggers: Option<Events> = None;
        while let Some(line) = reader.read_line() {
            if line.starts_with(":ST") {
                break;
            } else if line.contains('/') {
                let line_split: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if line_split.len() != 2 {
                    panic!("Failed to parse {line}. You must write sink's rule line as trigger|sink edge");
                }
                triggers = Some(Events::from_str(line_split[0]));
                sink_description = Some(Events::from_str(line_split[1]));
            } else {
                panic!("failed to parse expected loop rule");
            }
        }
        let triggers = triggers.expect("you must indicate the event that lead to the sink state for Sink target rule");

        let sink_description =
            sink_description.expect("you must indicate the event that lead to the sink state for Sink target rule");
        Ok(SinkTarget {
            triggers: triggers,
            sink_description: sink_description,
            name,
        })
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn apply(
        &mut self,
        graph: &Graph,
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

#[derive(Debug)]
pub(crate) struct SinkDescription {
    sink_description: Vec<Events>,
    name: String,
}

impl SinkDescription {
    ///Test if a sink state match one or several description made and add the corresponding description to a table.
    fn match_description(&self, graph: &Graph, sink_id: &NodeId) -> bool {
        let mut match_one_description;
        let outgoing_labels = graph.get_outgoing_labels(sink_id);
        for label in outgoing_labels.iter() {
            match_one_description = false;
            for description in self.sink_description.iter() {
                match_one_description = match_one_description || description.check_all(&label);
            }
            if !match_one_description {
                return false;
            }
        }
        true
    }

    fn inner_apply<'a>(&'a self, graph: &'a Graph) -> (HashSet<NodeId>, HashSet<(&'a MultiEdge, Vec<usize>)>) {
        let sinks = graph.get_sink_state();
        let mut nodes: HashSet<NodeId> = HashSet::new();

        for sink_id in sinks {
            if !self.match_description(graph, &sink_id) {
                nodes.insert(sink_id.clone());
            }
        }
        let mut edges: HashSet<(&MultiEdge, Vec<usize>)> = HashSet::new();
        for node in nodes.iter() {
            if let Some(edges_from_node) = graph.get_outgoing_edges(node) {
                for edge in edges_from_node.values() {
                    let label_len = edge.get_nb_label();
                    edges.insert((edge, (0..label_len).collect()));
                }
            }
        }
        (nodes, edges)
    }
}

impl Rule for SinkDescription {
    fn from_reader(reader: &mut dyn crate::utils::reader::Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut sinks_labels: Vec<Events> = Vec::new();
        while let Some(line) = reader.read_line() {
            if line.starts_with(":SD") {
                break;
            } else if line.contains('/') {
                sinks_labels.push(Events::from_str(line));
            } else {
                panic!("failed to parse expected loop rule");
            }
        }
        Ok(SinkDescription {
            sink_description: sinks_labels,
            name,
        })
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn apply(
        &mut self,
        graph: &Graph,
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

    use super::{SinkDescription, SinkTarget};

    #[test]
    fn test_sink_target() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "sink1" [shape=ellipse, style=filed, fillcolor=white, URL="2"];
            "sink2" [shape=ellipse, style=filed, fillcolor=white, URL="45"];
            "sink3" [shape=ellipse, style=filed, fillcolor=white, URL="82"];
            "0" -> "1" [fontsize=5, label="a/b", color=black];
            "0" -> "2" [fontsize=5, label="c/b", color=black];
            "0" -> "sink2" [fontsize=5, label="e/nok", color=black];
            "1" -> "2" [fontsize=5, label="c/d", color=black];
            "1" -> "sink2" [fontsize=5, label="e/ok2", color=black];
            "1" -> "sink1" [fontsize=5, label="a/b", color=black];
            "2" -> "1" [fontsize=5, label="c/d", color=black];
            "2" -> "sink3" [fontsize=5, label="a/b", color=black];
            "2" -> "sink3" [fontsize=5, label="e/ok", color=black];
            "sink1" -> "sink1" [fontsize=5, label="a/no_resp", color=black];
            "sink1" -> "sink1" [fontsize=5, label="c/no_resp", color=black];
            "sink1" -> "sink1" [fontsize=5, label="e/no_resp", color=black];
            "sink2" -> "sink2" [fontsize=5, label="a/nok", color=black];
            "sink2" -> "sink2" [fontsize=5, label="c/nok", color=black];
            "sink2" -> "sink2" [fontsize=5, label="e/no_resp", color=black];
            "sink3" -> "sink3" [fontsize=5, label="a/no_resp", color=black];
            "sink3" -> "sink3" [fontsize=5, label="c/no_resp", color=black];
            "sink3" -> "sink3" [fontsize=5, label="e/nok", color=black];
            
        }"#;
        //                   ┌────────────────────────────────────┐
        //                   │                                    │
        //                   │                     e/no_resp      │
        //                   │                   ┌───────────┐    │ e/nok
        //                   │                   ▼           │    ▼
        //                 ┌──────┐    c/nok   ┌────────────────────────────────────────────────┐   a/nok
        //                 │      │  ┌──────── │                                                │ ────────────┐
        //                 │  0   │  │         │                     sink2                      │             │
        //   ┌──────────── │      │  └───────▶ │                                                │ ◀───────────┘
        //   │             └──────┘            └────────────────────────────────────────────────┘
        //   │               │                   ▲                                  e/no_resp
        //   │               │ a/b               │ e/ok2                          ┌───────────┐
        //   │               ▼                   │                                ▼           │
        //   │             ┌───────────────────────────────────┐    c/no_resp   ┌───────────────┐   a/no_resp
        //   │             │                                   │  ┌──────────── │               │ ────────────┐
        //   │ c/b         │                 1                 │  │             │     sink1     │             │
        //   │             │                                   │  └───────────▶ │               │ ◀───────────┘
        //   │             └───────────────────────────────────┘                └───────────────┘
        //   │               │       ▲           │               a/b              ▲
        //   │               │ c/d   │ c/d       └────────────────────────────────┘
        //   │               ▼       │
        //   │             ┌───────────────────────────────────┐
        //   └───────────▶ │                 2                 │
        //                 └───────────────────────────────────┘
        //                   │       │             e/nok
        //                   │ a/b   │ e/ok      ┌───────────┐
        //                   ▼       ▼           ▼           │
        //     c/no_resp   ┌───────────────────────────────────┐   a/no_resp
        //   ┌──────────── │                                   │ ────────────┐
        //   │             │               sink3               │             │
        //   └───────────▶ │                                   │ ◀───────────┘
        //                 └───────────────────────────────────┘
        let graph: Graph = Graph::new(input, true);
        // let rule_text = r#"ST:test
        // a/b;e/ok|c/no_resp;a/no_resp;e/nok
        // :ST"#;
        let rule_text = r#"ST:test
        e/ok|a/no_resp;c/no_resp;e/nok
        :ST"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<SinkTarget>()
            .expect("expect Sink target rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert!(nodes.is_empty());
        assert!(edges.is_empty());

        let rule_text = r#"ST:test
        a/b|c/no_resp;a/no_resp;e/no_resp
        :ST"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<SinkTarget>()
            .expect("expect Sink target rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert!(nodes.contains(&NodeId::new("sink3")));
        assert!(nodes.contains(&NodeId::new("2")));
        assert!(nodes.contains(&NodeId::new("0")));
        assert!(nodes.contains(&NodeId::new("1")));
        assert_eq!(nodes.len(), 4);
        let edges_expected = r#"
        "0" -> "1" [fontsize=5, label="a/b", color=black];
        "2" -> "sink3" [fontsize=5, label="a/b", color=black];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert!(expected_edge_map.eq(&edges));
    }

    #[test]
    fn test_sink_description() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "sink1" [shape=ellipse, style=filed, fillcolor=white, URL="2"];
            "sink2" [shape=ellipse, style=filed, fillcolor=white, URL="45"];
            "sink3" [shape=ellipse, style=filed, fillcolor=white, URL="82"];
            "0" -> "1" [fontsize=5, label="a/b", color=black];
            "0" -> "2" [fontsize=5, label="c/b", color=black];
            "0" -> "sink2" [fontsize=5, label="e/nok", color=black];
            "1" -> "2" [fontsize=5, label="c/d", color=black];
            "1" -> "sink2" [fontsize=5, label="e/ok", color=black];
            "1" -> "sink1" [fontsize=5, label="a/b", color=black];
            "2" -> "1" [fontsize=5, label="c/d", color=black];
            "2" -> "sink3" [fontsize=5, label="a/b", color=black];
            "2" -> "sink3" [fontsize=5, label="e/ok", color=black];
            "sink1" -> "sink1" [fontsize=5, label="a/no_resp", color=black];
            "sink1" -> "sink1" [fontsize=5, label="c/no_resp", color=black];
            "sink1" -> "sink1" [fontsize=5, label="e/no_resp", color=black];
            "sink2" -> "sink2" [fontsize=5, label="a/nok", color=black];
            "sink2" -> "sink2" [fontsize=5, label="c/nok", color=black];
            "sink2" -> "sink2" [fontsize=5, label="e/no_resp", color=black];
            "sink3" -> "sink3" [fontsize=5, label="a/no_resp", color=black];
            "sink3" -> "sink3" [fontsize=5, label="c/no_resp", color=black];
            "sink3" -> "sink3" [fontsize=5, label="e/nok", color=black];
            
        }"#;
        //                   ┌────────────────────────────────────┐
        //                   │                                    │
        //                   │                     e/no_resp      │
        //                   │                   ┌───────────┐    │ e/nok
        //                   │                   ▼           │    ▼
        //                 ┌──────┐    c/nok   ┌────────────────────────────────────────────────┐   a/nok
        //                 │      │  ┌──────── │                                                │ ────────────┐
        //                 │  0   │  │         │                     sink2                      │             │
        //   ┌──────────── │      │  └───────▶ │                                                │ ◀───────────┘
        //   │             └──────┘            └────────────────────────────────────────────────┘
        //   │               │                   ▲                                  e/no_resp
        //   │               │ a/b               │ e/ok                           ┌───────────┐
        //   │               ▼                   │                                ▼           │
        //   │             ┌───────────────────────────────────┐    c/no_resp   ┌───────────────┐   a/no_resp
        //   │             │                                   │  ┌──────────── │               │ ────────────┐
        //   │ c/b         │                 1                 │  │             │     sink1     │             │
        //   │             │                                   │  └───────────▶ │               │ ◀───────────┘
        //   │             └───────────────────────────────────┘                └───────────────┘
        //   │               │       ▲           │               a/b              ▲
        //   │               │ c/d   │ c/d       └────────────────────────────────┘
        //   │               ▼       │
        //   │             ┌───────────────────────────────────┐
        //   └───────────▶ │                 2                 │
        //                 └───────────────────────────────────┘
        //                   │       │             e/nok
        //                   │ a/b   │ e/ok      ┌───────────┐
        //                   ▼       ▼           ▼           │
        //     c/no_resp   ┌───────────────────────────────────┐   a/no_resp
        //   ┌──────────── │                                   │ ────────────┐
        //   │             │               sink3               │             │
        //   └───────────▶ │                                   │ ◀───────────┘
        //                 └───────────────────────────────────┘
        let graph: Graph = Graph::new(input, true);
        // let rule_text = r#"ST:test
        // a/b;e/ok|c/no_resp;a/no_resp;e/nok
        // :ST"#;
        let rule_text = r#"SD:test
        c/no_resp;a/no_resp;e/no_resp
        :SD"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<SinkDescription>()
            .expect("expect Sink target rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&NodeId::new("sink2")));
        assert!(nodes.contains(&NodeId::new("sink3")));
        let edges_expected = r#"
        "sink2"->"sink2" [fontsize=5,label="a/nok"];
        "sink2"->"sink2" [fontsize=5,label="c/nok"];
        "sink2"->"sink2" [fontsize=5,label="e/no_resp"];
        "sink3"->"sink3" [fontsize=5,label="e/nok"];
        "sink3"->"sink3" [fontsize=5,label="a/no_resp"];
        "sink3"->"sink3" [fontsize=5,label="c/no_resp"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert!(expected_edge_map.eq(&edges));

        let rule_text = r#"SD:test
        c/no_resp;a/no_resp;e/no_resp
        c/nok;a/nok;e/no_resp
        :SD"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<SinkDescription>()
            .expect("expect Sink target rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains(&NodeId::new("sink3")));
        let edges_expected = r#"
        "sink3"->"sink3" [fontsize=5,label="e/nok"];
        "sink3"->"sink3" [fontsize=5,label="a/no_resp"];
        "sink3"->"sink3" [fontsize=5,label="c/no_resp"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert!(expected_edge_map.eq(&edges));
    }
}
