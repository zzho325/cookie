use crate::app::model::focus::{Focusable, Focused};
use crate::chat::*;
use crate::impl_focusable;
use ratatui::widgets::ListState;

#[derive(Default)]
pub struct SessionManager {
    session_summaries: Vec<ChatSession>,
    list_state: ListState,
    focused: bool,
}

impl_focusable!(SessionManager, Focused::SessionManager);

impl SessionManager {
    pub fn session_summaries(&self) -> &[ChatSession] {
        &self.session_summaries
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn select_next(&mut self) -> Option<String> {
        match self.list_state.selected() {
            Some(i) if i + 1 < self.session_summaries.len() => {
                self.list_state.select_next();
                self.session_summaries.get(i + 1).map(|s| s.id.clone())
            }
            // select first if no current selection
            None => {
                self.list_state.select(Some(0));
                self.session_summaries.first().map(|s| s.id.clone())
            }
            _ => None,
        }
    }

    pub fn select_previous(&mut self) -> Option<String> {
        match self.list_state.selected() {
            Some(i) if i > 0 => {
                self.list_state.select_previous();
                self.session_summaries.get(i - 1).map(|s| s.id.clone())
            }
            _ => None,
        }
    }

    /// Selects the list item with id `session_id` if provided, and None otherwise.
    pub fn set_selected(&mut self, session_id: Option<String>) {
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
        session_summaries: Vec<ChatSession>,
        session_id: Option<String>,
    ) {
        self.session_summaries = session_summaries;
        self.session_summaries.sort_by(|a, b| {
            b.updated_at
                .as_ref()
                .map(|t| (t.seconds, t.nanos))
                .cmp(&a.updated_at.as_ref().map(|t| (t.seconds, t.nanos)))
        });

        self.set_selected(session_id);
    }

    /// Updates title of given session summary.
    pub fn handle_session_summary(&mut self, session_summary: ChatSession) {
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
