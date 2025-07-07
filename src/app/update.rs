use crossterm::event::KeyCode;
use tracing::warn;

use crate::{
    app::model::{Command, Message, Model, scroll::Scrollable as _},
    service::models::ServiceResp,
};

/// Updates model with message and optionally create next message for chained update and command
/// for side effect.
pub fn update(model: &mut Model, msg: Message) -> (Option<Message>, Option<Command>) {
    match msg {
        Message::Key(code) => return handle_key_code(model, code),
        Message::ServiceResp(resp) => {
            if let ServiceResp::ChatMessage(a) = resp {
                model.messages.append_message(a);
            } else {
                // TODO: show refusal
                warn!("received refusal")
            }
        }
        Message::Send => {
            // only send response if not waiting
            // TODO: implement timeout for pending resp
            if !model.messages.is_pending_resp() {
                let q = model.input_editor.input().to_string();
                let cmd = Command::SendMessage(q);
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

fn handle_key_code(model: &mut Model, code: KeyCode) -> (Option<Message>, Option<Command>) {
    if model.input_editor.is_editing {
        let editor = &mut model.input_editor;
        match code {
            KeyCode::Char(c) => editor.enter_char(c),
            KeyCode::Backspace => _ = editor.input.pop(),
            KeyCode::Esc => editor.is_editing = false,
            KeyCode::Enter => return (Some(Message::Send), None),
            _ => {}
        }
    } else {
        match code {
            KeyCode::Char('q') => model.should_quit = true,
            KeyCode::Char('i') => model.input_editor.is_editing = true,
            KeyCode::Down => model.messages.scroll_down(),
            KeyCode::Up => model.messages.scroll_up(),
            _ => {}
        }
    }
    (None, None)
}
