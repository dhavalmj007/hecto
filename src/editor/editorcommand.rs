use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::editor::terminal::Size;

pub enum Direction {
    Up,
    Down,
    Right,
    Left,
    Home,
    End,
    PageUp,
    PageDown,
}

pub enum EditorCommand {
    Move(Direction),
    Quit,
    Insert(char),
    Tab,
    Backspace,
    Delete,
    Enter,
    Resize(Size),
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(EditorCommand::Quit),
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    Ok(EditorCommand::Insert(c))
                }
                (KeyCode::Tab, _) => Ok(EditorCommand::Tab),
                (KeyCode::Up, _) => Ok(EditorCommand::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(EditorCommand::Move(Direction::Down)),
                (KeyCode::Right, _) => Ok(EditorCommand::Move(Direction::Right)),
                (KeyCode::Left, _) => Ok(EditorCommand::Move(Direction::Left)),
                (KeyCode::Home, _) => Ok(EditorCommand::Move(Direction::Home)),
                (KeyCode::End, _) => Ok(EditorCommand::Move(Direction::End)),
                (KeyCode::PageUp, _) => Ok(EditorCommand::Move(Direction::PageUp)),
                (KeyCode::PageDown, _) => Ok(EditorCommand::Move(Direction::PageDown)),
                (KeyCode::Enter, _) => Ok(EditorCommand::Enter),
                (KeyCode::Backspace, _) => Ok(EditorCommand::Backspace),
                (KeyCode::Delete, _) => Ok(EditorCommand::Delete),
                _ => Err(format!("Event not supported -> {:?}", code)),
            },
            Event::Resize(width_u16, height_u16) => {
                #[allow(clippy::as_conversions)]
                let height = height_u16 as usize;

                #[allow(clippy::as_conversions)]
                let width = width_u16 as usize;
                Ok(EditorCommand::Resize(Size { height, width }))
            }
            _ => Err("Event not supported".to_string()),
        }
    }
}
