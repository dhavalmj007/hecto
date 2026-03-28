use std::{fs::read_to_string, io::Error};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn load(filename: &str) -> Result<Self, Error> {
        let file_content = read_to_string(filename)?;
        let mut vec = Vec::new();
        for line in file_content.lines() {
            vec.push(line.to_string());
        }
        Ok(Self { lines: vec })
    }
}
