use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::model::{Command, Message, Model},
    models::ServiceResp,
};

pub type Update = (Option<Message>, Option<Command>);

/// Updates model with message and optionally create next message for chained update and command
/// for side effect.
pub fn update(model: &mut Model, msg: Message) -> Update {
    match msg {
        Message::Key(evt) => return handle_key_event(model, evt),
        Message::ServiceResp(resp) => {
            return handle_service_resp(model, resp);
        }
        Message::Send => {
            return model.session.handle_user_message();
        }
        Message::CrosstermClose => {
            model.should_quit = true;
        }
    }
    (None, None)
}

fn handle_service_resp(model: &mut Model, resp: ServiceResp) -> Update {
    match resp {
        ServiceResp::ChatMessage(assistant_message) => {
            model.session.handle_assistant_message(assistant_message)
        }
        _ => todo!(),
    }
}

// TODO: clean this up
fn handle_key_event(model: &mut Model, keyevent: KeyEvent) -> Update {
    if model.session.is_editing {
        return model.session.handle_key_event(keyevent);
    } else {
        match keyevent.code {
            KeyCode::Char('q') => model.should_quit = true,
            KeyCode::Char('i') => model.session.is_editing = true,
            _ => {
                return model.session.handle_key_event(keyevent);
            }
        }
    }
    (None, None)
}
