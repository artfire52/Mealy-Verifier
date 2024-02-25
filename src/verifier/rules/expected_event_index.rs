use std::collections::HashSet;

#[cfg(test)]
use std::any::Any;

use crate::{utils, verifier::event::Events};

use super::{Graph, MultiEdge, NodeId, Rule};
#[derive(Debug)]
pub(crate) struct ExpectedTransitionIndex {
    name: String,
    index: usize,
    event: Events,
}
impl ExpectedTransitionIndex {
    fn match_inner_event(&self, event: &str) -> bool {
        self.event.check_all(event)
    }
    fn inner_apply<'a>(&'a self, graph: &'a Graph) -> (HashSet<NodeId>, HashSet<(&'a MultiEdge, Vec<usize>)>) {
        let sink_nodes: HashSet<&NodeId> = graph.get_sink_state_set();
        let mut seen: HashSet<(NodeId, usize)> = HashSet::new();
        let mut nodes: HashSet<NodeId> = HashSet::new();
        let mut edges = HashSet::new();
        let root_node = graph.get_root().expect("could not be reached on root graph");
        let mut execution_stack = Vec::new();
        let neighbors = graph.neighbors_edges_iterator(root_node);
        execution_stack.push((root_node.clone(), neighbors, 0));
        while let Some((node_id, neighbors, depth)) = execution_stack.last_mut() {
            seen.insert((node_id.clone(), depth.clone()));
            let next_depth = *depth + 1;
            if let Some((neighbor_id, edge)) = neighbors.pop() {
                if *depth == self.index && !sink_nodes.contains(&neighbor_id) {
                    let mut indexes = Vec::new();
                    for (index, label) in edge.get_labels().iter().enumerate() {
                        if !self.match_inner_event(label) {
                            indexes.push(index);
                        }
                    }
                    if !indexes.is_empty() {
                        edges.insert((edge, indexes));
                        nodes.insert(node_id.clone());
                        nodes.insert(neighbor_id.clone());
                    }
                }
                if next_depth <= self.index && !seen.contains(&(neighbor_id.clone(), next_depth)) {
                    let neighbors_next = graph.neighbors_edges_iterator(&neighbor_id);
                    execution_stack.push((neighbor_id.clone(), neighbors_next, next_depth));
                }
            } else {
                execution_stack.pop();
            }
        }
        (nodes, edges)
    }
}

impl Rule for ExpectedTransitionIndex {
    fn from_reader(reader: &mut dyn crate::utils::reader::Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut event = None;
        let mut index = None;
        while let Some(line) = reader.read_line() {
            if line.contains("/") {
                event = Some(Events::from_str(line));
            } else if line.starts_with(":ETI") {
                break;
            } else {
                index = Some(
                    line.trim()
                        .parse::<usize>()
                        .expect("failed to parse index of ExpectedTransitionIndex rule due to :{line}"),
                );
            }
        }
        Ok(ExpectedTransitionIndex {
            event: event.expect("failed to parse ExpectedTransition index"),
            index: index.expect("failed to parse ExpectedTransition index"),
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
mod test {
    //https://dot-to-ascii.ggerganov.com/
    use crate::{
        graph::{edgemap::EdgeMap, nodeid::NodeId, Graph},
        verifier::rules::{expected_event_index::ExpectedTransitionIndex, parse_rule_from_str},
    };

    #[test]
    fn test_expected_event_index() {
        let input = r#"
        digraph "graph"{
            "0" [shape=doubleoctagon, style=filled, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "2" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "sink" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "0" -> "1" [fontsize=5, label="0/1", URL="t0"];
            "1" -> "2" [fontsize=5, label="1/2", URL="t0"];
            "1" -> "2" [fontsize=5, label="1/nok", URL="t0"];
            "2" -> "sink" [fontsize=5, label="2/s", URL="t0"];
            "1" -> "sink" [fontsize=5, label="1/nok", URL="t0"];
        }"#;
        //         ╔══════╗
        //         ║  0   ║
        //         ╚══════╝
        //           │
        //           │ 0/1
        //           ▼
        //         ┌──────┐
        // ┌────── │  1   │ ─┐
        // │       └──────┘  │
        // │         │       │
        // │         │ 1/2   │ 1/nok
        // │         ▼       │
        // │       ┌──────┐  │
        // │ 1/nok │  2   │ ◀┘
        // │       └──────┘
        // │         │
        // │         │ 2/s
        // │         ▼
        // │       ┌──────┐
        // └─────▶ │ sink │
        //         └──────┘
        let graph: Graph = Graph::new(input, true);
        let rule_text = r#"ETI:test
        1
        1/2
        :ETI"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<ExpectedTransitionIndex>()
            .expect("expect ExpectedTransitionIndex rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&NodeId::new("1")));
        assert!(nodes.contains(&NodeId::new("2")));
        let edges_expected = r#"
        "1"->"2" [fontsize=5,label="1/nok"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert_eq!(expected_edge_map, edges);
    }
}
