use crate::{
    app::App,
    types::{AppState, ConfirmPrompt, ConfirmSubject, FocusedTab},
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

    pub fn apply_cursor_shape(&mut self) {
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
    }
}
