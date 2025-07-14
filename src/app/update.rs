use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::model::{Command, Message, Model, scroll::Scrollable as _},
    models::{LlmSettings, ServiceReq, ServiceResp, configs::OpenAIConfig},
    service::client::api::OpenAIModel,
};

/// Updates model with message and optionally create next message for chained update and command
/// for side effect.
pub fn update(model: &mut Model, msg: Message) -> (Option<Message>, Option<Command>) {
    match msg {
        Message::Key(evt) => return handle_key_event(model, evt),
        Message::ServiceResp(resp) => {
            if let ServiceResp::ChatMessage(a) = resp {
                model.messages.append_message(a);
            } else {
                // TODO: show refusal
                tracing::warn!("received refusal")
            }
        }
        Message::Send => {
            // only send response if not waiting
            // TODO: implement timeout for pending resp
            if !model.messages.is_pending_resp() {
                let q = model.input_editor.input().to_string();
                let req = crate::models::ServiceReq::ChatMessage(q);
                let cmd = Command::ServiceReq(req);
                // move input to pending
                model.messages.send_question(model.input_editor.input());
                model.input_editor.clear();
                // send cmd
                return (None, Some(cmd));
            }
        }
        Message::CrosstermClose => {
            model.should_quit = true;
        }
    }
    (None, None)
}

fn handle_key_event(
    model: &mut Model,
    KeyEvent {
        code,
        modifiers,
        kind: _,
        state: _,
    }: KeyEvent,
) -> (Option<Message>, Option<Command>) {
    if model.input_editor.is_editing {
        let editor = &mut model.input_editor;
        match (code, modifiers) {
            (KeyCode::Char(c), _) => editor.enter_char(c),
            (KeyCode::Backspace, _) => editor.delete_char(),
            (KeyCode::Left, _) => editor.move_cursor_left(),
            (KeyCode::Right, _) => editor.move_cursor_right(),
            (KeyCode::Esc, _) => editor.is_editing = false,
            (KeyCode::Enter, KeyModifiers::ALT) => editor.enter_char('\n'),
            (KeyCode::Enter, _) => return (Some(Message::Send), None),
            (KeyCode::Down, _) => editor.move_cursor_down(),
            (KeyCode::Up, _) => editor.move_cursor_up(),
            _ => {}
        }
    } else {
        match code {
            KeyCode::Char('q') => model.should_quit = true,
            KeyCode::Char('i') => model.input_editor.is_editing = true,
            KeyCode::Down => model.messages.scroll_down(),
            KeyCode::Up => model.messages.scroll_up(),
            KeyCode::Right => {
                // for now for test
                let llm_settings = LlmSettings::OpenAI {
                    model: OpenAIModel::Gpt4oMini,
                    web_search: false,
                };
                tracing::debug!("send setting update");
                let req = crate::models::ServiceReq::UpdateSettings(llm_settings);
                let cmd = Command::ServiceReq(req);
                return (None, Some(cmd));
            }
            _ => {}
        }
    }
    (None, None)
}
