use crate::{
    app::{
        model::focus::{Focusable, Focused},
        view::messages_viewport::MessagesViewport,
    },
    chat::*,
};

#[derive(Default)]
pub struct Messages {
    chat_events: Vec<ChatEvent>,
    stream_message: Option<MessageDelta>,
    is_pending: bool,
    focused: bool,
    title: Option<String>,
    pub viewport: MessagesViewport,
}

impl Messages {
    pub fn is_pending(&self) -> bool {
        self.is_pending
    }

    pub fn chat_events(&self) -> &[ChatEvent] {
        &self.chat_events
    }

    pub fn set_chat_events(&mut self, chat_events: Vec<ChatEvent>) {
        self.chat_events = chat_events;
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    pub fn stream_message(&self) -> Option<&MessageDelta> {
        self.stream_message.as_ref()
    }

    // ----------------------------------------------------------------
    // Scroll.
    // ----------------------------------------------------------------

    pub fn scroll_down(&mut self) {
        self.viewport.scroll_state_mut().scroll_down();
    }

    pub fn scroll_up(&mut self) {
        self.viewport.scroll_state_mut().scroll_up();
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

    /// Handles state update on sending user chat message.
    pub fn handle_send(&mut self) {
        self.viewport.scroll_to_top();
        self.is_pending = true;
    }

    /// Handles chat events streamed from service.
    pub fn handle_chat_event_stream(&mut self, chat_event: ChatEvent) {
        if self.is_pending() {
            match &chat_event.payload {
                Some(chat_event::Payload::Message(message)) if message.role() == Role::User => {
                    self.chat_events.push(chat_event);
                }
                Some(chat_event::Payload::Message(message))
                    if message.role() == Role::Assistant =>
                {
                    self.chat_events.push(chat_event);
                    // mark state as complete on getting full text.
                    self.stream_message = None;
                    self.is_pending = false;
                }
                Some(chat_event::Payload::MessageDelta(message_delta)) => {
                    if let Some(stream_message) = &mut self.stream_message {
                        stream_message.delta.push_str(&message_delta.delta);
                    } else {
                        self.stream_message = Some(MessageDelta {
                            delta: message_delta.delta.clone(),
                        });
                    }
                }
                _ => {}
            }
        } else {
            tracing::error!("receiving orphan response")
        }

        // tracing::debug!("messages {:?}", self.chat_events);
        self.viewport
            .build_lines(self.chat_events.as_slice(), self.stream_message.as_ref());
    }

    /// Handles chat events loaded from storage.
    pub fn handle_chat_events(&mut self, mut chat_events: Vec<ChatEvent>) {
        // pick up pending message.
        if let Some(event) = chat_events
            .pop_if(|event| matches!(event.payload, Some(chat_event::Payload::MessageDelta(_))))
            && let Some(chat_event::Payload::MessageDelta(message_delta)) = event.payload
        {
            self.stream_message = Some(message_delta);
            self.is_pending = true;
        }
        self.chat_events = chat_events.into_iter().collect();
        self.viewport
            .build_lines(self.chat_events.as_slice(), self.stream_message.as_ref());
    }
}

impl Focusable for Messages {
    fn set_focus(&mut self, focused: bool) {
        // clear visual selection if navigating away.
        if !focused {
            self.viewport.clear_visual_selection();
        }
        self.focused = focused
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
    fn to_focused(&self) -> Focused {
        Focused::Messages
    }
}
