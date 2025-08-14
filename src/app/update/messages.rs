use crossterm::event::{KeyCode, KeyEvent};

use crate::app::model::Model;
use crate::app::{Message, update::Update};

pub fn handle_key_event(
    model: &mut Model,
    KeyEvent {
        code,
        modifiers: _,
        kind: _,
        state: _,
    }: KeyEvent,
) -> Update {
    let messages = &mut model.session.messages;
    match code {
        KeyCode::Char('q') => model.quit(),
        KeyCode::Char('e') => model.toggle_sidebar(),
        KeyCode::Char('n') => return (Some(Message::NewChat), None),
        KeyCode::Tab => model.shift_focus(),
        KeyCode::Char('i') => return (Some(Message::Editing), None),
        KeyCode::Char('s') => return (Some(Message::Setting), None),
        KeyCode::Down => messages.scroll_down(),
        KeyCode::Up => messages.scroll_up(),
        _ => {}
    }
    (None, None)
}
