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

use crate::app::types::{Command, Message};

#[derive(Debug, Default)]
pub struct Model {
    should_quit: bool,

    question: String,
    answer: String,
    is_editing: bool,
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
            Message::Key(code) => Self::handle_key_code(model, code),
            Message::ServiceResp(msg) => {
                model.answer = msg.clone();
                (None, None)
            }
            Message::CrosstermClose => {
                model.should_quit = true;
                (None, None)
            }
        }
    }

    fn handle_key_code(model: &mut Model, code: KeyCode) -> (Option<Message>, Option<Command>) {
        if model.is_editing {
            match code {
                KeyCode::Esc => {
                    model.is_editing = false;
                }
                KeyCode::Enter => {
                    model.is_editing = false;
                    return (None, Some(Command::ServiceReq(model.question.clone())));
                }
                KeyCode::Char(c) => model.question.push(c),
                KeyCode::Backspace => {
                    model.question.pop();
                }
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

        let texts = Text::from(vec![
            Line::from(vec!["Q: ".into(), model.question.clone().into()]),
            Line::from(vec!["A: ".into(), model.answer.clone().into()]),
        ]);

        frame.render_widget(Paragraph::new(texts).centered().block(block), frame.area());
    }
}
