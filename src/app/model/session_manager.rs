use ratatui::widgets::ListState;

use crate::{app::update::Update, models::SessionSummary};

pub struct SessionManager {
    session_summaries: Vec<SessionSummary>,
    list_state: ListState,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            session_summaries: Vec::new(),
            list_state: ListState::default(),
        }
    }

    pub fn session_summaries(&self) -> &[SessionSummary] {
        &self.session_summaries
    }

    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn update(&mut self, session_summaries: Vec<SessionSummary>) -> Update {
        tracing::debug!("sessions {session_summaries:?}");
        self.session_summaries = session_summaries;
        self.session_summaries
            .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        (None, None)
    }
}
