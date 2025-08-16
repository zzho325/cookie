use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Stylize, palette::tailwind},
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget},
};

pub struct ErrorPopup<'a> {
    msg: &'a str,
}

impl<'a> ErrorPopup<'a> {
    pub fn new(msg: &'a str) -> Self {
        Self { msg }
    }
}

impl Widget for ErrorPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // clears out the background
        Clear.render(area, buf);

        let block = Block::bordered().title(Line::from("Error Message").centered());
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Inner area
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
        let [message_area, footnote_area] = vertical.areas(inner_area);

        let message = Paragraph::new(self.msg);
        message.render(message_area, buf);

        let footnote = Line::from("Press any key to exit")
            .fg(tailwind::GRAY.c800)
            .centered();
        footnote.render(footnote_area, buf);
    }
}
