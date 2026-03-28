mod buffer;
use crate::editor::terminal::{Position, Size, Terminal};
use buffer::Buffer;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
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
            if let Some(line) = self.buffer.lines.get(current_row) {
                let truncated_line = if line.len() >= width {
                    &line[0..width]
                } else {
                    line
                };
                Self::render_line(current_row, truncated_line)?;
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                Self::render_line(current_row, "~")?;
            }

            // if self.buffer.is_empty() {
            //     Self::render_welcome_screen()?;
            // } else {
            //     self.render_buffer()?;
            // }
        }
        self.needs_redraw = false;
        Ok(())
    }

    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
        Terminal::move_caret_to(Position { col: at, row: 0 })?;
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
        self.needs_redraw = true;
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
        }
    }
}
