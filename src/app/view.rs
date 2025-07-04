use crate::app::{components::chat::Chat, model::Model};
use ratatui::Frame;

pub fn render_ui(model: &mut Model, frame: &mut Frame) {
    frame.render_widget(
        Chat {
            history_messages: &model.history_messages,
            input: model.input_editor.input(),
            pending_question: model.pending_question.as_deref(),
        },
        frame.area(),
    );
}
