use std::process::Output;

use color_eyre::Result;

use crate::service::{
    Service,
    client::{
        OpenAIClient,
        api::{ContentItem, InputItem, OutputItem, ResponsesReq, Role},
    },
    models::ServiceResp,
};

impl<C: OpenAIClient> Service<C> {
    pub async fn fetch_response(&mut self, msg: String) -> Result<()> {
        let resp = self
            .open_ai_client
            .responses(ResponsesReq {
                model: "gpt-4o".into(),
                input: vec![InputItem {
                    role: Role::User,
                    content: msg,
                }],
                ..ResponsesReq::default()
            })
            .await?;

        tracing::debug!("resp {resp:?}");
        let mut texts = Vec::new();
        for output in &resp.output {
            let OutputItem::Message { content, .. } = output;
            for item in content {
                if let ContentItem::OutputText { text } = item {
                    texts.push(text.clone());
                }
            }
        }
        self.resp_tx
            .send(ServiceResp::ChatMessage(texts.join("")))?;

        Ok(())
    }
}
