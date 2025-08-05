use crate::{
    app::view::{messages_viewport::MessagesViewport, widgets::scroll::ScrollState},
    models::{ChatEvent, ChatEventPayload, ChatMessage, MessageDelta},
};

#[derive(Default)]
pub struct Messages {
    chat_messages: Vec<ChatMessage>,
    stream_message: Option<MessageDelta>,
    is_pending: bool,
    scroll_state: ScrollState,
    pub viewport: MessagesViewport,
}

impl Messages {
    pub fn is_pending(&self) -> bool {
        self.is_pending
    }

    pub fn chat_messages(&self) -> &[ChatMessage] {
        &self.chat_messages
    }

    pub fn set_chat_messages(&mut self, chat_messages: Vec<ChatMessage>) {
        self.chat_messages = chat_messages;
    }

    pub fn stream_message(&self) -> Option<&MessageDelta> {
        self.stream_message.as_ref()
    }

    // ----------------------------------------------------------------
    // Scroll.
    // ----------------------------------------------------------------

    pub fn scroll_down(&mut self) {
        self.scroll_state.scroll_down();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_state.scroll_up();
    }

    pub fn scroll_state(&self) -> &ScrollState {
        &self.scroll_state
    }

    pub fn set_viewport_width(&mut self, viewport_width: usize) {
        self.viewport.set_viewport_width(viewport_width);
    }

    // ----------------------------------------------------------------
    // Event handling.
    // ----------------------------------------------------------------

    /// Resets to default on nagivating.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Handles sending user chat message.
    pub fn handle_user_chat_message(&mut self, user_chat_message: ChatMessage) {
        self.chat_messages.push(user_chat_message);
        self.is_pending = true;

        self.viewport
            .build_lines(self.chat_messages.as_slice(), self.stream_message.as_ref());
    }

    /// Handles chat events streamed from streaming API.
    pub fn handle_chat_event_stream(&mut self, chat_event: ChatEvent) {
        if self.is_pending() {
            match chat_event.payload() {
                ChatEventPayload::Message(_) => {
                    if let Ok(msg) = TryInto::<ChatMessage>::try_into(chat_event) {
                        self.chat_messages.push(msg);
                    }
                    // mark state as complete on getting full text.
                    self.stream_message = None;
                    self.is_pending = false;
                }
                ChatEventPayload::MessageDelta(message_delta) => {
                    if let Some(stream_message) = &mut self.stream_message {
                        stream_message.delta_mut().push_str(&message_delta.delta);
                    } else {
                        self.stream_message = Some(MessageDelta {
                            delta: message_delta.delta.clone(),
                        });
                    }
                }
                ChatEventPayload::ToolEvent(_) => {}
            }
        } else {
            tracing::error!("receiving orphan response")
        }

        self.viewport
            .build_lines(self.chat_messages.as_slice(), self.stream_message.as_ref());
    }

    /// Handle chat events loaded from storage.
    pub fn handle_chat_events(&mut self, chat_events: Vec<ChatEvent>) {
        self.chat_messages = chat_events
            .into_iter()
            .filter_map(|event| event.try_into().ok())
            .collect();

        self.viewport
            .build_lines(self.chat_messages.as_slice(), self.stream_message.as_ref());
    }
}
