use crate::{
    app::model::setting_manager::SettingManager, service::llms::open_ai::api::OPENAI_MODELS,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, palette::tailwind},
    text::Line,
    widgets::{Block, Clear, List, ListItem, StatefulWidget, Widget},
};

const SELECTED_STYLE: Style = Style::new()
    .fg(tailwind::ROSE.c100)
    .bg(tailwind::GRAY.c800)
    .add_modifier(Modifier::BOLD);

impl Widget for &mut SettingManager {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // clears out the background
        Clear.render(area, buf);
        let block = Block::bordered().title(Line::from("Model").centered());

        let items: Vec<ListItem> = OPENAI_MODELS
            .iter()
            .map(|m| ListItem::from(m.display_name()))
            .collect();
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE);

        StatefulWidget::render(list, area, buf, self.list_state_mut());
    }
}
