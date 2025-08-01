use chrono::{DateTime, Utc};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    app::model::messages::Messages,
    models::{ChatMessage, settings::LlmSettings},
};

impl Messages {
    /// Generate prefix spans from message metadata.
    // TODO: handle narrow width, clean up spans and add unit test
    fn prefix(settings: &LlmSettings, elapsed_secs: Option<i64>) -> Vec<Vec<Span>> {
        let provider = settings.provider_name();
        let model = settings.model_name();
        // compute elapsed seconds
        let elapsed = elapsed_secs.map_or_else(|| "-".to_string(), |s| format!("{s}s"));

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

impl Widget for &Messages {
    /// Renders history messages pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut messages = Text::raw("");

        // history messages
        let mut iter = self.chat_messages().iter().peekable();
        while let Some(chat_message) = iter.next() {
            match chat_message.payload().role {
                crate::models::Role::User => {
                    // calculate elapsed duration if next message is from assistant
                    let start = *chat_message.created_at();
                    let elapsed_secs = iter
                        .peek()
                        .map(|next| (*next.created_at() - start).num_seconds());
                    let prefix = Messages::prefix(chat_message.llm_settings(), elapsed_secs);
                    let lines = prefix
                        .iter()
                        .enumerate()
                        .map(|(i, base)| {
                            let mut spans = base.clone();
                            if i == 1 {
                                spans.push(Span::raw(chat_message.payload().msg.clone()));
                            }
                            Line::from(spans)
                        })
                        .collect::<Vec<_>>();
                    messages.extend(Text::from(lines));
                }
                crate::models::Role::Assistant => {
                    messages.extend(tui_markdown::from_str(&chat_message.payload().msg));
                    messages.extend(Text::from(""));
                }
            }
        }

        // stream in progress
        if let Some(stream_message) = self.stream_message() {
            messages.extend(tui_markdown::from_str(&stream_message.delta));
        }

        Paragraph::new(messages)
            .wrap(Wrap { trim: false })
            .scroll(self.scroll_state().scroll_offset())
            .render(area, buf);
    }
}
