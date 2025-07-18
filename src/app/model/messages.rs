use crate::{
    app::model::scroll::{ScrollState, Scrollable},
    models::{ChatMessage, LlmSettings},
};

#[derive(Default)]
pub struct Messages {
    history_messages: Vec<ChatMessage>,
    pending: Option<(ChatMessage, LlmSettings)>,
    scroll_state: ScrollState,
}

impl Messages {
    pub fn receive_response(&mut self, assistant_message: ChatMessage) {
        if let Some((user_message, _)) = self.pending.take() {
            self.history_messages.push(user_message);
            self.history_messages.push(assistant_message);
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

    pub fn history_messages(&self) -> &Vec<ChatMessage> {
        &self.history_messages
    }

    pub fn pending_question(&self) -> Option<&(ChatMessage, LlmSettings)> {
        self.pending.as_ref()
    }

    pub fn reset(&mut self) {
        self.history_messages.clear();
        self.pending = None;
        self.scroll_state.reset();
    }

    #[cfg(test)]
    #[doc(hidden)]
    pub fn set_history_messages(&mut self, history_messages: Vec<ChatMessage>) {
        self.history_messages = history_messages;
    }
}

impl Scrollable for Messages {
    fn scroll_offset(&self) -> (u16, u16) {
        self.scroll_state.scroll_offset()
    }

    fn scroll_state(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }
}
