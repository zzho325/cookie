use std::borrow::Cow;
use textwrap::{Options, WordSeparator, wrap};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::app::model::scroll::{ScrollState, Scrollable};

#[derive(Debug, Default, Clone, Copy)]
pub enum WrapMode {
    /// Vim-style: fill to the column limit, then break.
    #[default]
    Character,
    /// Word-wrap: only break at word boundaries (spaces, punctuation, etc).
    // TODO: make this configurable.
    Word,
}

#[derive(Debug)]
struct Paragraph {
    /// Current input buffer.
    input: String,
    lines: Vec<String>,
    /// Byte offset of start of paragraph.
    byte_offset: usize,
    /// Line number offset.
    line_offset: usize,
}

impl Paragraph {
    fn new(input: String, byte_offset: usize, line_offset: usize) -> Self {
        Self {
            input,
            lines: Vec::new(),
            byte_offset,
            line_offset,
        }
    }

    /// Recalculates paragraph lines.
    /// TODO: only recalcualte changed paragraphs.
    fn reflow(&mut self, view_width: usize, wrap_mode: WrapMode) {
        let input = &self.input;
        match wrap_mode {
            WrapMode::Character => {
                let mut current_width = 0;
                let mut line_byte_offset = 0;
                let mut lines = Vec::new();
                // iterate graphemes
                for (grapheme_byte_offset, grapheme) in input.grapheme_indices(true) {
                    let grapheme_width = UnicodeWidthStr::width(grapheme);
                    // create new line when width is reached
                    if current_width + grapheme_width > view_width {
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
                self.lines = lines;
            }
            WrapMode::Word => {
                // TODO: confirm options and unify with ratatui wrap_mode
                // FIXME: don't recreate it every time
                let opts = Options::new(view_width)
                    .break_words(true)
                    .word_separator(WordSeparator::UnicodeBreakProperties)
                    // TODO: update dependency after textwrap's next release
                    .preserve_trailing_space(true);

                let lines = wrap(input, opts.clone());
                self.lines = lines.into_iter().map(Cow::into_owned).collect();
            }
        }
    }

    /// Returns cursor position given a byte idx against cached lines, assuming it's in this
    /// paragraph.
    fn find_cursor_position(&self, cursor_byte_idx: usize) -> (u16 /*x*/, u16 /*y*/) {
        let mut line_byte_offset = 0;
        for (line_idx, line) in self.lines.iter().enumerate() {
            if cursor_byte_idx < self.byte_offset + line_byte_offset + line.len() {
                let x = line[..(cursor_byte_idx - self.byte_offset - line_byte_offset)].width();
                let y = self.line_offset + line_idx;
                return (x as u16, y as u16);
            }
            line_byte_offset += line.len();
        }

        // handle cursor at the end of paragraph
        let current_width = self.lines.last().map_or(0, |s| s.width());
        let x = current_width;
        let y = self.line_offset + self.lines.len() - 1;
        (x as u16, y as u16)
    }

    /// Returns cursor byte idx given a position against cached visual lines, assuming it's in
    /// this paragraph.
    fn find_byte_idx(&self, cursor_position: (u16 /*x*/, u16 /*y*/)) -> usize {
        let (x, y) = cursor_position;
        let mut line_idx = y as usize - self.line_offset;
        // clamp to last line if out of paragraph bound
        line_idx = line_idx.clamp(0, self.lines.len() - 1);

        let line = &self.lines[line_idx];
        let line_byte_offset: usize = self.lines[..line_idx].iter().map(|l| l.len()).sum();

        // iterate graphemes
        let mut current_width = 0;
        for (grapheme_byte_offset, grapheme) in line.grapheme_indices(true) {
            let grapheme_width = UnicodeWidthStr::width(grapheme);
            if current_width == x as usize {
                return self.byte_offset + line_byte_offset + grapheme_byte_offset;
            }
            current_width += grapheme_width;
        }
        // clamp to last byte if out of line bound
        self.byte_offset + line_byte_offset + line.len()
    }
}

#[derive(Default)]
pub struct Editor {
    /// Current value of the input.
    pub input: String,
    /// Current cursor char idx in input.
    pub char_idx: usize,
    pub is_editing: bool,
    /// Wrap mode, should not change after bootstrap.
    wrap_mode: WrapMode,

    pub scroll_state: ScrollState,
    /// Current view_width.
    view_width: usize,
    /// Current paragraphs holding visual lines wrapped with view_width.
    paragraphs: Vec<Paragraph>,
    /// If paragraphs are up-to-date with `input`, `view_width`.
    needs_reflow: bool,
}

impl Editor {
    pub fn new(input: String, is_editing: bool, wrap_mode: WrapMode) -> Self {
        let char_idx = input.chars().count();
        Self {
            input,
            char_idx,
            is_editing,
            wrap_mode,
            ..Editor::default()
        }
    }

    /// Returns current input.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Recalculates `self.paragraphs` if it's not up-to-date for the current `input`, `view_width`.
    pub fn maybe_reflow(&mut self) {
        if !self.needs_reflow {
            return;
        }
        let input = &self.input;
        let mut paragraphs = Vec::new();

        let mut byte_offset = 0;
        let mut line_offset = 0;

        for paragraph_input in input.split('\n') {
            let mut paragraph =
                Paragraph::new(paragraph_input.to_string(), byte_offset, line_offset);
            paragraph.reflow(self.view_width, self.wrap_mode);

            byte_offset += paragraph_input.len() + 1; // count for '\n'
            line_offset += paragraph.lines.len();

            paragraphs.push(paragraph);
        }

        self.paragraphs = paragraphs;
        self.needs_reflow = false;
    }

    pub fn set_width(&mut self, view_width: usize) {
        if view_width != self.view_width {
            self.view_width = view_width;
            self.needs_reflow = true;
        }
    }

    /// Returns visual lines given a view_width.
    pub fn lines(&mut self) -> Vec<String> {
        self.maybe_reflow();
        let mut lines = Vec::new();
        for paragraph in &self.paragraphs {
            lines.extend(paragraph.lines.clone());
        }
        lines
    }

    /// Returns current cursor position.
    pub fn cursor_position(&mut self) -> (u16 /*x*/, u16 /*y*/) {
        self.maybe_reflow();
        let cursor_byte_idx = self.byte_idx();
        for paragraph in &self.paragraphs {
            // if cursor is in this paragrah, find its line and find cursor
            // inclusive on the right side to take account for \n
            if cursor_byte_idx >= paragraph.byte_offset
                && cursor_byte_idx <= paragraph.byte_offset + paragraph.input.len()
            {
                let (x, y) = paragraph.find_cursor_position(cursor_byte_idx);
                return (x.min(self.view_width as u16), y);
            }
        }
        tracing::warn!("cursor position not found");
        (0u16, 0u16)
    }

    /// Returns cursor char idx given a position.
    fn find_char_idx(&mut self, cursor_position: (u16 /*x*/, u16 /*y*/)) -> usize {
        self.maybe_reflow();
        let (_, y) = cursor_position;
        let mut paragraph_idx = self.paragraphs.partition_point(|p| {
            // first partition: {paragraphs before cursor}
            (p.line_offset as u16 + p.lines.len() as u16) <= y
        });
        // clamp to last paragraph if out of input bound
        paragraph_idx = paragraph_idx.clamp(0, self.paragraphs.len() - 1);
        let paragraph = &self.paragraphs[paragraph_idx];
        let byte_idx = paragraph.find_byte_idx(cursor_position);
        self.input[..byte_idx].chars().count()
    }

    pub fn enter_char(&mut self, new_char: char) {
        let idx = self.byte_idx();
        self.input.insert(idx, new_char);
        self.move_cursor_right();
        self.needs_reflow = true;
    }

    pub fn delete_char(&mut self) {
        if self.char_idx != 0 {
            // not using `remove` since it works on bytes instead of the chars
            let current_idx = self.char_idx;
            let from_left_to_current_idx = current_idx - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_idx);
            let after_char_to_delete = self.input.chars().skip(current_idx);

            // put all characters together except the selected one
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
            self.needs_reflow = true;
        }
    }

    /// Clears input.
    pub fn clear(&mut self) {
        self.input = String::new();
        self.char_idx = 0;
        self.needs_reflow = true;
    }

    pub fn move_cursor_down(&mut self) {
        let (x, mut y) = self.cursor_position();
        y = y.saturating_add(1);
        self.char_idx = self.find_char_idx((x, y));
    }

    pub fn move_cursor_up(&mut self) {
        let (x, mut y) = self.cursor_position();
        y = y.saturating_sub(1);
        self.char_idx = self.find_char_idx((x, y));
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_idx.saturating_sub(1);
        self.char_idx = self.clamp_cursor_idx(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_idx.saturating_add(1);
        self.char_idx = self.clamp_cursor_idx(cursor_moved_right);
    }

    /// Returns current cursor byte idx.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte idx based on the idx of the character.
    fn byte_idx(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_idx)
            .unwrap_or(self.input.len())
    }

    fn clamp_cursor_idx(&self, new_char_idx: usize) -> usize {
        new_char_idx.clamp(0, self.input.chars().count())
    }
}

impl Scrollable for Editor {
    fn scroll_offset(&self) -> (u16, u16) {
        self.scroll_state.scroll_offset()
    }

