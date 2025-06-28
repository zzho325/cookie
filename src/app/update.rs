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
                model.history.push((q.clone(), a.clone()));
            } else {
                warn!("received answer while no question is pending")
            }
            model.pending_question = None;
        }
        Message::SendQuestion => {
            // only send response if not waiting
            // TODO: implement timeout for pending resp
            if model.pending_question.is_none() {
                let cmd = Command::ServiceReq(model.input.clone());
                // move input to pending
                model.pending_question = Some(model.input.clone());
                model.input.clear();
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
    if model.is_editing {
        match code {
            KeyCode::Char(c) => model.input.push(c),
            KeyCode::Backspace => _ = model.input.pop(),
            KeyCode::Esc => model.is_editing = false,
            KeyCode::Enter => return (Some(Message::SendQuestion), None),
            _ => {}
        }
    } else {
        match code {
            KeyCode::Char('q') => model.should_quit = true,
            KeyCode::Char('i') => model.is_editing = true,
            _ => {}
        }
    }
    (None, None)
}
