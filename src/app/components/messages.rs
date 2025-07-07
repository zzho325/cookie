use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::app::model::messages::Messages;

pub struct MessagesView<'a> {
    pub history_messages: &'a [(String, String)],
    pub pending_question: Option<&'a str>,
}

impl Widget for MessagesView<'_> {
    /// Renders history passanges pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // history messages
        let mut messages = Text::raw("");
        for (q, a) in self.history_messages {
            messages.extend(Text::from(q.clone()));
            messages.extend(tui_markdown::from_str(a));
        }
        tracing::debug!("{messages:?}");
        if let Some(q) = self.pending_question {
            messages.extend(Text::from(q));
        }

        Paragraph::new(messages)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl<'a> From<&'a Messages> for MessagesView<'a> {
    fn from(messages: &'a Messages) -> Self {
        MessagesView {
            history_messages: &messages.history_messages(),
            pending_question: messages.pending_question(),
        }
    }
}
