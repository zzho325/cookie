use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::{
        components::messages::MessagesView,
        model::{editor::Editor, messages::Messages},
    },
    service::models::LlmProvider,
};

const BORDER_LINE_COUNT_SIDE: usize = 1;
const BORDER_LINE_COUNT: usize = BORDER_LINE_COUNT_SIDE * 2;

const MIN_INPUT_LINE_CPUNT: usize = 1 + BORDER_LINE_COUNT;
const MAX_INPUT_RATIO: f32 = 0.3;

pub struct ChatState {
    pub cursor_position: Option<(u16, u16)>,
}

pub struct ChatView<'a> {
    pub messages: &'a Messages,
    pub input_editor: &'a mut Editor,
    pub llm: &'a LlmProvider,
}

impl StatefulWidget for ChatView<'_> {
    type State = ChatState;

    /// Renders chat pane with input block starting with height = 1 (excluding border) and increase
    /// height as input length increases with a maximum of 0.3 * widget area.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ChatState) {
        // use textwrap for height calculation and rendering for consistency
        let input_width = area.width.saturating_sub(BORDER_LINE_COUNT as u16) as usize;
        let wrapped_input = self.input_editor.lines(input_width);
        let cursor_position = self.input_editor.cursor_position(input_width);

        // calculate input and history messages area height
        let input_line_count = wrapped_input.len() + BORDER_LINE_COUNT;
        let max_input_line_count = (area.height as f32 * MAX_INPUT_RATIO).floor() as usize;
        let input_height = input_line_count
            .max(MIN_INPUT_LINE_CPUNT)
            .min(max_input_line_count) as u16;

        let message_height = area.height.saturating_sub(input_height);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(message_height),
                Constraint::Length(input_height),
            ])
            .split(area);

        let messages = MessagesView::from(self.messages);
        messages.render(layout[0], buf);

        // input
        let input_lines: Vec<Line> = wrapped_input.into_iter().map(Line::from).collect();
        let text = Text::from(input_lines);

        // construct title
        // TODO: centralize style here and prompt style
        let provider = self.llm.provider_name();
        let model = self.llm.model_name();
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

        Paragraph::new(text)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .padding(Padding::horizontal(1))
                    .title(title.left_aligned()),
            )
            .render(layout[1], buf);

        // set cursor position if editing
        if self.input_editor.is_editing {
            let (x, y) = cursor_position;
            state.cursor_position = Some((
                x + BORDER_LINE_COUNT_SIDE as u16,
                y + message_height + BORDER_LINE_COUNT_SIDE as u16,
            ));
        } else {
            state.cursor_position = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{
        components::chat::ChatState,
        model::{
            editor::{Editor, WrapMode},
            messages::{HistoryMessage, MessageMetadata, Messages},
        },
    };

    #[test]
    fn render_chat() {
        let req_time = std::time::Instant::now();
        let resp_time = req_time + std::time::Duration::from_secs(2);

        let llm = crate::service::models::LlmProvider::OpenAI {
            model: "gpt-4o".to_string(),
            web_search: false,
        };
        let history_llm = llm.clone();
        let history_message = HistoryMessage {
            user_msg: "history question".to_string(),
            assistant_msg: "history answer".to_string(),
            metadata: MessageMetadata {
                llm: history_llm,
                req_time,
                resp_time: Some(resp_time),
            },
        };

        let chat = super::ChatView {
            messages: &Messages {
                history_messages: vec![history_message],
                ..Messages::default()
            },
            input_editor: &mut Editor::new("repeat this".repeat(3), false, WrapMode::default()),
            llm: &llm,
        };
        let chat_state = &mut ChatState {
            cursor_position: None,
        };

        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(20, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_stateful_widget(chat, frame.area(), chat_state))
            .unwrap();
        insta::assert_snapshot!(terminal.backend());
    }
}
