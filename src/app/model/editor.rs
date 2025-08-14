use crate::{
    app::{
        model::focus::{Focusable, Focused},
        view::editor_viewport::EditorViewport,
    },
    impl_focusable,
};

#[derive(Debug, Default, Clone, Copy)]
pub enum WrapMode {
    /// Vim-style: fill to the column limit, then break.
    #[default]
    Character,
    /// Word-wrap: only break at word boundaries (spaces, punctuation, etc).
    // TODO: make this configurable.
    Word,
}

#[derive(Default)]
pub struct Editor {
    /// Current value of the input.
    pub input: String,
    /// Current cursor char idx in input.
    pub cursor_char_idx: usize,
    focused: bool,
    is_editing: bool,

    pub viewport: EditorViewport,
}

impl_focusable!(Editor, Focused::InputEditor);

impl Editor {
    pub fn new(input: String, wrap_mode: WrapMode) -> Self {
        let char_idx = input.chars().count();
        Self {
            input,
            cursor_char_idx: char_idx,
            viewport: EditorViewport::new(wrap_mode),
            ..Default::default()
        }
    }

    /// Returns current input.
    pub fn input(&self) -> &str {
        &self.input
    }

    #[cfg(test)]
    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn is_editing(&self) -> bool {
        self.is_editing
    }

    pub fn set_is_editing(&mut self, is_editing: bool) {
        self.is_editing = is_editing;
    }

    /// Returns current cursor byte idx.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte idx based on the idx of the character.
    fn cursor_byte_idx(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_char_idx)
            .unwrap_or(self.input.len())
    }

    pub fn set_viewport_width(&mut self, viewport_width: usize) {
        self.viewport
            .set_viewport_width(viewport_width, &self.input, self.cursor_byte_idx())
    }

    // ----------------------------------------------------------------
    // Input change.
    // ----------------------------------------------------------------

    // TODO: don't reflow with full input on editing.
    pub fn enter_char(&mut self, new_char: char) {
        let idx = self.cursor_byte_idx();
        self.input.insert(idx, new_char);
        self.viewport.reflow(&self.input);

        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_char_idx != 0 {
            // not using `remove` since it works on bytes instead of the chars
            let current_idx = self.cursor_char_idx;
            let from_left_to_current_idx = current_idx - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_idx);
            let after_char_to_delete = self.input.chars().skip(current_idx);

            // put all characters together except the selected one
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.viewport.reflow(&self.input);

            self.move_cursor_left();
        }
    }

    /// Clears input.
    pub fn clear(&mut self) {
        self.input = String::new();
        self.viewport.reflow(&self.input);

        self.cursor_char_idx = 0;
        self.viewport.update_cursor_position(self.cursor_byte_idx());
    }

    // ----------------------------------------------------------------
    // Cursor nagivation.
    // ----------------------------------------------------------------

    pub fn move_cursor_down(&mut self) {
        let (x, mut y) = self.viewport.cursor_position();
        y = y.saturating_add(1);

        let target_cursor_byte_idx = self.viewport.find_cursor_byte_idx((x, y));
        let target_cursor_char_idx = self.input[..target_cursor_byte_idx].chars().count();
        self.clamp_and_update_cursor_position(target_cursor_char_idx);
    }

    pub fn move_cursor_up(&mut self) {
        let (x, mut y) = self.viewport.cursor_position();
        y = y.saturating_sub(1);

        let target_cursor_byte_idx = self.viewport.find_cursor_byte_idx((x, y));
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
        self.viewport.update_cursor_position(self.cursor_byte_idx());
    }
}

#[cfg(test)]
mod tests {
    use crate::app::model::editor::{Editor, WrapMode};

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
            Case {
                description: "Chinese",
                input: "taro\n芋泥奶茶",
                char_idx: 3,
                view_width: 11,
                expected_positions: vec![(2, 1), (2, 0)],
                expected_char_indices: vec![6, 2],
            },
        ];
        for case in cases {
            let mut editor = Editor::new(case.input.to_string(), WrapMode::Character);
            editor.cursor_char_idx = case.char_idx;
            // editor.set_max_height(10);
            editor.set_viewport_width(case.view_width);
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
            positions.push(editor.viewport.cursor_position());
            char_indices.push(editor.cursor_char_idx);
            editor.move_cursor_up();
            positions.push(editor.viewport.cursor_position());
            char_indices.push(editor.cursor_char_idx);
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
