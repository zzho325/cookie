use ratatui::prelude::BlockExt;
use ratatui::widgets::Paragraph;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Block, ScrollbarState, StatefulWidget, Widget},
};

#[derive(Default, Clone)]
pub struct ScrollState {
    pub vertical_scroll_offset: usize,
    pub vertical_scroll_bar_state: ScrollbarState,
    pub cursor_position: Option<(u16 /*x*/, u16 /*y*/)>,
}

impl ScrollState {
    pub fn with_cursor(mut self) -> Self {
        self.cursor_position = Some((0, 0));
        self
    }

    pub fn scroll_offset(&self) -> (u16, u16) {
        (self.vertical_scroll_offset as u16, 0)
    }

    pub fn cursor_position(&self) -> Option<(u16, u16)> {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, cursor_position: (u16 /*x*/, u16 /*y*/)) {
        self.cursor_position = Some(cursor_position);
    }

    pub fn cursor_viewport_position(&self) -> Option<(u16, u16)> {
        self.cursor_position
            .map(|(x, y)| (x, y - self.vertical_scroll_offset as u16))
    }

    pub fn scroll_down(&mut self) {
        self.vertical_scroll_offset = self.vertical_scroll_offset.saturating_add(1);
        self.vertical_scroll_bar_state = self
            .vertical_scroll_bar_state
            .position(self.vertical_scroll_offset);
    }

    pub fn scroll_up(&mut self) {
        self.vertical_scroll_offset = self.vertical_scroll_offset.saturating_sub(1);
        self.vertical_scroll_bar_state = self
            .vertical_scroll_bar_state
            .position(self.vertical_scroll_offset);
    }

    /// Scrolls just enough so that line at height is visible.
    pub fn ensure_line_visible(&mut self, height: usize) {
        if let Some((_, y)) = self.cursor_position {
            let line = y as usize;
            if line < self.vertical_scroll_offset {
                self.vertical_scroll_offset = line;
            } else if line >= self.vertical_scroll_offset + height {
                self.vertical_scroll_offset = (line + 1).saturating_sub(height);
            }
            self.vertical_scroll_bar_state = self
                .vertical_scroll_bar_state
                .position(self.vertical_scroll_offset);
        }
    }

    pub fn reset(&mut self) {
        self.vertical_scroll_offset = 0;
        self.vertical_scroll_bar_state = self
            .vertical_scroll_bar_state
            .position(self.vertical_scroll_offset);
    }
}

pub struct AutoScroll<'a> {
    /// An optional block to wrap the widget in.
    pub(crate) block: Option<Block<'a>>,
    pub content: Text<'a>,
}

impl<'a> AutoScroll<'a> {
    /// Creates a new Scrollable from any value that can be converted to Text.
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            block: None,
            content: content.into(),
        }
    }

    /// Wraps the list with a custom [`Block`] widget.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a, T> From<T> for AutoScroll<'a>
where
    T: Into<Text<'a>>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl StatefulWidget for AutoScroll<'_> {
    type State = ScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(block) = &self.block {
            block.render(area, buf);
        }
        let inner_area = self.block.inner_if_some(area);
        state.ensure_line_visible(inner_area.height as usize);
        Widget::render(
            Paragraph::new(self.content).scroll(state.scroll_offset()),
            inner_area,
            buf,
        );
    }
}
