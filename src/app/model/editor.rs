#[derive(Debug, Default)]
pub struct Editor {
    /// Current value of the input.
    pub input: String,
    /// Position of cursor in the editor area.
    // pub character_index: usize,
    pub is_editing: bool,
}

impl Editor {
    pub fn new(input: String, is_editing: bool) -> Self {
        Self { input, is_editing }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn clear(&mut self) {
        self.input = String::new();
    }
}
