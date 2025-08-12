use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
            (KeyCode::Char(c), _) => editor.enter_char(c),
            (KeyCode::Backspace, _) => editor.delete_char(),
            (KeyCode::Left, _) => editor.move_cursor_left(),
            (KeyCode::Right, _) => editor.move_cursor_right(),
            (KeyCode::Esc, _) => editor.set_is_editing(false),
            (KeyCode::Enter, KeyModifiers::ALT) => editor.enter_char('\n'),
            (KeyCode::Enter, _) => return (Some(Message::Send), None),
            (KeyCode::Down, _) => editor.move_cursor_down(),
            (KeyCode::Up, _) => editor.move_cursor_up(),
            _ => {}
        }
    } else {
        match code {
            KeyCode::Char('q') => model.quit(),
            KeyCode::Char('e') => model.toggle_sidebar(),
            KeyCode::Char('n') => return (Some(Message::NewChat), None),
            KeyCode::Tab => model.shift_focus(),
            KeyCode::Char('i') => return (Some(Message::Editing), None),
            KeyCode::Char('s') => return (Some(Message::Setting), None),
            KeyCode::Enter => return (Some(Message::Send), None),
            KeyCode::Left | KeyCode::Char('h') => editor.move_cursor_left(),
            KeyCode::Right | KeyCode::Char('l') => editor.move_cursor_right(),
            KeyCode::Down | KeyCode::Char('j') => editor.move_cursor_down(),
            KeyCode::Up | KeyCode::Char('k') => editor.move_cursor_up(),
            _ => {}
        }
    }
    (None, None)
}
