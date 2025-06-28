use crossterm::event::KeyCode;

#[derive(Debug)]
pub struct Model {
    pub should_quit: bool,

    pub input: String,
    pub history: Vec<(String, String)>, // (queston, answer)
    pub pending_question: Option<String>,

    pub is_editing: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            should_quit: false,
            input: String::new(),
            history: vec![],
            is_editing: true,
            pending_question: None,
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