    fn scroll_state(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }
}

#[cfg(test)]
mod tests {
    use crate::app::model::editor::{Editor, WrapMode};

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
            let mut editor = Editor::new(case.input.to_string(), true, WrapMode::Character);
            editor.char_idx = case.char_idx;

            editor.set_width(case.view_width);
            let lines = editor.lines();
            let cursor_position = editor.cursor_position();
            assert_eq!(lines, case.lines, "{} lines", case.description,);
            assert_eq!(
                cursor_position, case.cursor_position,
                "{} cursor position",
                case.description,
            );

            let char_idx = editor.find_char_idx(cursor_position);
            assert_eq!(
                char_idx, case.char_idx,
                "{} cursor char index",
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
            let mut editor = Editor::new(case.input.to_string(), true, WrapMode::Word);
            editor.char_idx = case.char_idx;

            editor.set_width(case.view_width);
            let lines = editor.lines();
            let cursor_position = editor.cursor_position();
            assert_eq!(lines, case.lines, "{} lines", case.description,);
            assert_eq!(
                cursor_position, case.cursor_position,
                "{} cursor position",
                case.description,
            );

            let char_idx = editor.find_char_idx(cursor_position);
            assert_eq!(
                char_idx, case.clamped_char_idx,
                "{} cursor char index",
                case.description,
            )
        }
    }

