use uuid::Uuid;

use crate::{
    app::model::{
        editor::{Editor, WrapMode},
        messages::Messages,
    },
    chat::*,
    llm::*,
};

pub struct Session {
    /// Session Id of current session. None for new session before sending first message.
    pub session_id: Option<String>,
    llm_settings: LlmSettings,
    pub messages: Messages,
    pub input_editor: Editor,
}

impl Session {
    pub fn new(llm_settings: LlmSettings) -> Self {
        Self {
            session_id: None,
            llm_settings,
            messages: Messages::default(),
            input_editor: Editor::new(String::new(), WrapMode::default()),
        }
    }

    pub fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    pub fn set_messages(&mut self, messages: Messages) {
        self.messages = messages;
    }

    pub fn llm_settings(&self) -> LlmSettings {
        self.llm_settings
    }

    pub fn set_llm_settings(&mut self, llm_settings: LlmSettings) {
        self.llm_settings = llm_settings;
    }

    /// Clears everything except for editor input with given settings.
    pub fn reset(&mut self, settings: LlmSettings) {
        self.session_id = None;
        self.llm_settings = settings;

        self.messages.reset();
    }

    // ----------------------------------------------------------------
    // Event handlers.
    // ----------------------------------------------------------------

    /// If not already pending response, and input editor is not empty, sends user message to
    /// service, create session_id if this is a draft chat, i.e., session_id not populated.
    /// Returns the user message.
    pub fn handle_sending_user_message(&mut self) -> Option<ChatEvent> {
        // only send response if no response is pending or in progress
        // TODO: implement timeout for pending resp
        if self.messages.is_pending() {
            return None;
        }
        let msg = self.input_editor.input().to_string();
        // early return if input is empty.
        if msg.is_empty() {
            return None;
        }

        let msg_ = msg.clone();
        let session_id = if let Some(id) = &self.session_id {
            id.clone()
        } else {
            let id = Uuid::new_v4().to_string();
            self.session_id = Some(id.clone());
            id
        };

        let payload = chat_event::Payload::Message(crate::chat::Message {
            role: Role::User as i32,
            msg: msg_,
        });
        let user_message = ChatEvent::new(session_id, Some(self.llm_settings), payload);
        self.messages.handle_send();
        self.input_editor.clear();
        Some(user_message)
    }

    pub fn handle_chat_event(&mut self, chat_event: ChatEvent) {
        // assign session with session id
        match &self.session_id {
            Some(id) if id == &chat_event.session_id => {
                self.messages.handle_chat_event_stream(chat_event);
            }
            _ => {}
        }
    }

    /// Replaces current content with given session except for editor input.
    pub fn handle_session(&mut self, session: ChatSession) {
        self.session_id = Some(session.id);
        self.llm_settings = session.llm_settings.unwrap_or_default();

        self.messages.reset();
        self.messages.set_title(Some(session.title));
        self.messages.handle_chat_events(session.events);
    }

    /// Updates title if `session_summary` is for current session.
    pub fn handle_session_summary(&mut self, session_summary: ChatSession) {
        if let Some(session_id) = self.session_id.clone()
            && session_id == session_summary.id
        {
            self.messages.set_title(Some(session_summary.title))
        }
    }
}
