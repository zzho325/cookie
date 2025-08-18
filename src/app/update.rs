mod input_editor;
mod messages;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{
        Command, Message,
        model::{Model, focus::Focused, setting_manager::SettingManager},
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
            if let Some(user_message) = model.session.handle_sending_user_message() {
                model.selected_session_id = Some(user_message.session_id.clone());
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
            model.shift_focus_to(Focused::InputEditor);
            model.session.input_editor.set_is_editing(true);
        }
        Message::Setting => match &mut model.setting_manager_popup {
            None => {
                model.setting_manager_popup =
                    Some(SettingManager::new(model.session.llm_settings()))
            }
            Some(setting_manager) => {
                model
                    .session
                    .set_llm_settings(setting_manager.llm_settings());
                model.setting_manager_popup = None;
            }
        },
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
        ServiceResp::ChatEvent(chat_event) => model.session.handle_chat_event(chat_event),
        ServiceResp::Sessions(session_summaries) => model
            .session_manager
            .handle_session_summaries(session_summaries, model.selected_session_id.clone()),
        ServiceResp::Session(session) => {
            if model.selected_session_id.as_ref() == Some(&session.id) {
                model.session.handle_session(session);
            }
        }
        ServiceResp::SessionSummary(session_summary) => {
            if model.selected_session_id.as_ref() == Some(&session_summary.id) {
                model
                    .session
                    .handle_session_summary(session_summary.clone());
            }
            model
                .session_manager
                .handle_session_summary(session_summary);
        }
        ServiceResp::Error(msg) => model.error_message = Some(msg),
    }
    (None, None)
}

fn handle_key_event(model: &mut Model, keyevent: KeyEvent) -> Update {
    // Quit on any key if there is an error message.
    if model.error_message.is_some() {
        model.quit()
    }

    if let Some(setting_manager) = &mut model.setting_manager_popup {
        match keyevent.code {
            KeyCode::Down | KeyCode::Char('j') => {
                setting_manager.select_next();
                return (None, None);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                setting_manager.select_previous();
                return (None, None);
            }
            KeyCode::Esc => model.setting_manager_popup = None,
            KeyCode::Enter => return (Some(Message::Setting), None),
            _ => {}
        }
        return (None, None);
    }

    match model.focused {
        Focused::InputEditor => {
            return input_editor::handle_key_event(model, keyevent);
        }
        Focused::Messages => {
            return messages::handle_key_event(model, keyevent);
        }
        Focused::SessionManager => match keyevent.code {
            KeyCode::Char('q') => model.quit(),
            KeyCode::Char('e') => model.toggle_sidebar(),
            KeyCode::Char('n') => return (Some(Message::NewChat), None),
            KeyCode::Char('i') => return (Some(Message::Editing), None),
            KeyCode::Char('s') => return (Some(Message::Setting), None),
            KeyCode::Down | KeyCode::Char('j') => {
                model.selected_session_id = model.session_manager.select_next();
                let maybe_msg = model.selected_session_id.clone().map(Message::GetSession);
                return (maybe_msg, None);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                model.selected_session_id = model.session_manager.select_previous();
                let maybe_msg = model.selected_session_id.clone().map(Message::GetSession);
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
    use rstest::{fixture, rstest};
    use tracing::{
        level_filters::LevelFilter,
        subscriber::{self, DefaultGuard},
    };
    use tracing_subscriber::fmt::{format::FmtSpan, time::Uptime};

    use crate::{
        app::{
            model::{Model, focus::Focused},
            update::{self, handle_key_event},
        },
        models::configs::Config,
    };

    #[fixture]
    fn with_tracing() -> DefaultGuard {
        let subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            .with_timer(Uptime::default())
            .with_max_level(LevelFilter::TRACE)
            .with_span_events(FmtSpan::ENTER)
            .finish();
        subscriber::set_default(subscriber)
    }

    #[rstest]
    fn navigation(_with_tracing: DefaultGuard) {
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
                description: "sidebar hidden, key tab navigates from editor to messages",
                focused: Focused::InputEditor,
                show_sidebar: false,
                is_editing: false,
                key_event: KeyCode::Tab.into(),
                expected_focused: Focused::Messages,
                expected_show_sidebar: false,
                expected_is_editing: false,
            },
            Case {
                description: "sidebar hidden, key tab navigates from messages to editor",
                focused: Focused::Messages,
                show_sidebar: false,
                is_editing: false,
                key_event: KeyCode::Tab.into(),
                expected_focused: Focused::InputEditor,
                expected_show_sidebar: false,
                expected_is_editing: false,
            },
            Case {
                description: "sidebar open, key tab navigates from session manager to messages",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Tab.into(),
                expected_focused: Focused::Messages,
                expected_show_sidebar: true,
                expected_is_editing: false,
            },
            Case {
                description: "sidebar open, key tab navigates from input editor to session manager",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Tab.into(),
                expected_focused: Focused::Messages,
                expected_show_sidebar: true,
                expected_is_editing: false,
            },
            Case {
                description: "key e opens and navigates to sidebar",
                focused: Focused::InputEditor,
                show_sidebar: false,
                is_editing: false,
                key_event: KeyCode::Char('e').into(),
                expected_focused: Focused::SessionManager,
                expected_show_sidebar: true,
                expected_is_editing: false,
            },
            Case {
                description: "key e closes sidebar",
                focused: Focused::Messages,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Char('e').into(),
                expected_focused: Focused::Messages,
                expected_show_sidebar: false,
                expected_is_editing: false,
            },
            Case {
                description: "key e closes and navigates away from sidebar",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Char('e').into(),
                expected_focused: Focused::InputEditor,
                expected_show_sidebar: false,
                expected_is_editing: false,
            },
            Case {
                description: "key i navigates from session manager to session and enter editing",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyCode::Char('i').into(),
                expected_focused: Focused::InputEditor,
                expected_show_sidebar: true,
                expected_is_editing: true,
            },
        ];

        for case in cases {
            let mut model = Model::new(Config::default());
            model.session.input_editor.set_is_editing(case.is_editing);
            if case.show_sidebar {
                model.toggle_sidebar();
            }
            model.shift_focus_to(case.focused);
            let (maybe_msg, _) = handle_key_event(&mut model, case.key_event);
            if let Some(msg) = maybe_msg {
                update::update(&mut model, msg);
            }
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
                model.session.input_editor.is_editing(),
                case.expected_is_editing,
                "{} is_editing",
                case.description
            );
        }
    }
}
