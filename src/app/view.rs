use crate::app::{components::chat::Chat, model::Model};
use ratatui::{
    Frame,
    style::Stylize as _,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};
use textwrap::wrap;

pub fn render_ui(model: &mut Model, frame: &mut Frame) {
    // let title = Line::from(" Cookie ".bold());
    // let block = Block::bordered()
    //     .title(title.centered())
    //     .border_set(border::THICK);

    frame.render_widget(
        Chat {
            history_messages: &model.history_messages,
            input: &model.input,
            pending_question: model.pending_question.as_deref(),
        },
        frame.area(),
    );
}
