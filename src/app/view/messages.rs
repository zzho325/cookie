use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::app::model::messages::Messages;

impl Widget for &mut Messages {
    /// Renders history messages pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.set_viewport_width(area.width as usize);
        let mut messages = Text::raw("");
        let styled_lines = self.viewport.lines();
        let lines: Vec<Line> = styled_lines.iter().map(Line::from).collect();
        messages.extend(lines);

        Paragraph::new(messages)
            .wrap(Wrap { trim: false })
            .scroll(self.scroll_state().scroll_offset())
            .render(area, buf);
    }
}
