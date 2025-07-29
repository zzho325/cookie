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
    fn prefix(
        settings: &LlmSettings,
        created_at: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Vec<Vec<Span>> {
        let provider = settings.provider_name();
        let model = settings.model_name();
        // compute elapsed seconds
        let elapsed = if let Some((req_at, resp_at)) = created_at {
            let elapsed_secs = (resp_at - req_at).num_seconds();
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

impl Widget for &Messages {
    /// Renders history messages pane.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // history messages
        let mut messages = Text::raw("");

        for chunk in self.chat_messages().chunks_exact(2) {
            let user_message: &ChatMessage = &chunk[0];
            let assistant_message: &ChatMessage = &chunk[1];

            let prefix = Messages::prefix(
                user_message.llm_settings(),
                Some((*user_message.created_at(), *assistant_message.created_at())),
            );
            let lines = prefix
                .iter()
                .enumerate()
                .map(|(i, base)| {
                    let mut spans = base.clone();
                    if i == 1 {
                        spans.push(Span::raw(user_message.payload().msg.clone()));
                    }
                    Line::from(spans)
                })
                .collect::<Vec<_>>();
            messages.extend(Text::from(lines));

            // llm response
            messages.extend(tui_markdown::from_str(&assistant_message.payload().msg));
            messages.extend(Text::from(""));
        }
        if let Some((user_message, settings)) = self.pending() {
            let prefix = Messages::prefix(settings, None);
            let lines = prefix
                .iter()
                .enumerate()
                .map(|(i, base)| {
                    let mut spans = base.clone();
                    if i == 1 {
                        spans.push(Span::raw(user_message.payload().msg.clone()));
                    }
                    Line::from(spans)
                })
                .collect::<Vec<_>>();
            messages.extend(Text::from(lines));
        }

        Paragraph::new(messages)
            .wrap(Wrap { trim: false })
            .scroll(self.scroll_state().scroll_offset())
            .render(area, buf);
    }
}
