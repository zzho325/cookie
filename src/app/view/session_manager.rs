use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Stylize, palette::tailwind},
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, StatefulWidget, Widget},
};

use crate::{
    app::model::{focus::Focusable, session_manager::SessionManager},
    models::SessionSummary,
};

impl From<&SessionSummary> for ListItem<'_> {
    fn from(value: &SessionSummary) -> Self {
        let title = if value.title.is_empty() {
            "New Chat".to_string()
        } else {
            value.title.clone()
        };

        let line = Line::from(title);

        ListItem::new(line)
    }
}

impl Widget for &mut SessionManager {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let styled_title = if self.is_focused() {
            "Sessions".fg(tailwind::AMBER.c400).bold()
        } else {
            "Sessions".fg(tailwind::AMBER.c300)
        };
        let block = Block::new()
            .borders(Borders::RIGHT)
            .title(Line::from(styled_title).centered());

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .session_summaries()
            .iter()
            .map(ListItem::from)
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            // .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, self.list_state());
    }
}
