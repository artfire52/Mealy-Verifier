use std::collections::HashSet;

use crate::{
    utils,
    verifier::event::{Event, Events},
};

use super::{Graph, MultiEdge, NodeId, Rule};
#[cfg(test)]
use std::any::Any;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum State {
    UnInit,               //before seing init
    LookingStartSequence, // we are looking for the first element of the sequence
    LookingForSequence,   //init has been seen. we are looking for the sequence
    End,                  //released has been reached.
}
#[derive(Debug)]
pub(crate) struct ExpectedTransitionSequence {
    name: String,
    events_sequence: Events,
    ignore_events:Option<Events>,
    init_events: Option<Events>,
    end_events: Option<Events>,
    init_state: State,
}

impl ExpectedTransitionSequence {
    fn sequence_start(&self, event_str: &str) -> (bool, bool) {
        (
            self.events_sequence.check_input(0, event_str),
            self.events_sequence.check(0, event_str),
        )
    }

    fn updating_index(&self, index: usize, event_str: &str, state: &State) -> (usize, bool, State) {
        if let Some(ignore)=&self.ignore_events{
            if ignore.check_all(event_str){
               return  (index, false, *state);
            }
        }
        match state {
            State::UnInit => {
                if let Some(init) = &self.init_events {
                    if init.check_all(event_str) {
                        (index, false, State::LookingStartSequence)
                    } else {
                        (index, false, State::UnInit)
                    }
                } else {
                    unreachable!()
                }
            }
            State::LookingStartSequence => {
                let (sequence_start, output_is_good) = self.sequence_start(event_str);
                if sequence_start {
                    if output_is_good {
                        (1, !output_is_good, State::LookingForSequence)
                    } else {
                        (0, !output_is_good, State::LookingStartSequence)
                    }
                } else {
                    (0, false, State::LookingStartSequence)
                }
            }
            State::LookingForSequence => {
                if let Some(end) = &self.end_events {
                    if end.check_all(event_str) {
                        return (index, false, State::End);
                    }
                }

                if self.events_sequence.check(index, event_str) {
                    (index + 1, false, State::LookingStartSequence)
                } else {
                    (index, true, State::LookingForSequence)
                }
            }
            State::End => (index, false, State::End),
        }
    }

    fn inner_apply<'a>(&'a self, graph: &'a Graph) -> (HashSet<NodeId>, HashSet<(&'a MultiEdge, Vec<usize>)>) {
        let sink_node: HashSet<NodeId> = graph.get_sink_state().iter().map(|n| n.clone()).collect();
        let mut nodes: HashSet<NodeId> = HashSet::new();
        let mut seen: HashSet<(NodeId, usize, State)> = HashSet::new();
        let mut edges = HashSet::new();
        let root_node = graph.get_root().expect("could not be reached on root graph");
        let mut execution_stack = Vec::new();
        let neighbors = graph.neighbors_edges_iterator(root_node);
        execution_stack.push((root_node.clone(), neighbors, 0, self.init_state.clone()));
        while let Some((node_id, neighbors, index_on_sequence, state)) = execution_stack.last_mut() {
            seen.insert((node_id.clone(), index_on_sequence.clone(), state.clone()));
            if let Some((neighbor_id, edge)) = neighbors.pop() {
                if sink_node.contains(&neighbor_id) {
                    continue;
                }
                let mut indexes = Vec::new();
                let mut sequence_indexes = Vec::new();
                let mut sequences_states = Vec::new();
                for (index, label) in edge.get_labels().iter().enumerate() {
                    let (new_sequence_index, error, new_state) = self.updating_index(*index_on_sequence, label, state);
                    if error {
                        indexes.push(index)
                    } else if !seen.contains(&(neighbor_id.clone(), new_sequence_index, new_state)) {
                        sequence_indexes.push(new_sequence_index);
                        sequences_states.push(new_state)
                    }
                }
                if !indexes.is_empty() {
                    edges.insert((edge, indexes));
                    nodes.insert(node_id.clone());
                    nodes.insert(neighbor_id.clone());
                }
                let neighbors_next = graph.neighbors_edges_iterator(&neighbor_id);
                for (i, s) in sequence_indexes.iter().zip(sequences_states.iter()) {
                    execution_stack.push((neighbor_id.clone(), neighbors_next.clone(), *i, *s));
                }
            } else {
                execution_stack.pop();
            }
        }
        (nodes, edges)
    }
}

