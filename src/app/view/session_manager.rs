use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, Stylize, palette::tailwind},
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, StatefulWidget},
};

use crate::app::model::focus::Focusable;
use crate::{
    app::{model::session_manager::SessionManager, view::utils::area::Area},
    chat::ChatSession,
    models::constants::NEW_SESSION_TITLE,
};

impl From<&ChatSession> for ListItem<'_> {
    fn from(value: &ChatSession) -> Self {
        let title = if value.title.is_empty() {
            NEW_SESSION_TITLE.to_string()
        } else {
            value.title.clone()
        };

        let line = Line::from(title);

        ListItem::new(line)
    }
}
const SELECTED_STYLE: Style = Style::new()
    .bg(tailwind::ZINC.c200)
    .add_modifier(Modifier::BOLD);

impl StatefulWidget for &mut SessionManager {
    type State = Area;

    fn render(self, area: Rect, buf: &mut Buffer, state_area: &mut Area) {
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
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, self.list_state_mut());
        state_area.height = area.height;
        state_area.width = area.width - 1;
    }
}
