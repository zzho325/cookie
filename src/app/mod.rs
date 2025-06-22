use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures_util::stream::StreamExt;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize as _,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

pub struct App {
    // frontend <> backend channels
    req_tx: UnboundedSender<String>,
    resp_rx: UnboundedReceiver<String>,

    // prototyping
    question: String,
    answer: String,
    is_editing: bool,
    exit: bool,
}

impl App {
    pub fn new(
        req_tx: UnboundedSender<String>,
        resp_rx: UnboundedReceiver<String>,
    ) -> Result<Self> {
        Ok(Self {
            req_tx,
            resp_rx,
            question: String::new(),
            answer: String::new(),
            is_editing: true,
            exit: false,
        })
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let mut event_reader = EventStream::new();

        loop {
            // draw first so we see the latest state immediately
            terminal.draw(|f| self.draw(f))?;

            select! {
                // key event from Crossterm
                maybe_evt = event_reader.next() => {
                   let exit = self.handle_erossterm_event(maybe_evt)?;
                   if exit {break;}
                }
                // service response
                resp = self.resp_rx.recv() => {
                    match resp {
                        Some(ans) => {
                            self.answer = ans;
                        }
                        None => {
                            // TODO: service task hung up
                             break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// handle crossterm event and return true to exit
    pub fn handle_erossterm_event(
        &mut self,
        maybe_evt: Option<Result<Event, std::io::Error>>,
    ) -> Result<bool> {
        // If the stream closed, bail out
        let evt = match maybe_evt {
            Some(Ok(e)) => e,
            Some(Err(e)) => {
                // TODO: handle error
                eprintln!("event stream error: {e}");
                return Ok(false);
            }
            // TODO: when will None happen?
            None => return Ok(true),
        };

        // only care about key‐presses as crossterm also emits key release and repeat events on
        // Windows
        if let Event::Key(key) = evt {
            if key.kind == KeyEventKind::Press {
                self.handle_key_code(key.code)?;
                if self.exit {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn handle_key_code(&mut self, code: KeyCode) -> Result<()> {
        if self.is_editing {
            match code {
                KeyCode::Esc => {
                    self.is_editing = false;
                }
                KeyCode::Enter => {
                    self.req_tx.send(self.question.clone())?;
                    self.is_editing = false;
                }
                KeyCode::Char(c) => self.question.push(c),
                KeyCode::Backspace => {
                    self.question.pop();
                }
                _ => {}
            }
        } else {
            match code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Char('i') => self.is_editing = true,
                _ => {}
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Cookie ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let texts = Text::from(vec![
            Line::from(vec!["Q: ".into(), self.question.clone().into()]),
            Line::from(vec!["A: ".into(), self.answer.clone().into()]),
        ]);

        Paragraph::new(texts)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
