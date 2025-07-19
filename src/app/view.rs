mod chat;
pub mod constants;
mod messages;
mod session_manager;
pub mod widgets;

use crate::app::model::Model;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position},
};

pub fn render_ui(model: &mut Model, frame: &mut Frame) {
    let chat_state = &mut chat::ChatState {
        cursor_position: None,
    };
    if model.show_sidebar {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Percentage(90)])
            .split(frame.area());

        frame.render_widget(&mut model.session_manager, layout[0]);

        frame.render_stateful_widget(
            chat::ChatView {
                is_editing: model.session.is_editing,
                messages: &model.session.messages,
                input_editor: &mut model.session.input_editor,
                llm_settings: &model.session.llm_settings,
            },
            layout[1],
            chat_state,
        );

        if let Some((mut x, y)) = chat_state.cursor_position {
            x += layout[0].width;
            frame.set_cursor_position(Position::new(x, y));
        }
    } else {
        frame.render_stateful_widget(
            chat::ChatView {
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
}
