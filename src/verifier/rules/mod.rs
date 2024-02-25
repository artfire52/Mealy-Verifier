mod conditional;
mod expected_event_index;
mod expected_event_sequence;
mod output;
mod restricted_events;
mod sink;
mod unexpected_cycle;
#[cfg(test)]
use std::any::Any;
use std::path::PathBuf;
use core::fmt::Debug;

use crate::{
    graph::prelude::*,
    utils::reader::{Reader, ReaderFile},
};

use self::{
    conditional::Conditional,
    expected_event_index::ExpectedTransitionIndex,
    expected_event_sequence::ExpectedTransitionSequence,
    output::Output,
    restricted_events::RestrictedEvents,
    sink::{SinkDescription, SinkTarget},
    unexpected_cycle::UnexpectedCycle,
};

pub(crate) trait Rule: Debug {
    ///A mathod to read the rule from a reader. The name is given.
    fn from_reader(reader: &mut dyn Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized;

    ///Get the name of the rule
    fn get_name(&self) -> &str;

    ///Apply the rule to obtain the output subgrpah
    /// The rules has to write the files within apply
    /// the output folder is the path to the folder where the output of the rules has to be.
    fn apply(&mut self, graph: &Graph, output_folder: &mut PathBuf);
    // fn as_any(&self) -> &dyn Any ;
    // fn inner_apply(&mut self,graph:&Graph,cycles: &HashMap<NodeId, Vec<Cycle>>)->;
    #[cfg(test)]
    fn as_any(&self) -> &dyn Any;
}

//Rule parsing
fn get_name(line: &str) -> String {
    let el: Vec<&str> = line.split(":").collect();
    match el.get(1) {
        Some(name) => {
            if name.trim().len() == 0 {
                panic!("Rules need a name")
            }
            name.trim().to_string()
        }
        None => panic!("Rules need a name"),
    }
}

pub(crate) fn parse_file(path_to_file: &str) -> Vec<Box<dyn Rule>> {
    let mut reader = match ReaderFile::open(path_to_file) {
        Ok(reader) => reader,
        Err(e) => {
            panic!("Error while reading rule file '{}' : {}", path_to_file, e.to_string())
        }
    };
    let mut ret: Vec<Box<dyn Rule>> = Vec::new();
    while let Some(line) = reader.read_line() {
        if line.starts_with("UC:") {
            let name = get_name(line);
            match UnexpectedCycle::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("SD:") {
            let name = get_name(line);
            match SinkDescription::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("ST:") {
            let name = get_name(line);
            match SinkTarget::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("CT:") {
            let name = get_name(line);
            match Conditional::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("ETS:") {
            let name = get_name(line);
            match ExpectedTransitionSequence::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("ETI:") {
            let name = get_name(line);
            match ExpectedTransitionIndex::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("OR:") {
            let name = get_name(line);
            match Output::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("RE:") {
            let name = get_name(line);
            match RestrictedEvents::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        }
    }
    ret
}

#[cfg(test)]
pub(crate) fn parse_rule_from_str(rules_str: &str) -> Vec<Box<dyn Rule>> {
    use crate::utils::reader::test_reader::TestReader;

    let mut reader = match TestReader::from_text(rules_str) {
        Ok(r) => r,
        Err(_) => panic!("unable to create the test reader to parse rule"),
    };
    let mut ret: Vec<Box<dyn Rule>> = Vec::new();
    while let Some(line) = reader.read_line() {
        if line.starts_with("UC:") {
            let name = get_name(line);
            match UnexpectedCycle::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("SD:") {
            let name = get_name(line);
            match SinkDescription::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("ST:") {
            let name = get_name(line);
            match SinkTarget::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("CT:") {
            let name = get_name(line);
            match Conditional::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("ETS:") {
            let name = get_name(line);
            match ExpectedTransitionSequence::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("ETI:") {
            let name = get_name(line);
            match ExpectedTransitionIndex::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("OR:") {
            let name = get_name(line);
            match Output::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        } else if line.starts_with("RE:") {
            let name = get_name(line);
            match RestrictedEvents::from_reader(&mut reader, name) {
                Ok(rule) => ret.push(Box::new(rule)),
                Err(e) => {
                    panic!("failed to parse rule file:{}", e.to_string())
                }
            }
        }
    }

    ret
}
