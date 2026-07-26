mod overlays;
mod states;

use crate::{
    app::App,
    types::AppState
}

impl App {
    pub fn draw(&mut self, frame: &mut ratatui::Frame) {
        self.apply_cursor_shape();

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
