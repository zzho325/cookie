use crossterm::event::{KeyCode, KeyEvent};

use crate::app::Command;
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
        KeyCode::Char('n') => return (Some(Message::NewSession), None),
        KeyCode::Tab => model.shift_focus(),
        KeyCode::Char('i') => return (Some(Message::Editing), None),
        KeyCode::Char('s') => return (Some(Message::Setting), None),
        KeyCode::Char('v') => messages.viewport.toggle_visual_selection(),
        KeyCode::Char('y') => {
            if let Some(selected) = messages.viewport.yank_visual_selection() {
                return (None, Some(Command::CopyToClipboard(selected)));
            }
        }
        // KeyCode::Down => messages.scroll_down(),
        // KeyCode::Up => messages.scroll_up(),
        KeyCode::Left | KeyCode::Char('h') => messages.viewport.move_cursor_left(),
        KeyCode::Right | KeyCode::Char('l') => messages.viewport.move_cursor_right(),
        KeyCode::Down | KeyCode::Char('j') => messages.viewport.move_cursor_down(),
        KeyCode::Up | KeyCode::Char('k') => messages.viewport.move_cursor_up(),
        _ => {}
    }
    (None, None)
}
