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
    /// Returns if a mouse event is within the area.
    pub fn contains_mouse_event(&self, evt: MouseEvent) -> bool {
        if evt.row < self.row || evt.row >= self.row + self.height {
            return false;
        }
        if evt.column < self.column || evt.column >= self.column + self.width {
            return false;
        }
        true
    }
}
