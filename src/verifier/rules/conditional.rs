use super::super::event::Event;
use super::super::premise::Premise;
use super::Rule;
use crate::graph::edgemap::EdgeMap;
use crate::graph::element::{Element, Elements};
use crate::graph::node::Node;
use crate::graph::prelude::*;
use indexmap::IndexSet;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

#[cfg(test)]
use std::any::Any;
#[derive(Debug)]
pub(crate) struct Conditional {
    name: String,
    premises: Vec<Premise>,
    action: Event,
}

impl Rule for Conditional {
    fn from_reader(reader: &mut dyn crate::utils::reader::Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut rule = Self::empty();
        rule.name = name;
        while let Some(line) = reader.read_line() {
            if line.starts_with(":CT") {
                break;
            } else if line.contains('|') {
                rule.add_premise(&line);
            } else {
                rule.add_action(&line);
            }
        }
        Ok(rule)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn apply(&mut self, graph: &Graph, output_folder: &mut PathBuf) {
        let action_node = self.find_node_action(graph);
        output_folder.push(self.get_name());
        if let Err(e) = fs::create_dir_all(&output_folder) {
            panic!("failed to create output directory due to :{}", e.to_string());
        }
        for (index, node_id) in action_node.iter().enumerate() {
            let (nodes, edges) = self.inner_apply(graph, &node_id);
            self.write_file(nodes, edges, output_folder, index, &node_id).unwrap();
        }
        output_folder.pop();
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Conditional {
    fn empty() -> Self {
        Conditional {
            name: String::new(),
            premises: Vec::new(),
            action: Event::empty(),
        }
    }

    fn add_premise(&mut self, premise_str: &str) {
        self.premises.push(Premise::new(premise_str));
    }

    fn add_action(&mut self, action_str: &str) {
        self.action = Event::new(action_str);
    }

    fn find_node_action(&self, graph: &Graph) -> Vec<NodeId> {
        let mut result = Vec::new();
        for node_id in graph.get_nodes_id() {
            if graph.has_label(node_id, &self.action) {
                result.push(node_id.clone());
            }
        }
        result
    }

    fn add_nodes_and_edges(
        graph: &Graph,
        path_node: &IndexSet<(NodeId, usize, bool, Option<Rc<str>>)>,
        output_node_id: &mut HashSet<(NodeId, usize, bool)>,
        output_node: &mut HashSet<Node>,
        output_edges: &mut EdgeMap,
        extra_node:Option<(NodeId, usize, bool,Rc<str>)>,
    ) {
        for chunk in path_node.iter().collect::<Vec<_>>().windows(2) {
            if chunk.len() == 2 {
                //build nodes
                let (node_id_to_add, index_to_add, bool_counter_event_to_add, _) = &chunk[0];
                let from = Self::node_add_label(&graph, &node_id_to_add, *index_to_add, *bool_counter_event_to_add);
                output_node_id.insert((node_id_to_add.clone(), *index_to_add, *bool_counter_event_to_add));
                let (node_id_to_add, index_to_add, bool_counter_event_to_add, label) = &chunk[1];
                let to = Self::node_add_label(&graph, &node_id_to_add, *index_to_add, *bool_counter_event_to_add);
                output_node_id.insert((node_id_to_add.clone(), *index_to_add, *bool_counter_event_to_add));

                let from_id = from.get_node_id().clone();
                let to_id = to.get_node_id().clone();
                let label = label.as_ref().unwrap();

                //add the nodes to the output
                output_node.insert(from);
                output_node.insert(to);

                //add the corresponding edge
                let edge = MultiEdge::from(from_id, to_id, &label, Elements::default_edge(&label));
                output_edges.add_edge(&edge);
            }
        }
        if let Some((to_id,index,bool_counter_event,label)) =extra_node{
            let (from_id,from_index,from_bool_counter_event,_) =path_node.last().unwrap();
            let to_id=Self::make_node_id(&to_id,index,bool_counter_event);
            let from_id=Self::make_node_id(from_id,*from_index,*from_bool_counter_event);
            let edge = MultiEdge::from(from_id.clone(), to_id, &label, Elements::default_edge(&label));
            output_edges.add_edge(&edge);
        }
    }

    fn inner_apply_add_cycle(&self,graph:&Graph,action_node_id: &NodeId,mut output_node:HashSet<Node>,mut output_node_id:HashSet<(NodeId, usize, bool)>,mut output_edges:EdgeMap)->(HashSet<Node>, EdgeMap) {
        let mut visited: HashSet<(NodeId, usize, bool)> = HashSet::new();
        let mut execution_stack: Vec<(NodeId, Vec<(NodeId, &MultiEdge)>, Option<Rc<str>>, (usize, bool))> = Vec::new();
        let mut path_node: IndexSet<(NodeId, usize, bool, Option<Rc<str>>)> = IndexSet::new();
        let neighbors = graph.neighbors_tranposed_edges(action_node_id);
        execution_stack.push((action_node_id.clone(), neighbors, None, (self.premises.len(), false)));
        while let Some((node_id, neighbors, from_label, (state_index, state_counter_event))) =
        execution_stack.last_mut()
    {
        path_node.insert((node_id.clone(), *state_index, *state_counter_event, from_label.clone()));
        visited.insert((node_id.clone(), *state_index, *state_counter_event));
        //explore neighbors
        if let Some((neighbor_id, edge_to_reach_neighbor_id)) = neighbors.pop() {
            //we check the transition to neighbors to see how the state evolved

            let state_index_copy = state_index.clone();
            let state_counter_event_copy = state_counter_event.clone();
            for label in edge_to_reach_neighbor_id.get_label_iterator() {
                let (new_state_index, mut new_state_ce) = self.check_label_on_state(state_index_copy, label);
                new_state_ce = new_state_ce || state_counter_event_copy;
                if visited.contains(&(neighbor_id.clone(), new_state_index, new_state_ce))
                    && output_node_id.contains(&(neighbor_id.clone(), new_state_index, new_state_ce))
                    && !self.action.check(label)
                {
                    let mut extra_node=None;
                    if !path_node.insert((neighbor_id.clone(), new_state_index, new_state_ce, Some(label.clone()))){
                        extra_node=Some((neighbor_id.clone(), new_state_index, new_state_ce, label.clone()));
                    }
                    Self::add_nodes_and_edges(
                        graph,
                        &path_node,
                        &mut output_node_id,
                        &mut output_node,
                        &mut output_edges,
                        extra_node,
                    );
                    path_node.pop();
                   
                }
                if !visited.contains(&(neighbor_id.clone(), new_state_index, new_state_ce))
                    && (new_state_index != 0 || new_state_ce)
                {
                    let neighbors = graph.neighbors_tranposed_edges(&neighbor_id);
                    execution_stack.push((
                        neighbor_id.clone(),
                        neighbors,
                        Some(label.clone()),
                        (new_state_index, new_state_ce),
                    ));
                }
            }
        } else {
            path_node.pop();
            execution_stack.pop();
        }
    }
    (output_node, output_edges)
    }

    fn inner_apply_preleminary(&self, graph: &Graph, action_node_id: &NodeId) -> (HashSet<Node>,HashSet<(NodeId, usize, bool)>, EdgeMap) {
        let mut visited: HashSet<(NodeId, usize, bool)> = HashSet::new();
        let mut execution_stack: Vec<(NodeId, Vec<(NodeId, &MultiEdge)>, Option<Rc<str>>, (usize, bool))> = Vec::new();
        let mut path_node: IndexSet<(NodeId, usize, bool, Option<Rc<str>>)> = IndexSet::new();
        let mut output_node: HashSet<Node> = HashSet::new();
        let mut output_node_id: HashSet<(NodeId, usize, bool)> = HashSet::new();
        let mut output_edges: EdgeMap = EdgeMap::new();

        //initialize the execution stack
        let neighbors = graph.neighbors_tranposed_edges(action_node_id);
        execution_stack.push((action_node_id.clone(), neighbors, None, (self.premises.len(), false)));
        //state: state_index=k index means that counter event with <=k-1 are effecient (Ik in previous algorithm)
        //bool indicates if an effective counter event has been reached (true, one at least was present, false: none)
        while let Some((node_id, neighbors, from_label, (state_index, state_counter_event))) =
            execution_stack.last_mut()
        {
            //We check that the state is not 0.
            //if the state is 0 and there no issue then from this state on the path the rule is mandatory true.
            path_node.insert((node_id.clone(), *state_index, *state_counter_event, from_label.clone()));
            visited.insert((node_id.clone(), *state_index, *state_counter_event));
            if (graph.is_starting_node(node_id) && (*state_index != 0 || *state_counter_event))
                || (*state_index == 0 && *state_counter_event)
            {
                Self::add_nodes_and_edges(
                    graph,
                    &path_node,
                    &mut output_node_id,
                    &mut output_node,
                    &mut output_edges,
                    None,
                );
            }
            //explore neighbors
            if let Some((neighbor_id, edge_to_reach_neighbor_id)) = neighbors.pop() {
                //we check the transition to neighbors to see how the state evolved

                let state_index_copy = state_index.clone();
                let state_counter_event_copy = state_counter_event.clone();
                for label in edge_to_reach_neighbor_id.get_label_iterator() {
                    let (new_state_index, mut new_state_ce) = self.check_label_on_state(state_index_copy, label);
                    new_state_ce = new_state_ce || state_counter_event_copy;
                    if visited.contains(&(neighbor_id.clone(), new_state_index, new_state_ce))
                        && output_node_id.contains(&(neighbor_id.clone(), new_state_index, new_state_ce))
                        && !self.action.check(label)
                    {
                        path_node.insert((neighbor_id.clone(), new_state_index, new_state_ce, Some(label.clone())));
                        Self::add_nodes_and_edges(
                            graph,
                            &path_node,
                            &mut output_node_id,
                            &mut output_node,
                            &mut output_edges,
                            None,
                        );
                        path_node.pop();
                    }
                    if !visited.contains(&(neighbor_id.clone(), new_state_index, new_state_ce))
                        && (new_state_index != 0 || new_state_ce)
                    {
                        let neighbors = graph.neighbors_tranposed_edges(&neighbor_id);
                        execution_stack.push((
                            neighbor_id.clone(),
                            neighbors,
                            Some(label.clone()),
                            (new_state_index, new_state_ce),
                        ));
                    }
                }
            } else {
                path_node.pop();
                execution_stack.pop();
            }
        }
        (output_node,output_node_id,output_edges)
    }

    fn inner_apply(&self, graph: &Graph, action_node_id: &NodeId) -> (HashSet<Node>, EdgeMap){
        let (nodes,nodes_id,edges)=self.inner_apply_preleminary(graph,action_node_id);
        self.inner_apply_add_cycle(graph, action_node_id, nodes, nodes_id, edges)
    }

    fn check_label_on_state(&self, index: usize, label: &Rc<str>) -> (usize, bool) {
        //A counter event should not match an event from premise (otherwise the rules is not well formulated).
        let bool_ret = self.check_premise_counter_event(label, &index);
        if index == 0 {
            return (index, bool_ret);
        }
        if self.premises[index - 1].check_event(label) {
            return (index - 1, bool_ret);
        }
        return (index, bool_ret);
    }

    //return true if an effective counter event has been found
    fn check_premise_counter_event(&self, label: &Rc<str>, index: &usize) -> bool {
        //if index equal zero then we return false
        if *index == 0 {
            return false;
        }
        //[..index] index is excluded so we only check to index-1 premises
        for premise in self.premises[..*index].iter() {
            if premise.check_counter_event(label) {
                return true;
            }
        }
        false
    }

    fn write_file(
        &self,
        nodes: HashSet<Node>,
        edges: EdgeMap,
        output_folder: &mut PathBuf,
        index: usize,
        action_node_id: &NodeId,
    ) -> std::io::Result<()> {
        if nodes.is_empty() || edges.is_empty() {
            return Ok(());
        }
        output_folder.push(format!("{index}_ce.dot"));
        let mut output = String::from("digraph \"Automata\" { \n");
        let mut file = match fs::OpenOptions::new().write(true).truncate(true).open(&output_folder) {
            Ok(f) => f,
            Err(_) => File::create(&output_folder)?,
        };
        let action_node_id_str = format!("{}_{}_{}", action_node_id, self.premises.len(), false);
        for mut node in nodes.into_iter() {
            let node_line;
            if node.get_node_id() == &action_node_id_str {
                node.add_element(Element::new_color("color=red").unwrap());
            }
            node_line = node.to_string();
            output.push_str(&node_line)
        }
        let edges_tranposed = edges.transpose();
        output.push_str(&edges_tranposed.to_string());
        let action_edge=format!("\t\"{}_{}_false\"->\"{}_{}_false\"[color=red,label=\"{}\",fontsize=5];",action_node_id,self.premises.len(),action_node_id,self.premises.len(),self.action);
        output.push_str(&action_edge);
        output.push_str("}\n");
        file.write_all(output.as_bytes())?;
        output_folder.pop();
        Ok(())
    }

    //add label for the node. Because in the output we want several node to have the same name.
    fn node_add_label(graph: &Graph, nodeid: &NodeId, index: usize, counter_event: bool) -> Node {
        if let Some(node) = graph.get_node(nodeid) {
            let mut new_node = node.clone();
            let name = new_node.get_node_id();
            let new_node_id = Self::make_node_id_string(name, index, counter_event);
            new_node.add_element(Element::new(&format!("label={}", name)).unwrap());
            new_node.modify_name(&new_node_id);
            return new_node;
        }
        panic!("failed to retrieve a node, in conditional");
    }

    fn make_node_id_string(node_id:&NodeId,index:usize,counter_event: bool)->String{
        format!("{}_{}_{}", node_id, index, counter_event)
    }
    fn make_node_id(node_id:&NodeId,index:usize,counter_event: bool)->NodeId{
        NodeId::new(&format!("{}_{}_{}", node_id, index, counter_event))
    }
}

#[cfg(test)]
mod test {
    //https://dot-to-ascii.ggerganov.com/
    use std::collections::HashMap;

    use crate::{
        graph::{edgemap::EdgeMap, node::Node},
        verifier::rules::parse_rule_from_str,
    };

    use super::{Conditional, Graph, NodeId};

    #[test]
    fn conditional_test() {
        let input = r#"
        digraph "graph"{
            "0" [shape=doubleoctagon, style=filled, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "2" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "3" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "g1" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "g2" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "b1" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "b2" [shape=ellipse, style=filled, fillcolor=white, URL="0"];
            "0" -> "1" [fontsize=5, label="p1/p1", URL="t0"];
            "0" -> "1" [fontsize=5, label="osef/osef", URL="t0"];
            "1" -> "2" [fontsize=5, label="osef/osef", URL="t0"];
            "1" -> "2" [fontsize=5, label="pp1/pp1", URL="t0"];
            "2" -> "3" [fontsize=5, label="osef/osef", URL="t0"];
            "2" -> "3" [fontsize=5, label="p2/p2", URL="t0"];
            "3" -> "3" [fontsize=5, label="action/action", URL="t0"];
            "0" -> "g1" [fontsize=5, label="p1/p1", URL="t0"];
            "g1" -> "g2" [fontsize=5, label="p2/p2", URL="t0"];
            "g2" -> "3" [fontsize=5, label="osef/osef", URL="t0"];
            "0" -> "b1" [fontsize=5, label="p1/p1", URL="t0"];
            "b1" -> "b2" [fontsize=5, label="p2/p2", URL="t0"];
            "b2" -> "3" [fontsize=5, label="pp1/pp1", URL="t0"];
        }"#;
        //         ┌────┐  p2/p2     ┌────────────┐
        //         │ g2 │ ◀───────── │     g1     │
        //         └────┘            └────────────┘
        //           │                 ▲
        //           └─────────────────┼─────────────────────────────────┐
        //                             │                                 │
        // ┌────┐  p2/p2   ┌────┐  p1/p1     ╔════════════╗              │
        // │ b2 │ ◀─────── │ b1 │ ◀───────── ║     0      ║ ─┐           │
        // └────┘          └────┘            ╚════════════╝  │           │
        // │                                 │               │           │
        // │                                 │ p1/p1         │ osef/osef │
        // │                                 ▼               │           │
        // │                               ┌────────────┐    │           │
        // │                     ┌──────── │     1      │   ◀┘           │
        // │                     │         └────────────┘                │
        // │                     │           │                           │
        // │                     │ pp1/pp1   │ osef/osef                 │
        // │                     │           ▼                           │
        // │                     │         ┌────────────┐                │
        // │                     └───────▶ │     2      │ ─┐             │
        // │                               └────────────┘  │             │
        // │                                 │             │             │
        // │                                 │ osef/osef   │ p2/p2       │ osef/osef
        // │                                 ▼             ▼             ▼
        // │                               ┌────────────────────────────────────────┐   action/action
        // │                               │                                        │ ────────────────┐
        // │                    pp1/pp1    │                   3                    │                 │
        // └─────────────────────────────▶ │                                        │ ◀───────────────┘
        //                                 └────────────────────────────────────────┘

        let rule = r#"CT:test42
        p1/p1|pp1/pp1
        p2/p2|pp2/pp2
        action/action
        :CT"#;
        let graph = Graph::new(input, true);
        let rule = parse_rule_from_str(rule);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<Conditional>()
            .expect("expect conditional rule");
        let (nodes, edges) = rule.inner_apply(&graph, &NodeId::new("3"));
        let edges = edges.transpose();
        let node_ids = vec![
            "b1_1_true",
            "0_2_true",
            "0_1_false",
            "1_2_true",
            "0_0_true",
            "3_2_false",
            "0_2_false",
            "1_1_false",
            "0_1_true",
            "2_2_false",
            "1_1_true",
            "1_2_false",
            "2_1_false",
            "b2_2_true",
        ];
        let node_ids: Vec<NodeId> = node_ids.iter().map(|e| NodeId::new(e)).collect();
        let nodes: HashMap<NodeId, &Node> = nodes.iter().map(|n| (n.get_node_id().clone(), n)).collect();
        for node_id in node_ids.iter() {
            assert!(nodes.contains_key(node_id));
        }
        assert_eq!(nodes.len(), node_ids.len());
        let edges_expected = r#"
        "b2_2_true"->"3_2_false" [fontsize=5,label="pp1/pp1"];
        "2_2_false"->"3_2_false" [fontsize=5,label="osef/osef"];
        "2_1_false"->"3_2_false" [fontsize=5,label="p2/p2"];
        "1_1_false"->"2_1_false" [fontsize=5,label="osef/osef"];
        "1_1_true"->"2_1_false" [fontsize=5,label="pp1/pp1"];
        "0_1_true"->"1_1_true" [fontsize=5,label="osef/osef"];
        "0_0_true"->"1_1_true" [fontsize=5,label="p1/p1"];
        "0_0_true"->"b1_1_true" [fontsize=5,label="p1/p1"];
        "0_1_false"->"1_1_false" [fontsize=5,label="osef/osef"];
        "1_2_true"->"2_2_false" [fontsize=5,label="pp1/pp1"];
        "1_2_false"->"2_2_false" [fontsize=5,label="osef/osef"];
        "0_2_true"->"1_2_true" [fontsize=5,label="osef/osef"];
        "0_2_true"->"1_2_true" [fontsize=5,label="p1/p1"];
        "0_2_false"->"1_2_false" [fontsize=5,label="osef/osef"];
        "0_2_false"->"1_2_false" [fontsize=5,label="p1/p1"];
        "b1_1_true"->"b2_2_true" [fontsize=5,label="p2/p2"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }

        assert_eq!(expected_edge_map, edges);
    }

    #[test]
    fn conditional_test_cycle() {
        let input = r#"
        digraph "Automata" {
            "0" [shape=doubleoctagon, style=filled, fillcolor=white, URL="0"];
            "0bis" [shape=doubleoctagon, style=filled, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filled, fillcolor=white, URL="1"];
            "4" [shape=ellipse, style=filled, fillcolor=white, URL="4"];
            "3" [shape=ellipse, style=filled, fillcolor=white, URL="3"];
            "2" [shape=ellipse, style=filled, fillcolor=white, URL="2"];
            "0" -> "0" [fontsize=5, label="open_secure_channel_request / OpnRepOK,", URL="t1"];
            "0bis" -> "0" [fontsize=5, label="u / u", URL="t1"];
            "0" -> "0bis" [fontsize=5, label="u / u", URL="t1"];
            "0" -> "1" [fontsize=5, label="open_secure_channel_request / OpnRepOK,", URL="t1"];
            "1" -> "4" [fontsize=5, label="create_session / CreSesResOK,", URL="t19"];
            "4" -> "4" [fontsize=5, label="open_secure_channel_request / OpnRepOK,", URL="t57"];
            "4" -> "4" [fontsize=5, label="read_req / ReadRepOK,", URL="t67"];
            "4" -> "4" [fontsize=5, label="write_req / WriteRepOK,", URL="t68"];
            "4" -> "2" [fontsize=5, label="hello / Ack,", URL="t56"];
            "4" -> "3" [fontsize=5, label="close_secure_channel_request / Eof,", URL="t60"];
        }"#;
        let rule = r#"CT:restricted_address_space_access
        active_session+active_session_cert/AcSesResOK,|close_session/*CloSesResOK,*
        read_req+write_req/*ReadRepOK,*+*WriteRepOK,*
        :CT"#;
        let graph = Graph::new(input, true);
        let rule = parse_rule_from_str(rule);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<Conditional>()
            .expect("expect conditional rule");
        let (nodes, edges) = rule.inner_apply(&graph, &NodeId::new("4"));
        let edges = edges.transpose();
        let node_ids = vec![
            "0_1_false",
            "1_1_false",
            "4_1_false",
            "0bis_1_false",
        ];
        let node_ids: Vec<NodeId> = node_ids.iter().map(|e| NodeId::new(e)).collect();
        let nodes: HashMap<NodeId, &Node> = nodes.iter().map(|n| (n.get_node_id().clone(), n)).collect();
        for node_id in node_ids.iter() {
            assert!(nodes.contains_key(node_id));
        }
        assert_eq!(nodes.len(), node_ids.len());
        let edges_expected = r#""1_1_false"->"4_1_false" [fontsize=5,label="create_session / CreSesResOK,"];
	    "4_1_false"->"4_1_false" [fontsize=5,label="open_secure_channel_request / OpnRepOK,"];
	    "0_1_false"->"0bis_1_false" [fontsize=5,label="u / u"];
	    "0_1_false"->"1_1_false" [fontsize=5,label="open_secure_channel_request / OpnRepOK,"];
	    "0_1_false"->"0_1_false" [fontsize=5,label="open_secure_channel_request / OpnRepOK,"];
	    "0bis_1_false"->"0_1_false" [fontsize=5,label="u / u"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }

        assert_eq!(expected_edge_map, edges);
        println!("{edges}");

    }
}
