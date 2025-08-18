use std::borrow::Cow;
use textwrap::{Options, WordSeparator, wrap};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::app::{
    model::editor::WrapMode,
    view::{
        utils::paragraph::{Paragraph, Slicable},
        widgets::scroll::ScrollState,
    },
};

impl Slicable for String {
    fn slice(&self, offset: usize, len: usize) -> Self {
        self[offset..offset + len].to_string()
    }
}

#[derive(Default)]
pub struct EditorViewport {
    /// Wrap mode, should not change after bootstrap.
    wrap_mode: WrapMode,

    /// Logical paragraphs wrapped with viewport_width.
    paragraphs: Vec<Paragraph<String>>,
    /// Available visual width.
    viewport_width: usize,

    scroll_state: ScrollState,
}

impl EditorViewport {
    pub fn new(wrap_mode: WrapMode) -> Self {
        Self {
            wrap_mode,
            scroll_state: ScrollState::default().with_cursor(),
            ..Default::default()
        }
    }

    /// Returns current cursor position.
    pub fn cursor_position(&self) -> (u16 /*x*/, u16 /*y*/) {
        self.scroll_state.cursor_position().unwrap_or((0, 0))
    }

    pub fn scroll_state(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }

    pub fn set_viewport_width(
        &mut self,
        viewport_width: usize,
        input: &str,
        cursor_byte_idx: usize,
    ) {
        if viewport_width != self.viewport_width {
            self.viewport_width = viewport_width;
            self.reflow(input);
            self.update_cursor_position(cursor_byte_idx);
        }
    }

    /// Returns visual lines given a view_width.
    pub fn lines(&mut self) -> Vec<String> {
        self.paragraphs
            .iter()
            .flat_map(|p| p.lines().iter().cloned())
            .collect()
    }

    /// Wraps `input` into visual lines.
    fn wrap(&mut self, input: &str) -> Vec<String> {
        match self.wrap_mode {
            WrapMode::Character => {
                let mut current_width = 0;
                let mut line_byte_offset = 0;
                let mut lines = Vec::new();
                // iterate graphemes
                for (grapheme_byte_offset, grapheme) in input.grapheme_indices(true) {
                    let grapheme_width = UnicodeWidthStr::width(grapheme);
                    // create new line when width is reached
                    if current_width + grapheme_width > self.viewport_width {
                        lines.push(input[(line_byte_offset)..grapheme_byte_offset].to_string());
                        line_byte_offset = grapheme_byte_offset;
                        current_width = 0;
                    }
                    current_width += grapheme_width;
                }
                // push the remainder
                if line_byte_offset < input.len() {
                    lines.push(input[line_byte_offset..].to_string());
                }
                // handle empty paragraph
                if input.is_empty() {
                    lines.push(String::new());
                }
                lines
            }
            WrapMode::Word => {
                let wrap_opts = Options::new(self.viewport_width)
                    .break_words(true)
                    .word_separator(WordSeparator::UnicodeBreakProperties)
                    .preserve_trailing_space(true);

                let lines = wrap(input, wrap_opts);
                lines.into_iter().map(Cow::into_owned).collect()
            }
        }
    }
    //

