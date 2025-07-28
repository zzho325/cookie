use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::model::Model;
use crate::app::{Message, update::Update};

pub fn handle_session_key_event(
    model: &mut Model,
    KeyEvent {
        code,
        modifiers,
        kind: _,
        state: _,
    }: KeyEvent,
) -> Update {
    let session = &mut model.session;
    if session.is_editing {
        let editor = &mut session.input_editor;
        match (code, modifiers) {
            (KeyCode::Char(c), _) => editor.enter_char(c),
            (KeyCode::Backspace, _) => editor.delete_char(),
            (KeyCode::Left, _) => editor.move_cursor_left(),
            (KeyCode::Right, _) => editor.move_cursor_right(),
            (KeyCode::Esc, _) => session.is_editing = false,
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
            KeyCode::Tab => {
                if model.show_sidebar {
                    model.shift_focus()
                }
            }
            KeyCode::Char('i') => return (Some(Message::Editing), None),
            KeyCode::Down => session.messages.scroll_down(),
            KeyCode::Up => session.messages.scroll_up(),
            // KeyCode::Right => {
            //     // for now for test
            //     let llm_settings = LlmSettings::OpenAI {
            //         model: OpenAImodel::Gpt4oMini,
            //         web_search: false,
            //     };
            //     tracing::debug!("send setting update");
            //     let req = crate::models::ServiceReq::UpdateSettings(llm_settings);
            //     let cmd = Command::ServiceReq(req);
            //     return (None, Some(cmd));
            // }
            _ => {}
        }
    }
    (None, None)
}
