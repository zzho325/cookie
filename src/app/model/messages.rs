use std::{sync::Arc, time::Instant};

use crate::{
    app::model::scroll::{ScrollState, Scrollable},
    models::LlmSettings,
};

pub struct MessageMetadata {
    pub llm: LlmSettings,
    pub req_time: Instant,
    pub resp_time: Option<Instant>,
}

pub struct HistoryMessage {
    pub user_msg: String,
    pub assistant_msg: String,
    pub metadata: MessageMetadata,
}

pub struct PendingMessage {
    pub user_msg: String,
    pub metadata: MessageMetadata,
}

#[derive(Default)]
pub struct Messages {
    pub llm_settings: Arc<LlmSettings>,
    pub history_messages: Vec<HistoryMessage>,
    pub pending: Option<PendingMessage>,
    pub scroll_state: ScrollState,
}

impl Messages {
    pub fn new(settings: Arc<LlmSettings>) -> Self {
        Self {
            llm_settings: settings,
            ..Self::default()
        }
    }

    pub fn append_message(&mut self, assistant_msg: String) {
        if let Some(mut pending) = self.pending.take() {
            pending.metadata.resp_time = Some(Instant::now());
            let history_msg = HistoryMessage {
                user_msg: pending.user_msg,
                assistant_msg,
                metadata: pending.metadata,
            };
            self.history_messages.push(history_msg);
            self.pending = None;
        } else {
            // TODO: report error
            tracing::warn!("received answer while no question is pending")
        }
    }

    pub fn send_question(&mut self, user_msg: &str) {
        let metadata = MessageMetadata {
            llm: (*self.llm_settings).clone(),
            req_time: Instant::now(),
            resp_time: None,
        };
        self.pending = Some(PendingMessage {
            user_msg: user_msg.to_string(),
            metadata,
        });
    }

    pub fn is_pending_resp(&self) -> bool {
        self.pending.is_some()
    }

    pub fn history_messages(&self) -> &Vec<HistoryMessage> {
        &self.history_messages
    }

    pub fn pending_question(&self) -> Option<&PendingMessage> {
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
