mod buffer;
use crate::editor::editorcommand::Direction::{
    self, Down, End, Home, Left, PageDown, PageUp, Right, Up,
};
use crate::editor::editorcommand::EditorCommand;
use crate::editor::terminal::{Position, Size, Terminal};
use buffer::Buffer;
use std::cmp::min;
use std::ops::DerefMut;
use std::{io::Error, ops::Deref};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Tracking location of caret on the document
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

impl Location {
    pub fn new(x: usize, y: usize) -> Self {
        Location { x, y }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
struct ScrollOffset(Location);

// impl ScrollOffset {
//     pub fn increment(&mut self, x: usize, y: usize) {
//         self.0.x = self.0.x.saturating_add(x);
//         self.0.y = self.0.y.saturating_add(y);
//     }
// }

impl Deref for ScrollOffset {
    type Target = Location;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScrollOffset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    pub location: Location,
    scroll_offset: ScrollOffset,
}

impl View {
    pub fn render(&mut self) -> Result<(), Error> {
        if !self.needs_redraw {
            return Ok(());
        }

        let Size { height, width } = self.size;
        if height == 0 || width == 0 {
            return Ok(());
        }

        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;

        for current_row in 0..height {
            let doc_row = current_row + self.scroll_offset.y;
            if let Some(line) = self.buffer.lines.get(doc_row) {
                let start = self.scroll_offset.x;
                let end = start + width;
                let visible_line = line.get_visible_fragment(start..end);
                Self::render_line(current_row, &visible_line)?;
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                Self::render_line(current_row, "~")?;
            }
        }
        self.needs_redraw = false;
        Ok(())
    }

    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
        Terminal::move_caret_to(Position { row: at, col: 0 })?;
        Terminal::clear_line()?;
        Terminal::print(line_text)?;
        Ok(())
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return " ".to_string();
        }

        let welcome_msg = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_msg.len();

        if width <= len {
            return "~".to_string();
        }

        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;
        let mut full_msg = format!("~{}{}", " ".repeat(padding), welcome_msg);
        full_msg.truncate(width);
        full_msg
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll_location_into_view();
        self.needs_redraw = true;
    }

    pub fn delete(&mut self, location: Location) {
        self.buffer.delete(location);
    }

    pub fn insert(&mut self, location: Location, c: char) {
        self.buffer.insert(location, c);
    }

    pub fn set_redraw(&mut self, redraw: bool) {
        self.needs_redraw = redraw;
    }

    pub fn line_len(&self, at: usize) -> Option<usize> {
        self.buffer.line_len(at)
    }

    // pub fn set_cursor_location(&mut self, location: Location) {
    //     unimplemented!()
    // }

    // pub fn caret_location(&self) -> Location {
    //     self.location
    // }

    pub fn screen_position(&self) -> Position {
        let col_x = self.buffer.col_of(self.location);
        let col = col_x.saturating_sub(self.scroll_offset.x);
        let row = self.location.y.saturating_sub(self.scroll_offset.y);
        Position::new(col, row)
    }

    pub fn move_caret(&mut self, code: Direction) -> Result<(), Error> {
        let Location { mut x, mut y } = self.location;
        let Size { height, .. } = self.size;
        match code {
            Up => y = y.saturating_sub(1),
            Down => y = y.saturating_add(1),
            Right => {
                let line_len = self.buffer.line_len(y).unwrap_or(0);
                if x >= line_len && y < self.buffer.height() {
                    x = 0;
                    y = y.saturating_add(1)
                } else if x < line_len {
                    x = x.saturating_add(1);
                }
            }
            Left => {
                if x == 0 && y > 0 {
                    x = self.buffer.line_len(y.saturating_sub(1)).unwrap_or(0);
                    y = y.saturating_sub(1);
                } else {
                    x = x.saturating_sub(1)
                }
            }
            Home => x = 0,
            End => x = self.buffer.line_len(y).unwrap_or(0),
            PageUp => y = y.saturating_sub(height.saturating_sub(1)),
            PageDown => y = y.saturating_add(height.saturating_sub(1)),
        }

        y = min(y, self.buffer.height());
        x = self.buffer.line_len(y).unwrap_or(0).min(x);
        let location = Location::new(x, y);
        self.location = location;

        self.scroll_location_into_view();
        Ok(())
    }

    pub fn handle_char_insert(&mut self, c: char) -> Result<(), Error> {
        let Location { x, y } = self.location;
        let before_line_len = self.buffer.line_len(y).unwrap_or(0);

        self.insert(self.location, c);
        let after_line_len = self.buffer.line_len(y).unwrap_or(0);
        let new_x = if after_line_len > before_line_len {
            x.saturating_add(1)
        } else {
            x
        };
        self.location = Location { x: new_x, y };
        self.scroll_location_into_view();
        self.set_redraw(true);

        Ok(())
    }

    pub fn handle_enter(&mut self) -> Result<(), Error> {
        self.buffer.split_line(self.location);
        self.move_caret(Right)?;
        self.scroll_location_into_view();
        self.set_redraw(true);
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), Error> {
        if self.location.x == 0 && self.location.y > 0 {
            self.location.x = self
                .line_len(self.location.y.saturating_sub(1))
                .unwrap_or(0);
            self.buffer.join_line(self.location.y.saturating_sub(1));
            self.location.y = self.location.y.saturating_sub(1);
        } else if self.location.x > 0 {
            self.location.x = self.location.x.saturating_sub(1);
            self.delete(self.location);
        }
        self.scroll_location_into_view();
        self.set_redraw(true);
        Ok(())
    }

    pub fn handle_delete(&mut self) -> Result<(), Error> {
        let Location { x, y } = self.location;
        let line_len = self.buffer.line_len(y).unwrap_or(0);
        if x == line_len && y < self.buffer.height() {
            self.buffer.join_line(y);
        } else if x < line_len {
            self.delete(self.location);
        }
        self.scroll_location_into_view();
        self.set_redraw(true);
        Ok(())
    }

    pub fn handle_command(&mut self, command: EditorCommand) -> Result<(), Error> {
        match command {
            EditorCommand::Insert(c) => self.handle_char_insert(c)?,
            EditorCommand::Tab => self.handle_char_insert('\t')?,
            EditorCommand::Move(direction) => self.move_caret(direction)?,
            EditorCommand::Enter => self.handle_enter()?,
            EditorCommand::Backspace => self.handle_backspace()?,
            EditorCommand::Delete => self.handle_delete()?,
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {
                unreachable!("Quit command should be handled by Editor, not View")
            }
        };

        Ok(())
    }

    fn scroll_location_into_view(&mut self) {
        let Size { height, width } = self.size;
        let old_scroll_offset = self.scroll_offset;
        let col_x = self.buffer.col_of(self.location);

        if col_x >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = col_x.saturating_sub(width).saturating_add(1);
        }

        if col_x < self.scroll_offset.x {
            self.scroll_offset.x = col_x;
        }

        if self.location.y >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = self.location.y.saturating_sub(height).saturating_add(1);
        }

        if self.location.y < self.scroll_offset.y {
            self.scroll_offset.y = self.location.y;
        }

        if old_scroll_offset != self.scroll_offset {
            self.needs_redraw = true;
        }
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            location: Location::default(),
            scroll_offset: ScrollOffset::default(),
        }
    }
}
