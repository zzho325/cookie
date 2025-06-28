use crossterm::event::KeyCode;

/// drives update
pub enum Message {
    Key(KeyCode),
    ServiceResp(String),
    SendQuestion,
    CrosstermClose,
}

/// side effect of update
pub enum Command {
    ServiceReq(String),
}
