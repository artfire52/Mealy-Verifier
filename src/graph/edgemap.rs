use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use indexmap::{IndexMap, IndexSet};

use crate::utils::unquote;

use super::{element::Elements, multi_edge::MultiEdge, nodeid::NodeId};
type EgdgeMapIntern = HashMap<NodeId, MultiEdge>;
type EdgeMapInternal = IndexMap<NodeId, EgdgeMapIntern>;

#[derive(Clone, Debug)]
pub(crate) struct EdgeMap(EdgeMapInternal);

impl EdgeMap {
    //Wrapper to handle map of edges
    //multiedge are used to merge different edges with same source and destination and
    //different labels.
    pub(crate) fn new() -> Self {
        EdgeMap(EdgeMapInternal::new())
    }

    pub(crate) fn add_edge_from_str(&mut self, edge_str: &str) {
        let (from, to, label, elements) = Self::parse_label(edge_str);
        self.inner_add_edge(from, to, &label, elements)
    }

    fn inner_add_edge(&mut self, from: NodeId, to: NodeId, label: &str, elements: Elements) {
        match self.0.get_mut(&from) {
            Some(map) => {
                //An edge starting from this node exist
                match map.get_mut(&to) {
                    Some(multi_edge) => {
                        //there is an other edge with the same destination node
                        //Hence we add the label to the multiedge
                        multi_edge.add_label(label, elements);
                    }
                    None => {
                        //there is no other edge with the same destination node
                        //Hence, we create the multiedge
                        let edge = MultiEdge::from(from, to.clone(), label, elements);
                        map.insert(to, edge);
                    }
                }
            }
            None => {
                //An edge starting from this node does not exist
                let mut edge_map = HashMap::new();
                edge_map.insert(to.clone(), MultiEdge::from(from.clone(), to, label, elements));
                self.0.insert(from, edge_map);
            }
        }
    }

    fn parse_label(line_str: &str) -> (NodeId, NodeId, String, Elements) {
        let line: String = line_str.trim().to_string();
        let index = line.find("[").expect("Malformed file");
        let elements: Elements = Elements::new(&line[index + 1..]);
        let label = match elements.get_label() {
            Some(str) => unquote(str).to_string(),
            None => panic!("Failed to found label for transition {:?}", line),
        };
        let index_state_separator = line.find("-").expect("Malformed file");
        let from = NodeId::new(&line[..index_state_separator]);
        let to = NodeId::new(&line[index_state_separator + 2..index]);
        (from, to, label, elements)
    }

    pub(crate) fn transpose(&self) -> Self {
        let mut result = Self::new();
        for edge_map in self.0.values() {
            for edge in edge_map.values() {
                let (from, to, labels, elements) = edge.get_inner();
                result.inner_add_edge_for_transpose(to, from, labels, elements);
            }
        }
        result
    }

    fn inner_add_edge_for_transpose(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: IndexSet<Rc<str>>,
        elements: IndexSet<Elements>,
    ) {
        match self.0.get_mut(&from) {
            Some(map) => {
                //An edge starting from this node exist
                let edge = MultiEdge::from_multiple_labels(from, to.clone(), label, elements);
                map.insert(to, edge);
            }
            None => {
                //An edge starting from this node does not exist
                let mut edge_map = HashMap::new();
                edge_map.insert(
                    to.clone(),
                    MultiEdge::from_multiple_labels(from.clone(), to, label, elements),
                );
                self.0.insert(from, edge_map);
            }
        }
    }

    pub(crate) fn add_edge(&mut self, multi_edge: &MultiEdge) {
        let (from, to, labels, elements) = multi_edge.get_inner();
        for (label, element) in labels.iter().zip(elements) {
            self.inner_add_edge(from.clone(), to.clone(), label, element);
        }
    }

    pub(crate) fn add_edge_with_indexes(&mut self, multi_edge: &MultiEdge,indexes:&[usize]) {
        let (from, to, labels, elements) = multi_edge.get_inner();
        for i in indexes{
            self.inner_add_edge(from.clone(), to.clone(), labels.get_index(*i).unwrap(), elements.get_index(*i).unwrap().clone());
        }

    }

}

impl Deref for EdgeMap {
    type Target = EdgeMapInternal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EdgeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for EdgeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_stirng = String::new();
        for edge_hashmap in self.0.values() {
            for edge in edge_hashmap.values() {
                output_stirng.push_str(&edge.to_string());
            }
        }

        write!(f, "{}", output_stirng)
    }
}

impl PartialEq for EdgeMap {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for (node, map) in other.0.iter() {
            if !self.0.contains_key(node) {
                return false;
            }
            let self_map = self.0.get(node).unwrap();
            if self_map.len() != map.len() {
                return false;
            }
            for (to, edge) in map {
                if !self_map.contains_key(to) {
                    return false;
                }
                let self_edge = self_map.get(to).unwrap();
                if !self_edge.eq(edge) {
                    return false;
                }
            }
        }
        true
    }
}

impl PartialEq<HashSet<(&MultiEdge, Vec<usize>)>> for EdgeMap {
    fn eq(&self, other: &HashSet<(&MultiEdge, Vec<usize>)>) -> bool {
        for (edge, label_index) in other.iter() {
            let from = edge.get_source();
            let to = edge.get_dest();
            match self.0.get(from) {
                Some(map) => {
                    if let Some(self_edge) = map.get(to) {
                        let labels = edge.get_labels();
                        let labels: HashSet<_> = labels
                            .iter()
                            .enumerate()
                            .filter_map(|(index, label)| {
                                if label_index.contains(&index) {
                                    Some(label)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let self_labels = self_edge.get_labels();
                        if labels.len() != self_labels.len() {
                            return false;
                        }
                        for i in self_labels.iter() {
                            if !labels.contains(i) {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }
}
