use std::{fs::read_to_string, io::Error};

use crate::editor::view::Location;

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

    pub fn delete_backward(&mut self, location: Location) {
        if let Some(line) = self.lines.get_mut(location.y)
            && location.x < line.len()
        {
            line.remove(location.x);
        }
    }

    pub fn insert(&mut self, location: Location, c: char) {
        if let Some(line) = self.lines.get_mut(location.y) {
            debug_assert!(location.x <= line.len(), "Location is out of bounds");
            line.insert(location.x, c);
        } else {
            self.lines.push(String::new());
            self.lines.last_mut().unwrap().insert(location.x, c);
        }
    }

    pub fn line_len(&self, at: usize) -> Option<usize> {
        self.lines.get(at).map(|line| line.len())
    }

    pub fn join_line(&mut self, at: usize) {
        if at + 1 >= self.lines.len() {
            return;
        }

        let line = self.lines.remove(at + 1);
        self.lines[at].push_str(&line);
    }

    pub fn split_line(&mut self, Location { x, y }: Location) {
        if let Some(line) = self.lines.get_mut(y) {
            let reminder = line.split_off(x);
            self.lines.insert(y + 1, reminder);
        }
    }
}
