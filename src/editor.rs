mod terminal;
mod view;

use crate::editor::terminal::Terminal;
use crate::editor::terminal::{Position, Size};
use crate::editor::view::View;
use crossterm::event::KeyCode::{Down, End, Home, Left, PageDown, PageUp, Right, Up};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::cmp::min;
use std::io::Error;

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
pub struct Editor {
    should_quit: bool,
    location: Location,
    view: View,
    _terminal: Terminal,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            should_quit: false,
            location: Location::default(),
            view: View::default(),
            _terminal: Terminal::new()?,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.handle_args();
        let result = self.repl();
        result?;
        Ok(())
    }

    fn handle_args(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        if let Some(filename) = args.get(1) {
            self.view.load(filename);
        }
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }

            let event = read()?;
            self.evaluate_event(event)?;
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                (KeyCode::Char(c), _) => {
                    let Location { x, y } = self.location;
                    self.view.insert(self.location, c);
                    let new_x = x + 1;
                    let new_y = y;
                    Terminal::move_caret_to(Position {
                        col: new_x,
                        row: new_y,
                    })?;
                    self.location = Location { x: new_x, y: new_y };
                    self.view.set_redraw(true);
                    self.view.render()?;
                }
                (Up | Down | Right | Left | Home | End | PageDown | PageUp, _) => {
                    self.move_caret(code)?;
                }
                (KeyCode::Enter, _) => {
                    self.location.y += 1;
                    self.location.x = 0;
                    self.view.insert(self.location, '\r');
                    self.view.set_redraw(true);
                    self.view.render()?;
                }
                (KeyCode::Backspace, _) => {
                    if self.location.x == 0 && self.location.y > 0 {
                        self.location.y = self.location.y.saturating_sub(1);
                        self.location.x = self.view.line_len(self.location.y).unwrap_or(0);
                    } else {
                        self.location.x = self.location.x.saturating_sub(1);
                    }
                    self.view.delete(self.location);
                    self.view.set_redraw(true);
                    self.view.render()?;
                }
                _ => {}
            },
            Event::Resize(width_u16, height_u16) => {
                #[allow(clippy::as_conversions)]
                let height = height_u16 as usize;

                #[allow(clippy::as_conversions)]
                let width = width_u16 as usize;

                self.view.resize(Size { height, width });
            }
            _ => {}
        }
        Terminal::execute()?;
        Ok(())
    }

    fn move_caret(&mut self, code: KeyCode) -> Result<(), Error> {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size()?;
        match code {
            Up => y = y.saturating_sub(1),
            Down => y = min(height.saturating_sub(1), y.saturating_add(1)),
            Right => x = min(width.saturating_sub(1), x.saturating_add(1)),
            Left => x = x.saturating_sub(1),
            Home => x = 0,
            End => x = width.saturating_sub(1),
            PageUp => y = 0,
            PageDown => y = height.saturating_sub(1),
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
        Terminal::move_caret_to(Position::default())?;
        if self.should_quit {
            // Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            self.view.render()?;
            let position = Position::new(self.location.x, self.location.y);
            Terminal::move_caret_to(position)?;
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
}
