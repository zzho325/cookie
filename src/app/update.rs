use crossterm::event::KeyCode;
use tracing::warn;

use crate::app::model::{Command, Message, Model};

/// Updates model with message and optionally create next message for chained update and command
/// for side effect.
pub fn update(model: &mut Model, msg: Message) -> (Option<Message>, Option<Command>) {
    match msg {
        Message::Key(code) => return handle_key_code(model, code),
        Message::ServiceResp(a) => {
            if let Some(q) = model.pending_question.as_ref() {
                model.history_messages.push((q.clone(), a.clone()));
            } else {
                warn!("received answer while no question is pending")
            }
            model.pending_question = None;
        }
        Message::SendQuestion => {
            // only send response if not waiting
            // TODO: implement timeout for pending resp
            if model.pending_question.is_none() {
                let cmd = Command::ServiceReq(model.input_editor.input().to_string());
                // move input to pending
                model.pending_question = Some(model.input_editor.input().to_string());
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
            KeyCode::Char(c) => editor.input.push(c),
            KeyCode::Backspace => _ = editor.input.pop(),
            KeyCode::Esc => editor.is_editing = false,
            KeyCode::Enter => return (Some(Message::SendQuestion), None),
            _ => {}
        }
    } else {
        match code {
            KeyCode::Char('q') => model.should_quit = true,
            KeyCode::Char('i') => model.input_editor.is_editing = true,
            _ => {}
        }
    }
    (None, None)
}
