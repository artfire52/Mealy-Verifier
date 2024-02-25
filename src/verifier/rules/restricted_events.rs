use std::collections::HashSet;

use crate::{utils, verifier::event::Events};

use super::{Graph, MultiEdge, NodeId, Rule};
#[cfg(test)]
use std::any::Any;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum State {
    UnInit,     //before seing init
    Restricted, //init has been seen. Restriction apply
    Released,   //released has been reached.
    Cancel,     // rule does not apply
}
#[derive(Debug)]
pub(crate) struct RestrictedEvents {
    name: String,
    init: Option<Events>,
    release: Events,
    authorized: Events,
    cancel_rule: Option<Events>,
}

impl RestrictedEvents {
    fn is_authorized(&self, event_str: &str) -> bool {
        self.authorized.check_all(event_str)
    }

    fn end(&self, event_str: &str) -> bool {
        self.release.check_all(event_str)
    }

    fn start(&self, event_str: &str) -> bool {
        match &self.init {
            Some(init_event) => init_event.check_all(event_str),
            None => true,
        }
    }

    fn cancel(&self, event_str: &str) -> bool {
        match &self.cancel_rule {
            Some(cancel) => cancel.check_all(event_str),
            None => false,
        }
    }

    fn get_init_state(&self) -> State {
        match self.init {
            Some(_) => State::UnInit,
            None => State::Restricted,
        }
    }

    fn update_state(&self, state: &State, event_str: &str) -> (bool, State) {
        match state {
            State::UnInit => {
                if self.start(event_str) {
                    return (false, State::Restricted);
                } else if self.cancel(event_str) {
                    return (false, State::Cancel);
                } else {
                    return (false, State::UnInit);
                }
            }
            State::Restricted => {
                if self.end(event_str) {
                    return (false, State::Released);
                } else if self.is_authorized(event_str) {
                    return (false, State::Restricted);
                } else {
                    return (true, State::Restricted);
                }
            }
            State::Released => {
                return (false, State::Released);
            }
            State::Cancel => {
                return (false, State::Cancel);
            }
        }
    }

    fn inner_apply<'a>(&'a self, graph: &'a Graph) -> (HashSet<NodeId>, HashSet<(&MultiEdge, Vec<usize>)>) {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();
        let mut execution_stack: Vec<(NodeId, Vec<(NodeId, &MultiEdge)>, State)> = Vec::new();
        let mut seen: HashSet<(NodeId, State)> = HashSet::new();
        let sink_nodes = graph.get_sink_state_set();
        let root_id = graph.get_root().clone().unwrap();
        execution_stack.push((
            root_id.clone(),
            graph.neighbors_edges_iterator(&root_id),
            self.get_init_state(),
        ));
        while let Some((node_id, neighbors, current_state)) = execution_stack.last_mut() {
            seen.insert((node_id.clone(), current_state.clone()));
            if let Some((neighbor_id, edge)) = neighbors.pop() {
                if sink_nodes.contains(&neighbor_id) {
                    continue;
                }
                let mut indexes_next = HashSet::new();
                let mut indexes_error = Vec::new();
                for (index, label) in edge.get_labels().iter().enumerate() {
                    let (error, new_state) = self.update_state(current_state, label);
                    if error {
                        indexes_error.push(index)
                    }
                    indexes_next.insert(new_state);
                }
                for new_state in indexes_next {
                    if !seen.contains(&(neighbor_id.clone(), new_state.clone())) {
                        let new_neighbors = graph.neighbors_edges_iterator(&neighbor_id);
                        execution_stack.push((neighbor_id.clone(), new_neighbors, new_state.clone()));
                    }
                }
                if !indexes_error.is_empty() {
                    nodes.insert(edge.get_source().clone());
                    nodes.insert(edge.get_dest().clone());
                    edges.insert((edge, indexes_error));
                }
            } else {
                execution_stack.pop();
            }
            seen.len();
        }

        (nodes, edges)
    }
}

