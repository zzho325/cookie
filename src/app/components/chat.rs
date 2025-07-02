use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use textwrap::wrap;

const BORDER_LINE_COUNT: usize = 2;
const MIN_INPUT_LINE_CPUNT: usize = 1 + BORDER_LINE_COUNT;
const MAX_INPUT_RATIO: f32 = 0.3;

pub struct Chat<'a> {
    pub history_messages: &'a [(String, String)],
    pub pending_question: Option<&'a str>,
    pub input: &'a str,
}

impl Widget for Chat<'_> {
    /// Renders chat pane with input block starting with height = 1 (excluding border) and increase
    /// height as input length increases with a maximum of 0.3 * widget area.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input_width = area.width.saturating_sub(BORDER_LINE_COUNT as u16) as usize;
        let input_line_count = wrap(self.input, input_width).len() + BORDER_LINE_COUNT;
        let max_input_line_count = (area.height as f32 * MAX_INPUT_RATIO).floor() as usize;
        let input_height = input_line_count
            .max(MIN_INPUT_LINE_CPUNT)
            .min(max_input_line_count) as u16;

        let message_height = area.height.saturating_sub(input_height as u16);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(message_height),
                Constraint::Length(input_height),
            ])
            .split(area);

        // history messages
        let mut lines = vec![];
        for (q, a) in self.history_messages {
            lines.push(Line::from(vec!["• ".into(), q.clone().into()]));
            lines.push(Line::from(vec!["• ".into(), a.clone().into()]));
            lines.push(Line::from("")); // blank line
        }
        if let Some(q) = self.pending_question {
            lines.push(Line::from(vec!["•: ".into(), q.into()]));
        }
        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .render(layout[0], buf);

        // input
        Paragraph::new(self.input.to_string())
            .block(Block::bordered().title("🚀:"))
            .wrap(Wrap { trim: false })
            .render(layout[1], buf);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn render_chat() {
        let chat = super::Chat {
            history_messages: &[("history question".to_string(), "history answer".to_string())],
            pending_question: None,
            input: &"repeat this".repeat(3),
        };
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(20, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(chat, frame.area()))
            .unwrap();
        insta::assert_snapshot!(terminal.backend());
    }
}
