mod app;
mod models;
mod service;

use color_eyre::{Result, eyre::Context};
use tokio::sync::mpsc;

use crate::{
    app::App,
    models::{LlmSettings, ServiceReq, ServiceResp, configs::Configs},
    service::ServiceBuilder,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = init_logging();

    // TODO: handle error better
    let cfg = Configs::load().wrap_err_with(|| "load config")?;
    let mut default_llm_settings = cfg.derive_llm_settings();

    // #[cfg(debug_assertions)]
    {
        let args: Vec<String> = std::env::args().collect();
        let use_mock = args.iter().any(|a| a == "--mock");

        if use_mock {
            default_llm_settings = LlmSettings::Mock {
                latency: std::time::Duration::ZERO,
            }
        }
    }

    // frontend <> backend channels
    let (req_tx, req_rx) = mpsc::unbounded_channel::<ServiceReq>();
    let (resp_tx, resp_rx) = mpsc::unbounded_channel::<ServiceResp>();

    // spawn backend service and tui app, both *should* only return on irrecoverable error
    let svc_llm_settings = default_llm_settings.clone();
    let svc_fut = async move {
        let service = ServiceBuilder::new(svc_llm_settings, req_rx, resp_tx).build();
        service.run().await
    };

    let app_fut = async move {
        let mut app = App::new(req_tx, resp_rx)?;
        app.run(default_llm_settings).await
        // req_tx is dropped here and will shutdown backend service
    };

    // return either when both complete with Ok or when the first complete with Err
    let res = tokio::try_join!(svc_fut, app_fut);

    ratatui::restore();

    // propagate the first error
    res.map(|(_svc_ok, _tui_ok)| ())
}

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    // creates logs/YYYY-MM-DD/service.log rotating daily
    let file_appender = tracing_appender::rolling::daily("logs", "service.log");
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::ERROR.into())
        .from_env_lossy();
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .init();

    _guard // keep guard alive so logs are flushed on drop
}
