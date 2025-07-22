use ratatui::widgets::ListState; 
use crate::models::SessionSummary;
use crate::app::model::focus::Focusable;

#[derive(Default)]
pub struct SessionManager {
    session_summaries: Vec<SessionSummary>,
    list_state: ListState,
    focused: bool,
}

crate::impl_focusable!(SessionManager);

// impl_focusable!(Session);
impl SessionManager {
    pub fn session_summaries(&self) -> &[SessionSummary] {
        &self.session_summaries
    }

    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    /// Sets `session_summaries` and optionally set summary with id `session_id` as selected.
    pub fn handle_sessions_update(
        &mut self,
        session_summaries: Vec<SessionSummary>,
        session_id: Option<uuid::Uuid>,
    ) {
        tracing::debug!("sessions {session_summaries:?}");
        self.session_summaries = session_summaries;
        self.session_summaries
            .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        let selected =
            session_id.and_then(|id| self.session_summaries.iter().position(|s| s.id == id));
        self.list_state.select(selected);
    }

    pub fn select_next(&mut self) {
        if self
            .list_state
            .selected()
            .is_some_and(|i| i.saturating_add(1) < self.session_summaries.len())
        {
            self.list_state.select_next();
        }
    }

    pub fn select_previous(&mut self) {
        self.list_state().select_previous();
    }

    pub fn selected(&self) -> Option<uuid::Uuid> {
        self.list_state
            .selected()
            .map(|i| self.session_summaries[i].id)
    }
}
