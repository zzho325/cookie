mod input_editor;
mod messages;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

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
        Message::CrosstermClose => {
            model.should_quit = true;
        }
        Message::MouseEvent(evt) => return handle_mouse_event(model, evt),
        Message::ServiceResp(resp) => {
            return handle_service_resp(model, resp);
        }

        /* ----- model wide activities ----- */
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
        Message::NewSession => {
            model.new_draft_chat();
        }
        Message::DeleteSession => {
            if let Some(session_id) = &model.selected_session_id {
                return (
                    Some(Message::SelectNextSession),
                    Some(Command::ServiceReq(ServiceReq::DeleteSession(
                        session_id.to_string(),
                    ))),
                );
            }
        }
        Message::SelectNextSession => {
            let maybe_cmd = model
                .handle_select_next_session()
                .map(|id| Command::ServiceReq(ServiceReq::GetSession(id)));
            return (None, maybe_cmd);
        }
        Message::SelectPrevSession => {
            let maybe_cmd = model
                .handle_select_prev_session()
                .map(|id| Command::ServiceReq(ServiceReq::GetSession(id)));
            return (None, maybe_cmd);
        }

        /* ----- editor activities ----- */
        Message::Paste(data) => {
            if model.focused == Focused::InputEditor {
                model.session.input_editor.paste_data(&data);
            }
        }
        Message::ExternalEditingComplete(data) => {
            model.session.input_editor.handle_editting_in_editor(data);
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

fn handle_key_event(model: &mut Model, evt: KeyEvent) -> Update {
    // Quit on any key if there is an error message.
    if model.error_message.is_some() {
        model.quit()
    }

    if let Some(setting_manager) = &mut model.setting_manager_popup {
        match evt.code {
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
            return input_editor::handle_key_event(model, evt);
        }
        Focused::Messages => {
            return messages::handle_key_event(model, evt);
        }
        Focused::SessionManager => match (evt.code, evt.modifiers) {
            (KeyCode::Char('q'), _) => model.quit(),
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => model.toggle_sidebar(),
            (KeyCode::Char('n'), _) => return (Some(Message::NewSession), None),
            (KeyCode::Char('d'), _) => return (Some(Message::DeleteSession), None),
            (KeyCode::Char('i'), _) => return (Some(Message::Editing), None),
            (KeyCode::Char('s'), _) => return (Some(Message::Setting), None),
            (KeyCode::Down | KeyCode::Char('j'), _) => {
                return (Some(Message::SelectNextSession), None);
            }
            (KeyCode::Up | KeyCode::Char('k'), _) => {
                return (Some(Message::SelectPrevSession), None);
            }
            (KeyCode::Tab, _) => model.shift_focus(),
            _ => {}
        },
    }
    (None, None)
}

fn handle_mouse_event(model: &mut Model, evt: MouseEvent) -> Update {
    tracing::debug!(?evt);
    // TODO: handle popups
    tracing::debug!(session_manager=?model.session_manager.area());
    tracing::debug!(input_editor_area=?model.session.input_editor.viewport.area());
    tracing::debug!(messages_area=?model.session.messages.viewport.area());

    if model.show_sidebar
        && let Some(evt) = model.session_manager.area().maybe_mouse_event(evt)
    {
        tracing::debug!("session_manager mouse event");
    } else if let Some(evt) = model
        .session
        .messages
        .viewport
        .area()
        .maybe_mouse_event(evt)
    {
        return messages::handle_mouse_event(model, evt);
    } else if let Some(evt) = model
        .session
        .input_editor
        .viewport
        .area()
        .maybe_mouse_event(evt)
    {
        return input_editor::handle_mouse_event(model, evt);
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
                description: "key CTRL e opens and navigates to sidebar",
                focused: Focused::InputEditor,
                show_sidebar: false,
                is_editing: false,
                key_event: KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
                expected_focused: Focused::SessionManager,
                expected_show_sidebar: true,
                expected_is_editing: false,
            },
            Case {
                description: "key CTRL e closes sidebar",
                focused: Focused::Messages,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
                expected_focused: Focused::Messages,
                expected_show_sidebar: false,
                expected_is_editing: false,
            },
            Case {
                description: "key CTRL e closes and navigates away from sidebar",
                focused: Focused::SessionManager,
                show_sidebar: true,
                is_editing: false,
                key_event: KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
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
