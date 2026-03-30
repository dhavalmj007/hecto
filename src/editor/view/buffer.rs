use std::{fs::read_to_string, io::Error};

use crate::editor::Location;

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

    pub fn delete_backward(&mut self, location: Location) -> () {
        match self.lines.get_mut(location.y) {
            Some(line) => {
                if location.x < line.len() {
                    line.remove(location.x);
                }
            }
            None => {}
        }
    }

    pub fn insert(&mut self, location: Location, c: char) {
        match self.lines.get_mut(location.y) {
            Some(line) => {
                debug_assert!(location.x <= line.len(), "Location is out of bounds");
                line.insert(location.x, c);
            }
            None => {
                self.lines.push(String::new());
                self.lines.last_mut().unwrap().insert(location.x, c);
            }
        }
    }

    pub fn line_len(&self, at: usize) -> Option<usize> {
        self.lines.get(at).map(|line| line.trim().len())
    }
}
