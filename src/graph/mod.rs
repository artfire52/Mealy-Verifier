pub(crate) mod edgemap;
pub(crate) mod element;
pub(crate) mod multi_edge;
pub(crate) mod node;
pub(crate) mod nodeid;
use indexmap::map::Keys;
use indexmap::IndexMap;
use node::Node;

use std::collections::HashSet;
use std::rc::Rc;

pub(crate) type NodeMap = IndexMap<NodeId, Node>;
// pub(crate) type EdgeMap = IndexMap<NodeId, HashSet<Rc<MultiEdge>>>;

//we use Reference counting to avoid complete copy of graph when using subgraph.
pub(crate) struct Graph {
    name: String,
    nodes: NodeMap,
    edges: EdgeMap,
    transpose_edges: EdgeMap,
    root: Option<NodeId>,
    sinks: Vec<NodeId>,
}
use crate::utils::reader::Reader;
use crate::utils::reader::ReaderFile;
use crate::verifier::event::Event;

use self::edgemap::EdgeMap;
use self::multi_edge::MultiEdge;
use self::nodeid::NodeId;
impl Graph {
    pub(crate) fn new_file(path_to_file: &str) -> Self {
        let mut reader = match ReaderFile::open(path_to_file) {
            Ok(reader) => reader,
            Err(e) => {
                panic!("Error while reading graph file '{}' : {}", path_to_file, e.to_string())
            }
        };
        let mut nodes: NodeMap = NodeMap::new();
        let mut edges: EdgeMap = EdgeMap::new();
        let mut node: Node;
        while let Some(line) = reader.read_line() {
            if Graph::is_node(&line) {
                node = Node::new(&line);
                nodes.insert(node.nodeid.clone(), node);
            } else if Graph::is_edge(line) {
                edges.add_edge_from_str(line);
            }
        }
        let name_vec: Vec<&str> = path_to_file.split("/").collect();
        let name: String;
        if name_vec.len() > 1 {
            name = format!("{}_{}", name_vec[name_vec.len() - 2], name_vec[name_vec.len() - 1]);
        } else {
            name = name_vec.last().unwrap().to_string();
        }
        let transpose = edges.transpose();
        let mut graph = Graph {
            name: name,
            nodes,
            edges,
            transpose_edges: transpose,
            root: None,
            sinks: Vec::new(),
        };
        graph.identify_sink_state();
        graph.identify_start_state();

        graph
    }

    fn is_edge(line: &str) -> bool {
        line.contains("->")
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    fn is_node(line: &str) -> bool {
        !Graph::is_edge(line) && line.contains("[")
    }
    pub(crate) fn _set_root(&mut self, root_id: &str) {
        self.root = Some(NodeId::new(root_id));
    }

    fn identify_start_state(&mut self) {
        //this function will be looking for the first node that has only outgoing edges.
        //Mealy machine shoudl only have one starting state.
        if self.root.is_some() {
            self.root = None;
        }
        for node in self.nodes.keys() {
            match &self.transpose_edges.get(node) {
                Some(edge) => {
                    let mut is_start = true;
                    for from in edge.keys() {
                        is_start = is_start && (from == node);
                    }
                    if is_start {
                        if self.root.is_some() {
                            panic!("several starting state found, exactly one is expected");
                        }
                        self.root = Some(node.clone());
                    }
                }
                None => {
                    if self.root.is_some() {
                        panic!("several starting state found, exactly one is expected");
                    }
                    self.root = Some(node.clone());
                }
            }
            if node=="0" && self.root.is_none(){
                if self.root.is_some() {
                    panic!("several starting state found, exactly one is expected");
                }
                self.root = Some(node.clone());
            }
        }
        if self.root.is_none() {
            panic!("no starting state on the mealy machine");
        }
    }

    //from a node id give the neighbors and
    pub(crate) fn neighbors_edges_iterator(&self, node_id: &NodeId) -> Vec<(NodeId, &MultiEdge)> {
        let mut ret: Vec<(NodeId, &MultiEdge)> = Vec::new();
        let edges = match self.edges.get(node_id) {
            None => return ret,
            Some(edge_from_node_id) => edge_from_node_id,
        };
        for (node, edge) in edges.iter() {
            ret.push((node.clone(), edge));
        }
        ret
    }


    pub(crate) fn neighbors_tranposed_edges(&self, node_id: &NodeId) -> std::vec::Vec<(NodeId, &MultiEdge)> {
        let mut ret: Vec<(NodeId, &MultiEdge)> = Vec::new();
        let edges = match self.transpose_edges.get(node_id) {
            None => return ret,
            Some(edge_from_node_id) => edge_from_node_id,
        };
        for (node, edge) in edges.iter() {
            ret.push((node.clone(), edge));
        }
        ret
    }

    pub(crate) fn is_starting_node(&self, node_id: &NodeId) -> bool {
        if let Some(e) = &self.root {
            return node_id == e;
        }
        false
    }

    pub(crate) fn _get_nodes(&self) -> HashSet<NodeId> {
        self.nodes.keys().map(|node_id| node_id.clone()).collect()
    }

    pub(crate) fn iter_node_id(&self) ->Keys<'_, NodeId, Node> {
        self.nodes.keys()
    }

