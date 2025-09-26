mod constants;
pub mod editor_viewport;
mod error_popup;
mod messages;
pub mod messages_viewport;
mod session;
mod session_manager;
mod setting_manager;
mod utils;
pub mod widgets;

use crate::app::{model::Model, view::error_popup::ErrorPopup};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position},
};

pub fn render_ui(model: &mut Model, frame: &mut Frame) {
    let session_state = &mut session::SessionState::default();

    if model.show_sidebar {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Percentage(90)])
            .split(frame.area());

        frame.render_widget(&mut model.session_manager, layout[0]);

        frame.render_stateful_widget(&mut model.session, layout[1], session_state);

        if let Some((mut x, y)) = session_state.cursor_position {
            x += layout[0].width;
            frame.set_cursor_position(Position::new(x, y));
        }
    } else {
        frame.render_stateful_widget(&mut model.session, frame.area(), session_state);

        if let Some((x, y)) = session_state.cursor_position {
            frame.set_cursor_position(Position::new(x, y));
        }

        model
            .session
            .input_editor
            .viewport
            .set_area(session_state.input_editor_area.clone());
        model
            .session
            .messages
            .viewport
            .set_area(session_state.messages_area.clone());
    }

    if let Some(setting_manager) = &mut model.setting_manager_popup {
        let setting_area = utils::centered_rect(frame.area(), 30, 60);
        frame.render_widget(setting_manager, setting_area);
    }

    if let Some(error_message) = &model.error_message {
        let error_popup = ErrorPopup::new(error_message);
        let area = utils::centered_rect(frame.area(), 60, 30);
        frame.render_widget(error_popup, area);
    }
}
