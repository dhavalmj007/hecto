mod terminal;
mod view;

use crate::editor::terminal::Terminal;
use crate::editor::terminal::Size;
use crate::editor::view::View;
use crossterm::event::KeyCode::{Down, End, Home, Left, PageDown, PageUp, Right, Up};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use std::io::Error;

pub struct Editor {
    should_quit: bool,
    view: View,
    _terminal: Terminal,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            should_quit: false,
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

    #[allow(clippy::needless_pass_by_value)]
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
                (KeyCode::Char(c), _) => self.view.handle_char_insert(c)?,
                (Up | Down | Right | Left | Home | End | PageDown | PageUp, _) => {
                    self.view.move_caret(code)?
                }
                (KeyCode::Enter, _) => self.view.handle_enter()?,
                (KeyCode::Backspace, _) => self.view.handle_backspace()?,
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

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        // Terminal::move_caret_to(Position::default())?;
        if self.should_quit {
            // Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            self.view.render()?;
            let position = self.view.screen_position();
            Terminal::move_caret_to(position)?;
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
    }
}
