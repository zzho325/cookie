pub mod editor;
pub mod focus;
pub mod messages;
pub mod session;
pub mod session_manager;

use crate::{
    app::model::{
        focus::{Focusable, Focused},
        session::Session,
        session_manager::SessionManager,
    },
    models::configs::Configs,
};

pub struct Model {
    pub configs: Configs,
    pub session: Session,
    pub session_manager: SessionManager,

    pub show_sidebar: bool,
    pub focused: Focused,
    focus_order: Vec<fn(&mut Model) -> &mut dyn Focusable>,
    pub should_quit: bool,
}

impl Model {
    pub fn new(configs: Configs) -> Self {
        // FIXME: fix config usage
        let default_llm_settings = configs.derive_llm_settings();

        let mut this = Self {
            configs,
            session: Session::new(default_llm_settings),
            session_manager: SessionManager::default(),
            show_sidebar: false,
            should_quit: false,
            focused: Focused::Session,
            focus_order: Vec::new(),
        };

        this.focus_order.push(|m| &mut m.session);
        this.focus_order.push(|m| &mut m.session_manager);
        this.focus_order[0](&mut this).set_focus(true);
        this
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_sidebar(&mut self) {
        if !self.show_sidebar {
            self.shift_focus_to(Focused::SessionManager);
        } else {
            self.shift_focus_to(Focused::Session);
        }
        self.show_sidebar = !self.show_sidebar;
    }
}
