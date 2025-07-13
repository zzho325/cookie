use color_eyre::{Result, eyre::Ok};

use crate::{
    models::ServiceResp,
    service::{Service, client::LlmReq},
};

impl Service {
    pub async fn fetch_response(&mut self, msg: String) -> Result<()> {
        let resp = self
            .llm_client
            .responses(LlmReq {
                msg,
                previous_response_id: self.previous_response_id.clone(),
                settings: self.llm_settings.clone(),
            })
            .await?;

        self.previous_response_id = Some(resp.id);
        self.resp_tx.send(ServiceResp::ChatMessage(resp.msg))?;
        Ok(())
    }
}
