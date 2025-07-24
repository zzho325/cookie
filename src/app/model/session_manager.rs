use crate::app::model::focus::Focusable;
use crate::models::SessionSummary;
use ratatui::widgets::ListState;

#[derive(Default)]
pub struct SessionManager {
    session_summaries: Vec<SessionSummary>,
    list_state: ListState,
    focused: bool,
}

crate::impl_focusable!(SessionManager);

impl SessionManager {
    pub fn session_summaries(&self) -> &[SessionSummary] {
        &self.session_summaries
    }

    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
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

    /// Updates selected list item to item of session_id if exists, and None otherwise.
    pub fn set_selected(&mut self, session_id: Option<uuid::Uuid>) {
        let selected =
            session_id.and_then(|id| self.session_summaries.iter().position(|s| s.id == id));
        self.list_state.select(selected);
    }

    // ----------------------------------------------------------------
    // Event handlers.
    // ----------------------------------------------------------------

    /// Replaces current `session_summaries` with updated list while keeping sorted order and
    /// current `session_id` selection.
    pub fn handle_session_summaries(&mut self, session_summaries: Vec<SessionSummary>) {
        tracing::debug!("sessions {session_summaries:?}");
        let selected_id = self.selected();

        self.session_summaries = session_summaries;
        self.session_summaries
            .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        self.set_selected(selected_id);
    }

    /// Updates title of given session summa.ry
    pub fn handle_session_summary(&mut self, session_summary: SessionSummary) {
        tracing::debug!("updating session manager with {session_summary:?}");
        if let Some(current) = self
            .session_summaries
            .iter_mut()
            .find(|s| s.id == session_summary.id)
        {
            current.title = session_summary.title;
        }
    }
}
