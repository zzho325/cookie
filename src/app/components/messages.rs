use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Span, Text},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::app::model::{messages::Messages, scroll::Scrollable as _};

pub struct MessagesView<'a> {
    pub history_messages: &'a [(String, String)],
    pub pending_question: Option<&'a str>,
    pub scroll_offset: (u16, u16),
}

impl Widget for MessagesView<'_> {
    /// Renders history passanges pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // history messages
        let mut messages = Text::raw("");

        let sep = Span::styled(
            "─".repeat(area.width as usize),
            Style::default().add_modifier(Modifier::DIM),
        );

        for (idx, (q, a)) in self.history_messages.iter().enumerate() {
            // user question
            if idx != 0 {
                messages.extend(Text::from(sep.clone()));
            }
            messages.extend(Text::from(q.clone()));

            // llm response
            messages.extend(Text::from(sep.clone()));
            messages.extend(tui_markdown::from_str(a));
        }
        if let Some(q) = self.pending_question {
            messages.extend(Text::from(sep.clone()));
            messages.extend(Text::from(q));
        }

        Paragraph::new(messages)
            .wrap(Wrap { trim: false })
            .scroll(self.scroll_offset)
            .render(area, buf);
    }
}

impl<'a> From<&'a Messages> for MessagesView<'a> {
    fn from(messages: &'a Messages) -> Self {
        MessagesView {
            history_messages: messages.history_messages(),
            pending_question: messages.pending_question(),
            scroll_offset: messages.scroll_offset(),
        }
    }
}
