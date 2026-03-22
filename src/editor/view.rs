mod buffer;
use crate::editor::terminal::{Position, Size, Terminal};
use buffer::Buffer;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    pub needs_redraw: bool,
}

impl View {
    pub fn render_welcome_screen() -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            #[allow(clippy::integer_division)]
            if current_row == height / 3 {
                Self::draw_welcome_msg()?;
            } else {
                Self::draw_empty_row()?;
            }
            Terminal::move_caret_to(Position::new(current_row, 0))?;
            // if current_row.saturating_add(1) < height {
            //     Terminal::print("\r\n")?;
            // }
        }
        Ok(())
    }

    fn render_buffer(&self) -> Result<(), Error> {
        let Size { height, width } = Terminal::size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            if let Some(line) = self.buffer.lines.get(current_row) {
                let mut line = line.clone();
                line.truncate(width);
                Terminal::print(&line)?;
                // Terminal::print("\r\n")?;
                Terminal::move_caret_to(Position::new(current_row, 0))?;
                continue;
            }
            Self::draw_empty_row()?;
            // if current_row.saturating_add(1) < height {
            //     Terminal::print("\r\n")?;
            // }
        }
        Ok(())
    }

    pub fn render(&self) -> Result<(), Error> {
        if self.buffer.is_empty() {
            Self::render_welcome_screen()?;
        } else {
            self.render_buffer()?;
        }
        Ok(())
    }

    fn draw_welcome_msg() -> Result<(), Error> {
        let mut welcome_msg = format!("{NAME} editor -- version {VERSION}");
        let width = Terminal::size()?.width;
        let len = welcome_msg.len();
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len)) / 2;
        let space = " ".repeat(padding.saturating_sub(1)); // account for the ~
        welcome_msg = format!("~{space}{welcome_msg}");
        welcome_msg.truncate(width);
        Terminal::print(&welcome_msg)
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
    }
}
