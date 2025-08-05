use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::app::model::messages::Messages;

impl Widget for &mut Messages {
    /// Renders history messages pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.set_viewport_width(area.width as usize);
        let styled_lines = self.viewport.lines();
        let lines: Vec<Line> = styled_lines.iter().map(Line::from).collect();
        let messages = lines;

        Paragraph::new(messages)
            .scroll(self.viewport.scroll_state().scroll_offset())
            .render(area, buf);
    }
}
