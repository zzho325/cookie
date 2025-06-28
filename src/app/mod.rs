pub mod types;

use color_eyre::{
    Result,
    eyre::{Context as _, bail},
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures_util::stream::StreamExt;
use ratatui::{
    Frame,
    style::Stylize as _,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tracing::warn;

use crate::app::types::{Command, Message};

#[derive(Debug)]
pub struct Model {
    should_quit: bool,

    input: String,
    history: Vec<(String, String)>, // (queston, answer)
    pending_question: Option<String>,

    is_editing: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            should_quit: false,
            input: String::new(),
            history: vec![],
            is_editing: true,
            pending_question: None,
        }
    }
}

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

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let mut model = Model::default();

        let mut event_reader = EventStream::new();

        while !model.should_quit {
            // draw first so we see the latest state immediately
            terminal.draw(|f| Self::view(&mut model, f))?;
            let mut maybe_msg = select! {
                // key event from Crossterm
                maybe_evt = event_reader.next() => {
                    Self::handle_crossterm_event(&model, maybe_evt)?
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
                let (next_msg, maybe_cmd) = Self::update(&mut model, msg);
                maybe_msg = next_msg;
                if let Some(Command::ServiceReq(req)) = maybe_cmd {
                    self.req_tx.send(req)?
                }
            }
        }

        Ok(())
    }

    /// convert crossterm event to message
    pub fn handle_crossterm_event(
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

    /// update model with message and optionally create next message for chained update and command
    /// for side effect
    fn update(model: &mut Model, msg: Message) -> (Option<Message>, Option<Command>) {
        match msg {
            Message::Key(code) => return Self::handle_key_code(model, code),
            Message::ServiceResp(a) => {
                if let Some(q) = model.pending_question.as_ref() {
                    model.history.push((q.clone(), a.clone()));
                } else {
                    warn!("received answer while no question is pending")
                }
                model.pending_question = None;
            }
            Message::SendQuestion => {
                // only send response if not waiting
                // TODO: implement timeout for pending resp
                if model.pending_question.is_none() {
                    let cmd = Command::ServiceReq(model.input.clone());
                    // move input to pending
                    model.pending_question = Some(model.input.clone());
                    model.input.clear();
                    // send cmd
                    return (None, Some(cmd));
                }
            }
            Message::CrosstermClose => {
                model.should_quit = true;
            }
        }
        (None, None)
    }

    fn handle_key_code(model: &mut Model, code: KeyCode) -> (Option<Message>, Option<Command>) {
        if model.is_editing {
            match code {
                KeyCode::Char(c) => model.input.push(c),
                KeyCode::Backspace => _ = model.input.pop(),
                KeyCode::Esc => model.is_editing = false,
                KeyCode::Enter => return (Some(Message::SendQuestion), None),
                _ => {}
            }
        } else {
            match code {
                KeyCode::Char('q') => model.should_quit = true,
                KeyCode::Char('i') => model.is_editing = true,
                _ => {}
            }
        }
        (None, None)
    }

    fn view(model: &mut Model, frame: &mut Frame) {
        let title = Line::from(" Cookie ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let mut lines = vec![];
        for (q, a) in &model.history {
            lines.push(Line::from(vec!["•: ".into(), q.clone().into()]));
            lines.push(Line::from(vec!["•: ".into(), a.clone().into()]));
            lines.push(Line::from("")); // blank line
        }
        if let Some(q) = model.pending_question.as_ref() {
            lines.push(Line::from(vec!["•: ".into(), q.into()]));
        }

        lines.push(Line::from(vec!["🚀: ".into(), model.input.clone().into()]));

        frame.render_widget(
            Paragraph::new(Text::from(lines)).centered().block(block),
            frame.area(),
        );
    }
}
