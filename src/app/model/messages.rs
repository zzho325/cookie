use std::sync::Arc;

use crate::{
    app::model::scroll::{ScrollState, Scrollable},
    models::{ChatMessage, LlmSettings},
};

#[derive(Default)]
pub struct Messages {
    pub llm_settings: Arc<LlmSettings>,
    pub history_messages: Vec<ChatMessage>,
    pub pending: Option<(ChatMessage, LlmSettings)>,
    pub scroll_state: ScrollState,
}

impl Messages {
    pub fn new(settings: Arc<LlmSettings>) -> Self {
        Self {
            llm_settings: settings,
            ..Self::default()
        }
    }

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

    pub fn send_question(&mut self, user_chat_message: ChatMessage) {
        self.pending = Some((user_chat_message, (*self.llm_settings).clone()));
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
}

impl Scrollable for Messages {
    fn scroll_offset(&self) -> (u16, u16) {
        self.scroll_state.scroll_offset()
    }

    fn scroll_state(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }
}