    pub(crate) fn _get_nodes_iterator(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().map(|node_id| node_id.clone())
    }

    pub(crate) fn get_outgoing_labels(&self, node_id: &NodeId) -> Vec<Rc<str>> {
        let mut result = Vec::new();
        if let Some(edges_map) = self.edges.get(node_id) {
            for edge in edges_map.values() {
                let labels = edge.get_labels();
                result.extend(labels);
            }
        }
        result
    }

    pub(crate) fn get_outgoing_edges(&self, node_id: &NodeId) -> Option<&std::collections::HashMap<NodeId, MultiEdge>> {
        self.edges.get(node_id)
    }

    pub(crate) fn get_root(&self) -> Option<&NodeId> {
        self.root.as_ref()
    }


    pub(crate) fn get_nodes_id(&self) -> Keys<'_, NodeId, Node> {
        self.nodes.keys()
    }

    pub(crate) fn get_node(&self, nodeid: &NodeId) -> Option<&Node> {
        self.nodes.get(nodeid)
    }

    pub(crate) fn has_label(&self, node_id: &NodeId, label: &Event) -> bool {
        if let Some(edge) = self.edges.get(node_id) {
            for edge in edge.values() {
                if edge.has_label(label) {
                    return true;
                }
            }
        }
        false
    }
    
    fn identify_sink_state(&mut self){
        let mut result = Vec::new();
        let mut is_sink: bool;
        for source in self.nodes.keys() {
            if let Some(edgemap) = self.edges.get(source) {
                is_sink = true;
                for dest in edgemap.keys() {
                    if !dest.eq(source) {
                        is_sink = false;
                        break;
                    }
                }
                if is_sink {
                    result.push(source.clone());
                }
            } else {
                result.push(source.clone());
            }
        }
        self.sinks=result;
    }
    pub(crate) fn get_sink_state(&self) -> &Vec<NodeId> {
       &self.sinks
    }

    pub(crate) fn get_sink_state_set(&self) -> HashSet<&NodeId> {
        self.sinks.iter().collect()
    }

    pub(crate) fn iter_edges(&self) -> Vec<&MultiEdge> {
        let mut res_iter: Vec<&MultiEdge> = Vec::new();
        for (_, edges) in self.edges.iter() {
            for edge in edges.values() {
                res_iter.push(edge);
            }
        }
        res_iter
    }
}

#[cfg(test)]
impl Graph{
    pub(crate) fn get_edge(&self, from: &NodeId, to: &NodeId) -> Option<&MultiEdge> {
        if let Some(edges) = self.edges.get(from) {
            match edges.get(to) {
                Some(multi_edge) => return Some(multi_edge),
                None => return None,
            };
        } else {
            None
        }
    }
}

pub(crate) mod prelude {
    // pub(crate) use super::super::algorithm::cycle::Cycle;
    pub(crate) use super::multi_edge::MultiEdge;
    pub(crate) use super::nodeid::NodeId;
    pub(crate) use super::Graph;
}
#[cfg(test)]
impl Graph {
    pub(crate) fn new(content: &str, starting_state: bool) -> Self {
        let lines: Vec<&str> = content.split("\n").collect();
        let mut nodes: NodeMap = NodeMap::new();
        let mut edges: EdgeMap = EdgeMap::new();
        let mut node: Node;
        for line in lines {
            if Graph::is_node(&line) {
                node = Node::new(&line);
                nodes.insert(node.nodeid.clone(), node);
            } else if Graph::is_edge(line) {
                edges.add_edge_from_str(line);
            }
        }
        let transpose = edges.transpose();
        let mut graph = Graph {
            name: "test_graph".to_string(),
            nodes,
            edges,
            transpose_edges: transpose,
            root: None,
            sinks: Vec::new(),
        };
        graph.identify_sink_state();
        if starting_state {
            graph.identify_start_state();
        }

        graph
    }
}

