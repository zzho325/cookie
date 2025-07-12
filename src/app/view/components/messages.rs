use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::app::model::{
    messages::{HistoryMessage, MessageMetadata, Messages, PendingMessage},
    scroll::Scrollable as _,
};

pub struct MessagesView<'a> {
    pub history_messages: &'a [HistoryMessage],
    pub pending_question: Option<&'a PendingMessage>,
    pub scroll_offset: (u16, u16),
}

impl MessagesView<'_> {
    /// Generate prefix spans from message metadata.
    // TODO: handle narrow width, clean up spans and add unit test
    fn prefix(metadata: &MessageMetadata) -> Vec<Vec<Span>> {
        let provider = metadata.llm.provider_name();
        let model = metadata.llm.model_name();
        // compute elapsed seconds
        let elapsed = if let Some(resp_time) = metadata.resp_time {
            let elapsed_secs = resp_time.duration_since(metadata.req_time).as_secs();
            format!("{}s", elapsed_secs)
        } else {
            "-".to_string()
        };

        vec![
            vec![
                Span::raw("┌─> "),
                Span::styled(
                    provider,
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" on "),
                Span::styled(model, Style::default().fg(Color::LightBlue)),
                Span::raw(" ["),
                Span::styled(elapsed, Style::default().fg(Color::LightMagenta)),
                Span::raw("]"),
            ],
            vec![Span::raw("└─> ")],
        ]
    }
}

impl Widget for MessagesView<'_> {
    /// Renders history passanges pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // history messages
        let mut messages = Text::raw("");

        for HistoryMessage {
            user_msg,
            assistant_msg,
            metadata,
        } in self.history_messages
        {
            let prefix = MessagesView::prefix(metadata);
            let lines = prefix
                .iter()
                .enumerate()
                .map(|(i, base)| {
                    let mut spans = base.clone();
                    if i == 1 {
                        spans.push(Span::raw(user_msg));
                    }
                    Line::from(spans)
                })
                .collect::<Vec<_>>();
            messages.extend(Text::from(lines));

            // llm response
            messages.extend(tui_markdown::from_str(assistant_msg));
            messages.extend(Text::from(""));
        }
        if let Some(PendingMessage { user_msg, metadata }) = self.pending_question {
            let prefix = MessagesView::prefix(metadata);
            let lines = prefix
                .iter()
                .enumerate()
                .map(|(i, base)| {
                    let mut spans = base.clone();
                    if i == 1 {
                        spans.push(Span::raw(user_msg));
                    }
                    Line::from(spans)
                })
                .collect::<Vec<_>>();
            messages.extend(Text::from(lines));
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
