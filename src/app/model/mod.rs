pub mod editor;
pub mod messages;
pub mod scroll;

use std::sync::Arc;

use crossterm::event::KeyEvent;

use crate::{
    app::model::{
        editor::{Editor, WrapMode},
        messages::Messages,
    },
    service::models::{LlmProvider, ServiceReq, ServiceResp},
};

/// Mutable settings.
#[derive(Default)]
pub struct Settings {
    pub llm: LlmProvider,
}

pub struct Model {
    pub settings: Arc<Settings>,
    pub should_quit: bool,
    pub messages: Messages,
    pub input_editor: Editor,
}

impl Model {
    pub fn new(settings: Settings) -> Self {
        let shared_settings = Arc::new(settings);
        Self {
            settings: shared_settings.clone(),
            should_quit: false,
            messages: Messages::new(shared_settings),
            // by default editting
            input_editor: Editor::new(String::new(), true, WrapMode::default()),
        }
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
    SendMessage(String),
}

impl Command {
    /// If this `Command` corresponds to a service request, return `Some(_)`, otherwise return `None`.
    pub fn into_service_req(self) -> Option<ServiceReq> {
        let Command::SendMessage(msg) = self;
        Some(ServiceReq::ChatMessage(msg))
    }
}
