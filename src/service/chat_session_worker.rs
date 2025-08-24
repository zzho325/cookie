use std::sync::Arc;

use crate::{
    chat::*,
    models::ServiceResp,
    service::{
        llms::{LlmClient, LlmClientRouter, LlmReq},
        stores::{chat_event_store::ChatEventStore, chat_session_store::ChatSessionStore},
    },
};
use color_eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt as _;

pub struct ChatSessionWorker {
    chat_rx: UnboundedReceiver<ChatEvent>,
    chat_session: ChatSession,
    llm_router: LlmClientRouter,
    resp_tx: UnboundedSender<ServiceResp>,
    chat_event_store: Arc<dyn ChatEventStore>,
    chat_session_store: Arc<dyn ChatSessionStore>,
}

impl ChatSessionWorker {
    pub fn new(
        chat_rx: UnboundedReceiver<ChatEvent>,
        chat_session: ChatSession,
        llm_router: LlmClientRouter,
        resp_tx: UnboundedSender<ServiceResp>,
        chat_event_store: Arc<dyn ChatEventStore>,
        chat_session_store: Arc<dyn ChatSessionStore>,
    ) -> Self {
        Self {
            chat_rx,
            chat_session,
            llm_router,
            resp_tx,
            chat_event_store,
            chat_session_store,
        }
    }

    /// Polls user messages from `chat_rx`, for each user message, constructs `LlmReq` and request
    /// llm response with streaming. Send response to tui and persist to db.
    pub async fn run(mut self) -> Result<()> {
        // TODO: close worker after inactivity
        while let Some(mut user_message) = self.chat_rx.recv().await {
            // ----------------------------------------------------------------
            // Persist user message and send it back to tui.
            // ----------------------------------------------------------------
            // persist user message and update timestamp
            user_message = self
                .chat_event_store
                .create_chat_event(user_message)
                .await?;

            // update settings if changed
            if self.chat_session.llm_settings != user_message.llm_settings {
                self.chat_session.llm_settings = user_message.llm_settings;
                self.chat_session_store
                    .update_chat_session(self.chat_session.clone())
                    .await?;
            }
            self.chat_session.events.push(user_message.clone());
            self.resp_tx.send(ServiceResp::ChatEvent(user_message))?;

            // ----------------------------------------------------------------
            // Prepare llm request.
            // ----------------------------------------------------------------
            // load history events
            let events = self
                .chat_session
                .events
                .iter()
                .filter_map(|e| e.payload.clone())
                .collect();

            let llm_req = LlmReq {
                events,
                instructions: None,
                settings: self.chat_session.llm_settings.unwrap_or_default(),
            };

            // ----------------------------------------------------------------
            // Stream request and handle response.
            // ----------------------------------------------------------------
            let session_id = self.chat_session.id.clone();
            let mut stream = self.llm_router.stream(llm_req).await?;
            while let Some(payload) = stream.next().await {
                let mut chat_event = ChatEvent::new(
                    session_id.clone(),
                    self.chat_session.llm_settings,
                    payload.clone(),
                );
                // persist non delta event and update timestamp
                match payload {
                    chat_event::Payload::Message(_) | chat_event::Payload::ToolEvent(_) => {
                        chat_event = self.chat_event_store.create_chat_event(chat_event).await?;
                        self.chat_session.events.push(chat_event.clone());
                    }
                    _ => {}
                }
                // send to tui
                self.resp_tx.send(ServiceResp::ChatEvent(chat_event))?;
            }
        }
        Ok(())
    }
}
