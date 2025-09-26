use std::time::SystemTime;

use itertools::Itertools;
use ratatui::{
    style::{Color, Modifier, Style, palette::tailwind},
    text::Span,
};
use textwrap::{Options, WordSeparator, wrap};

use crate::{
    app::view::{
        utils::{
            area::Area,
            markdown,
            paragraph::{Paragraph, Slicable},
            styled_line::StyledLine,
        },
        widgets::scroll::ScrollState,
    },
    chat::*,
    llm::*,
};

#[derive(Default)]
pub struct MessagesViewport {
    /// Aggregate paragraphs content.
    input: String,
    /// Logical paragraphs.
    paragraphs: Vec<Paragraph<StyledLine>>,
    /// Available visual width.
    viewport_width: usize,
    scroll_state: ScrollState,
    /// Current cursor char index.
    cursor_char_idx: usize,
    /// Selection start char index.
    selection_start_char_idx: Option<usize>,
    area: Area,
}

const HIGHLIGHT_STYLE: Style = Style::new().fg(tailwind::ZINC.c800).bg(tailwind::ZINC.c200);

impl MessagesViewport {
    pub fn lines(&self) -> Vec<StyledLine> {
        if let Some((start_offset, end_offset)) = self.visual_selection_byte_range() {
            return self
                .paragraphs
                .iter()
                .flat_map(|p| {
                    let lines = p.lines();
                    let mut new_lines: Vec<StyledLine> = Vec::with_capacity(lines.len());

                    let mut line_offset = p.byte_offset();
                    for line in lines.iter() {
                        let line_len = line.content().len();
                        if end_offset <= line_offset || start_offset > line_offset + line_len {
                            new_lines.push(line.clone());
                        } else {
                            let new_line = line.clone();
                            let start_offset = start_offset.saturating_sub(line_offset);
                            let end_offset = end_offset.min(line_offset + line_len) - line_offset;
                            let new_line = new_line
                                .patch_style(HIGHLIGHT_STYLE, Some((start_offset, end_offset)));
                            new_lines.push(new_line);
                        }
                        line_offset += line_len;
                    }
                    new_lines
                })
                .collect();
        }
        self.paragraphs
            .iter()
            .flat_map(|p| p.lines())
            .cloned()
            .collect()
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn scroll_state(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }

    pub fn scroll_state_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }

    pub fn area(&self) -> &Area {
        &self.area
    }

    pub fn set_area(&mut self, area: Area) {
        self.area = area;
    }

    /// Returns current cursor position.
    pub fn cursor_position(&self) -> (u16 /*x*/, u16 /*y*/) {
        self.scroll_state.cursor_position().unwrap_or((0, 0))
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
        chat_events: &[ChatEvent],
        stream_message: Option<&MessageDelta>,
    ) {
        let mut lines: Vec<StyledLine> = vec![];

        // history messages
        let mut iter = chat_events
            .iter()
            .filter(|e| matches!(e.payload, Some(chat_event::Payload::Message(_))))
            .peekable();

        while let Some(chat_event) = iter.next() {
            let chat_event::Payload::Message(Message { role, msg }) =
                chat_event.payload.clone().unwrap()
            else {
                unreachable!()
            };

            match Role::try_from(role).expect("Invalid role") {
                Role::User => {
                    // calculate elapsed duration if next message is from assistant
                    let start: SystemTime = chat_event.created_at.unwrap().try_into().unwrap();

                    let elapsed_secs = iter.peek().map(|next| {
                        let next_time: SystemTime =
                            next.created_at.unwrap_or_default().try_into().unwrap();
                        next_time
                            .duration_since(start)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0)
                    });
                    let prefix_line = Self::make_prompt_line(
                        &chat_event.llm_settings.unwrap_or_default(),
                        elapsed_secs,
                    );
                    lines.push(prefix_line);

                    let mut chat_message_lines: Vec<StyledLine> = msg
                        .lines()
                        .map(|l| StyledLine::from(l.to_string()))
                        .collect();
                    if let Some(styled_line) = chat_message_lines.get_mut(0) {
                        styled_line.insert_prefix(Span::raw("└─> "));
                    }
                    lines.extend(chat_message_lines);
                }
                Role::Assistant => {
                    let styled_lines = markdown::from_str(&msg);
                    lines.extend(styled_lines);
                }
                Role::Unspecified => unreachable!("Unpecified role"),
            }
        }

        // stream in progress
        if let Some(stream_message) = stream_message {
            let styled_lines = markdown::from_str(&stream_message.delta);
            lines.extend(styled_lines);
        }

        // add an extra new line in the end so that we move cursor to the new message line on new
        // message
        let styled_lines = StyledLine::from(String::from(""));
        lines.push(styled_lines);

