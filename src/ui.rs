mod overlays;
mod states;

use crate::{
    app::App,
    types::{AppState, FocusedTab},
};

use crossterm::{cursor::SetCursorStyle, execute};
use edtui::EditorMode;

impl App {
    pub fn draw(&mut self, frame: &mut ratatui::Frame) {
        let cursor_style = if matches!(self.state, AppState::Note)
            && matches!(self.focused_tab, FocusedTab::Editor)
        {
            match self.editor.mode {
                EditorMode::Normal => SetCursorStyle::SteadyBlock,
                EditorMode::Insert => SetCursorStyle::SteadyBar,
                EditorMode::Visual | EditorMode::Search => SetCursorStyle::SteadyUnderScore,
            }
        } else {
            SetCursorStyle::DefaultUserShape
        };

        if cursor_style != self.last_cursor_mode {
            let _ = execute!(std::io::stdout(), cursor_style);
            self.last_cursor_mode = cursor_style;
        }

        match self.state {
            AppState::Menu => self.menu(frame),
            AppState::VaultSelect => self.vault_select(frame),
            AppState::Note => self.note(frame),
        }

        if self.need_help {
            self.draw_help(frame, frame.area());
        }

        if let Some(prompt) = &self.confirm {
            self.draw_confirm(frame, frame.area(), prompt);
        }

        if let Some(prompt) = &self.file_create {
            self.draw_file_create(frame, frame.area(), prompt);
        }

        if let Some(prompt) = &self.file_rename {
            self.draw_file_rename(frame, frame.area(), prompt);
        }
    }
}
