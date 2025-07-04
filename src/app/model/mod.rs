pub mod editor;

use crossterm::event::KeyCode;

use crate::app::model::editor::Editor;

#[derive(Debug)]
pub struct Model {
    pub should_quit: bool,

    pub history_messages: Vec<(String, String)>, // (queston, answer)
    pub pending_question: Option<String>,

    pub input_editor: Editor,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            should_quit: false,
            history_messages: vec![],
            pending_question: None,
            // By default editting
            input_editor: Editor::new(String::new(), true),
        }
    }
}

/// Drives update.
pub enum Message {
    Key(KeyCode),
    ServiceResp(String),
    SendQuestion,
    CrosstermClose,
}

/// Side effect of update.
pub enum Command {
    ServiceReq(String),
}