    #[test]
    fn move_cursor_vertical() {
        #[derive(Default)]
        struct Case {
            description: &'static str,
            input: &'static str,
            char_idx: usize,
            view_width: usize,
            expected_positions: Vec<(u16, u16)>,
            expected_char_indices: Vec<usize>,
        }
        let cases = vec![
            Case {
                description: "two lines",
                input: "hello\nworld",
                char_idx: 3,
                view_width: 10,
                expected_positions: vec![(3, 1), (3, 0)],
                expected_char_indices: vec![9, 3],
            },
            Case {
                description: "only one line",
                input: "hello",
                char_idx: 3,
                view_width: 10,
                expected_positions: vec![(3, 0), (3, 0)],
                expected_char_indices: vec![3, 3],
            },
            Case {
                description: "two lines with clamping",
                input: "hello world hi",
                char_idx: 5,
                view_width: 11,
                expected_positions: vec![(3, 1), (3, 0)],
                expected_char_indices: vec![14, 3],
            },
            Case {
                description: "Chinese",
                input: "芋泥奶茶，\n咖啡",
                char_idx: 2,
                view_width: 11,
                expected_positions: vec![(4, 1), (4, 0)],
                expected_char_indices: vec![8, 2],
            },
        ];
        for case in cases {
            let mut editor = Editor::new(case.input.to_string(), true, WrapMode::Character);
            editor.char_idx = case.char_idx;
            editor.set_width(case.view_width);
            let mut char_indices: Vec<usize> = Vec::new();
            let mut positions: Vec<(u16, u16)> = Vec::new();

            for (i, b) in case.input.as_bytes().iter().enumerate() {
                println!("  [{}] = 0x{:02X} ({})", i, b, b);
            }

            println!("\nChars:");
            for (i, c) in case.input.char_indices().enumerate() {
                println!("  [{:?}] = '{:?}'", i, c);
            }

            editor.move_cursor_down();
            positions.push(editor.cursor_position());
            char_indices.push(editor.char_idx);
            editor.move_cursor_up();
            positions.push(editor.cursor_position());
            char_indices.push(editor.char_idx);
            assert_eq!(
                positions, case.expected_positions,
                "{} cursor positions",
                case.description,
            );
            assert_eq!(
                char_indices, case.expected_char_indices,
                "{} cursor char indices",
                case.description,
            );
        }
    }
}
