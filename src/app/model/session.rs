use uuid::Uuid;

use crate::{
    app::model::{
        editor::{Editor, WrapMode},
        messages::Messages,
    },
    models::{self, ChatMessage, LlmSettings, ServiceReq},
};

pub struct Session {
    pub session_id: Option<Uuid>,
    pub title: Option<String>,
    pub llm_settings: LlmSettings,
    pub messages: Messages,
    pub input_editor: Editor,
    pub is_editing: bool,
}

impl Session {
    pub fn new(llm_settings: LlmSettings) -> Self {
        Self {
            session_id: None,
            title: None,
            llm_settings,
            messages: Messages::default(),
            input_editor: Editor::new(String::new(), WrapMode::default()),
            // by default editting
            is_editing: true,
        }
    }

    pub fn session_id(&self) -> Option<Uuid> {
        self.session_id
    }

    pub fn handle_user_message(&mut self) -> Option<ServiceReq> {
        // only send response if not waiting
        // TODO: implement timeout for pending resp
        if self.messages.is_pending_resp() {
            return None;
        }
        let msg = self.input_editor.input().to_string();
        let msg_ = msg.clone();

        let session_id = self.session_id.unwrap_or_else(Uuid::new_v4);
        let user_message = ChatMessage::new(session_id, crate::models::Role::User, msg_);
        self.messages
            .send_question(user_message.clone(), self.llm_settings.clone());
        let req = match self.session_id {
            Some(_) => ServiceReq::ChatMessage(user_message.clone()),
            None => {
                self.session_id = Some(session_id);
                ServiceReq::NewSession {
                    settings: self.llm_settings.clone(),
                    user_message,
                }
            }
        };

        self.input_editor.clear();
        Some(req)
    }

    pub fn handle_assistant_message(&mut self, assistant_message: ChatMessage) {
        // assign session with session id
        match self.session_id {
            Some(session_id) if session_id == assistant_message.session_id => {
                self.messages.receive_response(assistant_message);
            }
            _ => {}
        }
    }

    pub fn reset(&mut self, settings: LlmSettings) {
        self.session_id = None;
        self.title = None;
        self.llm_settings = settings;
        self.is_editing = true;

        self.messages.reset();
        self.input_editor.clear();
    }

    pub fn load_session(&mut self, session: models::Session) {
        self.session_id = Some(session.id);
        self.title = Some(session.title);
        self.llm_settings = session.settings;

        self.messages.reset();
        self.messages.set_chat_messages(session.chat_messages);
    }
}
