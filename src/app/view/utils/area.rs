use crossterm::event::MouseEvent;

#[derive(Default, Clone, Debug)]
pub struct Area {
    /// The starting column position of the area.
    pub column: u16,
    /// The starting row position of the area.
    pub row: u16,
    /// The vertical extent or height of the area.
    pub height: u16,
    /// The horizontal extent or width of the area.
    pub width: u16,
}

impl Area {
    /// Returns a mouse event shifted w.r.t. the area if it's within the area. Returns None
    /// otherwise.
    pub fn maybe_mouse_event(&self, evt: MouseEvent) -> Option<MouseEvent> {
        if evt.row < self.row || evt.row >= self.row + self.height {
            return None;
        }

        if evt.column < self.column || evt.column >= self.column + self.width {
            return None;
        }

        let mut evt = evt;
        evt.row -= self.row;
        evt.column -= self.column;
        Some(evt)
    }
}
