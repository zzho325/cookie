pub mod editor;
pub mod focus;
pub mod messages;
pub mod session;
pub mod session_manager;

use crate::{
    app::model::{
        focus::{Focusable, Focused},
        session::Session,
        session_manager::SessionManager,
    },
    models::configs::Configs,
};

pub struct Model {
    pub configs: Configs,
    /// Current session.
    pub session: Session,
    pub session_manager: SessionManager,
    /// Source of truth for selected session id.
    /// It might not be in session_summaries for newly created session.
    /// It might not be the same as id in session while waiting for fetching selected session.
    pub selected_session_id: Option<uuid::Uuid>,

    pub show_sidebar: bool,
    pub focused: Focused,
    focus_order: Vec<fn(&mut Model) -> &mut dyn Focusable>,
    pub should_quit: bool,
}

impl Model {
    pub fn new(configs: Configs) -> Self {
        // FIXME: fix config usage
        let default_llm_settings = configs.derive_llm_settings();

        let mut this = Self {
            configs,
            session: Session::new(default_llm_settings),
            session_manager: SessionManager::default(),
            selected_session_id: None,
            show_sidebar: false,
            should_quit: false,
            focused: Focused::Session,
            focus_order: Vec::new(),
        };

        this.focus_order.push(|m| &mut m.session);
        this.focus_order.push(|m| &mut m.session_manager);
        this.focus_order[0](&mut this).set_focus(true);
        this
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_sidebar(&mut self) {
        if !self.show_sidebar {
            self.shift_focus_to(Focused::SessionManager);
        } else {
            self.shift_focus_to(Focused::Session);
        }
        self.show_sidebar = !self.show_sidebar;
    }

    /// Opens an new empty chat and enables editing.
    pub fn new_draft_chat(&mut self) {
        self.session.reset(self.configs.derive_llm_settings());
        self.selected_session_id = None;
        self.session_manager.set_selected(None);
        self.shift_focus_to(Focused::Session);
        self.session.is_editing = true;
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        app::model::{Model, focus::Focused, messages::Messages},
        models::{ChatMessage, Role, SessionSummary},
    };

    #[test]
    fn new_draft_chat() {
        // ----------------------------------------------------------------
        // Setup model.
        // ----------------------------------------------------------------
        let llm_settings = crate::models::LlmSettings::OpenAI {
            model: crate::service::client::api::OpenAIModel::Gpt4o,
            web_search: false,
        };
        let session_id = Uuid::new_v4();
        let title = "Awesome chat".to_string();

        let mut messages = Messages::default();
        messages.send_question(
            ChatMessage::new(
                session_id,
                Role::User,
                llm_settings.clone(),
                "history question".to_string(),
            ),
            llm_settings.clone(),
        );
        messages.receive_response(ChatMessage::new(
            session_id,
            Role::Assistant,
            llm_settings.clone(),
            "history reponse".to_string(),
        ));
        messages.send_question(
            ChatMessage::new(
                session_id,
                Role::User,
                llm_settings.clone(),
                "pending question".to_string(),
            ),
            llm_settings.clone(),
        );
        messages.scroll_down();

        let mut model = Model::new(crate::models::configs::Configs::default());
        model.session.set_title(Some(title.clone()));
        model.session.set_messages(messages);
        model.session.is_editing = false;
        *model.session.input_editor.input_mut() = "repeat this".repeat(3);
        model.selected_session_id = Some(session_id);

        model.session_manager.handle_session_summaries(
            vec![SessionSummary {
                id: session_id,
                title,
                updated_at: chrono::Utc::now(),
            }],
            model.selected_session_id,
        );

        // ----------------------------------------------------------------
        // Verify new chat behavior.
        // ----------------------------------------------------------------
        model.new_draft_chat();

        // Session history, pending messages and messages scroll srate are reset.
        assert_eq!(
            model.session.messages.chat_messages().len(),
            0,
            "history messages are reset"
        );
        assert!(
            !model.session.messages.is_pending_resp(),
            "pending messages are reset"
        );
        assert_eq!(
            model.session.messages.scroll_state().scroll_offset(),
            (0u16, 0u16),
            "messages scroll state is reset"
        );

        // Session manager reset.
        assert!(
            model.selected_session_id.is_none(),
            "model selected session id is reset"
        );
        assert!(
            model.session_manager.list_state().selected().is_none(),
            "session manager selected is reset"
        );

        // Input editor remains.
        assert!(
            !model.session.input_editor.input().is_empty(),
            "input remains"
        );

        // Focus on editor and in editing mode.
        assert_eq!(
            model.focused,
            Focused::Session,
            "focus is shifted to session"
        );
        assert!(model.session.is_editing, "editing is enabled")
    }
}
