use async_trait::async_trait;
use color_eyre::Result;

use crate::service::client::{
    OpenAIClient,
    api::{ContentItem, OutputItem, ResponsesReq, ResponsesResp, Role},
};

/// Mock OpenAI Rest API client.
///
/// It simply echos request message back.
#[cfg(debug_assertions)]
pub struct MockOpenAIClient {}

#[async_trait]
#[cfg(debug_assertions)]
impl OpenAIClient for MockOpenAIClient {
    async fn responses(&self, req: ResponsesReq) -> Result<ResponsesResp> {
        let message = req.input[0].content.clone();
        Ok(ResponsesResp {
            id: "mock-response-id".to_string(),
            output: vec![OutputItem::Message {
                role: Role::Assistant,
                content: vec![ContentItem::OutputText { text: message }],
            }],
        })
    }
}