    /// Recalculates `self.paragraphs`.
    pub fn reflow(&mut self, input: &str) {
        let mut paragraphs = Vec::new();

        let mut byte_offset = 0;
        let mut line_offset = 0;

        for paragraph_input in input.split('\n') {
            let lines = self.wrap(paragraph_input);
            let mut paragraph = Paragraph::build(paragraph_input.to_string());

            let line_count = lines.len();
            paragraph.reflow(lines, byte_offset, line_offset);
            paragraphs.push(paragraph);

            byte_offset += paragraph_input.len() + 1; // count for '\n'
            line_offset += line_count;
        }

        self.paragraphs = paragraphs;
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
        paragraph_idx = paragraph_idx.clamp(0, self.paragraphs.len() - 1);
        let paragraph = &self.paragraphs[paragraph_idx];
        paragraph.find_cursor_byte_idx(cursor_position)
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{model::editor::WrapMode, view::editor_viewport::EditorViewport};

    fn cursor_byte_idx(input: &str, char_idx: usize) -> usize {
        input
            .char_indices()
            .map(|(i, _)| i)
            .nth(char_idx)
            .unwrap_or(input.len())
    }

    #[test]
    fn lines_and_cursor_charactor_mode() {
        #[derive(Default)]
        struct Case {
            description: &'static str,
            input: &'static str,
            char_idx: usize,
            view_width: usize,
            lines: Vec<&'static str>,
            cursor_position: (u16, u16),
        }
        let cases = vec![
            Case {
                description: "empty input",
                view_width: 5,
                lines: vec![""],
                ..Default::default()
            },
            Case {
                description: "one line, cursor at the end of input",
                input: "hello",
                char_idx: 5,
                view_width: 10,
                lines: vec!["hello"],
                cursor_position: (5, 0),
            },
            Case {
                description: "one line, cursor in the middle of input",
                input: "hello",
                char_idx: 2,
                view_width: 10,
                lines: vec!["hello"],
                cursor_position: (2, 0),
            },
            Case {
                description: "two lines with new line",
                input: "hello\nworld",
                char_idx: 6,
                view_width: 10,
                lines: vec!["hello", "world"],
                cursor_position: (0, 1),
            },
            Case {
                description: "two lines with new line, cursor at newline",
                input: "hello\nworld",
                char_idx: 5,
                view_width: 10,
                lines: vec!["hello", "world"],
                cursor_position: (5, 0),
            },
            Case {
                description: "two lines with new line, second line empty, cursor at newline",
                input: "hello\n",
                char_idx: 6,
                view_width: 10,
                lines: vec!["hello", ""],
                cursor_position: (0, 1),
            },
            Case {
                description: "two lines with wrap",
                input: "hello world",
                char_idx: 7,
                view_width: 6,
                lines: vec!["hello ", "world"],
                cursor_position: (1, 1),
            },
            Case {
                description: "two lines with wrap, cursor before break",
                input: " hello  world ",
                char_idx: 6,
                view_width: 7,
                lines: vec![" hello ", " world "],
                cursor_position: (6, 0),
            },
            Case {
                description: "two lines with wrap, cursor after break",
                input: " hello  world ",
                char_idx: 7,
                view_width: 7,
                lines: vec![" hello ", " world "],
                cursor_position: (0, 1),
            },
            Case {
                description: "Chinese",
                input: "芋泥奶茶\n",
                char_idx: 4,
                view_width: 10,
                lines: vec!["芋泥奶茶", ""],
                cursor_position: (8, 0),
            },
        ];
        for case in cases {
            let mut viewport = EditorViewport::new(WrapMode::Character);
            let byte_idx = cursor_byte_idx(case.input, case.char_idx);
            viewport.set_viewport_width(case.view_width, case.input, byte_idx);

            let lines = viewport.lines();
            assert_eq!(lines, case.lines, "{} lines", case.description);

            let cursor_position = viewport.cursor_position();
            assert_eq!(
                cursor_position, case.cursor_position,
                "{} cursor byte index to position",
                case.description,
            );

            let byte_idx_ = viewport.find_cursor_byte_idx(cursor_position);
            assert_eq!(
                byte_idx_, byte_idx,
                "{} cursor position to byte index",
                case.description,
            )
        }
    }

    #[test]
    fn lines_and_cursor_word_mode() {
        #[derive(Default)]
        struct Case {
            description: &'static str,
            input: &'static str,
            char_idx: usize,
            view_width: usize,
            lines: Vec<&'static str>,
            cursor_position: (u16, u16),
            clamped_char_idx: usize,
        }
        let cases = vec![
            Case {
                description: "empty input",
                view_width: 5,
                lines: vec![""],
                ..Default::default()
            },
            Case {
                description: "one line, cursor at the end of input",
                input: "hello",
                char_idx: 5,
                view_width: 10,
                lines: vec!["hello"],
                cursor_position: (5, 0),
                clamped_char_idx: 5,
            },
            Case {
                description: "one line, cursor in the middle of input",
                input: "hello world",
                char_idx: 5,
                view_width: 10,
                lines: vec!["hello ", "world"],
                cursor_position: (5, 0),
                clamped_char_idx: 5,
            },
            Case {
                description: "one line with trailing whitespace",
                input: " hello   ",
                char_idx: 9,
                view_width: 7,
                lines: vec![" hello   "],
                cursor_position: (7, 0),
                clamped_char_idx: 7,
            },
            Case {
                description: "two lines with new line, cursor at newline",
                input: "hello\nworld",
                char_idx: 5,
                view_width: 10,
                lines: vec!["hello", "world"],
                cursor_position: (5, 0),
                clamped_char_idx: 5,
            },
            Case {
                description: "two lines with new line, cursor after newline",
                input: "hello\nworld",
                char_idx: 6,
                view_width: 10,
                lines: vec!["hello", "world"],
                cursor_position: (0, 1),
                clamped_char_idx: 6,
            },
            Case {
                description: "two lines with new line, second line empty, cursor at newline",
                input: "hello\n",
                char_idx: 6,
                view_width: 10,
                lines: vec!["hello", ""],
                cursor_position: (0, 1),
                clamped_char_idx: 6,
            },
            Case {
                description: "two lines with soft wrap",
                input: "hello world",
                char_idx: 7,
                view_width: 6,
                lines: vec!["hello ", "world"],
                cursor_position: (1, 1),
                clamped_char_idx: 7,
            },
            Case {
                description: "two lines with soft wrap, cursor before break with trailing whitespace",
                input: " hello   world ",
                char_idx: 8,
                view_width: 7,
                lines: vec![" hello   ", "world "],
                cursor_position: (7, 0),
                clamped_char_idx: 7,
            },
            Case {
                description: "two lines with soft wrap, cursor after break",
                input: " hello  world ",
                char_idx: 8,
                view_width: 7,
                lines: vec![" hello  ", "world "],
                cursor_position: (0, 1),
                clamped_char_idx: 8,
            },
            Case {
                description: "three lines",
                input: " hello,  world\n",
                char_idx: 15,
                view_width: 7,
                lines: vec![" hello,  ", "world", ""],
                cursor_position: (0, 2),
                clamped_char_idx: 15,
            },
            Case {
                description: "Chinese",
                input: "芋泥奶茶\n",
                char_idx: 4,
                view_width: 10,
                lines: vec!["芋泥奶茶", ""],
                cursor_position: (8, 0),
                clamped_char_idx: 4,
            },
        ];
        for case in cases {
            let mut viewport = EditorViewport::new(WrapMode::Word);
            let byte_idx = cursor_byte_idx(case.input, case.char_idx);
            viewport.set_viewport_width(case.view_width, case.input, byte_idx);

            let lines = viewport.lines();
            assert_eq!(lines, case.lines, "{} lines", case.description);

            let cursor_position = viewport.cursor_position();
            assert_eq!(
                cursor_position, case.cursor_position,
                "{} cursor byte index to position",
                case.description,
            );

            let byte_idx_ = viewport.find_cursor_byte_idx(cursor_position);
            assert_eq!(
                byte_idx_,
                cursor_byte_idx(case.input, case.clamped_char_idx),
                "{} cursor position to byte index",
                case.description,
            )
        }
    }
}
