use crate::app::model::setting_manager::SettingManager;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, palette::tailwind},
    widgets::{Block, Borders, Widget},
};

impl Widget for &mut SettingManager {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Setting")
            .borders(Borders::NONE)
            .style(Style::default().bg(tailwind::AMBER.c400));
        block.render(area, buf);
    }
}
