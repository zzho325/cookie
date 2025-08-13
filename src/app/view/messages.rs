use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::app::{model::messages::Messages, view::widgets::scroll::AutoScroll};

impl Widget for &mut Messages {
    /// Renders history messages pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.set_viewport_width(area.width as usize);
        let styled_lines = self.viewport.lines();
        let lines: Vec<Line> = styled_lines.iter().map(|&l| Line::from(l)).collect();

        let text = Text::from(lines);
        let scrollable = AutoScroll::from(text).block(Block::new());
        scrollable.render(area, buf, self.viewport.scroll_state());
    }
}
