use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use std::io::{stdout, Error, Write};
use std::panic::take_hook;

#[derive(Debug, Copy, Clone, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    pub fn new(col: usize, row: usize) -> Self {
        Position { col, row }
    }
}

/// Represents the Terminal.
/// Edge Case for platforms where `usize` < `u16`:
/// Regardless of the actual size of the Terminal, this representation
/// only spans over at most `usize::MAX` or `u16::size` rows/columns, whichever is smaller.
/// Each size returned truncates to min(`usize::MAX`, `u16::MAX`)
/// And should you attempt to set the cursor out of these bounds, it will also be truncated.
pub struct Terminal {}

#[derive(Default, Copy, Clone)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

impl Terminal {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            Self::terminate().unwrap();
            current_hook(panic_info);
        }));
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        Ok(Self {})
    }

    pub fn terminate() -> Result<(), Error> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), Error> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    /// Returns the current size of this Terminal.
    /// Edge Case for systems with `usize` < `u16`:
    /// * A `Size` representing the terminal size. Any coordinate `z` truncated to `usize` if `usize` < `z` < `u16`
    pub fn size() -> Result<Size, Error> {
        let (width, height) = size()?;
        #[allow(clippy::as_conversions)]
        Ok(Size {
            height: height as usize,
            width: width as usize,
        })
    }

    /// Moves the cursor to the given Position.
    /// # Arguments
    /// * `Position` - the `Position` to move the cursor to. Will be truncated to `u16::MAX` if bigger.
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        queue!(stdout(), MoveTo(position.col as u16, position.row as u16))?;
        Ok(())
    }

    pub fn hide_caret() -> Result<(), Error> {
        queue!(stdout(), Hide)
    }

    pub fn show_caret() -> Result<(), Error> {
        queue!(stdout(), Show)
    }

    pub fn print(s: &str) -> Result<(), Error> {
        queue!(stdout(), Print(s))
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = Self::terminate();
    }
}
