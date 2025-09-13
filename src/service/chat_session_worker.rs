use color_eyre::Result;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_stream::StreamExt as _;

use crate::{
    chat::*,
    models::ServiceResp,
    service::{
        llms::{LlmClient, LlmClientRouter, LlmReq},
        stores::{chat_event_store::ChatEventStore, chat_session_store::ChatSessionStore},
    },
};

/// ChatSessionWorkerHandle allows server to interact with each ChatSessionWorker. When
/// ChatSessionWorkerHandle is dropped, the corresponding woker is stopped.
pub struct ChatSessionWorkerHandle {
    chat_tx: UnboundedSender<ChatEvent>,
    chat_session: Arc<Mutex<ChatSession>>,
}

impl ChatSessionWorkerHandle {
    pub fn new(chat_tx: UnboundedSender<ChatEvent>, chat_session: Arc<Mutex<ChatSession>>) -> Self {
        Self {
            chat_tx,
            chat_session,
        }
    }

    /// Sends user message to worker.
    pub fn send_user_message(&self, user_message: ChatEvent) -> Result<()> {
        self.chat_tx.send(user_message)?;
        Ok(())
    }

    pub async fn get_chat_events(&mut self) -> ChatSession {
        let chat_session = self.chat_session.lock().await;
        chat_session.clone()
    }
}

pub struct ChatSessionWorker {
    chat_rx: UnboundedReceiver<ChatEvent>,
    chat_session: Arc<Mutex<ChatSession>>,
    llm_router: LlmClientRouter,
    resp_tx: UnboundedSender<ServiceResp>,
    chat_event_store: Arc<dyn ChatEventStore>,
    chat_session_store: Arc<dyn ChatSessionStore>,
}

impl ChatSessionWorker {
    pub fn new(
        chat_rx: UnboundedReceiver<ChatEvent>,
        chat_session: Arc<Mutex<ChatSession>>,
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
            user_message = self
                .chat_event_store
                .create_chat_event(user_message)
                .await?;
            self.resp_tx
                .send(ServiceResp::ChatEvent(user_message.clone()))?;

            // ----------------------------------------------------------------
            // Update chat session and prepare llm request.
            // ----------------------------------------------------------------
            let llm_req = {
                let mut chat_session = self.chat_session.lock().await;

                // update settings if changed.
                if chat_session.llm_settings != user_message.llm_settings {
                    chat_session.llm_settings = user_message.llm_settings;
                    self.chat_session_store
                        .update_chat_session(chat_session.clone())
                        .await?;
                }
                // append user message.
                chat_session.events.push(user_message);

                // load history events
                let events = chat_session
                    .events
                    .iter()
                    .filter_map(|e| e.payload.clone())
                    .collect();

                LlmReq {
                    events,
                    instructions: None,
                    settings: chat_session.llm_settings.unwrap_or_default(),
                }
            };

            // ----------------------------------------------------------------
            // Stream request and handle response.
            // ----------------------------------------------------------------
            let mut stream = self.llm_router.stream(llm_req).await?;
            while let Some(payload) = stream.next().await {
                let chat_event = {
                    let mut chat_session = self.chat_session.lock().await;

                    let mut chat_event = ChatEvent::new(
                        chat_session.id.clone(),
                        chat_session.llm_settings,
                        payload.clone(),
                    );

                    // persist non delta event and update timestamp
                    match payload {
                        chat_event::Payload::Message(_) => {
                            // if the last message is message delta, pop it
                            chat_session.events.pop_if(|event| {
                                matches!(event.payload, Some(chat_event::Payload::MessageDelta(_)))
                            });
                            chat_event =
                                self.chat_event_store.create_chat_event(chat_event).await?;
                            chat_session.events.push(chat_event.clone());
                        }
                        chat_event::Payload::MessageDelta(message_delta) => {
                            // if the last message is message delta, just append delta to it,
                            // otherwise append the delta
                            if let Some(event) = chat_session.events.last_mut()
                                && let Some(chat_event::Payload::MessageDelta(
                                    ref mut last_delta_message,
                                )) = event.payload
                            {
                                last_delta_message.delta.push_str(&message_delta.delta);
                            } else {
                                chat_session.events.push(chat_event.clone());
                            }
                        }
                        chat_event::Payload::ToolEvent(_) => {
                            chat_event =
                                self.chat_event_store.create_chat_event(chat_event).await?;
                            chat_session.events.push(chat_event.clone());
                        }
                    }
                    chat_event
                };
                // send to tui
                self.resp_tx.send(ServiceResp::ChatEvent(chat_event))?;
            }
        }
        Ok(())
    }
}
