//https://stackoverflow.com/questions/45882329/read-large-files-line-by-line-in-rust
use std::{
    fs::File,
    io::{self, prelude::*},
};

pub(crate) trait Reader {
    fn read_line(&mut self) -> Option<&mut String>;
}
pub struct ReaderFile {
    reader: io::BufReader<File>,
    buffer: String,
}
impl ReaderFile {
    pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let buffer = String::with_capacity(1024);
        Ok(Self { reader, buffer })
    }
}
impl Reader for ReaderFile {
    fn read_line(&mut self) -> Option<&mut String> {
        self.buffer.clear();

        let res = self.reader.read_line(&mut self.buffer);
        match res {
            Ok(0) | Err(_) => {
                return None;
            }
            Ok(_) => Some(&mut self.buffer),
        }
    }
}
#[cfg(test)]
pub(crate) mod test_reader {
    use std::io::{self};

    use super::Reader;

    pub struct TestReader {
        lines: Vec<String>,
        index: usize,
    }

    impl TestReader {
        pub fn from_text(text: &str) -> io::Result<Self> {
            let lines: Vec<&str> = text.trim().split("\n").collect();
            Ok(Self {
                lines: lines.iter().map(|e| e.trim().to_string()).collect(),
                index: 0,
            })
        }
        pub fn read_line(&mut self) -> Option<&mut String> {
            if self.index >= self.lines.len() {
                return None;
            }
            let to_return = &mut self.lines[self.index];
            self.index += 1;
            return Some(to_return);
        }
    }

    impl Reader for TestReader {
        fn read_line(&mut self) -> Option<&mut String> {
            if self.index >= self.lines.len() {
                return None;
            }
            let to_return = &mut self.lines[self.index];
            self.index += 1;
            return Some(to_return);
        }
    }
}
