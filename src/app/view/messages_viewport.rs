use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use textwrap::{Options, WordSeparator, wrap};

use crate::{
    app::view::{utils::markdown, widgets::scroll::ScrollState},
    models::{ChatMessage, MessageDelta, settings::LlmSettings},
};

#[derive(Debug, PartialEq)]
pub struct StyleSlice {
    byte_offset: usize,
    len: usize,
    /// A Ratatui `Style`.
    style: Style,
}

impl StyleSlice {
    fn new(byte_offset: usize, len: usize, style: Style) -> Self {
        Self {
            byte_offset,
            len,
            style,
        }
    }

    fn start_idx(&self) -> usize {
        self.byte_offset
    }

    fn end_idx(&self) -> usize {
        self.byte_offset + self.len
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct StyledLine {
    /// The literal text to render.
    content: String,

    /// A Ratatui `Style` applied to the whole line.
    style: Style,

    /// Style slices in current line.
    style_slices: Vec<StyleSlice>,
}

impl From<String> for StyledLine {
    fn from(value: String) -> Self {
        let style_slice = StyleSlice::new(0, value.len(), Style::default());
        Self {
            content: value,
            style: Style::default(),
            style_slices: vec![style_slice],
        }
    }
}

impl StyledLine {
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn style_slices_mut(&mut self) -> &mut Vec<StyleSlice> {
        &mut self.style_slices
    }

    /// Appends content to line.
    pub fn append(&mut self, content: impl Into<String>, style: Style) {
        let content: String = content.into();
        let byte_offset = self.content.len();
        let style_slice = StyleSlice::new(byte_offset, content.len(), style);
        self.content += &content;
        self.style_slices.push(style_slice);
    }

    /// Inserts prefix span to line.
    pub fn insert_prefix(&mut self, prefix: Span) {
        let len = prefix.content.len();
        self.content.insert_str(0, &prefix.content);
        for style_slices in &mut self.style_slices {
            style_slices.byte_offset += len;
        }
        self.style_slices
            .insert(0, StyleSlice::new(0, len, prefix.style));
    }

    pub fn patch_style(&mut self, style: Style) {
        self.style = self.style.patch(style)
    }

    /// Returns a subslice of `StyledLine` starting at `offset` of length `len`.
    pub fn get(&self, offset: usize, len: usize) -> StyledLine {
        let style_slices = self
            .style_slices
            .iter()
            .filter_map(|s| {
                if s.start_idx() >= offset + len || s.end_idx() < offset {
                    None
                } else {
                    // start = min(current start, offset)
                    let start = s.start_idx().max(offset);
                    // len = end - start = min(current end, offset + len) - start
                    let len = s
                        .end_idx()
                        .min(offset.saturating_add(len))
                        .saturating_sub(start);
                    Some(StyleSlice::new(start.saturating_sub(offset), len, s.style))
                }
            })
            .collect();

        StyledLine {
            content: self
                .content
                .get(offset..offset + len)
                .unwrap_or("")
                .to_string(),
            style: self.style,
            style_slices,
        }
    }
}

impl From<&StyledLine> for Line<'_> {
    fn from(line: &StyledLine) -> Self {
        let spans: Vec<Span> = line
            .style_slices
            .iter()
            .map(|s| {
                let content: String = line.content[s.byte_offset..s.byte_offset + s.len].into();
                Span::from(content).style(s.style)
            })
            .collect();
        let tui_line = Line::from(spans);
        tui_line.patch_style(line.style)
    }
}

#[derive(Default)]
pub struct MessagesViewport {
    /// Logical paragraphs.
    paragraphs: Vec<StyledLine>,
    /// Visual lines.
    lines: Vec<StyledLine>,
    /// Available visual width.
    viewport_width: usize,
    scroll_state: ScrollState,
}

impl MessagesViewport {
    pub fn lines(&self) -> &[StyledLine] {
        &self.lines
    }

    pub fn scroll_state(&self) -> &ScrollState {
        &self.scroll_state
    }

    pub fn scroll_state_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }

    /// Creates prompt line as `StyledLine`.
    fn make_prompt_line(settings: &LlmSettings, elapsed_sec: Option<i64>) -> StyledLine {
        let provider = settings.provider_name();
        let model = settings.model_name();
        let mut line = StyledLine::default();
        let elapsed = elapsed_sec.map_or_else(|| "-".to_string(), |s| format!("{s}s"));

        line.append("┌─> ", Style::default());
        line.append(
            provider,
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );
        line.append(" on ", Style::default());
        line.append(model, Style::default().fg(Color::LightBlue));
        line.append(" [", Style::default());
        line.append(elapsed, Style::default().fg(Color::LightMagenta));
        line.append("]", Style::default());
        line
    }

    // Creates text wrap option.
    fn make_wrap_opts(&self) -> Options {
        Options::new(self.viewport_width)
            .break_words(true)
            .word_separator(WordSeparator::UnicodeBreakProperties)
            .preserve_trailing_space(true)
    }

    /// Sets viewport width.
    pub fn set_viewport_width(&mut self, viewport_width: usize) {
        if viewport_width != self.viewport_width {
            self.viewport_width = viewport_width;
            self.reflow();
        }
    }

    /// Builds lines from chat messages.
    // TODO: cache history messages.
    pub fn build_lines(
        &mut self,
        chat_messages: &[ChatMessage],
        stream_message: Option<&MessageDelta>,
    ) {
        let mut paragraphs: Vec<StyledLine> = vec![];

        // history messages
        let mut iter = chat_messages.iter().peekable();
        while let Some(chat_message) = iter.next() {
            match chat_message.payload().role {
                crate::models::Role::User => {
                    // calculate elapsed duration if next message is from assistant
                    let start = *chat_message.created_at();
                    let elapsed_secs = iter
                        .peek()
                        .map(|next| (*next.created_at() - start).num_seconds());
                    let prefix_line =
                        Self::make_prompt_line(chat_message.llm_settings(), elapsed_secs);
                    paragraphs.push(prefix_line);

                    let mut lines: Vec<StyledLine> = chat_message
                        .payload()
                        .msg
                        .lines()
                        .map(|l| StyledLine::from(l.to_string()))
                        .collect();
                    if let Some(styled_line) = lines.get_mut(0) {
                        styled_line.insert_prefix(Span::raw("└─> "));
                    }
                    paragraphs.extend(lines);
                }
                crate::models::Role::Assistant => {
                    let styled_lines = markdown::from_str(&chat_message.payload().msg);
                    paragraphs.extend(styled_lines);
                }
            }
        }

        // stream in progress
        if let Some(stream_message) = stream_message {
            let styled_lines = markdown::from_str(&stream_message.delta);
            paragraphs.extend(styled_lines);
        }

        self.paragraphs = paragraphs;
        self.reflow();
    }

    /// Recalculates `self.lines`.
    pub fn reflow(&mut self) {
        let mut styled_lines: Vec<StyledLine> = vec![];

        for paragraph in &self.paragraphs {
            let lines = wrap(paragraph.content(), self.make_wrap_opts());
            let mut offset = 0;
            for line in lines {
                let len = line.len();
                let styled_line = paragraph.get(offset, len);
                offset += len;
                styled_lines.push(styled_line);
            }
        }
        self.lines = styled_lines;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_state
            .set_vertical_scroll_offset(self.lines.len())
    }
}
