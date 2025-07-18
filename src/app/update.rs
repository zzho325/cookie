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
            KeyCode::Char('n') => {
                model.session.reset(model.configs.derive_llm_settings());
                model.shift_focus_to(Focused::Session);
            }
            KeyCode::Tab => model.shift_focus(),
            _ => {}
        },
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent};

    use crate::{
        app::{
            model::{Focused, Model},
            update::handle_key_event,
        },
        models::configs::Configs,
    };

    #[test]
    fn navigation() {
        struct Case {
            description: &'static str,
            focused: Focused,
            show_sidebar: bool,
            is_editing: bool,
            key_event: KeyEvent,
            expected_focused: Focused,
            expected_show_sidebar: bool,
        }

        let cases = vec![
            Case {
                description: "key tab does not navigate focus when sidebar is hidden",
                focused: Focused::Session,
                show_sidebar: false,
                is_editing: false,
                key_event: KeyCode::Tab.into(),
                expected_focused: Focused::Session,
                expected_show_sidebar: false,
            },
            Case {
                description: "key s opens and navigates to sidebar",
                focused: Focused::Session,
                show_sidebar: false,
                is_editing: false,
                key_event: KeyCode::Char('s').into(),
                expected_focused: Focused::SessionManager,
                expected_show_sidebar: true,
            },
            Case {
                description: "key s closes and navigates away from sidebar",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Char('s').into(),
                expected_focused: Focused::Session,
                expected_show_sidebar: false,
            },
            Case {
                description: "Tab navigates from session manager to session",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Tab.into(),
                expected_focused: Focused::Session,
                expected_show_sidebar: true,
            },
        ];

        for case in cases {
            let mut model = Model::new(Configs::default());
            model.session.is_editing = case.is_editing;
            model.shift_focus_to(case.focused);
            model.show_sidebar = case.show_sidebar;
            handle_key_event(&mut model, case.key_event);
            assert_eq!(
                model.focused, case.expected_focused,
                "{} focused",
                case.description
            );
            assert_eq!(
                model.show_sidebar, case.expected_show_sidebar,
                "{} show_sidebar",
                case.description
            );
        }
    }
}
