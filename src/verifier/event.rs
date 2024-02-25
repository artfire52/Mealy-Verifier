use core::fmt::Debug;
use std::fmt::Display;
use wildmatch::WildMatch;

#[derive(Debug)]
pub(crate) struct Events {
    events: Vec<Event>,
}
impl Events {
    pub(crate) fn from_str(events_str: &str) -> Self {
        let events_str: Vec<&str> = events_str.split(';').map(|s| s.trim()).collect();
        let mut events: Vec<Event> = Vec::with_capacity(events_str.len());
        for event_str in events_str {
            events.push(Event::new(event_str));
        }
        Events { events }
    }
    pub(crate) fn empty() -> Self {
        Events { events: vec![] }
    }
    pub(crate) fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    pub(crate) fn check(&self, index: usize, event: &str) -> bool {
        match self.events.get(index) {
            Some(self_event) => return self_event.check(event),
            None => return true,
        }
    }

    pub(crate) fn check_input(&self, index: usize, event: &str) -> bool {
        match self.events.get(index) {
            Some(self_event) => return self_event.check_input(event),
            None => return true,
        }
    }

    pub(crate) fn check_all(&self, event: &str) -> bool {
        for event_ in self.events.iter() {
            if event_.check(event) {
                return true;
            }
        }
        return false;
    }

    pub(crate) fn len(&self) -> usize {
        self.events.len()
    }

}

/// An event has one input and one output.
/// The input could use one Inner Event such as a+b* to express
/// a lot of patterns
/// for the output several patterns are allowed seperated by ";"
#[derive(Debug, Clone)]
pub(crate) struct Event {
    input: Pattern,  //to server
    output: Pattern, // from server
}

impl Event {
    pub(crate) fn new(event_str: &str) -> Self {
        let split_result: Vec<&str> = event_str.split("/").collect();
        if split_result.len() != 2 {
            panic!("event parsing error {event_str}");
        }
        Self::from(split_result[0].trim(), split_result[1].trim())
    }

    pub(crate) fn empty() -> Self {
        Event {
            input: Pattern::empty(),
            output: Pattern::empty(),
        }
    }
    pub(crate) fn from(input: &str, output: &str) -> Event {
        Event {
            input: Pattern::from_str(input),
            output: Pattern::from_str(output),
        }
    }
    /// NEED TO ADD PARTIAL MATCH
    pub(crate) fn check(&self, event_str: &str) -> bool {
        //an event is written as emit/output to srv
        let event: Vec<&str> = event_str.split("/").map(|e| e.trim()).collect();
        let input = event[0];
        let output = event[1];
        if self.input.check(input) && self.output.check(output) {
            return true;
        } else {
            return false;
        }
    }


    pub(crate) fn check_input(&self, event_str: &str) -> bool {
        let event: Vec<&str> = event_str.split("/").map(|e| e.trim()).collect();
        let input = event[0];
        self.input.check(input)
    }
}
impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.input.to_string(), self.output.to_string())
    }
}
#[derive(Debug, Clone)]
enum InnerPattern {
    Positive(PositivePattern),
    Negative(NegativePattern),
    Negatives(NegativesPattern),
}

impl InnerPattern {
    fn check(&self, input: &str) -> bool {
        match &self {
            InnerPattern::Positive(pattern) => pattern.check(input),
            InnerPattern::Negative(pattern) => pattern.check(input),
            InnerPattern::Negatives(pattern) => pattern.check(input),
        }
    }
}
impl Display for InnerPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerPattern::Positive(s) => {
                write!(f, "{}", s)
            }
            InnerPattern::Negative(s) => {
                write!(f, "{}", s)
            }
            InnerPattern::Negatives(s) => {
                write!(f, "{}", s)
            }
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Pattern {
    elements: Vec<InnerPattern>,
}

impl Pattern {
    pub(crate) fn empty() -> Self {
        Self { elements: Vec::new() }
    }

    pub(crate) fn check(&self, input: &str) -> bool {
        for element in self.elements.iter() {
            if element.check(input) {
                return true;
            }
        }
        return false;
    }

    pub(crate) fn from_str(string: &str) -> Self
    where
        Self: Sized,
    {
        let line: Vec<&str> = string.split("+").map(|e| e.trim()).collect();
        let mut elements: Vec<InnerPattern> = Vec::new();
        for element in line {
            if element.starts_with("!(") {
                elements.push(InnerPattern::Negatives(NegativesPattern::from_str(element)));
            } else if element.starts_with("!") {
                elements.push(InnerPattern::Negative(NegativePattern::from_str(element)));
            } else {
                elements.push(InnerPattern::Positive(PositivePattern::from_str(element)));
            }
        }
        Self { elements }
    }
}
impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_string = String::new();
        for i in self.elements.iter() {
            output_string.push_str(&i.to_string());
        }
        write!(f, "{}", output_string)
    }
}
///Element of an input or output (transition)
#[derive(Debug, Clone)]
struct PositivePattern {
    inner_pattern: WildMatch,
}
impl PositivePattern {
    fn check(&self, input: &str) -> bool {
        self.inner_pattern.matches(input)
    }

