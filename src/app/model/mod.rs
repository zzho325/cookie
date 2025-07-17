pub mod editor;
pub mod messages;
pub mod scroll;
pub mod session_manager;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    app::{
        model::{
            editor::{Editor, WrapMode},
            messages::Messages,
            scroll::Scrollable as _,
            session_manager::SessionManager,
        },
        update::Update,
    },
    models::{ChatMessage, LlmSettings, ServiceReq, ServiceResp},
};

pub struct Model {
    pub default_llm_settings: LlmSettings,
    pub session: Session,
    pub session_manager: SessionManager,

    pub should_quit: bool,
    pub is_focused: bool,
}

impl Model {
    pub fn new(default_llm_settings: LlmSettings) -> Self {
        Self {
            default_llm_settings: default_llm_settings.clone(),
            should_quit: false,
            session: Session::new(default_llm_settings),
            session_manager: SessionManager::new(),
            is_focused: false,
        }
    }
}

pub struct Session {
    pub session_id: Option<Uuid>,
    pub llm_settings: Arc<LlmSettings>,
    pub messages: Messages,
    pub input_editor: Editor,
    pub is_editing: bool,
}

impl Session {
    pub fn new(default_llm_settings: LlmSettings) -> Self {
        let shared_llm_settings = Arc::new(default_llm_settings);
        Self {
            session_id: None,
            llm_settings: shared_llm_settings.clone(),
            messages: Messages::new(shared_llm_settings),
            input_editor: Editor::new(String::new(), WrapMode::default()),
            // by default editting
            is_editing: true,
        }
    }

    pub fn handle_user_message(&mut self) -> Update {
        // only send response if not waiting
        // TODO: implement timeout for pending resp
        if self.messages.is_pending_resp() {
            return (None, None);
        }
        let msg = self.input_editor.input().to_string();
        let msg_ = msg.clone();

        let session_id = self.session_id.unwrap_or_else(Uuid::new_v4);
        let user_message = ChatMessage::new(session_id, crate::models::Role::User, msg_);
        self.messages.send_question(user_message.clone());
        let req = match self.session_id {
            Some(_) => ServiceReq::ChatMessage(user_message.clone()),
            None => {
                self.session_id = Some(session_id);
                ServiceReq::NewSession {
                    settings: (*self.llm_settings).clone(),
                    user_message,
                }
            }
        };

        self.input_editor.clear();
        (None, Some(Command::ServiceReq(req)))
    }

    pub fn handle_assistant_message(&mut self, assistant_message: ChatMessage) -> Update {
        // assign session with session id
        match self.session_id {
            Some(session_id) if session_id == assistant_message.session_id => {
                self.messages.receive_response(assistant_message);
            }
            _ => {}
        }
        (None, None)
    }

    pub fn handle_key_event(
        &mut self,
        KeyEvent {
            code,
            modifiers,
            kind: _,
            state: _,
        }: KeyEvent,
    ) -> Update {
        if self.is_editing {
            let editor = &mut self.input_editor;
            match (code, modifiers) {
                (KeyCode::Char(c), _) => editor.enter_char(c),
                (KeyCode::Backspace, _) => editor.delete_char(),
                (KeyCode::Left, _) => editor.move_cursor_left(),
                (KeyCode::Right, _) => editor.move_cursor_right(),
                (KeyCode::Esc, _) => self.is_editing = false,
                (KeyCode::Enter, KeyModifiers::ALT) => editor.enter_char('\n'),
                (KeyCode::Enter, _) => return (Some(Message::Send), None),
                (KeyCode::Down, _) => editor.move_cursor_down(),
                (KeyCode::Up, _) => editor.move_cursor_up(),
                _ => {}
            }
        } else {
            match code {
                KeyCode::Down => self.messages.scroll_down(),
                KeyCode::Up => self.messages.scroll_up(),
                // KeyCode::Right => {
                //     // for now for test
                //     let llm_settings = LlmSettings::OpenAI {
                //         self: OpenAIself::Gpt4oMini,
                //         web_search: false,
                //     };
                //     tracing::debug!("send setting update");
                //     let req = crate::models::ServiceReq::UpdateSettings(llm_settings);
                //     let cmd = Command::ServiceReq(req);
                //     return (None, Some(cmd));
                // }
                _ => {}
            }
        }
        (None, None)
    }
}

/// Drives update.
pub enum Message {
    Key(KeyEvent),
    ServiceResp(ServiceResp),
    Send,
    CrosstermClose,
}

/// Side effect of update.
pub enum Command {
    ServiceReq(ServiceReq),
}

impl Command {
    /// If this `Command` corresponds to a service request, return `Some(_)`, otherwise return `None`.
    pub fn into_service_req(self) -> Option<ServiceReq> {
        let Command::ServiceReq(req) = self;
        Some(req)
    }
}
