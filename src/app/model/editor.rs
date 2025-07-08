use std::borrow::Cow;
use textwrap::{Options, WordSeparator, wrap};
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub enum WrapMode {
    /// Vim-style: fill to the column limit, then break.
    #[default]
    Character,
    /// Word-wrap: only break at word boundaries (spaces, punctuation, etc).
    // TODO: make this configurable.
    Word,
}

#[derive(Debug, Default)]
pub struct Editor {
    /// Current value of the input.
    pub input: String,
    /// Position of cursor in the editor area.
    pub char_index: usize,
    pub is_editing: bool,
    pub wrap_mode: WrapMode,
}

impl Editor {
    pub fn new(input: String, is_editing: bool, wrap_mode: WrapMode) -> Self {
        let char_index = input.chars().count();
        Self {
            input,
            char_index,
            is_editing,
            wrap_mode,
        }
    }

    /// Returns current input.
    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
        debug!("{:?}, cursor index {}", self.input, self.char_index)
    }

    /// Clear input.
    pub fn clear(&mut self) {
        self.input = String::new();
        self.char_index = 0;
    }

    /// Input and cursor index after soft-wrapping to `wrap_width` based on `wrap_mode`.
    pub fn wrapped_view(&self, wrap_width: usize) -> (Vec<Cow<'_, str>>, (u16 /*x*/, u16 /*y*/)) {
        let input = &self.input;
        let cursor_byte_index = self.byte_index();
        let mut lines = Vec::new();
        let (mut x, mut y) = (0, 0);

        // TODO: Refactor this and implement cursor position -> char index.
        // Cache soft-wrap per paragraph and only reflow affected lines.
        // For further peformance improvement, use piece- or rope-based buffers to make edits and
        // reflow O(log n).
        match self.wrap_mode {
            WrapMode::Character => {
                // let mut paragraph_line_offset = 0;
                let mut paragraph_byte_offset = 0;
                let mut found_cursor = false;

                for paragraph in input.split('\n') {
                    let mut current_width = 0;
                    let mut line_byte_offset = 0;

                    // iterate graphemes
                    for (grapheme_byte_offset, grapheme) in paragraph.grapheme_indices(true) {
                        let grapheme_width = UnicodeWidthStr::width(grapheme);
                        // create new line when width is reached
                        if current_width + grapheme_width > wrap_width {
                            lines.push(Cow::Borrowed(
                                &input[(paragraph_byte_offset + line_byte_offset)
                                    ..grapheme_byte_offset],
                            ));
                            line_byte_offset = grapheme_byte_offset;
                            current_width = 0;
                        }
                        current_width += grapheme_width;

                        // find cursor at the first grapheme with byte index >= cursor byte index
                        if !found_cursor
                            && cursor_byte_index <= paragraph_byte_offset + grapheme_byte_offset
                        {
                            x = current_width - 1;
                            y = lines.len();
                            found_cursor = true;
                        }
                    }

                    // handle cursor at the end of paragraph
                    if !found_cursor && cursor_byte_index == paragraph_byte_offset + paragraph.len()
                    {
                        x = current_width;
                        y = lines.len();
                        if x == wrap_width {
                            x = 0;
                            y += 1;
                        }
                        found_cursor = true;
                    }

                    // push the remainder
                    if line_byte_offset < paragraph.len() {
                        lines.push(Cow::Borrowed(&paragraph[line_byte_offset..]));
                    }

                    // handle empty paragraph
                    if paragraph.is_empty() {
                        lines.push(Cow::Borrowed(""));
                    }

                    paragraph_byte_offset += paragraph.len() + 1; // count for '\n'
                }

                debug!("{x}, {y}");
                (lines, (x as u16, y as u16))
            }
            WrapMode::Word => {
                // TODO: confirm options and unify with ratatui wrap
                let opts = Options::new(wrap_width)
                    .break_words(true)
                    .word_separator(WordSeparator::UnicodeBreakProperties)
                    // TODO: update dependency after textwrap's next release
                    .preserve_trailing_space(true);

                let mut paragraph_line_offset = 0;
                let mut paragraph_byte_offset = 0;
                let mut found_cursor = false;

                for paragraph in input.split('\n') {
                    // textwrap uses an optimal-fit algorithm which looks ahead and chooses line
                    // breaks which minimize the gaps left at ends of lines
                    let paragraph_lines = wrap(paragraph, opts.clone());

                    // if cursor is in this paragrah, find its line and find cursor
                    // inclusive on the right side to take account for \n
                    if cursor_byte_index >= paragraph_byte_offset
                        && cursor_byte_index <= paragraph_byte_offset + paragraph.len()
                    {
                        let mut line_byte_offset = 0;
                        for (line_index, line) in paragraph_lines.iter().enumerate() {
                            if cursor_byte_index
                                < paragraph_byte_offset + line_byte_offset + line.len()
                            {
                                x = line[..(cursor_byte_index
                                    - paragraph_byte_offset
                                    - line_byte_offset)]
                                    .graphemes(true)
                                    .count()
                                    .min(wrap_width);
                                y = paragraph_line_offset + line_index;
                                found_cursor = true;
                                break;
                            }
                            line_byte_offset += line.len();
                        }

                        // handle cursor at the end of paragraph
                        if !found_cursor {
                            let current_width = paragraph_lines
                                .last()
                                .map(|line| line.graphemes(true).count())
                                .unwrap_or(0);

                            x = current_width;
                            y = paragraph_line_offset + paragraph_lines.len() - 1;
                            if x == wrap_width {
                                x = 0;
                                y += 1;
                            }
                        }
                    }
                    lines.extend(paragraph_lines.iter().cloned());
                    paragraph_line_offset += paragraph_lines.len();
                    paragraph_byte_offset += paragraph.len() + 1; // count for '\n'
                }
                (lines, (x as u16, y as u16))
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_index != 0;
        if is_not_cursor_leftmost {
            // Not using `remove` since it works on bytes instead of the chars.
            let current_index = self.char_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_index)
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
            char_index: usize,
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
                char_index: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "one line, cursor in the middle of input",
                input: "hello",
                char_index: 2,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello"],
                expect_cursor_position: (2, 0),
            },
            Case {
                description: "two lines with new line",
                input: "hello\nworld",
                char_index: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with new line, cursor at newline",
                input: "hello\nworld",
                char_index: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "two lines with new line, second line empty, cursor at newline",
                input: "hello\n",
                char_index: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", ""],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with wrap",
                input: "hello world",
                char_index: 7,
                wrap_width: 6,
                expect_wrapped_input: vec!["hello ", "world"],
                expect_cursor_position: (1, 1),
            },
            Case {
                description: "two lines with wrap, cursor before break",
                input: " hello  world ",
                char_index: 6,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello ", " world "],
                expect_cursor_position: (6, 0),
            },
            Case {
                description: "two lines with wrap, cursor after break",
                input: " hello  world ",
                char_index: 7,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello ", " world "],
                expect_cursor_position: (0, 1),
            },
        ];
        for case in cases {
            let editor = Editor {
                input: case.input.to_string(),
                char_index: case.char_index,
                is_editing: true,
                wrap_mode: WrapMode::Character,
            };
            let (wrapped_input, cursor_position) = editor.wrapped_view(case.wrap_width);
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
            char_index: usize,
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
                char_index: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "one line, cursor in the middle of input",
                input: "hello world",
                char_index: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello ", "world"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "two lines with new line, cursor at newline",
                input: "hello\nworld",
                char_index: 5,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (5, 0),
            },
            Case {
                description: "two lines with new line, cursor after newline",
                input: "hello\nworld",
                char_index: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", "world"],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with new line, second line empty, cursor at newline",
                input: "hello\n",
                char_index: 6,
                wrap_width: 10,
                expect_wrapped_input: vec!["hello", ""],
                expect_cursor_position: (0, 1),
            },
            Case {
                description: "two lines with soft wrap",
                input: "hello world",
                char_index: 7,
                wrap_width: 6,
                expect_wrapped_input: vec!["hello ", "world"],
                expect_cursor_position: (1, 1),
            },
            Case {
                description: "two lines with soft wrap, cursor before break with trailing whitespace",
                input: " hello   world ",
                char_index: 8,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello   ", "world "],
                expect_cursor_position: (7, 0),
            },
            Case {
                description: "two lines with soft wrap, cursor after break",
                input: " hello  world ",
                char_index: 8,
                wrap_width: 7,
                expect_wrapped_input: vec![" hello  ", "world "],
                expect_cursor_position: (0, 1),
            },
        ];
        for case in cases {
            let editor = Editor {
                input: case.input.to_string(),
                char_index: case.char_index,
                is_editing: true,
                wrap_mode: WrapMode::Word,
            };
            let (wrapped_input, cursor_position) = editor.wrapped_view(case.wrap_width);
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
