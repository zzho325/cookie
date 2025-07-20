use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListItem, StatefulWidget, Widget},
};

use crate::{app::model::session_manager::SessionManager, models::SessionSummary};

impl From<&SessionSummary> for ListItem<'_> {
    fn from(value: &SessionSummary) -> Self {
        let summary = if value.title == "" {
            "New Chat".to_string()
        } else {
            value.title.clone()
        };

        let line = Line::from(format!("{}", summary));

        ListItem::new(line)
    }
}

impl Widget for &mut SessionManager {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::new().title(Line::raw("Sessions").centered());

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .session_summaries()
            .iter()
            .enumerate()
            .map(|(_, title)| ListItem::from(title))
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
