pub mod editor;
pub mod messages;
pub mod scroll;
pub mod session;
pub mod session_manager;

use std::default;

use crate::{
    app::model::{session::Session, session_manager::SessionManager},
    models::{LlmSettings, configs::Configs},
};

#[derive(Debug, PartialEq)]
pub enum Focused {
    SessionManager,
    Session,
}

pub struct Model {
    pub configs: Configs,
    pub session: Session,
    pub session_manager: SessionManager,

    pub show_sidebar: bool,
    pub focused: Focused,
    pub should_quit: bool,
}

impl Model {
    pub fn new(configs: Configs) -> Self {
        // FIXME: fix config usage
        let default_llm_settings = configs.derive_llm_settings();
        Self {
            configs,
            session: Session::new(default_llm_settings),
            session_manager: SessionManager::default(),
            focused: Focused::Session,
            show_sidebar: false,
            should_quit: false,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn shift_focus(&mut self) {
        match self.focused {
            Focused::SessionManager => self.focused = Focused::Session,
            Focused::Session => self.focused = Focused::SessionManager,
        }
    }

    pub fn shift_focus_to(&mut self, new_focused: Focused) {
        self.focused = new_focused;
    }

    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
    }
}
