mod terminal;
mod view;

use std::cmp::min;
use terminal::Terminal;

use crate::editor::terminal::{Position, Size};
use crate::editor::view::View;
use crossterm::event::Event::Key;
use crossterm::event::KeyCode::{Char, Down, End, Home, Left, PageDown, PageUp, Right, Up};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::queue;
use crossterm::style::Print;
use std::io::{stdout, Error};

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
    view: View,
}

impl Editor {
    pub fn run(&mut self) -> Result<(), Error> {
        self.view.needs_redraw = true;
        Terminal::initialize()?;
        self.handle_args();
        let result = self.repl();
        Terminal::terminate()?;
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

        if let Some((..)) = event.as_resize_event() {
            self.view.needs_redraw = true;
            self.view.render()?;
            let position = Position::default();
            Terminal::move_caret_to(position)?;
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
            if self.view.needs_redraw {
                self.view.render()?;
                let position = Position::default();
                Terminal::move_caret_to(position)?;
                self.view.needs_redraw = false;
            }
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
}
