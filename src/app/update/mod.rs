mod session;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{
        Command, Message,
        model::{Focused, Model},
    },
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
            if let Some(req) = model.session.handle_user_message() {
                return (None, Some(Command::ServiceReq(req)));
            }
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

        ServiceResp::Sessions(session_summaries) => model
            .session_manager
            .handle_sessions_update(session_summaries),
        _ => todo!(),
    }
    (None, None)
}

// TODO: clean this up
fn handle_key_event(model: &mut Model, keyevent: KeyEvent) -> Update {
    match model.focused {
        Focused::Session => {
            return session::handle_session_key_event(model, keyevent);
        }
        Focused::SessionManager => match keyevent.code {
            KeyCode::Char('q') => model.should_quit = true,
            _ => {}
        },
    }

    (None, None)
}
