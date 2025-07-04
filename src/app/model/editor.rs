use std::borrow::Cow;

use textwrap::{Options, WordSplitter, wrap};

#[derive(Debug, Default)]
pub struct Editor {
    /// Current value of the input.
    pub input: String,
    /// Position of cursor in the editor area.
    pub char_index: usize,
    pub is_editing: bool,
}

impl Editor {
    pub fn new(input: String, is_editing: bool) -> Self {
        let char_index = input.chars().count();
        Self {
            input,
            char_index,
            is_editing,
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
    }

    /// Clear input.
    pub fn clear(&mut self) {
        self.input = String::new();
        self.char_index = 0;
    }

    /// Input after soft-wrapping to `wrap_width`.
    ///
    /// Only lines up to current `char_index` if `up_to_index` is true.
    pub fn wrapped_input(&self, width: usize, up_to_index: bool) -> Vec<Cow<'_, str>> {
        let slice = if up_to_index {
            &self.input[..self.byte_index()]
        } else {
            &self.input
        };

        // TODO: disable triming white space.
        // TODO: confirm options and unify with ratatui wrap.
        wrap(slice, width)
    }

    /// Position (x, y) after soft-wrapping to `wrap_width`.
    pub fn cursor_position(&self, wrap_width: usize) -> (u16, u16) {
        let lines = self.wrapped_input(wrap_width, true);
        // x: display‐width of last line
        let x = lines
            .last()
            .map(|cow| textwrap::core::display_width(cow.as_ref()))
            .unwrap_or(0);

        // y: # of lines - 1
        let y = lines.len().saturating_sub(1);

        (x as u16, y as u16)
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
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
    use crate::app::model::editor::Editor;

    #[test]
    fn cursor_position() {
        #[derive(Default)]
        struct Case {
            description: &'static str,
            input: &'static str,
            char_index: usize,
            wrap_width: usize,
            expect: (u16, u16),
        }
        let cases = vec![
            Case {
                description: "empty input",
                wrap_width: 5,
                ..Default::default()
            },
            Case {
                description: "one line, cursor at the end of input",
                input: "hello",
                char_index: 5,
                wrap_width: 10,
                expect: (5, 0),
            },
            Case {
                description: "one line, cursor in the middle of input",
                input: "hello world",
                char_index: 5,
                wrap_width: 10,
                expect: (5, 0),
            },
            Case {
                description: "two lines with new line",
                input: "hello\nworld",
                char_index: 6,
                wrap_width: 10,
                expect: (0, 1),
            },
            Case {
                description: "two lines with soft wrap",
                input: "hello world",
                char_index: 7,
                wrap_width: 6,
                expect: (1, 1),
            },
            // FIXME: trimming trailing spaces makes cursor position tracking inaccurate.
            Case {
                description: "two lines with soft wrap, trailing spaces trimmed",
                input: "hello world",
                char_index: 6,
                wrap_width: 6,
                expect: (5, 0),
            },
            // Case {
            //     description: "three lines with new line and soft wrap",
            //     input: "hello,\nhow are you doing",
            //     char_index: 20,
            //     wrap_width: 10,
            //     expect: (2, 2),
            // },
        ];
        for case in cases {
            let editor = Editor {
                input: case.input.to_string(),
                char_index: case.char_index,
                is_editing: true,
            };
            let position = editor.cursor_position(case.wrap_width);
            assert_eq!(position, case.expect, "{}", case.description);
        }
    }
}
