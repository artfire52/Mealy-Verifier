use indexmap::IndexSet;
use std::{hash::Hash, rc::Rc, vec::IntoIter};

use crate::{utils::unquote, verifier::event::Event};

use super::{element::Elements, nodeid::NodeId};
#[derive(Clone, Eq, Debug)]
pub(crate) struct MultiEdge {
    from: NodeId,
    to: NodeId,
    label: IndexSet<Rc<str>>,
    elements: IndexSet<Elements>,
}

impl Hash for MultiEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
    }
}
impl MultiEdge {
    pub(crate) fn get_labels(&self) -> Vec<Rc<str>> {
        let mut result = Vec::new();
        for i in self.label.iter() {
            result.push(i.clone());
        }
        result
    }

    pub(crate) fn from(from: NodeId, to: NodeId, label: &str, elements: Elements) -> Self {
        Self {
            from,
            to,
            label: IndexSet::from([Rc::from(unquote(label))]),
            elements: IndexSet::from([elements]),
        }
    }

    pub(crate) fn from_multiple_labels(
        from: NodeId,
        to: NodeId,
        label: IndexSet<Rc<str>>,
        elements: IndexSet<Elements>,
    ) -> Self {
        Self {
            from,
            to,
            label,
            elements,
        }
    }

    pub(crate) fn add_label(&mut self, label: &str, elements: Elements) {
        let ret=self.label.insert(Rc::from(unquote(label)));
        if ret{
            self.elements.insert(elements);
        }
    }

    pub(crate) fn get_inner(&self) -> (NodeId, NodeId, IndexSet<Rc<str>>, IndexSet<Elements>) {
        (
            self.from.clone(),
            self.to.clone(),
            self.label.clone(),
            self.elements.clone(),
        )
    }

    pub(crate) fn get_dest(&self) -> &NodeId {
        &self.to
    }

    ///Has a label (equality)
    pub(crate) fn has_label(&self, event: &Event) -> bool {
        for label in self.label.iter() {
            if event.check(label) {
                return true;
            }
        }
        false
    }

    pub(crate) fn to_string_labels(&self, indexes: IntoIter<usize>) -> Vec<String> {
        // let output = String::new();
        let mut res = Vec::new();
        for index in indexes {
            let s = format!("\t \"{}\"->\"{}\" {};\n", self.from, self.to, self.elements[index]);
            res.push(s);
        }

        res
    }

    pub(crate) fn get_label_iterator(&self) -> indexmap::set::Iter<'_, Rc<str>> {
        self.label.iter()
    }

    pub(crate) fn get_source(&self) -> &NodeId {
        &self.from
    }

    pub(crate) fn get_nb_label(&self) -> usize {
        self.label.len()
    }
}

impl PartialEq for MultiEdge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.label == other.label
    }
}

impl std::fmt::Display for MultiEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_string = String::new();
        for elements in self.elements.iter() {
            output_string.push_str(&format!("\t \"{}\"->\"{}\" {};\n", self.from, self.to, elements));
        }
        write!(f, "{}", output_string)
    }
}

#[cfg(test)]
impl MultiEdge {
    pub(crate) fn new(line: &str) -> Self {
        let line: String = line.trim().to_string();
        let index = line.find("[").expect("Malformed file");
        let elements: Elements = Elements::new(&line[index + 1..]);

        let label = match elements.get_label() {
            Some(str) => unquote(str).to_string(),
            None => panic!("Failed to found label for transition {:?}", line),
        };
        let index2 = line.find("-").expect("Malformed file");
        Self {
            from: NodeId::new(&line[..index2]),
            to: NodeId::new(&line[index2 + 2..index]),
            label: IndexSet::from([Rc::from(label)]),
            elements: IndexSet::from([elements]),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use indexmap::IndexSet;

    use crate::graph::{element::Elements, multi_edge::MultiEdge, nodeid::NodeId};

    #[test]
    fn parsing_simple_edge() {
        let input = "    \"0\" -> \"1\" [fontsize=5, label=\"a/b\", color=black];".to_string();
        let edge = MultiEdge::new(&input);
        let expected_edge = MultiEdge {
            from: NodeId::new("0"),
            to: NodeId::new("1"),
            label: IndexSet::from([Rc::from("a/b")]),
            elements: IndexSet::from([Elements::new("fontsize=5, label=\"a/b\", color=black")]),
        };
        assert_eq!(edge, expected_edge);
    }
}