impl Rule for RestrictedEvents {
    fn from_reader(reader: &mut dyn crate::utils::reader::Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut init: Option<Events> = None;
        let mut release: Option<Events> = None;
        let mut authorized: Option<Events> = None;
        let mut cancel: Option<Events> = None;
        while let Some(line) = reader.read_line() {
            let line = line.trim_start();
            if line.starts_with("A:") {
                let split_line: Vec<_> = line.split(":").collect();
                if split_line.len() != 2 {
                    panic!("no authorized event for RestrictedEvents rule");
                }
                authorized = Some(Events::from_str(split_line[1].trim()))
            } else if line.starts_with("R:") {
                let split_line: Vec<_> = line.split(":").collect();
                if split_line.len() != 2 {
                    panic!("no release for RestrictedEvents rule");
                }
                release = Some(Events::from_str(split_line[1].trim()));
            } else if line.starts_with("I:") {
                let split_line: Vec<_> = line.split(":").collect();
                if split_line.len() != 2 {
                    panic!("no init for RestrictedEvents rule");
                }
                init = Some(Events::from_str(split_line[1].trim()));
            } else if line.starts_with("C:") {
                let split_line: Vec<_> = line.split(":").collect();
                if split_line.len() != 2 {
                    panic!("no release for RestrictedEvents rule");
                }
                cancel = Some(Events::from_str(split_line[1].trim()));
            } else if line.starts_with(":RE") {
                break;
            } else {
                panic!("failed to parse until Rule:{}", line);
            }
        }
        Ok(RestrictedEvents {
            name,
            init: init,
            authorized: authorized.expect("you must specify authorized event for restricted event rule"),
            release: release.expect("you must specify end event for restrictued event rule"),
            cancel_rule: cancel,
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

    use super::RestrictedEvents;

    #[test]
    fn test_restricted_event() {
        let input = r#"
        digraph "graph"{
            "0" [shape=doubleoctagon, style=filled, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "2" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "3" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "4" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "5" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "6" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "0" -> "1" [fontsize=5, label="nok/nok", URL="t0"];
            "0" -> "2" [fontsize=5, label="nok/nok", URL="t0"];
            "2" -> "sink" [fontsize=5, label="nok/nok", URL="t0"];
            "1" -> "3" [fontsize=5, label="init/init", URL="t0"];
            "3" -> "4" [fontsize=5, label="ok/ok", URL="t0"];
            "3" -> "5" [fontsize=5, label="release/release", URL="t0"];
            "4" -> "6" [fontsize=5, label="ok/ok", URL="t0"];
            "6" -> "5" [fontsize=5, label="nok/nok", URL="t0"];
            "5" -> "7" [fontsize=5, label="nok/nok", URL="t0"];
            "7" -> "sink" [fontsize=5, label="release/release", URL="t0"];
        }"#;

        // ┌───┐  nok/nok   ╔══════════════════╗
        // │ 2 │ ◀───────── ║        0         ║
        // └───┘            ╚══════════════════╝
        //   │                │
        //   │                │ nok/nok
        //   │                ▼
        //   │              ┌──────────────────┐
        //   │              │        1         │
        //   │              └──────────────────┘
        //   │                │
        //   │                │ init/init
        //   │                ▼
        //   │              ┌──────────────────┐
        //   │              │        3         │ ─┐
        //   │              └──────────────────┘  │
        //   │                │                   │
        //   │                │ ok/ok             │
        //   │                ▼                   │
        //   │              ┌──────────────────┐  │
        //   │              │        4         │  │
        //   │              └──────────────────┘  │
        //   │                │                   │
        //   │                │ ok/ok             │ release/release
        //   │                ▼                   │
        //   │              ┌──────────────────┐  │
        //   │              │        6         │  │
        //   │              └──────────────────┘  │
        //   │                │                   │
        //   │                │ nok/nok           │
        //   │                ▼                   │
        //   │              ┌──────────────────┐  │
        //   │              │        5         │ ◀┘
        //   │              └──────────────────┘
        //   │                │
        //   │                │ nok/nok
        //   │                ▼
        //   │              ┌──────────────────┐
        //   │              │        7         │
        //   │              └──────────────────┘
        //   │                │
        //   │                │ release/release
        //   │                ▼
        //   │   nok/nok    ┌──────────────────┐
        //   └────────────▶ │       sink       │
        //                  └──────────────────┘

        let rule = r#"RE:test42
        I:init/init
        A:ok/ok
        R:release/release
        :RE"#;
        let graph = Graph::new(input, true);
        let rule = parse_rule_from_str(rule);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<RestrictedEvents>()
            .expect("expect conditional rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        let node_ids = vec!["6", "5", "7"];
        let node_ids: Vec<NodeId> = node_ids.iter().map(|e| NodeId::new(e)).collect();
        for node_id in node_ids.iter() {
            assert!(nodes.contains(node_id));
        }
        assert_eq!(nodes.len(), node_ids.len());
        let edges_expected = r#"
        "6" -> "5" [fontsize=5, label="nok/nok"];
        "5" -> "7" [fontsize=5, label="nok/nok"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert_eq!(expected_edge_map, edges);
    }
}
