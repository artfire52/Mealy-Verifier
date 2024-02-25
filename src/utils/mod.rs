use std::{collections::HashSet, hash::Hash};

use indexmap::IndexSet;

pub(crate) mod output;
pub(crate) mod reader;
pub(crate) fn unquote(s: &str) -> &str {
    let start: usize;
    let end: usize;
    let mut index = s.find("\"");
    match index {
        Some(s) => start = s + 1,
        None => return s,
    }
    index = s[start..].find("\"");
    match index {
        Some(s) => end = s + start,
        None => return s,
    }
    &s[start..end]
}

pub(crate) fn _compare_index_set_and_set<T: Eq + Hash>(index_set: &IndexSet<T>, set: &HashSet<T>) -> bool {
    index_set.len() == set.len() && index_set.iter().all(|value| set.contains(value))
}

#[cfg(test)]
mod test {
    use super::unquote;
    #[test]
    fn test_unquoting() {
        let quoted = "\"o\"";
        let unquoted = "o";
        assert_eq!(unquoted, unquote(quoted));
        assert_eq!(unquoted, unquote(unquoted));
    }
}