#[cfg(test)]
mod tests {
  
    use std::collections::HashSet;

    use crate::graph::{multi_edge::MultiEdge, node::Node, nodeid::NodeId, Graph};

    #[test]
    fn read_graph() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="2"];
            "0" -> "1" [fontsize=5, label="a/b", color=black];
            "1" -> "1" [fontsize=5, label="a/b", color=black];
            "1" -> "2" [fontsize=5, label="a/b", color=black];
            "0" -> "2" [fontsize=5, label="a/b", color=black];
        }"#;
        let graph: Graph = Graph::new(input, false);
        let node = Node::new(r#""0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];"#);
        assert_eq!(node, *graph.nodes.get(&node.nodeid).unwrap());
        let node = Node::new(r#""1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];"#);
        assert_eq!(node, *graph.nodes.get(&node.nodeid).unwrap());
        let node = Node::new(r#""2" [shape=ellipse, style=filed, fillcolor=white, URL="2"];"#);
        assert_eq!(node, *graph.nodes.get(&node.nodeid).unwrap());

        let edge = MultiEdge::new(r#""0" -> "1" [fontsize=5, label="a/b", color=black];"#);
        assert_eq!(
            edge,
            graph.get_edge(&NodeId::new("0"), &NodeId::new("1")).unwrap().clone()
        );
        let edge = MultiEdge::new(r#""0" -> "2" [fontsize=5, label="a/b", color=black];"#);
        assert_eq!(
            edge,
            graph.get_edge(&NodeId::new("0"), &NodeId::new("2")).unwrap().clone()
        );

        let edge = MultiEdge::new(r#""1" -> "1" [fontsize=5, label="a/b", color=black];"#);
        assert_eq!(
            edge,
            graph.get_edge(&NodeId::new("1"), &NodeId::new("1")).unwrap().clone()
        );
        let edge = MultiEdge::new(r#""1" -> "2" [fontsize=5, label="a/b", color=black];"#);
        assert_eq!(
            edge,
            graph.get_edge(&NodeId::new("1"), &NodeId::new("2")).unwrap().clone()
        );

        assert_eq!(graph.nodes.len(), 3);

        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn sink_detection() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="2"];
            "0" -> "1" [fontsize=5, label="a/b", color=black];
            "1" -> "1" [fontsize=5, label="a/b", color=black];
            "1" -> "2" [fontsize=5, label="a/b", color=black];
            "0" -> "2" [fontsize=5, label="a/b", color=black];
            "2" -> "2" [fontsize=5, label="a/b", color=black];
        }"#;
        //         ┌──────┐
        //         │  0   │ ─┐
        //         └──────┘  │
        //           │       │
        //           │ a/b   │
        //           ▼       │
        //   a/b   ┌──────┐  │
        // ┌────── │      │  │
        // │       │  1   │  │ a/b
        // └─────▶ │      │  │
        //         └──────┘  │
        //           │       │
        //           │ a/b   │
        //           ▼       │
        //   a/b   ┌──────┐  │
        // ┌────── │      │  │
        // │       │  2   │  │
        // └─────▶ │      │ ◀┘
        //         └──────┘
        let graph: Graph = Graph::new(input, false);
        let sink_nodes = graph.get_sink_state().iter().cloned().collect::<HashSet<_>>();
        assert!(sink_nodes.contains(&NodeId::new("2")));
        assert_eq!(sink_nodes.len(), 1);
    }

    #[test]
    fn start_detection() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="2"];
            "0" -> "1" [fontsize=5, label="a/b", color=black];
            "1" -> "1" [fontsize=5, label="a/b", color=black];
            "1" -> "2" [fontsize=5, label="a/b", color=black];
            "0" -> "2" [fontsize=5, label="a/b", color=black];
        }"#;

        //         ┌──────┐
        //         │  0   │ ─┐
        //         └──────┘  │
        //           │       │
        //           │ a/b   │
        //           ▼       │
        //   a/b   ┌──────┐  │
        // ┌────── │      │  │
        // │       │  1   │  │ a/b
        // └─────▶ │      │  │
        //         └──────┘  │
        //           │       │
        //           │ a/b   │
        //           ▼       │
        //         ┌──────┐  │
        //         │  2   │ ◀┘
        //         └──────┘

        let graph: Graph = Graph::new(input, true);
        assert_eq!(*graph.get_root().unwrap(), NodeId::new("0"));
    }
}
