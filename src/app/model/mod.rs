pub mod editor;
pub mod messages;
pub mod scroll;
pub mod session;
pub mod session_manager;

use crate::{
    app::model::{session::Session, session_manager::SessionManager},
    models::LlmSettings,
};

#[derive(Debug, PartialEq)]
pub enum Focused {
    SessionManager,
    Session,
}

pub struct Model {
    pub default_llm_settings: LlmSettings,
    pub session: Session,
    pub session_manager: SessionManager,

    pub show_sidebar: bool,
    pub focused: Focused,
    pub should_quit: bool,
}

impl Model {
    pub fn new(default_llm_settings: LlmSettings) -> Self {
        Self {
            default_llm_settings: default_llm_settings.clone(),
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

    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
    }
}
