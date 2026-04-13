mod buffer;
use crate::editor::terminal::{Position, Size, Terminal};
use buffer::Buffer;
use crossterm::event::KeyCode;
use crossterm::event::KeyCode::{Down, End, Home, Left, PageDown, PageUp, Right, Up};
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

impl ScrollOffset {
    pub fn increment(&mut self, x: usize, y: usize) {
        self.0.x = self.0.x.saturating_add(x);
        self.0.y = self.0.y.saturating_add(y);
    }
}

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
                // let truncated_line = if line.len() >= width {
                //     &line[0..width]
                // } else {
                //     line
                // };
                let start = self.scroll_offset.x.min(line.len());
                let end = (start + width).min(line.len());
                let visible_line = &line[start..end];
                Self::render_line(current_row, visible_line)?;
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
        self.buffer.delete_backward(location);
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

    pub fn caret_location(&self) -> Location {
        self.location
    }

    pub fn screen_position(&self) -> Position {
        let col = self.location.x - self.scroll_offset.x;
        let row = self.location.y - self.scroll_offset.y;
        Position::new(col, row)
    }

    pub fn move_caret(&mut self, code: KeyCode) -> Result<(), Error> {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = self.size;
        match code {
            Up => y = y.saturating_sub(1),
            Down => y = y.saturating_add(1),
            Right => x = x.saturating_add(1),
            Left => x = x.saturating_sub(1),
            Home => x = 0,
            End => x = width.saturating_sub(1),
            PageUp => y = y.saturating_sub(height - 1),
            PageDown => y = y.saturating_add(height - 1),
            _ => (),
        }
        let location = Location::new(x, y);
        self.location = location;

        self.scroll_location_into_view();
        Ok(())
    }

    pub fn handle_char_insert(&mut self, c: char) -> Result<(), Error> {
        let Location { x, y } = self.location;
        self.insert(self.location, c);
        let new_x = x.saturating_add(1);
        let new_y = y;
        // Terminal::move_caret_to(Position {
        //     col: new_x,
        //     row: new_y,
        // })?;
        self.location = Location { x: new_x, y: new_y };
        self.scroll_location_into_view();
        self.set_redraw(true);
        self.render()?;

        Ok(())
    }

    pub fn handle_enter(&mut self) -> Result<(), Error> {
        self.location.y = self.location.y.saturating_add(1);
        self.location.x = 0;
        self.scroll_location_into_view();
        self.insert(self.location, '\r');
        self.set_redraw(true);
        self.render()?;
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), Error> {
        if self.location.x == 0 && self.location.y > 0 {
            self.location.y = self.location.y.saturating_sub(1);
            self.location.x = self.line_len(self.location.y).unwrap_or(0);
        } else {
            self.location.x = self.location.x.saturating_sub(1);
        }
        self.scroll_location_into_view();
        self.delete(self.location);
        self.set_redraw(true);
        self.render()?;
        Ok(())
    }

    fn scroll_location_into_view(&mut self) {
        let Size { height, width } = self.size;
        let old_scroll_offset = self.scroll_offset;

        if self.location.x >= self.scroll_offset.x + width {
            self.scroll_offset.x = self.location.x.saturating_sub(width) + 1
        }

        if self.location.x < self.scroll_offset.x {
            self.scroll_offset.x = self.location.x
        }

        if self.location.y >= self.scroll_offset.y + height {
            self.scroll_offset.y = self.location.y.saturating_sub(height) + 1
        }

        if self.location.y < self.scroll_offset.y {
            self.scroll_offset.y = self.location.y
        }

        if old_scroll_offset != self.scroll_offset {
            self.needs_redraw = true
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
