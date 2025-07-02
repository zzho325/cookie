pub mod components;
pub mod model;
pub mod update;
pub mod view;

use color_eyre::{
    Result,
    eyre::{Context as _, bail},
};
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::stream::StreamExt;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::app::model::{Command, Message, Model};

pub struct App {
    // frontend <> backend channels
    req_tx: UnboundedSender<String>,
    resp_rx: UnboundedReceiver<String>,
}

impl App {
    pub fn new(
        req_tx: UnboundedSender<String>,
        resp_rx: UnboundedReceiver<String>,
    ) -> Result<Self> {
        Ok(Self { req_tx, resp_rx })
    }

    /// Runs the application's main loop until the user quits.
    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let mut model = Model::default();

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
                if let Some(Command::ServiceReq(req)) = maybe_cmd {
                    self.req_tx.send(req)?
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
        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
            Ok(Some(Message::Key(key.code)))
        }
        Some(Ok(_)) => Ok(None),
        Some(Err(e)) => Err(e).context("reading crossterm event failed"),
        None => {
            // the EventStream has closed
            Ok(Some(Message::CrosstermClose))
        }
    }
}
