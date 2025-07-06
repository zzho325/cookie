#[derive(Debug)]
pub enum ServiceReq {
    ChatMessage(String),
}

pub enum ServiceResp {
    ChatMessage(String),
    Refusal(String),
}
