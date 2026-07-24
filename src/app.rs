use std::{fs, io, path::PathBuf};

use crossterm::{cursor::SetCursorStyle, execute};
use edtui::{EditorEventHandler, EditorMode, EditorState};
use ratatui::widgets::ListState;

use crate::types::{
    AppState, ConfirmPrompt, ConfirmSubject, FileCreate, FileRename, FocusedTab, NoteItem,
};

pub struct App {
    pub state: AppState,
    pub focused_tab: FocusedTab,
    pub exit: bool,
    pub vault_files: Vec<PathBuf>,
    pub list_state: ListState,
    pub current_vault: PathBuf,
    pub current_dir: PathBuf,
    pub confirm: Option<ConfirmPrompt>,
    pub file_create: Option<FileCreate>,
    pub file_rename: Option<FileRename>,
    pub note_files: Vec<NoteItem>,
    pub editor: EditorState,
    pub editor_handler: EditorEventHandler,
    pub current_note: PathBuf,
    pub note_changed: bool,
    pub saved_content: String,
    pub last_cursor_mode: Option<EditorMode>,
    pub need_help: bool,
}

impl App {
    pub fn new(vault: PathBuf) -> Self {
        let current_vault = if vault.as_os_str().is_empty() {
            vault.clone()
        } else {
            fs::canonicalize(&vault).unwrap_or(vault.clone())
        };

        let confirm = if !vault.as_os_str().is_empty() {
            Some(ConfirmPrompt {
                message: format!("Open {} as a vault?", vault.to_string_lossy()),
                subject: ConfirmSubject::StartVault,
            })
        } else {
            None
        };

        Self {
            state: AppState::Menu,
            focused_tab: FocusedTab::Explorer,
            exit: false,
            vault_files: Vec::new(),
            list_state: ListState::default(),
            current_dir: PathBuf::new(),
            current_vault,
            confirm,
            file_create: None,
            file_rename: None,
            note_files: Vec::new(),
            note_changed: false,
            editor: EditorState::default(),
            editor_handler: EditorEventHandler::default(),
            current_note: PathBuf::new(),
            saved_content: String::new(),
            last_cursor_mode: None,
            need_help: false,
        }
    }

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

    pub fn view(&mut self, frame: &mut ratatui::Frame) {
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

    fn apply_cursor_shape(&mut self) {
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
