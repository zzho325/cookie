use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::app::{
    components::messages::MessagesView,
    model::{editor::Editor, messages::Messages},
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
    pub input_editor: &'a Editor,
}

impl StatefulWidget for ChatView<'_> {
    type State = ChatState;

    /// Renders chat pane with input block starting with height = 1 (excluding border) and increase
    /// height as input length increases with a maximum of 0.3 * widget area.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ChatState) {
        // use textwrap for height calculation and rendering for consistency
        let input_width = area.width.saturating_sub(BORDER_LINE_COUNT as u16) as usize;
        let (wrapped_input, cursor_position) = self.input_editor.wrapped_view(input_width);

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
        //
        // Paragraph::new(messages)
        //     .wrap(Wrap { trim: false })
        //     .render(layout[0], buf);

        // input
        let input_lines: Vec<Line> = wrapped_input.into_iter().map(Line::from).collect();
        let text = Text::from(input_lines);
        Paragraph::new(text)
            .block(Block::bordered().title("🚀:"))
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
        model::editor::{Editor, WrapMode},
    };

    #[test]
    fn render_chat() {
        let chat = super::ChatView {
            history_messages: &[("history question".to_string(), "history answer".to_string())],
            pending_question: None,
            input_editor: &Editor::new("repeat this".repeat(3), false, WrapMode::default()),
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
