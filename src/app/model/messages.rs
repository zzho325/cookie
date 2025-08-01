use crate::{
    app::view::widgets::scroll::ScrollState,
    models::{ChatEvent, ChatEventPayload, ChatMessage},
};

#[derive(Default)]
pub struct Messages {
    chat_messages: Vec<ChatMessage>,
    pending: Option<ChatMessage>,
    scroll_state: ScrollState,
}

impl Messages {
    /// Handle chat events from server.
    pub fn handle_chat_event(&mut self, chat_event: ChatEvent) {
        if let Some(user_message) = self.pending.take() {
            self.chat_messages.push(user_message);
            if let Ok(msg) = TryInto::<ChatMessage>::try_into(chat_event) {
                self.chat_messages.push(msg);
            }
            self.pending = None;
        } else {
            match chat_event.payload() {
                ChatEventPayload::Message(p) => {
                    if let Some(message) = self.chat_messages.last_mut() {
                        *message.msg_mut() = p.msg.to_string();
                    }
                }
                ChatEventPayload::MessageDelta(p) => {
                    if let Some(message) = self.chat_messages.last_mut() {
                        message.msg_mut().push_str(&p.delta);
                    }
                }
                ChatEventPayload::ToolEvent(_) => {}
            }
        }
    }

    pub fn send_question(&mut self, user_chat_message: ChatMessage) {
        self.pending = Some(user_chat_message);
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

    pub fn handle_chat_events(&mut self, chat_events: Vec<ChatEvent>) {
        self.chat_messages = chat_events
            .into_iter()
            .filter_map(|event| event.try_into().ok())
            .collect();
    }

    pub fn pending(&self) -> Option<&ChatMessage> {
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