        self.input = lines
            .iter()
            .map(|p| p.content().replace("\t", "  "))
            .join("\n");
        self.paragraphs = lines.into_iter().map(Paragraph::build).collect();
        self.reflow();
    }

    /// Recalculates paragraph lines.
    pub fn reflow(&mut self) {
        let mut byte_offset = 0;
        let mut line_offset = 0;

        let wrap_opts = Options::new(self.viewport_width)
            .break_words(true)
            .word_separator(WordSeparator::UnicodeBreakProperties)
            .preserve_trailing_space(true);

        for paragraph in &mut self.paragraphs {
            let mut styled_lines: Vec<StyledLine> = vec![];

            let lines = wrap(paragraph.content().as_ref(), wrap_opts.clone());
            let mut offset = 0;
            for line in lines {
                let len = line.len();
                let styled_line = paragraph.content().slice(offset, len);
                offset += len;
                styled_lines.push(styled_line);
            }

            let line_count = styled_lines.len();
            paragraph.reflow(styled_lines, byte_offset, line_offset);
            byte_offset += paragraph.len() + 1; // count for '\n'
            line_offset += line_count;
        }
    }

    /// Sets cursor position given cursor byte index.
    pub fn update_cursor_position(&mut self, cursor_byte_idx: usize) {
        for paragraph in &self.paragraphs {
            // if cursor is in this paragrah, find its line and find cursor
            // inclusive on the right side to take account for \n
            if cursor_byte_idx >= paragraph.byte_offset()
                && cursor_byte_idx <= paragraph.byte_offset() + paragraph.len()
            {
                let (mut x, y) = paragraph.find_cursor_position(cursor_byte_idx);
                x = x.clamp(0, self.viewport_width as u16);
                self.scroll_state.set_cursor_position((x, y));
                return;
            }
        }
        tracing::warn!("cursor position not found");
    }

    /// Returns cursor byte idx given a cursor position.
    pub fn find_cursor_byte_idx(&mut self, cursor_position: (u16 /*x*/, u16 /*y*/)) -> usize {
        let (_, y) = cursor_position;
        let mut paragraph_idx = self.paragraphs.partition_point(|p| {
            // first partition: {paragraphs before cursor}
            (p.line_offset() as u16 + p.lines().len() as u16) <= y
        });
        // clamp to last paragraph if out of input bound
        paragraph_idx = paragraph_idx.clamp(0, self.paragraphs.len().saturating_sub(1));
        let paragraph = &self.paragraphs[paragraph_idx];
        paragraph.find_cursor_byte_idx(cursor_position)
    }

    // ----------------------------------------------------------------
    // Cursor nagivation.
    // ----------------------------------------------------------------
    /// Returns current byte idx for given char index..
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte idx based on the idx of the character.
    fn cursor_byte_idx(&self, char_idx: usize) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(char_idx)
            .unwrap_or(self.input.len())
    }

    pub fn move_cursor_down(&mut self) {
        let (x, mut y) = self.cursor_position();
        y = y.saturating_add(1);

        let target_cursor_byte_idx = self.find_cursor_byte_idx((x, y));
        let target_cursor_char_idx = self.input[..target_cursor_byte_idx].chars().count();
        self.clamp_and_update_cursor_position(target_cursor_char_idx);
    }

    /// Moves cursor to next line after end of buffer and clear screen.
    pub fn scroll_to_top(&mut self) {
        let target_cursor_char_idx = self.input.chars().count();
        self.clamp_and_update_cursor_position(target_cursor_char_idx);
        if let Some((_, y)) = self.scroll_state().cursor_position() {
            self.scroll_state.set_vertical_scroll_offset(y as usize);
        } else {
            tracing::warn!("messages missing cursor");
        }
    }

    pub fn move_cursor_up(&mut self) {
        let (x, mut y) = self.cursor_position();
        y = y.saturating_sub(1);

        let target_cursor_byte_idx = self.find_cursor_byte_idx((x, y));
        let target_cursor_char_idx = self.input[..target_cursor_byte_idx].chars().count();
        self.clamp_and_update_cursor_position(target_cursor_char_idx);
    }

    pub fn move_cursor_left(&mut self) {
        let target_cursor_char_idx = self.cursor_char_idx.saturating_sub(1);
        self.clamp_and_update_cursor_position(target_cursor_char_idx);
    }

    pub fn move_cursor_right(&mut self) {
        let target_cursor_char_idx = self.cursor_char_idx.saturating_add(1);
        self.cursor_char_idx = target_cursor_char_idx.clamp(0, self.input.chars().count());
        self.clamp_and_update_cursor_position(target_cursor_char_idx);
    }

    /// Updates cursor position to clamped target cursor position.
    fn clamp_and_update_cursor_position(&mut self, target_cursor_char_idx: usize) {
        self.cursor_char_idx = target_cursor_char_idx.clamp(0, self.input.chars().count());
        self.update_cursor_position(self.cursor_byte_idx(self.cursor_char_idx));
    }

    // ----------------------------------------------------------------
    // Visual selection.
    // ----------------------------------------------------------------
    /// Sets selection start `selection_start_char_idx` to current cursor char index if it's not
    /// already set; otherwise clears selection by sets `selection_start_char_idx` None.
    pub fn toggle_visual_selection(&mut self) {
        if self.selection_start_char_idx.is_some() {
            self.selection_start_char_idx = None;
        } else {
            self.selection_start_char_idx = Some(self.cursor_char_idx);
        }
    }

    /// Clears visual selection.
    pub fn clear_visual_selection(&mut self) {
        self.selection_start_char_idx = None;
    }

    /// Returns current visual selection range [start, end) in byte offset.
    fn visual_selection_byte_range(&self) -> Option<(usize /*start*/, usize /*end*/)> {
        if let Some(mut start_char_idx) = self.selection_start_char_idx {
            let mut end_char_idx = self.cursor_char_idx;
            if start_char_idx > end_char_idx {
                (start_char_idx, end_char_idx) = (end_char_idx, start_char_idx);
            }

            return Some((
                self.cursor_byte_idx(start_char_idx),
                self.cursor_byte_idx(end_char_idx + 1),
            ));
        }
        None
    }

    /// Returns current selected text and clear selection.
    pub fn yank_visual_selection(&mut self) -> Option<String> {
        if let Some((start, end)) = self.visual_selection_byte_range() {
            if let Some(selected) = self.input.get(start..end) {
                let selected = selected.to_string();
                self.toggle_visual_selection();
                return Some(selected);
            }
            tracing::warn!("invalid visual selection range {:?}", (start, end));
        }
        None
    }
}
