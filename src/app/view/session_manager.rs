use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListItem, StatefulWidget, Widget},
};

use crate::{app::model::session_manager::SessionManager, models::SessionSummary};

impl From<&SessionSummary> for ListItem<'_> {
    fn from(value: &SessionSummary) -> Self {
        let summary = if value.summary == "" {
            "New Chat".to_string()
        } else {
            value.summary.clone()
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
            .map(|(i, session_summary)| {
                // let color = alternate_colors(i);
                ListItem::from(session_summary)
            })
            .collect();

        tracing::debug!("{items:?}");
        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            // .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, self.list_state());
    }
}
