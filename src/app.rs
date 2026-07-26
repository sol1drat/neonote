use std::{io, path::PathBuf};

use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyEventKind},
    execute,
};
use edtui::{EditorEventHandler, EditorMode, EditorState};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};

use crate::{
    cache::{cache_exists, get_vault},
    types::{
        AppState, ConfirmPrompt, ConfirmSubject, FileCreate, FileRename, FocusedTab, NoteItem,
    },
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
    pub saved_content: String,
    pub last_cursor_mode: Option<EditorMode>,
    pub need_help: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|f| self.draw(f))?;
            if let Event::Key(k) = event::read()?
                && k.kind == KeyEventKind::Press
            {
                self.update(k);
            }
        }
        execute!(io::stdout(), SetCursorStyle::DefaultUserShape)?;
        Ok(())
    }

    pub fn new(vault: PathBuf) -> Self {
        let vault_passed = !vault.as_os_str().is_empty();

        let current_vault = if vault_passed {
            std::fs::canonicalize(&vault).unwrap_or(vault)
        } else if cache_exists() {
            get_vault().unwrap_or_default()
        } else {
            PathBuf::new()
        };

        let state = if !vault_passed && cache_exists() {
            AppState::Note
        } else {
            AppState::Menu
        };

        let confirm = if vault_passed {
            Some(ConfirmPrompt {
                message: format!("Open {} as a vault?", current_vault.to_string_lossy()),
                subject: ConfirmSubject::StartVault,
            })
        } else {
            None
        };

        let mut app = Self {
            state,
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
            editor: EditorState::default(),
            editor_handler: EditorEventHandler::default(),
            current_note: PathBuf::new(),
            saved_content: String::new(),
            last_cursor_mode: None,
            need_help: false,
        };

        if matches!(app.state, AppState::Note) {
            app.load_note_items();
        }

        app
    }
}
