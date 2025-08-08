use crate::app::model::focus::{Focusable, Focused};
use crate::impl_focusable;
use crate::models::SessionSummary;
use ratatui::widgets::ListState;

#[derive(Default)]
pub struct SessionManager {
    session_summaries: Vec<SessionSummary>,
    list_state: ListState,
    focused: bool,
}

impl_focusable!(SessionManager, Focused::SessionManager);

impl SessionManager {
    pub fn session_summaries(&self) -> &[SessionSummary] {
        &self.session_summaries
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn select_next(&mut self) -> Option<uuid::Uuid> {
        match self.list_state.selected() {
            Some(i) if i + 1 < self.session_summaries.len() => {
                self.list_state.select_next();
                self.session_summaries.get(i + 1).map(|s| s.id)
            }
            _ => None,
        }
    }

    pub fn select_previous(&mut self) -> Option<uuid::Uuid> {
        match self.list_state.selected() {
            Some(i) if i > 0 => {
                self.list_state.select_previous();
                self.session_summaries.get(i - 1).map(|s| s.id)
            }
            _ => None,
        }
    }

    /// Selects the list item with id `session_id` if provided, and None otherwise.
    pub fn set_selected(&mut self, session_id: Option<uuid::Uuid>) {
        let selected_list_idx =
            session_id.and_then(|id| self.session_summaries.iter().position(|s| s.id == id));
        self.list_state.select(selected_list_idx);
    }

    // ----------------------------------------------------------------
    // Event handlers.
    // ----------------------------------------------------------------

    /// Replaces current `session_summaries` with updated list while keeping sorted order and
    /// select item with id `selected_session_id`.
    pub fn handle_session_summaries(
        &mut self,
        session_summaries: Vec<SessionSummary>,
        session_id: Option<uuid::Uuid>,
    ) {
        self.session_summaries = session_summaries;
        self.session_summaries
            .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        self.set_selected(session_id);
    }

    /// Updates title of given session summary.
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
