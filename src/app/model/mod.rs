pub mod editor;
pub mod messages;
pub mod scroll;

use crossterm::event::KeyEvent;

use crate::{
    app::model::{
        editor::{Editor, WrapMode},
        messages::Messages,
    },
    service::models::{ServiceReq, ServiceResp},
};

pub struct Model {
    pub should_quit: bool,
    pub messages: Messages,
    pub input_editor: Editor,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            should_quit: false,
            messages: Messages::default(),
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
