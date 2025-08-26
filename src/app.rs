mod model;
mod update;
mod view;

use color_eyre::{Result, eyre::Context};
use crossterm::cursor::SetCursorStyle;
use crossterm::event::{EnableBracketedPaste, Event, EventStream, KeyEvent, KeyEventKind};
use crossterm::execute;

use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_stream::StreamExt;

use crate::{ServiceReq, ServiceResp, app::model::Model, models::configs::Config};

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
    pub async fn run(&mut self, cfg: Config) -> Result<()> {
        let mut terminal = ratatui::init();
        let mut model = Model::new(cfg);

        let mut event_reader = EventStream::new();
        // enable crossterm bracketed paste
        execute!(terminal.backend_mut(), EnableBracketedPaste)?;
        while !model.should_quit {
            // set cursor style based on editing status
            let set_cursor_style = match model.focused {
                model::focus::Focused::InputEditor if model.session.input_editor.is_editing() => {
                    SetCursorStyle::BlinkingBar
                }
                _ => SetCursorStyle::SteadyBlock,
            };
            execute!(terminal.backend_mut(), set_cursor_style)?;

            // draw first so we see the latest state immediately
            terminal.draw(|f| view::render_ui(&mut model, f))?;
            let mut maybe_msg = select! {
                // key event from Crossterm
                maybe_evt = event_reader.next() => {
                    handle_crossterm_event(&model, maybe_evt)?
                }
                // service response
                maybe_resp = self.resp_rx.recv() => {
                    maybe_resp.map(Message::ServiceResp)
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
        Some(Ok(Event::Paste(data))) => Ok(Some(Message::Paste(data))),
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
    /// Paste event from crossterm.
    Paste(String),
    ServiceResp(ServiceResp),
    /// Sends message.
    Send,
    /// Starts new empty chat at tui.
    NewChat,
    /// Editing input.
    Editing,
    /// Open setting manager or saves setting manager update and closes it.
    Setting,
    GetSession(String),
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
