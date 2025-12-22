use color_eyre::Result;
use tokio::sync::mpsc;

use crate::{
    chat::*,
    models::{ServiceReq, ServiceResp, configs::Config},
    service::ServiceBuilder,
};

pub async fn run(message: String, config: Config) -> Result<()> {
    let (req_tx, req_rx) = mpsc::unbounded_channel::<ServiceReq>();
    let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<ServiceResp>();

    // Spawn backend service
    tokio::spawn(async move {
        if let Some(service) = ServiceBuilder::new(req_rx, resp_tx).build() {
            let _ = service.run().await;
        }
    });

    // Create a chat event with the user message
    let llm_settings = config.derive_llm_settings();
    let session_id = uuid::Uuid::new_v4().to_string();

    let chat_event = ChatEvent::new(
        session_id,
        Some(llm_settings),
        chat_event::Payload::Message(Message {
            role: Role::User as i32,
            msg: message,
        }),
    );

    // Send the message
    req_tx.send(ServiceReq::ChatMessage(chat_event))?;

    // Collect and print the response
    let mut buffer = String::new();
    while let Some(resp) = resp_rx.recv().await {
        match resp {
            ServiceResp::ChatEvent(event) => {
                if let Some(payload) = event.payload {
                    match payload {
                        chat_event::Payload::MessageDelta(delta) => {
                            print!("{}", delta.delta);
                            use std::io::Write;
                            std::io::stdout().flush()?;
                            buffer.push_str(&delta.delta);
                        }
                        chat_event::Payload::Message(msg) if msg.role() == Role::Assistant => {
                            // Final message received, we can exit
                            if buffer.is_empty() {
                                println!("{}", msg.msg);
                            } else {
                                println!(); // New line after streaming
                            }
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            ServiceResp::Error(err) => {
                eprintln!("Error: {}", err);
                return Ok(());
            }
            _ => {}
        }
    }

    Ok(())
}
