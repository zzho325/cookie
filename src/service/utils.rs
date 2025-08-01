use std::pin::Pin;

use color_eyre::{
    Result,
    eyre::{Context as _, bail},
};
use futures_util::{Stream, StreamExt, stream::BoxStream};
use reqwest::header::AUTHORIZATION;
use reqwest_eventsource::{Event, EventSource};
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

pub async fn post_stream<U, T>(
    client: &reqwest::Client,
    url: String,
    api_key: String,
    payload: &U,
) -> Result<Pin<Box<dyn Stream<Item = Result<T>> + Send>>>
where
    U: serde::Serialize,
    T: DeserializeOwned + Send + 'static,
{
    let request_builder = client
        .post(url)
        .json(payload)
        .header(AUTHORIZATION, format!("Bearer {}", api_key));

    let event_source = EventSource::new(request_builder).wrap_err("failed to create SSE client")?;
    Ok(stream::<T>(event_source))
}

pub(crate) fn stream<T>(mut event_source: EventSource) -> BoxStream<'static, Result<T>>
where
    T: DeserializeOwned + Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(event) = event_source.next().await {
            match event {
                Err(_) => break, // stream ends
                Ok(Event::Message(msg)) => {
                    // tracing::debug!(msg.data);
                    let parsed = serde_json::from_str::<T>(msg.data.as_str())
                        .wrap_err_with(|| format!("deserialize SSE payload: {}", msg.data));
                    if tx.send(parsed).is_err() {
                        break;
                    }
                }
                Ok(Event::Open) => {} // ignore,
            }
        }
        event_source.close();
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
