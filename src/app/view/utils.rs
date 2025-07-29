use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// A centered rect of the given percentage.
pub fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = horizontal.areas(area);
    let [area] = vertical.areas(area);
    area
}
