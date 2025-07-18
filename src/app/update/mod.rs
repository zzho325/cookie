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

fn handle_key_event(model: &mut Model, keyevent: KeyEvent) -> Update {
    match model.focused {
        Focused::Session => {
            return session::handle_session_key_event(model, keyevent);
        }
        Focused::SessionManager => match keyevent.code {
            KeyCode::Char('q') => model.quit(),
            KeyCode::Char('s') => model.toggle_sidebar(),
            KeyCode::Tab => model.shift_focus(),
            _ => {}
        },
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;

    use crate::{
        app::{
            model::{Focused, Model},
            update::handle_key_event,
        },
        models::configs::Configs,
    };

    #[test]
    fn navigation() {
        let mut model = Model::new(Configs::default());

        handle_key_event(&mut model, KeyCode::Tab.into());
        assert_eq!(
            model.focused,
            Focused::Session,
            "key tab does not navigate focus when sidebar is hidden"
        );

        handle_key_event(&mut model, KeyCode::Esc.into());
        assert!(!model.session.is_editing, "key Esc toggles editing");

        handle_key_event(&mut model, KeyCode::Char('s').into());
        assert!(model.show_sidebar, "key s toggles sidebar");
        assert_eq!(
            model.focused,
            Focused::SessionManager,
            "key s navigates to sidebar"
        );

        handle_key_event(&mut model, KeyCode::Tab.into());
        assert_eq!(model.focused, Focused::Session, "tab nagivates to session");
    }
}
