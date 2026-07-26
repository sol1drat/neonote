mod overlays;
mod states;

use crate::{
    app::App,
    types::{AppState, FocusedTab},
};

use edtui::EditorMode;

impl App {
    pub fn draw(&mut self, frame: &mut ratatui::Frame) {
        let want = if matches!(self.state, AppState::Note)
            && matches!(self.focused_tab, FocusedTab::Editor)
        {
            Some(self.editor.mode)
        } else {
            None
        };

        if want != self.last_cursor_mode {
            let style = match want {
                Some(EditorMode::Normal) => SetCursorStyle::SteadyBlock,
                Some(EditorMode::Insert) => SetCursorStyle::SteadyBar,
                Some(EditorMode::Visual) => SetCursorStyle::SteadyUnderScore,
                Some(EditorMode::Search) => SetCursorStyle::SteadyUnderScore,
                None => SetCursorStyle::DefaultUserShape,
            };
            let _ = execute!(io::stdout(), style);
            self.last_cursor_mode = want;
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
