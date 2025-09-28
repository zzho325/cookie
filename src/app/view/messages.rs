use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Stylize as _, palette::tailwind},
    text::{Line, Text},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{
    app::{
        model::{focus::Focusable, messages::Messages},
        view::widgets::scroll::AutoScroll,
    },
    models::constants::NEW_SESSION_TITLE,
};

impl Widget for &mut Messages {
    /// Renders history messages pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // border with title
        let title = self
            .title()
            .cloned()
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| NEW_SESSION_TITLE.to_string());

        let styled_title = if self.is_focused() {
            title.fg(tailwind::AMBER.c400).bold()
        } else {
            title.fg(tailwind::AMBER.c300)
        };
        let block = Block::new().title(Line::from(styled_title).centered());

        self.set_viewport_width(area.width as usize);
        let styled_lines = self.viewport.lines();
        let lines: Vec<Line> = styled_lines.iter().map(Line::from).collect();

        let text = Text::from(lines);
        let scrollable = AutoScroll::from(text).block(block);
        scrollable.render(area, buf, self.viewport.scroll_state());
    }
}
