use crate::{
    app::view::widgets::scroll::ScrollState,
    models::{ChatMessage, LlmSettings},
};

#[derive(Default)]
pub struct Messages {
    chat_messages: Vec<ChatMessage>,
    pending: Option<(ChatMessage, LlmSettings)>,
    scroll_state: ScrollState,
}

impl Messages {
    pub fn receive_response(&mut self, assistant_message: ChatMessage) {
        if let Some((user_message, _)) = self.pending.take() {
            self.chat_messages.push(user_message);
            self.chat_messages.push(assistant_message);
            self.pending = None;
        } else {
            // TODO: report error
            tracing::warn!("received answer while no question is pending")
        }
    }

    pub fn send_question(&mut self, user_chat_message: ChatMessage, llm_settings: LlmSettings) {
        self.pending = Some((user_chat_message, llm_settings));
    }

    pub fn is_pending_resp(&self) -> bool {
        self.pending.is_some()
    }

    pub fn chat_messages(&self) -> &[ChatMessage] {
        &self.chat_messages
    }

    pub fn set_chat_messages(&mut self, chat_messages: Vec<ChatMessage>) {
        self.chat_messages = chat_messages;
    }

    pub fn pending(&self) -> Option<&(ChatMessage, LlmSettings)> {
        self.pending.as_ref()
    }

    pub fn reset(&mut self) {
        self.chat_messages.clear();
        self.pending = None;
        self.scroll_state.reset();
    }

    pub fn scroll_down(&mut self) {
        self.scroll_state.scroll_down();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_state.scroll_up();
    }

    pub fn scroll_state(&self) -> &ScrollState {
        &self.scroll_state
    }
}