impl Rule for ExpectedTransitionSequence {
    fn from_reader(reader: &mut dyn crate::utils::reader::Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut events_sequence = Events::empty();
        let mut init_events = None;
        let mut end_events = None;
        let mut ignore_events=None;
        while let Some(line) = reader.read_line() {
            let line = line.trim();
            if line.starts_with("I:") {
                let split_line: Vec<_> = line.split(":").collect();
                if split_line.len() != 2 {
                    panic!("no authorized event for RestrictedEvents rule");
                }
                init_events = Some(Events::from_str(split_line[1].trim()));
            } else if line.starts_with("E:") {
                let split_line: Vec<_> = line.split(":").collect();
                if split_line.len() != 2 {
                    panic!("no authorized event for RestrictedEvents rule");
                }
                end_events = Some(Events::from_str(split_line[1].trim()));
            } else if line.starts_with("Ig:") {
                let split_line: Vec<_> = line.split(":").collect();
                if split_line.len() != 2 {
                    panic!("no authorized event for RestrictedEvents rule");
                }
                ignore_events = Some(Events::from_str(split_line[1].trim()));
            }else if line.contains("/") {
                events_sequence.push(Event::new(line));
            } else if line.starts_with(":ETS") {
                break;
            } else {
                panic!("failed to parse ExpectedTransitionSequence: {}", line);
            }
        }
        let init_state = match init_events {
            Some(_) => State::UnInit,
            None => State::LookingStartSequence,
        };
        Ok(ExpectedTransitionSequence {
            name,
            events_sequence,
            ignore_events,
            init_events,
            end_events,
            init_state,
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
        verifier::rules::parse_rule_from_str,
    };

    use super::ExpectedTransitionSequence;

    #[test]
    fn test_expected_event_sequence() {
        let input = r#"
        digraph "graph"{
            "0" [shape=doubleoctagon, style=filled, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "2" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "3" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "4" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "sink" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "0" -> "1" [fontsize=5, label="0/1", URL="t0"];
            "1" -> "2" [fontsize=5, label="i1/no1", URL="t0"];
            "2" -> "sink" [fontsize=5, label="i2/no2", URL="t0"];
            "1" -> "3" [fontsize=5, label="i1/o1", URL="t0"];
            "3" -> "sink" [fontsize=5, label="i2/o2", URL="t0"];
            "3" -> "sink" [fontsize=5, label="i2/no2", URL="t0"];
            "3" -> "4" [fontsize=5, label="i2/no2", URL="t0"];
            "4" -> "sink" [fontsize=5, label="i3/o3", URL="t0"];
        }"#;

        //                 ╔═════════╗
        //                 ║    0    ║
        //                 ╚═════════╝
        //                   │
        //                   │ 0/1
        //                   ▼
        // ┌───┐  i1/no1   ┌─────────┐
        // │ 2 │ ◀──────── │    1    │
        // └───┘           └─────────┘
        //   │               │
        //   │               │ i1/o1
        //   │               ▼
        //   │             ┌─────────┐
        //   │    ┌─────── │    3    │ ─┐
        //   │    │        └─────────┘  │
        //   │    │          │          │
        //   │    │          │ i2/no2   │
        //   │    │          ▼          │
        //   │    │        ┌─────────┐  │
        //   │    │ i2/no2 │    4    │  │
        //   │    │        └─────────┘  │
        //   │    │          │          │
        //   │    │          │ i3/o3    │ i2/o2
        //   │    │          ▼          ▼
        //   │    │        ┌───────────────────┐
        //   │    └──────▶ │       sink        │
        //   │             └───────────────────┘
        //   │   i2/no2      ▲
        //   └───────────────┘

        let graph: Graph = Graph::new(input, true);
        let rule_text = r#"ETS:test
        i1/o1
        i2/o2
        i3/o3
        :ETS"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<ExpectedTransitionSequence>()
            .expect("expect ExpectedTransitionIndex rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        // println!("nodes")
        assert_eq!(nodes.len(), 4);
        assert!(nodes.contains(&NodeId::new("1")));
        assert!(nodes.contains(&NodeId::new("2")));
        assert!(nodes.contains(&NodeId::new("3")));
        assert!(nodes.contains(&NodeId::new("4")));
        let edges_expected = r#"
        "1"->"2" [fontsize=5,label="i1/no1"];
        "3"->"4" [fontsize=5, label="i2/no2"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert_eq!(expected_edge_map, edges);
    }

    #[test]
    fn test_expected_event_sequence_with_start_and_end() {
        let input = r#"
        digraph "graph"{
            "0" [shape=doubleoctagon, style=filled, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "2" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "3" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "4" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "5" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "6" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "sink" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "0" -> "1" [fontsize=5, label="0/1", URL="t0"];
            "1" -> "2" [fontsize=5, label="init/init", URL="t0"];
            "2" -> "4" [fontsize=5, label="i1/o1", URL="t0"];
            "4" -> "6" [fontsize=5, label="end/end", URL="t0"];
            "6" -> "sink" [fontsize=5, label="i2/no2", URL="t0"];
            "1" -> "3" [fontsize=5, label="Noinit/Noinit", URL="t0"];
            "3" -> "5" [fontsize=5, label="i1/o1", URL="t0"];
            "5" -> "sink" [fontsize=5, label="i2/no2", URL="t0"];
        }"#;

        //                             ╔════════════╗
        //                             ║     0      ║
        //                             ╚════════════╝
        //                               │
        //                               │ 0/1
        //                               ▼
        // ┌────────┐  Noinit/Noinit   ┌────────────┐
        // │   3    │ ◀─────────────── │     1      │
        // └────────┘                  └────────────┘
        //   │                           │
        //   │ i1/o1                     │ init/init
        //   ▼                           ▼
        // ┌────────┐                  ┌────────────┐
        // │   5    │                  │     2      │
        // └────────┘                  └────────────┘
        //   │                           │
        //   │                           │ i1/o1
        //   │                           ▼
        //   │                         ┌────────────┐
        //   │                         │     4      │
        //   │                         └────────────┘
        //   │                           │
        //   │                           │ end/end
        //   │                           ▼
        //   │                         ┌────────────┐
        //   │                         │     6      │
        //   │                         └────────────┘
        //   │                           │
        //   │                           │ i2/no2
        //   │                           ▼
        //   │        i2/no2           ┌────────────┐
        //   └───────────────────────▶ │    sink    │
        //                             └────────────┘

        let graph: Graph = Graph::new(input, true);
        let rule_text = r#"ETS:test
        I:init/init
        E:end/end
        i1/o1
        i2/o2
        i3/o3
        :ETS"#;
        let rule = parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<ExpectedTransitionSequence>()
            .expect("expect ExpectedTransitionIndex rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert!(nodes.is_empty());
        assert!(edges.is_empty());
    }
}
