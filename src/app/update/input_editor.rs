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
    let session = &mut model.session;
    let editor = &mut session.input_editor;
    if editor.is_editing() {
        match (code, modifiers) {
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => model.toggle_sidebar(),
            (KeyCode::Char(c), _) => editor.enter_char(c),
            (KeyCode::Backspace, _) => editor.delete_char(),
            (KeyCode::Left, _) => editor.move_cursor_left(),
            (KeyCode::Right, _) => editor.move_cursor_right(),
            (KeyCode::Esc, _) => editor.set_is_editing(false),
            (KeyCode::Tab, _) => model.shift_focus(),
            (KeyCode::Enter, KeyModifiers::SHIFT) => editor.enter_char('\n'),
            (KeyCode::Enter, _) => return (Some(Message::Send), None),
            (KeyCode::Down, _) => editor.move_cursor_down(),
            (KeyCode::Up, _) => editor.move_cursor_up(),
            _ => {}
        }
    } else {
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
                    Some(Command::ExternalEditing(editor.input().to_string())),
                );
            }
            (KeyCode::Enter, _) => return (Some(Message::Send), None),
            (KeyCode::Left | KeyCode::Char('h'), _) => editor.move_cursor_left(),
            (KeyCode::Right | KeyCode::Char('l'), _) => editor.move_cursor_right(),
            (KeyCode::Down | KeyCode::Char('j'), _) => editor.move_cursor_down(),
            (KeyCode::Up | KeyCode::Char('k'), _) => editor.move_cursor_up(),
            _ => {}
        }
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
    tracing::debug!("input editor mouse event");
     (None, None)
}
