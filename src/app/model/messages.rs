#[derive(Default)]
pub struct Messages {
    pub history_messages: Vec<(String, String)>, // (queston, answer)
    pub pending_question: Option<String>,
}

impl Messages {
    pub fn append_message(&mut self, a: String) {
        if let Some(q) = self.pending_question.as_ref() {
            self.history_messages.push((q.clone(), a));
            self.pending_question = None;
        } else {
            // TODO: report error
            tracing::warn!("received answer while no question is pending")
        }
    }

    pub fn send_question(&mut self, q: &str) {
        self.pending_question = Some(q.to_string());
    }

    pub fn is_pending_resp(&self) -> bool {
        self.pending_question.is_some()
    }

    pub fn history_messages(&self) -> &[(String, String)] {
        &self.history_messages
    }

    pub fn pending_question(&self) -> Option<&str> {
        self.pending_question.as_deref()
    }
}
