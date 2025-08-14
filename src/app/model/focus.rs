use crate::app::model::Model;

pub trait Focusable {
    fn set_focus(&mut self, focused: bool);
    fn is_focused(&self) -> bool;
    fn to_focused(&self) -> Focused;
}

#[macro_export]
macro_rules! impl_focusable {
    ($ty:ty, $variant:expr) => {
        impl Focusable for $ty {
            fn set_focus(&mut self, focused: bool) {
                self.focused = focused
            }
            fn is_focused(&self) -> bool {
                self.focused
            }
            fn to_focused(&self) -> Focused {
                $variant
            }
        }
    };
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Focused {
    InputEditor,
    Messages,
    SessionManager,
}

impl From<&mut dyn Focusable> for Focused {
    fn from(v: &mut dyn Focusable) -> Focused {
        v.to_focused()
    }
}

impl Model {
    pub fn get_focused_idx(&mut self) -> Option<usize> {
        for i in 0..self.focus_order.len() {
            let handler = self.focus_order[i];
            let focused_variant = handler(self).to_focused();
            if focused_variant == self.focused {
                return Some(i);
            }
        }
        None
    }

    /// Cycles to the next widget in `focus_order`.
    pub fn shift_focus(&mut self) {
        if let Some(old_idx) = self.get_focused_idx() {
            self.focus_order[old_idx](self).set_focus(false);

            // turn on new
            let new_idx = (old_idx + 1) % self.focus_order.len();
            let handler = self.focus_order[new_idx];
            self.focused = handler(self).into();
            handler(self).set_focus(true);
        } else {
            tracing::error!("current focus not in focusable vector");
        }
    }

    /// Shifts focus to new focused.
    pub fn shift_focus_to(&mut self, new: Focused) {
        tracing::debug!("shift focus to {new:?}");
        if let Some(old_idx) = self.get_focused_idx() {
            self.focus_order[old_idx](self).set_focus(false);
        } else {
            tracing::error!("current focus not in focusable list");
            return;
        }

        // turn on new
        self.focused = new;
        if let Some(new_idx) = self.get_focused_idx() {
            let handler = self.focus_order[new_idx];
            handler(self).set_focus(true);
        } else {
            tracing::error!("new focus not in focusable vector");
        }
    }
}
