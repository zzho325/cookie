use ratatui::widgets::ScrollbarState;

pub trait Scrollable {
    fn scroll_state(&mut self) -> &mut ScrollState;

    fn scroll_offset(&self) -> (u16 /*y*/, u16 /*x*/);

    fn scroll_up(&mut self) {
        self.scroll_state().scroll_up()
    }

    fn scroll_down(&mut self) {
        self.scroll_state().scroll_down()
    }
}

#[derive(Default)]
pub struct ScrollState {
    pub vertical_scroll_offset: usize,
    pub vertical_scroll_bar_state: ScrollbarState,
}

impl ScrollState {
    fn scroll_down(&mut self) {
        self.vertical_scroll_offset = self.vertical_scroll_offset.saturating_add(1);
        self.vertical_scroll_bar_state = self
            .vertical_scroll_bar_state
            .position(self.vertical_scroll_offset);
    }

    fn scroll_up(&mut self) {
        self.vertical_scroll_offset = self.vertical_scroll_offset.saturating_sub(1);
        self.vertical_scroll_bar_state = self
            .vertical_scroll_bar_state
            .position(self.vertical_scroll_offset);
    }

    pub fn scroll_offset(&self) -> (u16, u16) {
        (self.vertical_scroll_offset as u16, 0)
    }
}
