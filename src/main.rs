mod app;
mod service;

use color_eyre::Result;
use tokio::sync::mpsc;

use crate::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = init_logging();

    // frontend <> backend channels
    // TODO: replace String with Req / Resp enum
    let (req_tx, req_rx) = mpsc::unbounded_channel::<String>();
    let (resp_tx, resp_rx) = mpsc::unbounded_channel::<String>();

    // spawn backend service and tui app both *should* only return on irrecoverable error

    let svc_fut = async move { service::run_service_loop_with_openai(req_rx, resp_tx).await };
    let app_fut = async move {
        let mut app = App::new(req_tx, resp_rx)?;
        app.run().await
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
    let filter = tracing_subscriber::EnvFilter::new("debug"); // default level = INFO
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .init();

    _guard // keep guard alive so logs are flushed on drop
}
