mod model;
mod update;
mod view;

use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use crossterm::event::{Event, EventStream, KeyEvent, KeyEventKind};
use futures_util::stream::StreamExt;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::{ServiceReq, ServiceResp, app::model::Model, models::configs::Configs};

pub struct App {
    // frontend <> backend channels
    req_tx: UnboundedSender<ServiceReq>,
    resp_rx: UnboundedReceiver<ServiceResp>,
}

impl App {
    pub fn new(
        req_tx: UnboundedSender<ServiceReq>,
        resp_rx: UnboundedReceiver<ServiceResp>,
    ) -> Result<Self> {
        Ok(Self { req_tx, resp_rx })
    }

    /// Runs the application's main loop until the user quits.
    pub async fn run(&mut self, cfg: Configs) -> Result<()> {
        let mut terminal = ratatui::init();
        let mut model = Model::new(cfg);

        let mut event_reader = EventStream::new();

        while !model.should_quit {
            // draw first so we see the latest state immediately
            terminal.draw(|f| view::render_ui(&mut model, f))?;
            let mut maybe_msg = select! {
                // key event from Crossterm
                maybe_evt = event_reader.next() => {
                    handle_crossterm_event(&model, maybe_evt)?
                }
                // service response
                maybe_resp = self.resp_rx.recv() => {
                    if let Some(resp) = maybe_resp {
                        Some(Message::ServiceResp(resp))
                    } else {
                        bail!("service stream closed");
                    }
                }
            };

            // handle chained messages and side effect from update
            while let Some(msg) = maybe_msg {
                let (next_msg, maybe_cmd) = update::update(&mut model, msg);
                maybe_msg = next_msg;

                if let Some(cmd) = maybe_cmd {
                    if let Some(req) = cmd.into_service_req() {
                        self.req_tx.send(req)?
                    }
                }
            }
        }

        Ok(())
    }
}

/// Converts crossterm event to message.
fn handle_crossterm_event(
    _: &Model,
    maybe_evt: Option<Result<Event, std::io::Error>>,
) -> Result<Option<Message>> {
    match maybe_evt {
        Some(Ok(Event::Key(evt))) if evt.kind == KeyEventKind::Press => Ok(Some(Message::Key(evt))),
        Some(Ok(_)) => Ok(None),
        Some(Err(e)) => Err(e).context("reading crossterm event failed"),
        None => {
            // the EventStream has closed
            Ok(Some(Message::CrosstermClose))
        }
    }
}

/// Drives update.
pub enum Message {
    Key(KeyEvent),
    ServiceResp(ServiceResp),
    /// Sends message.
    Send,
    /// Starts new empty chat at tui.
    NewChat,
    /// Editing input.
    Editing,
    /// Open setting manager.
    Setting,
    GetSession(uuid::Uuid),
    CrosstermClose,
}

/// Side effect of update.
pub enum Command {
    ServiceReq(ServiceReq),
}

impl Command {
    /// If this `Command` corresponds to a service request, return `Some(_)`, otherwise return `None`.
    pub fn into_service_req(self) -> Option<ServiceReq> {
        let Command::ServiceReq(req) = self;
        Some(req)
    }
}