    fn from_str(string: &str) -> Self
    where
        Self: Sized,
    {
        Self {
            inner_pattern: WildMatch::new(string.trim()),
        }
    }
}
impl Display for PositivePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner_pattern.to_string())
    }
}
#[derive(Debug, Clone)]
struct NegativePattern {
    inner_pattern: WildMatch,
}

impl NegativePattern {
    fn check(&self, input: &str) -> bool {
        !self.inner_pattern.matches(input)
    }

    fn from_str(string: &str) -> Self
    where
        Self: Sized,
    {
        let string = string.trim();
        let index = string.find("!").expect("failed to parse negative element pattern");
        Self {
            inner_pattern: WildMatch::new(&string[index + 1..]),
        }
    }
}
impl Display for NegativePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner_pattern.to_string())
    }
}
#[derive(Debug, Clone)]
struct NegativesPattern {
    inner_patterns: Vec<WildMatch>,
}

impl NegativesPattern {
    fn check(&self, input: &str) -> bool {
        let mut result: bool = true;
        for p in self.inner_patterns.iter() {
            if p.matches(input) {
                result = false;
            }
        }
        result
    }

    fn from_str(string: &str) -> Self
    where
        Self: Sized,
    {
        let string = string.trim();
        let index = string.find("!(").expect("failed to parse negative element pattern");
        let index_end = string.find(")").expect("failed to parse negativeS element pattern");
        let string = &string[index + 2..index_end];
        let line: Vec<&str> = string.split("#").map(|e| e.trim()).collect();
        let mut inner_patterns = Vec::new();
        for negative_pattern in line {
            inner_patterns.push(WildMatch::new(negative_pattern))
        }
        Self {
            inner_patterns: inner_patterns,
        }
    }
}
impl Display for NegativesPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_string = String::new();
        for i in self.inner_patterns.iter() {
            output_string.push_str(&i.to_string());
        }
        write!(f, "{}", output_string)
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use crate::verifier::event::{Event, NegativePattern, NegativesPattern, Pattern, PositivePattern};

    #[test]
    fn test_event_check() {
        let event = String::from("a/b");
        let event_to_test = Event::new(&event);
        assert!(event_to_test.check(&event));
        let event = String::from("c/d");
        assert!(!event_to_test.check(&event));
        let event = String::from("a/d");
        assert!(!event_to_test.check(&event));
        let event = String::from("c/b");
        assert!(!event_to_test.check(&event));

        let event_to_test = Event::new("input/output1+output2+oupsi*+output1?");
        assert!(event_to_test.check("input/output1"));
        assert!(event_to_test.check("input/output2"));
        assert!(event_to_test.check("input/oupsi i cant't remember the output"));
        assert!(event_to_test.check("input/ output10 "));
        assert!(!event_to_test.check("input/output20"));
    }

    #[test]
    fn test_inner_positive_pattern() {
        let pattern = "b";
        let respect_pattern = "b";
        let does_not_respect_pattern = "c";
        let positive = PositivePattern::from_str(pattern);
        assert!(positive.check(respect_pattern));
        assert!(!positive.check(does_not_respect_pattern));

        let pattern = "b?";
        let respect_pattern = "bc";
        let does_not_respect_pattern = "bcc";
        let positive = PositivePattern::from_str(pattern);
        assert!(positive.check(respect_pattern));
        assert!(!positive.check(does_not_respect_pattern));

        let pattern = "b*";
        let respect_pattern = "bsssc";
        let positive = PositivePattern::from_str(pattern);
        assert!(positive.check(respect_pattern));
    }

    #[test]
    fn test_inner_negative_pattern() {
        let pattern = "!b";
        let respect_pattern = "zui";
        let does_not_respect_pattern = "b";
        let negative = NegativePattern::from_str(pattern);
        assert!(negative.check(respect_pattern));
        assert!(!negative.check(does_not_respect_pattern));
    }

    #[test]
    fn test_inner_negatives_pattern() {
        let pattern = "!(b#a)";
        let respect_pattern = "zui";
        let does_not_respect_pattern_first = "b";
        let does_not_respect_pattern_second = "a";
        let negative = NegativesPattern::from_str(pattern);
        assert!(negative.check(respect_pattern));
        assert!(!negative.check(does_not_respect_pattern_first));
        assert!(!negative.check(does_not_respect_pattern_second));
    }

    #[test]
    fn test_inner_pattern() {
        let pattern = "b+c";
        let respect_pattern = "c";
        let does_not_respect_pattern = "z";
        let pattern = Pattern::from_str(pattern);
        assert!(pattern.check(respect_pattern));
        let respect_pattern = "b";
        assert!(pattern.check(respect_pattern));
        assert!(!pattern.check(does_not_respect_pattern));

        let pattern = "!b+c";
        let respect_pattern = "sdlbsdvmklsdnklsdvnkl";
        let pattern = Pattern::from_str(pattern);
        assert!(pattern.check(respect_pattern));
    }
}
