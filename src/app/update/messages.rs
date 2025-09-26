use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::app::Command;
use crate::app::model::Model;
use crate::app::{Message, update::Update};

pub fn handle_key_event(
    model: &mut Model,
    KeyEvent {
        code,
        modifiers,
        kind: _,
        state: _,
    }: KeyEvent,
) -> Update {
    let messages = &mut model.session.messages;
    match (code, modifiers) {
        (KeyCode::Char('q'), _) => model.quit(),
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => model.toggle_sidebar(),
        (KeyCode::Char('n'), _) => return (Some(Message::NewSession), None),
        (KeyCode::Tab, _) => model.shift_focus(),
        (KeyCode::Char('i'), _) => return (Some(Message::Editing), None),
        (KeyCode::Char('s'), _) => return (Some(Message::Setting), None),
        (KeyCode::Char('e'), _) => {
            return (
                None,
                Some(Command::ExternalEditingReadOnly(
                    messages.viewport.input().to_string(),
                )),
            );
        }
        (KeyCode::Char('v'), _) => messages.viewport.toggle_visual_selection(),
        (KeyCode::Char('y'), _) => {
            if let Some(selected) = messages.viewport.yank_visual_selection() {
                return (None, Some(Command::CopyToClipboard(selected)));
            }
        }
        (KeyCode::Esc, _) => messages.viewport.clear_visual_selection(),
        // KeyCode::Down => messages.scroll_down(),
        // KeyCode::Up => messages.scroll_up(),
        (KeyCode::Left | KeyCode::Char('h'), _) => messages.viewport.move_cursor_left(),
        (KeyCode::Right | KeyCode::Char('l'), _) => messages.viewport.move_cursor_right(),
        (KeyCode::Down | KeyCode::Char('j'), _) => messages.viewport.move_cursor_down(),
        (KeyCode::Up | KeyCode::Char('k'), _) => messages.viewport.move_cursor_up(),
        _ => {}
    }
    (None, None)
}

pub fn handle_mouse_event(
    model: &mut Model,
    MouseEvent {
        kind,
        column,
        row,
        modifiers,
    }: MouseEvent,
) -> Update {
    tracing::debug!("messages mouse event");
     (None, None)
}
