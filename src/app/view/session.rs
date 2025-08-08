use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::tailwind},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, StatefulWidget, Widget},
};

use crate::{
    app::{
        model::{focus::Focusable, session::Session},
        view::{
            constants::{
                BORDER_THICKNESS, BORDER_THICKNESS_SIDE, MAX_INPUT_RATIO, MIN_INPUT_HEIGHT,
            },
            widgets::scroll::AutoScroll,
        },
    },
    models::constants::NEW_SESSION_TITLE,
};

pub struct SessionState {
    pub cursor_position: Option<(u16, u16)>,
}

impl StatefulWidget for &mut Session {
    type State = SessionState;

    /// Renders chat session with input block starting with MIN_INPUT_HEIGHT including border and
    /// increase height as input length increases with a maximum of MAX_INPUT_RATIO of widget area.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut SessionState) {
        // border with title
        let title = self
            .title()
            .map(String::as_str)
            .unwrap_or(NEW_SESSION_TITLE);
        let styled_title = if self.messages.is_focused() {
            title.fg(tailwind::AMBER.c400).bold()
        } else {
            title.fg(tailwind::AMBER.c300)
        };
        let block = Block::new().title(Line::from(styled_title).centered());
        let inner_area = block.inner(area);
        block.render(area, buf);

        // ----------------------------------------------------------------
        // Dynamic height and split area
        // ----------------------------------------------------------------

        // input width
        let input_content_width = inner_area.width.saturating_sub(BORDER_THICKNESS as u16) as usize;
        self.input_editor.set_viewport_width(input_content_width);

        // input height
        let max_input_height = (inner_area.height as f32 * MAX_INPUT_RATIO).floor() as usize;
        let lines = self.input_editor.viewport.lines();
        let input_height =
            (lines.len() + BORDER_THICKNESS_SIDE).clamp(MIN_INPUT_HEIGHT, max_input_height) as u16;

        // message height
        let message_height = inner_area.height.saturating_sub(input_height);

        let [message_area, input_area] = Layout::vertical([
            Constraint::Length(message_height),
            Constraint::Length(input_height),
        ])
        .areas(inner_area);

        self.messages.render(message_area, buf);

        // ----------------------------------------------------------------
        // Input
        // ----------------------------------------------------------------

        // TODO: centralize style here and prompt style
        let provider = self.llm_settings().provider_name();
        let model = self.llm_settings().model_name();
        let title: Line = Line::from(vec![
            Span::raw("─ "),
            Span::styled(
                provider,
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" on "),
            Span::styled(model, Style::default().fg(Color::LightBlue)),
            Span::raw(" "),
        ]);

        let input_lines: Vec<Line> = lines.into_iter().map(Line::from).collect();
        let text = Text::from(input_lines);
        let scrollable = AutoScroll::from(text).block(
            Block::new()
                .borders(Borders::TOP)
                .padding(Padding::horizontal(1))
                .title(title.left_aligned()),
        );
        scrollable.render(input_area, buf, self.input_editor.viewport.scroll_state());

        // ----------------------------------------------------------------
        // Cursor position
        // ----------------------------------------------------------------

        state.cursor_position = if self.input_editor.is_editing() {
            self.input_editor
                .viewport
                .scroll_state()
                .cursor_viewport_position()
                .map(|(x, y)| {
                    (
                        x + BORDER_THICKNESS_SIDE as u16,
                        y + message_height + BORDER_THICKNESS_SIDE as u16 * 2,
                    )
                })
        } else {
            None
        };
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use uuid::Uuid;

    use crate::{
        app::{model::messages::Messages, view::session::SessionState},
        models::{ChatMessage, Role},
    };

    #[test]
    fn render_session() {
        let user_message_created_at = chrono::Utc.with_ymd_and_hms(2025, 7, 10, 0, 0, 0).unwrap();
        let assistant_message_created_at =
            user_message_created_at + std::time::Duration::from_secs(2);

        let llm_settings = crate::models::settings::LlmSettings::OpenAI {
            model: crate::service::llms::open_ai::api::OpenAIModel::Gpt4o,
            web_search: false,
        };
        let session_id = Uuid::new_v4();

        let mut messages = Messages::default();
        let chat_messages: Vec<ChatMessage> = vec![
            ChatMessage::new(
                session_id,
                llm_settings.clone(),
                Role::User,
                "history question".to_string(),
            )
            .with_created_at(user_message_created_at),
            ChatMessage::new(
                session_id,
                llm_settings.clone(),
                Role::Assistant,
                "history answer".to_string(),
            )
            .with_created_at(assistant_message_created_at),
        ];
        messages.viewport.build_lines(&chat_messages, None);
        let mut session = super::Session::new(llm_settings);
        session.set_title(Some("Awesome chat".to_string()));
        session.set_messages(messages);
        session.input_editor.set_input("repeat this".repeat(3));
        let session_state = &mut SessionState {
            cursor_position: None,
        };

        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(20, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_stateful_widget(&mut session, frame.area(), session_state))
            .unwrap();
        insta::assert_snapshot!(terminal.backend());
    }
}
