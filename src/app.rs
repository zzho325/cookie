mod model;
mod update;
mod view;

use color_eyre::{Result, eyre::Context};
use crossterm::clipboard::CopyToClipboard;
use crossterm::{
    cursor::SetCursorStyle,
    event::{EnableBracketedPaste, Event, EventStream, KeyEvent, KeyEventKind},
    execute, terminal,
};
use ratatui::{Terminal, prelude::Backend};
use std::{env, fs, io::Write};
use tempfile::NamedTempFile;
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

                match maybe_cmd {
                    Some(Command::ServiceReq(req)) => self.req_tx.send(req)?,
                    Some(Command::ExternalEditing(initial)) => {
                        match external_editing(&mut terminal, &initial) {
                            Ok(data) => maybe_msg = Some(Message::ExternalEditingComplete(data)),
                            Err(e) => {
                                tracing::error!("failed to edit: {e}")
                            }
                        }
                    }
                    Some(Command::ExternalEditingReadOnly(initial)) => {
                        if let Err(e) = external_editing(&mut terminal, &initial) {
                            tracing::error!("failed to view: {e}")
                        }
                    }
                    Some(Command::CopyToClipboard(selected)) => {
                        if let Err(e) = copy_to_clipboard(&mut terminal, &selected) {
                            tracing::error!("failed to copy to clipboard: {e}")
                        }
                    }
                    None => {}
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
    CrosstermClose,

    ServiceResp(ServiceResp),

    /* ----- model wide activities ----- */
    /// Sends message.
    Send,
    /// Focuses on editor and enter editing mode.
    Editing,
    /// Opens setting manager or saves setting manager update and closes it.
    Setting,
    /// Starts new empty chat at tui.
    NewSession,
    /// Deletes currently selected session and navigates to the next session.
    DeleteSession,
    /// Selects next session in session manager.
    SelectNextSession,
    /// Selects previews session in session manager.
    SelectPrevSession,

    /* ----- editor activities ----- */
    /// Pastes event from crossterm.
    Paste(String),
    /// Updates editor input accordingly after editing with system's editor finished
    ExternalEditingComplete(String),
}

/// Side effect of update.
pub enum Command {
    ServiceReq(ServiceReq),
    /// Opens system's editor to continue editing.
    ExternalEditing(String),
    /// Opens system's editor with content, but throws away editing content.
    ExternalEditingReadOnly(String),
    /// Puts given input to system clipboard.
    CopyToClipboard(String),
}

/// Opens external editor to continue editing initial and return edited string.
fn external_editing<B>(terminal: &mut Terminal<B>, initial: &str) -> Result<String>
where
    B: Backend + Write,
{
    // prepare temp file
    let file = NamedTempFile::new()?;
    fs::write(file.path(), initial)?;

    // respect VISUAL/EDITOR; fallback to vim
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "vim".to_string());

    // leave raw & alt screen
    terminal::disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        crossterm::event::DisableBracketedPaste,
        crossterm::terminal::LeaveAlternateScreen
    )?;

    // launch editor and wait
    std::process::Command::new(editor)
        .arg(file.path())
        .status()?;
    let input = fs::read_to_string(file.path())?;

    // re-enter raw & alt screen
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableBracketedPaste,
    )?;
    terminal::enable_raw_mode()?;
    terminal.clear()?;

    Ok(input)
}

/// Copies text to system clipboard.
fn copy_to_clipboard<B>(terminal: &mut Terminal<B>, text: &str) -> Result<()>
where
    B: Backend + Write,
{
    execute!(
        terminal.backend_mut(),
        CopyToClipboard::to_clipboard_from(text)
    )?;
    Ok(())
}
