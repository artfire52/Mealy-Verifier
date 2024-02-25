use std::{fmt::Display, rc::Rc};

use regex::Regex;

/// Element that describe an edge or a node in a graph (for dot format)
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub(crate) struct Element {
    field: String,
    value: String,
}

impl Element {
    pub(crate) fn new_vec(elmtns_str: &str) -> Vec<Self> {
        let mut elements: Vec<Element> = Vec::new();
        // let start = std::time::Instant::now();
        let re = Regex::new(r#"\w+\s*=\s*("[^"]*"|[^],])\s*[^],]*"#).unwrap();
        for cap in re.captures_iter(elmtns_str) {
            if let Some(e) = Element::new(&cap[0]) {
                elements.push(e);
            }
        }
        elements
    }
    pub(crate) fn new(elmtns_str: &str) -> Option<Self> {
        let el: Vec<&str> = elmtns_str.split("=").collect();
        if el[0].trim() == "color" {
            return None;
        }
        Some(Element {
            field: el[0].trim().to_string(),
            value: el[1].trim().to_string(),
        })
    }

    pub(crate) fn new_color(elmtns_str: &str) -> Option<Self> {
        let el: Vec<&str> = elmtns_str.split("=").collect();
        Some(Element {
            field: el[0].trim().to_string(),
            value: el[1].trim().to_string(),
        })
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.field == "label" {
            write!(f, "{}=\"{}\"", self.field, crate::utils::unquote(&self.value))
        } else {
            write!(f, "{}={}", self.field, self.value)
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub(crate) struct Elements {
    inner: Vec<Element>,
}
impl Elements {
    pub(crate) fn new(line: &str) -> Self {
        Elements {
            inner: Element::new_vec(line),
        }
    }

    pub(crate) fn get_label(&self) -> Option<&str> {
        for element in self.inner.iter() {
            if element.field == "label" {
                return Some(&element.value);
            }
        }
        return None;
    }

    pub(crate) fn default_edge(label: &Rc<str>) -> Self {
        let font_size = Element {
            field: "fontsize".to_string(),
            value: "5".to_string(),
        };
        let label = Element {
            field: "label".to_string(),
            value: format!("\"{}\"", label),
        };
        let mut inner = Vec::new();
        inner.push(font_size);
        inner.push(label);
        Self { inner }
    }

}

impl Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut elements_str = String::from("[");
        elements_str.push_str(&self.inner[0].to_string());
        for element in self.inner[1..].iter() {
            elements_str.push_str(",");
            elements_str.push_str(&element.to_string());
        }
        elements_str.push_str("]");
        write!(f, "{}", elements_str)
    }
}

#[cfg(test)]
mod tests {
    use super::Element;
    #[test]
    fn parsing_elements_vec() {
        let expected_elements = vec![
            Element {
                field: "shape".to_string(),
                value: "ellipse".to_string(),
            },
            Element {
                field: "style".to_string(),
                value: "filed".to_string(),
            },
            Element {
                field: "fillcolor".to_string(),
                value: "white".to_string(),
            },
            Element {
                field: "URL".to_string(),
                value: "\"0\"".to_string(),
            },
            Element {
                field: "label".to_string(),
                value: "a/b".to_string(),
            },
        ];
        let elements = Element::new_vec("shape=ellipse, style=filed, fillcolor=white, URL=\"0\", label=a/b]");
        assert_eq!(elements, expected_elements);

        let expected_elements = vec![
            Element {
                field: "shape".to_string(),
                value: "ellipse".to_string(),
            },
            Element {
                field: "style".to_string(),
                value: "filed".to_string(),
            },
            Element {
                field: "fillcolor".to_string(),
                value: "white".to_string(),
            },
            Element {
                field: "URL".to_string(),
                value: "\"0\"".to_string(),
            },
            Element {
                field: "label".to_string(),
                value: "\"a/b\"".to_string(),
            },
        ];
        let elements = Element::new_vec("shape=ellipse, style=filed, fillcolor=white, URL=\"0\", label=\"a/b\"]");
        assert_eq!(elements, expected_elements);
    }
    #[test]
    fn parsing_element() {
        let expected_elements = Element {
            field: "shape".to_string(),
            value: "ellipse".to_string(),
        };
        let elements = Element::new("shape=ellipse").unwrap();
        assert_eq!(elements, expected_elements);
    }
}
