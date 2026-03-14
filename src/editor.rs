mod terminal;

use terminal::Terminal;

use crate::editor::terminal::{Position, Size};
use crossterm::event::Event::Key;
use crossterm::event::KeyCode::Char;
use crossterm::event::{read, Event, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{execute, queue};
use crossterm::style::Print;
use std::io::{stdout, Error, Write};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quite: bool,
    position: Position,
}

impl Editor {
    pub const fn default() -> Self {
        Self {
            should_quite: false,
            position: Position { x: 0, y: 0 },
        }
    }

    pub fn update_position(&mut self, x: usize, y: usize) {
        self.position = Position { x, y };
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
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
                    let position = Position {
                        x: self.position.x + 1,
                        y: self.position.y,
                    };
                    // Terminal::move_cursor_to(position)?;
                    stdout.flush()?;
                    self.update_position(position.x + 1, position.y);
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor()?;
        if self.should_quite {
            Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            Self::draw_rows()?;
            let position = Position { x: 0, y: 0 };
            Terminal::move_cursor_to(position)?;
            self.position = position;
        }
        Terminal::show_cursor()?;
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
