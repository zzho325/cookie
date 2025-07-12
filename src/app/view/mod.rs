mod components;

use crate::app::{
    model::Model,
    view::components::chat::{ChatState, ChatView},
};
use ratatui::{Frame, layout::Position};

pub fn render_ui(model: &mut Model, frame: &mut Frame) {
    let chat_state = &mut ChatState {
        cursor_position: None,
    };
    frame.render_stateful_widget(
        ChatView {
            messages: &model.messages,
            input_editor: &mut model.input_editor,
            llm: &model.settings.llm,
        },
        frame.area(),
        chat_state,
    );

    if let Some((x, y)) = chat_state.cursor_position {
        frame.set_cursor_position(Position::new(x, y));
    }
}
