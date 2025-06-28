use crate::app::model::Model;
use ratatui::{
    Frame,
    style::Stylize as _,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

pub fn render_ui(model: &mut Model, frame: &mut Frame) {
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
