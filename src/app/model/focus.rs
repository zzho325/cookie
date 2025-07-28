use crate::app::model::Model;

pub trait Focusable {
    fn set_focus(&mut self, focused: bool);
    fn is_focused(&self) -> bool;
}

#[macro_export]
macro_rules! impl_focusable {
    ($ty:ty) => {
        impl Focusable for $ty {
            fn set_focus(&mut self, focused: bool) {
                self.focused = focused
            }
            fn is_focused(&self) -> bool {
                self.focused
            }
        }
    };
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Focused {
    Session,
    SessionManager,
}

impl From<usize> for Focused {
    fn from(v: usize) -> Self {
        const VARIANTS: [Focused; 2] = [Focused::Session, Focused::SessionManager];
        VARIANTS[v % VARIANTS.len()]
    }
}

impl Model {
    /// Cycle to the next widget in `focus_order`.
    pub fn shift_focus(&mut self) {
        // turn off old
        let old_idx = self.focused as usize;
        self.focus_order[old_idx](self).set_focus(false);
        // turn on new
        let new_idx = (old_idx + 1) % self.focus_order.len();
        self.focused = new_idx.into();
        self.focus_order[new_idx](self).set_focus(true);
    }

    // Shift focus to new focused.
    pub fn shift_focus_to(&mut self, new: Focused) {
        // turn off old
        let old_idx = self.focused as usize;
        self.focus_order[old_idx](self).set_focus(false);
        // turn on new
        self.focused = new;
        let new_idx = new as usize;
        self.focus_order[new_idx](self).set_focus(true);
    }
}
