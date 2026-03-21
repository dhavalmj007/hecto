mod terminal;

use std::cmp::min;
use terminal::Terminal;

use crate::editor::terminal::{Position, Size};
use crossterm::event::Event::Key;
use crossterm::event::KeyCode::{Char, Down, End, Home, Left, PageDown, PageUp, Right, Up};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::queue;
use crossterm::style::Print;
use std::io::{stdout, Error};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Tracking location of caret on the document
#[derive(Copy, Clone, Default)]
struct Location {
    x: usize,
    y: usize,
}

impl Location {
    fn new(x: usize, y: usize) -> Self {
        Location { x, y }
    }
}

/// Tracking position of caret on terminal
#[derive(Default)]
pub struct Editor {
    should_quite: bool,
    location: Location,
}

impl Editor {
    pub fn run(&mut self) -> Result<(), Error> {
        Terminal::initialize()?;
        let result = self.repl();
        Terminal::terminate()?;
        result?;
        Ok(())
    }

    fn repl(&mut self) -> Result<(), Error> {
        self.refresh_screen()?;
        loop {
            if self.should_quite {
                break;
            }

            let event = read()?;
            self.evaluate_event(&event)?;
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        if let Key(KeyEvent {
            code,
            modifiers,
            state: _state,
            kind: KeyEventKind::Press,
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quite = true;
                }
                Char(c) => {
                    let mut stdout = stdout();
                    queue!(stdout, Print(format!("{c}")))?;
                }
                Up | Down | Right | Left | Home | End | PageDown | PageUp => {
                    self.move_caret(*code)?;
                }
                _ => (),
            }
        }
        Terminal::execute()?;
        Ok(())
    }

    fn move_caret(&mut self, code: KeyCode) -> Result<(), Error> {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size()?;
        let cur_y = self.location.y;
        let cur_x = self.location.x;
        match code {
            Up => x = cur_x.saturating_sub(1),
            Down => x = min(height.saturating_sub(1), cur_x.saturating_add(1)),
            Right => y = min(width.saturating_sub(1), cur_y.saturating_add(1)),
            Left => y = cur_y.saturating_sub(1),
            Home => y = 0,
            End => y = width.saturating_sub(1),
            PageUp => x = 0,
            PageDown => x = height.saturating_sub(1),
            _ => (),
        }
        let location = Location::new(x, y);
        let position = Position::new(x, y);
        self.location = location;
        Terminal::move_caret_to(position)?;
        Terminal::execute()?;

        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        if self.should_quite {
            Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            Self::draw_rows()?;
            let position = Position::default();
            Terminal::move_caret_to(position)?;
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
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
        Terminal::print(welcome_msg)
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")
    }

    fn draw_rows() -> Result<(), std::io::Error> {
        let Size { height, .. } = Terminal::size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            #[allow(clippy::integer_division)]
            if current_row == height / 3 {
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
}
