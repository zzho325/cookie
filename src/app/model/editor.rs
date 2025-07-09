use std::borrow::Cow;
use textwrap::{Options, WordSeparator, wrap};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

    /// Recalculate paragraph lines.
    fn reflow(&mut self, wrap_width: usize, wrap_mode: WrapMode) {
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
                    if current_width + grapheme_width > wrap_width {
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
                let opts = Options::new(wrap_width)
                    .break_words(true)
                    .word_separator(WordSeparator::UnicodeBreakProperties)
                    // TODO: update dependency after textwrap's next release
                    .preserve_trailing_space(true);

                let lines = wrap(input, opts.clone());
                self.lines = lines.into_iter().map(Cow::into_owned).collect();
            }
        }
    }

    /// Find cursor position if it's in this paragraph.
    fn find_cursor_position(
        &self,
        wrap_width: usize,
        cursor_byte_idx: usize,
    ) -> (u16 /*x*/, u16 /*y*/) {
        let mut line_byte_offset = 0;
        for (line_idx, line) in self.lines.iter().enumerate() {
            if cursor_byte_idx < self.byte_offset + line_byte_offset + line.len() {
                let x = line[..(cursor_byte_idx - self.byte_offset - line_byte_offset)]
                    .width()
                    .min(wrap_width); // tailing white spaces from word wrap mode
                let y = self.line_offset + line_idx;
                return (x as u16, y as u16);
            }
            line_byte_offset += line.len();
        }

        // handle cursor at the end of paragraph, handle tailing white spaces from word wrap mode
        let current_width = self.lines.last().map_or(0, |s| s.width()).min(wrap_width);

        let mut x = current_width;
        let mut y = self.line_offset + self.lines.len() - 1;
        if x == wrap_width {
            x = 0;
            y += 1;
        }
        (x as u16, y as u16)
    }
}

#[derive(Debug, Default)]
pub struct Editor {
    /// Current value of the input.
    pub input: String,
    /// Position of cursor in the editor area.
    pub char_idx: usize,
    pub is_editing: bool,
    pub wrap_mode: WrapMode,

    paragraphs: Vec<Paragraph>,
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

    pub fn enter_char(&mut self, new_char: char) {
        let idx = self.byte_idx();
        self.input.insert(idx, new_char);
        self.move_cursor_right();
    }

    /// Clear input.
    pub fn clear(&mut self) {
        self.input = String::new();
        self.char_idx = 0;
    }

    pub fn lines(&mut self, wrap_width: usize) -> Vec<String> {
        let input = &self.input;
        let mut lines = Vec::new();
        let mut paragraphs = Vec::new();

        let mut byte_offset = 0;
        let mut line_offset = 0;

        // TODO: instead of reflow all paragraphs, cache and update on edits / width change.
        for paragraph_input in input.split('\n') {
            let mut paragraph =
                Paragraph::new(paragraph_input.to_string(), byte_offset, line_offset);
            paragraph.reflow(wrap_width, self.wrap_mode);

            byte_offset += paragraph_input.len() + 1; // count for '\n'
            line_offset += paragraph.lines.len();

            lines.extend(paragraph.lines.clone());
            paragraphs.push(paragraph);
        }

        self.paragraphs = paragraphs;
        lines
    }

    pub fn cursor_position(&self, wrap_width: usize) -> (u16 /*x*/, u16 /*y*/) {
        let cursor_byte_idx = self.byte_idx();
        for paragraph in &self.paragraphs {
            // if cursor is in this paragrah, find its line and find cursor
            // inclusive on the right side to take account for \n
            if cursor_byte_idx >= paragraph.byte_offset
                && cursor_byte_idx <= paragraph.byte_offset + paragraph.input.len()
            {
                return paragraph.find_cursor_position(wrap_width, cursor_byte_idx);
            }
        }
        tracing::warn!("cursor position not found");
        (0u16, 0u16)
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_idx.saturating_sub(1);
        self.char_idx = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_idx.saturating_add(1);
        self.char_idx = self.clamp_cursor(cursor_moved_right);
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_idx != 0;
        if is_not_cursor_leftmost {
            // Not using `remove` since it works on bytes instead of the chars.
            let current_idx = self.char_idx;
            let from_left_to_current_idx = current_idx - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_idx);
            let after_char_to_delete = self.input.chars().skip(current_idx);

            // Put all characters together except the selected one.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    /// Returns the byte idx based on the character position.
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

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
}

#[cfg(test)]
mod tests {
    use crate::app::model::editor::{Editor, WrapMode};

