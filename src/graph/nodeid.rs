use std::rc::Rc;

use crate::utils::unquote;

#[derive(Debug, Eq, Clone, Hash)]
pub(crate) struct NodeId {
    pub(crate) inner: Rc<str>,
}

impl NodeId {
    pub(crate) fn from_str(node_line: &str) -> (NodeId, &str) {
        let line = node_line.trim();
        let index = line.find("[").unwrap();
        let nodeid = NodeId {
            inner: Rc::from(unquote(line[..index].trim())),
        };
        (nodeid, &line[index..])
    }

    pub(crate) fn new(node_str: &str) -> Self {
        let line: String = node_str.trim().to_string();
        NodeId {
            inner: Rc::from(unquote(&line)),
        }
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref().eq(other.inner.as_ref())
    }
}
impl PartialEq<Rc<str>> for NodeId {
    fn eq(&self, other: &Rc<str>) -> bool {
        self.inner.as_ref().eq(other.as_ref())
    }
}
impl PartialEq<str> for NodeId {
    fn eq(&self, other: &str) -> bool {
        self.inner.as_ref().eq(other)
    }
}

impl PartialEq<String> for NodeId {
    fn eq(&self, other: &String) -> bool {
        (&*self.inner).eq(&*other)
    }
}
#[cfg(test)]
impl NodeId {
    pub(crate) fn get_inner(&self) -> &Rc<str> {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::nodeid::NodeId;
    #[test]
    pub(crate) fn test_new() {
        let node_id = NodeId::new("5");
        assert_eq!(node_id.inner.as_ref(), "5");
        assert!(!node_id.get_inner().contains("\""));
    }

    #[test]
    fn test_from_str() {
        let (node_id, other) =
            NodeId::from_str("\"5\" [shape=ellipse,style=filled,fillcolor=white,URL=\"3\", color=red];");
        assert_eq!(node_id.inner.as_ref(), "5");
        assert_eq!(
            other,
            "[shape=ellipse,style=filled,fillcolor=white,URL=\"3\", color=red];"
        );
        assert!(!node_id.get_inner().contains("\""));
    }
}
