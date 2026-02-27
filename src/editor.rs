use crossterm::event::read;
use crossterm::event::Event::Key;
use crossterm::event::KeyCode::Char;
use crossterm::terminal::enable_raw_mode;

pub struct Editor {}

impl Editor {
    pub fn default() -> Self {
        Editor {}
    }

    pub fn run(&self) {
        enable_raw_mode().unwrap();
        loop {
            match read() {
                Ok(Key(event)) => {
                    println!("{event:?} \r");

                    if let Char(c) = event.code
                        && c == 'q' {
                            break;
                        }
                }

                Err(err) => println!("Error: {err}"),
                _ => (),
            }
        }
    }
}
