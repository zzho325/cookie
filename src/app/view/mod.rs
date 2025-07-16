mod components;
pub mod constants;

use crate::app::{
    model::Model,
    view::components::chat::{ChatState, ChatView},
};
use ratatui::{Frame, layout::Position};

pub fn render_ui(model: &mut Model, frame: &mut Frame) {
    tracing::debug!("render_ui");
    let chat_state = &mut ChatState {
        cursor_position: None,
    };
    frame.render_stateful_widget(
        ChatView {
            is_editing: model.session.is_editing,
            messages: &model.session.messages,
            input_editor: &mut model.session.input_editor,
            llm_settings: &model.session.llm_settings,
        },
        frame.area(),
        chat_state,
    );

    if let Some((x, y)) = chat_state.cursor_position {
        frame.set_cursor_position(Position::new(x, y));
    }
}
