use color_eyre::{
    Result,
    eyre::{Context as _, bail},
};
use reqwest::header::AUTHORIZATION;
use serde::{Serialize, de::DeserializeOwned};

pub async fn post<U: Serialize, T: DeserializeOwned>(
    client: &reqwest::Client,
    url: String,
    api_key: String,
    payload: &U,
) -> Result<T> {
    let resp = client
        .post(url)
        .json(payload)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .send()
        .await
        .wrap_err("failed to send request")?;

    handle_resp(resp).await
}

async fn handle_resp<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    let body = resp.text().await.wrap_err("failed to read response body")?;
    if !status.is_success() {
        // TODO: return error
        bail!("request failed: HTTP {status} with body:\n{body}");
    }

    let result: T = serde_json::from_str(&body)
        .wrap_err_with(|| format!("could not deserialize response body:\n{}", body))?;

    Ok(result)
}
