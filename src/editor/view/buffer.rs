use std::{
    fs::read_to_string,
    io::Error,
    ops::{Index, Range},
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::editor::view::Location;

#[derive(Debug)]
enum CharWidth {
    Half,
    Full,
}

#[derive(Debug)]
struct GraphemesEntry {
    grapheme: String,
    replacement: Option<char>,
    rendered_width: CharWidth,
    byte_start: usize,
    byte_end: usize,
}

#[derive(Debug)]
pub(crate) struct Line {
    text: String,
    graphemes: Vec<GraphemesEntry>,
}

impl Line {
    fn new(s: &str) -> Self {
        Line {
            text: s.to_string(),
            graphemes: Self::create_graphemes(s, 0),
        }
    }

    fn create_graphemes(s: &str, byte_offset: usize) -> Vec<GraphemesEntry> {
        s.grapheme_indices(true)
            .map(|(start, g)| {
                let replacement = Self::replacement_char(g);
                GraphemesEntry {
                    grapheme: g.to_string(),
                    replacement,
                    rendered_width: if replacement.is_some() {
                        CharWidth::Half
                    } else {
                        match g.width() {
                            2 => CharWidth::Full,
                            _ => CharWidth::Half, // Default to half for unexpected widths
                        }
                    },
                    byte_start: start + byte_offset,
                    byte_end: start + byte_offset + g.len(),
                }
            })
            .collect()
    }

    fn replacement_char(g: &str) -> Option<char> {
        match g {
            " " => None,
            "\t" => Some(' '),
            _ if g.width() > 0 && g.trim().is_empty() => Some('␣'),
            _ if {
                let mut chars = g.chars();
                if let Some(ch) = chars.next() {
                    ch.is_control() && chars.next().is_none()
                } else {
                    false
                }
            } =>
            {
                Some('▯')
            }
            _ if g.width() == 0 => Some('·'),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.graphemes.len()
    }

    pub fn col_of(&self, grapheme_index: usize) -> usize {
        self.graphemes[..grapheme_index].iter().fold(0, |acc, g| {
            let width = match g.rendered_width {
                CharWidth::Half => 1,
                CharWidth::Full => 2,
            };

            width + acc
        })
    }

    fn insert(&mut self, at: usize, ch: char) {
        let byte_pos = self
            .graphemes
            .get(at)
            .map(|g| g.byte_start)
            .unwrap_or(self.text.len());
        self.text.insert(byte_pos, ch);
        let new_graphemes = Self::create_graphemes(&self.text[byte_pos..], byte_pos);
        self.graphemes.splice(at.., new_graphemes);
    }

    fn remove(&mut self, at: usize) {
        let GraphemesEntry {
            grapheme: _,
            replacement: _,
            rendered_width: _,
            byte_start,
            byte_end,
        } = self.graphemes[at];

        self.text.drain(byte_start..byte_end);
        let new_graphemes = Self::create_graphemes(&self.text[byte_start..], byte_start);
        self.graphemes.splice(at.., new_graphemes);
    }

    fn push_str(&mut self, line: &str) {
        let len = self.text.len();

        self.text.push_str(&line);

        let new_graphemes = Self::create_graphemes(&line, len);
        self.graphemes.extend(new_graphemes);
    }

    fn split_off(&mut self, at: usize) -> Line {
        if let Some(grapheme) = self.graphemes.get(at) {
            let remainder = self.text.split_off(grapheme.byte_start);
            self.graphemes.truncate(at);
            Line::new(&remainder)
        } else {
            Line::new("")
        }
    }

    pub fn get_visible_fragment(&self, range: Range<usize>) -> String {
        if range.is_empty() || self.graphemes.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        let mut current_col = 0;

        for entry in &self.graphemes {
            let width = match entry.rendered_width {
                CharWidth::Half => 1,
                CharWidth::Full => 2,
            };
            if current_col + width <= range.start {
                current_col += width;
                continue;
            }

            if current_col >= range.end {
                break;
            }

            if current_col < range.start {
                result.push(' ');
            } else if current_col + width > range.end {
                result.push(' ');
            } else {
                if let Some(ch) = entry.replacement {
                    result.push(ch);
                } else {
                    result.push_str(&entry.grapheme);
                }
            }

            current_col += width;
        }

        result
    }
}

impl Index<Range<usize>> for Line {
    type Output = str;
    fn index(&self, range: Range<usize>) -> &str {
        if range.is_empty() || self.graphemes.is_empty() {
            return "";
        }

        let grapheme_start = self
            .graphemes
            .iter()
            .scan(0, |col, g| {
                *col += match g.rendered_width {
                    CharWidth::Half => 1,
                    CharWidth::Full => 2,
                };
                Some(*col)
            })
            .position(|cum_col| cum_col > range.start)
            .unwrap_or(0);

        let grapheme_end = self
            .graphemes
            .iter()
            .scan(0, |col, g| {
                *col += match g.rendered_width {
                    CharWidth::Half => 1,
                    CharWidth::Full => 2,
                };
                Some(*col)
            })
            .position(|cum_col| cum_col >= range.end)
            .unwrap_or(self.len().saturating_sub(1));

        let byte_start = self.graphemes[grapheme_start].byte_start;
        let byte_end = self.graphemes[grapheme_end].byte_end;

        &self.text[byte_start..byte_end]
    }
}

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub fn load(filename: &str) -> Result<Self, Error> {
        let file_content = read_to_string(filename)?;
        let mut vec = Vec::new();
        for line in file_content.lines() {
            vec.push(Line::new(line));
        }
        Ok(Self { lines: vec })
    }

    pub fn delete(&mut self, location: Location) {
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
            self.lines.push(Line::new(""));
            self.lines.last_mut().unwrap().insert(location.x, c);
        }
    }

    pub fn line_len(&self, at: usize) -> Option<usize> {
        self.lines.get(at).map(|line| line.len())
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn join_line(&mut self, at: usize) {
        if at + 1 >= self.lines.len() {
            return;
        }

        let line = self.lines.remove(at + 1);
        self.lines[at].push_str(&line.text);
    }

    pub fn split_line(&mut self, Location { x, y }: Location) {
        if let Some(line) = self.lines.get_mut(y) {
            let remainder = line.split_off(x);
            self.lines.insert(y + 1, remainder);
        }
    }

    pub fn col_of(&self, location: Location) -> usize {
        self.lines
            .get(location.y)
            .map(|line| line.col_of(location.x))
            .unwrap_or(0)
    }
}
