use ratatui::widgets::ListState;

use crate::models::SessionSummary;

#[derive(Default)]
pub struct SessionManager {
    session_summaries: Vec<SessionSummary>,
    list_state: ListState,
}

impl SessionManager {
    pub fn session_summaries(&self) -> &[SessionSummary] {
        &self.session_summaries
    }

    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn handle_sessions_update(&mut self, session_summaries: Vec<SessionSummary>) {
        tracing::debug!("sessions {session_summaries:?}");
        self.session_summaries = session_summaries;
        self.session_summaries
            .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    }
}
