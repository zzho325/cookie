use color_eyre::Result;

use crate::service::{
    Service,
    client::api::{ContentItem, InputItem, OutputItem, ResponsesReq, Role},
    models::ServiceResp,
};

impl Service {
    pub async fn fetch_response(&mut self, msg: String) -> Result<()> {
        let resp = self
            .open_ai_client
            .responses(ResponsesReq {
                model: "gpt-4o-mini".into(),
                input: vec![InputItem {
                    role: Role::User,
                    content: msg,
                }],
                previous_response_id: self.previous_response_id.clone(),
                ..ResponsesReq::default()
            })
            .await?;

        tracing::debug!("resp {resp:?}");
        // TODO: assert role and handle refusal
        let mut texts = Vec::new();
        for output in &resp.output {
            let OutputItem::Message { content, .. } = output;
            for item in content {
                if let ContentItem::OutputText { text } = item {
                    texts.push(text.clone());
                }
            }
        }
        self.previous_response_id = Some(resp.id);
        self.resp_tx
            .send(ServiceResp::ChatMessage(texts.join("")))?;

        Ok(())
    }
}
