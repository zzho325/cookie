mod session;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{
        Command, Message,
        model::{Model, focus::Focused},
    },
    models::{ServiceReq, ServiceResp},
};

pub type Update = (Option<Message>, Option<Command>);

/// Updates model with message and optionally creates next message for chained update and command
/// for side effect.
pub fn update(model: &mut Model, msg: Message) -> Update {
    match msg {
        Message::Key(evt) => return handle_key_event(model, evt),
        Message::ServiceResp(resp) => {
            return handle_service_resp(model, resp);
        }
        // TODO: add a unit test for sending message and create session.
        Message::Send => {
            if let Some(user_message) = model.session.handle_send() {
                model.selected_session_id = Some(user_message.session_id());
                return (
                    None,
                    Some(Command::ServiceReq(ServiceReq::ChatMessage(
                        user_message.clone(),
                    ))),
                );
            }
        }
        Message::NewChat => {
            model.new_draft_chat();
        }
        Message::Editing => {
            model.shift_focus_to(Focused::Session);
            model.session.is_editing = true;
        }
        Message::GetSession(id) => {
            return (None, Some(Command::ServiceReq(ServiceReq::GetSession(id))));
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
            .handle_session_summaries(session_summaries, model.selected_session_id),
        ServiceResp::Session(session) => {
            if model.selected_session_id == Some(session.id) {
                model.session.handle_session(session);
            }
        }
        ServiceResp::SessionSummary(session_summary) => {
            if model.selected_session_id == Some(session_summary.id) {
                model
                    .session
                    .handle_session_summary(session_summary.clone());
            }
            model
                .session_manager
                .handle_session_summary(session_summary);
        }
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
            KeyCode::Char('n') => return (Some(Message::NewChat), None),
            KeyCode::Char('i') => return (Some(Message::Editing), None),
            KeyCode::Down | KeyCode::Char('j') => {
                model.selected_session_id = model.session_manager.select_next();
                let maybe_msg = model.selected_session_id.map(Message::GetSession);
                return (maybe_msg, None);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                model.selected_session_id = model.session_manager.select_previous();
                let maybe_msg = model.selected_session_id.map(Message::GetSession);
                return (maybe_msg, None);
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
            model::{Model, focus::Focused},
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
            expected_is_editing: bool,
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
                expected_is_editing: false,
            },
            Case {
                description: "key s opens and navigates to sidebar",
                focused: Focused::Session,
                show_sidebar: false,
                is_editing: false,
                key_event: KeyCode::Char('s').into(),
                expected_focused: Focused::SessionManager,
                expected_show_sidebar: true,
                expected_is_editing: false,
            },
            Case {
                description: "key s closes and navigates away from sidebar",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Char('s').into(),
                expected_focused: Focused::Session,
                expected_show_sidebar: false,
                expected_is_editing: false,
            },
            Case {
                description: "Tab navigates from session manager to session",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Tab.into(),
                expected_focused: Focused::Session,
                expected_show_sidebar: true,
                expected_is_editing: false,
            },
            Case {
                description: "key i navigates from session manager to session and enter editing",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Char('i').into(),
                expected_focused: Focused::Session,
                expected_show_sidebar: true,
                expected_is_editing: true,
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
            assert_eq!(
                model.session.is_editing, case.expected_is_editing,
                "{} is_editing",
                case.description
            );
        }
    }
}
