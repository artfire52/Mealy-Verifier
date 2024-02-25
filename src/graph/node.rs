use super::{element::Element, nodeid::NodeId};
// use crate::utils::*;
#[derive(Debug, Eq, Clone, Hash)]
pub(crate) struct Node {
    pub(crate) nodeid: NodeId,
    pub(crate) elements: Vec<Element>,
}

impl Node {
    pub(crate) fn new(line: &str) -> Self {
        let line: String = line.trim().to_string();
        let (nodeid, line) = NodeId::from_str(&line);
        Node {
            nodeid: nodeid,
            elements: Element::new_vec(line),
        }
    }

    pub(crate) fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub(crate) fn get_node_id(&self) -> &NodeId {
        &self.nodeid
    }

    pub(crate) fn modify_name(&mut self, name: &str) {
        self.nodeid = NodeId::new(name);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.nodeid.eq(&other.nodeid)
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut elements_str = String::from("[");
        elements_str.push_str(&self.elements[0].to_string());
        for element in self.elements[1..].iter() {
            elements_str.push_str(",");
            elements_str.push_str(&element.to_string());
        }
        elements_str.push_str("];");
        write!(f, "\t \"{}\" {}\n", self.nodeid, elements_str)
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use crate::graph::{element::Element, nodeid::NodeId};
    #[test]
    fn parsing_node() {
        let input = "     \"0\" [shape=ellipse, style=filed, fillcolor=white, URL=\"0\"];".to_string();
        let node = Node::new(&input);

        let expected_node = Node {
            nodeid: NodeId::new("0"),
            elements: Element::new_vec("shape=ellipse, style=filed, fillcolor=white, URL=\"0\""),
        };
        assert_eq!(node, expected_node);
    }

    #[test]
    fn modify_name_test() {
        let input = "     \"0\" [shape=ellipse, style=filed, fillcolor=white, URL=\"0\"];".to_string();
        let mut node = Node::new(&input);

        let expected_node = Node {
            nodeid: NodeId::new("1"),
            elements: Element::new_vec("shape=ellipse, style=filed, fillcolor=white, URL=\"0\""),
        };
        node.modify_name("1");
        assert_eq!(node, expected_node);
    }
}