    #[test]
    fn wraped_view_charactor_mode() {
        #[derive(Default)]
        struct Case {
            description: &'static str,
            input: &'static str,
            char_idx: usize,
            wrap_width: usize,
            expect_wrapped_input: Vec<&'static str>,
            expect_cursor_position: (u16, u16),
        }
        let cases = vec![
            Case {
                description: "empty input",
                wrap_width: 5,
                expect_wrapped_input: vec![""],
                ..Default::default()
            },
            Case {
                description: "one line, cursor at the end of input",
                input: "hello",
                char_idx: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "one line, cursor in the middle of input",
                input: "hello",
                char_idx: 2,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello"],
                expect_cursor_position: (2, 0),
            },
            Case {
                description: "two lines with new line",
                input: "hello\nworld",
                char_idx: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with new line, cursor at newline",
                input: "hello\nworld",
                char_idx: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "two lines with new line, second line empty, cursor at newline",
                input: "hello\n",
                char_idx: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", ""],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with wrap",
                input: "hello world",
                char_idx: 7,
                wrap_width: 6,
                expect_wrapped_input: vec!["hello ", "world"],
                expect_cursor_position: (1, 1),
            },
            Case {
                description: "two lines with wrap, cursor before break",
                input: " hello  world ",
                char_idx: 6,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello ", " world "],
                expect_cursor_position: (6, 0),
            },
            Case {
                description: "two lines with wrap, cursor after break",
                input: " hello  world ",
                char_idx: 7,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello ", " world "],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "Chinese",
                input: "芋泥奶茶\n",
                char_idx: 4,
                wrap_width: 10,
                expect_wrapped_input: vec!["芋泥奶茶", ""],
                expect_cursor_position: (8, 0),
            },
        ];
        for case in cases {
            let mut editor = Editor::new(case.input.to_string(), true, WrapMode::Character);
            editor.char_idx = case.char_idx;

            let wrapped_input = editor.lines(case.wrap_width);
            let cursor_position = editor.cursor_position(case.wrap_width);
            assert_eq!(
                wrapped_input, case.expect_wrapped_input,
                "{} wrapped inpput",
                case.description,
            );
            assert_eq!(
                cursor_position, case.expect_cursor_position,
                "{} cursor position",
                case.description
            );
        }
    }

    #[test]
    fn wrapped_view_word_mode() {
        #[derive(Default)]
        struct Case {
            description: &'static str,
            input: &'static str,
            char_idx: usize,
            wrap_width: usize,
            expect_wrapped_input: Vec<&'static str>,
            expect_cursor_position: (u16, u16),
        }
        let cases = vec![
            Case {
                description: "empty input",
                wrap_width: 5,
                expect_wrapped_input: vec![""],
                ..Default::default()
            },
            Case {
                description: "one line, cursor at the end of input",
                input: "hello",
                char_idx: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "one line, cursor in the middle of input",
                input: "hello world",
                char_idx: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello ", "world"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "two lines with new line, cursor at newline",
                input: "hello\nworld",
                char_idx: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "two lines with new line, cursor after newline",
                input: "hello\nworld",
                char_idx: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with new line, second line empty, cursor at newline",
                input: "hello\n",
                char_idx: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", ""],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with soft wrap",
                input: "hello world",
                char_idx: 7,
                wrap_width: 6,
                expect_wrapped_input: vec!["hello ", "world"],
                expect_cursor_position: (1, 1),
            },
            Case {
                description: "two lines with soft wrap, cursor before break with trailing whitespace",
                input: " hello   world ",
                char_idx: 8,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello   ", "world "],
                expect_cursor_position: (7, 0),
            },
            Case {
                description: "two lines with soft wrap, cursor after break",
                input: " hello  world ",
                char_idx: 8,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello  ", "world "],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "three lines",
                input: " hello,  world\n",
                char_idx: 15,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello,  ", "world", ""],
                expect_cursor_position: (0, 2),
            },
            Case {
                description: "Chinese",
                input: "芋泥奶茶\n",
                char_idx: 4,
                wrap_width: 10,
                expect_wrapped_input: vec!["芋泥奶茶", ""],
                expect_cursor_position: (8, 0),
            },
        ];
        for case in cases {
            let mut editor = Editor::new(case.input.to_string(), true, WrapMode::Word);
            editor.char_idx = case.char_idx;
            let wrapped_input = editor.lines(case.wrap_width);
            let cursor_position = editor.cursor_position(case.wrap_width);
            assert_eq!(
                wrapped_input, case.expect_wrapped_input,
                "{} wrapped inpput",
                case.description,
            );
            assert_eq!(
                cursor_position, case.expect_cursor_position,
                "{} cursor position",
                case.description
            );
        }
    }
}
