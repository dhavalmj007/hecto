use crate::editor::terminal::{Size, Terminal};
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View;

impl View {
    pub fn render() -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            #[allow(clippy::integer_division)]
            if current_row == 0 {
                Terminal::print("Hello World!")?;
            } else if current_row == height / 3 {
                Self::draw_welcome_msg()?;
            } else {
                Self::draw_empty_row()?;
            }
            if current_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
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
}
