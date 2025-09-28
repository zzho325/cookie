use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::tailwind},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, StatefulWidget, Widget},
};

use crate::app::{
    model::{focus::Focusable, session::Session},
    view::{
        constants::{MAX_INPUT_RATIO, MIN_INPUT_HEIGHT},
        utils::area::Area,
        widgets::scroll::AutoScroll,
    },
};

#[derive(Default)]
pub struct SessionState {
    pub cursor_position: Option<(u16, u16)>,
    pub messages_area: Area,
    pub input_editor_area: Area,
}

impl StatefulWidget for &mut Session {
    type State = SessionState;

    /// Renders chat session with input block starting with MIN_INPUT_HEIGHT including border and
    /// increase height as input length increases with a maximum of MAX_INPUT_RATIO of widget area.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut SessionState) {
        // ----------------------------------------------------------------
        // Dynamic height and split area
        // ----------------------------------------------------------------

        // input width
        let input_content_width = area.width.saturating_sub(2) as usize;
        self.input_editor.set_viewport_width(input_content_width);

        // input height
        let max_input_height = (area.height as f32 * MAX_INPUT_RATIO).floor() as usize;
        let lines = self.input_editor.viewport.lines();
        let input_height = (lines.len() + 1).clamp(MIN_INPUT_HEIGHT, max_input_height) as u16;

        // message height
        let messages_height = area.height.saturating_sub(input_height);

        let [messages_area, input_editor_area] = Layout::vertical([
            Constraint::Length(messages_height),
            Constraint::Length(input_height),
        ])
        .areas(area);

        // ----------------------------------------------------------------
        // Messages
        // ----------------------------------------------------------------

        self.messages.render(messages_area, buf);
        state.messages_area.height += messages_height;
        state.messages_area.width = area.width;

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
        scrollable.render(
            input_editor_area,
            buf,
            self.input_editor.viewport.scroll_state(),
        );

        state.input_editor_area.row += messages_height + 1;
        state.input_editor_area.height = messages_height + 1;
        state.input_editor_area.width = messages_height + 1;

        // ----------------------------------------------------------------
        // Cursor position
        // ----------------------------------------------------------------

        state.cursor_position = if self.messages.is_focused() {
            self.messages
                .viewport
                .scroll_state()
                .cursor_viewport_position()
                .map(|(x, y)| (x, y + 1))
        } else if self.input_editor.is_focused() {
            self.input_editor
                .viewport
                .scroll_state()
                .cursor_viewport_position()
                .map(|(x, y)| (x + 1, y + messages_height + 2))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use prost_types::Timestamp;
    use uuid::Uuid;

    use crate::{
        app::{model::messages::Messages, view::session::SessionState},
        chat::*,
        llm::*,
    };

    #[test]
    fn render_session() {
        let dt = chrono::Utc.with_ymd_and_hms(2025, 7, 10, 0, 0, 0).unwrap();
        let user_message_created_at = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let assistant_message_created_at = Timestamp {
            seconds: dt.timestamp() + 2,
            nanos: dt.timestamp_subsec_nanos() as i32,
        };

        let llm_settings = LlmSettings {
            provider: Some(crate::llm::llm_settings::Provider::OpenAi(OpenAiSettings {
                model: OpenAiModel::Gpt4o as i32,
                web_search: false,
            })),
        };
        let session_id = Uuid::new_v4().to_string();

        let mut messages = Messages::default();
        let chat_messages: Vec<ChatEvent> = vec![
            ChatEvent::new(
                session_id.clone(),
                Some(llm_settings),
                chat_event::Payload::Message(Message {
                    role: Role::User as i32,
                    msg: "history question".to_string(),
                }),
            )
            .with_created_at(user_message_created_at),
            ChatEvent::new(
                session_id.clone(),
                Some(llm_settings),
                chat_event::Payload::Message(Message {
                    role: Role::Assistant as i32,
                    msg: "history answer".to_string(),
                }),
            )
            .with_created_at(assistant_message_created_at),
        ];
        messages.viewport.build_lines(&chat_messages, None);
        messages.set_title(Some("Awesome chat".to_string()));
        let mut session = super::Session::new(llm_settings);
        session.set_messages(messages);
        session.input_editor.set_input("repeat this".repeat(3));
        let session_state = &mut SessionState::default();

        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(20, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_stateful_widget(&mut session, frame.area(), session_state))
            .unwrap();
        insta::assert_snapshot!(terminal.backend());
    }
}
