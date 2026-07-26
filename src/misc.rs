use crate::{
    app::App,
    types::{ConfirmPrompt, ConfirmSubject},
};

impl App {
    pub fn select_next(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(i.saturating_add(1)));
    }

    pub fn select_previous(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.list_state.select(Some(i.saturating_sub(1)));
        }
    }

    pub fn confirm_exit(&mut self) {
        self.confirm = Some(ConfirmPrompt {
            message: "Are you sure you want to quit?".into(),
            subject: ConfirmSubject::Exit,
        });
    }
}
