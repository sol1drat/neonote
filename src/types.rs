use std::path::PathBuf;

pub enum AppState {
    Menu,
    VaultSelect,
    Note,
}

pub enum FocusedTab {
    Explorer,
    Editor,
}

pub enum ConfirmSubject {
    Vault,
    Exit,
    StartVault,
}

pub struct ConfirmPrompt {
    pub message: String,
    pub subject: ConfirmSubject,
}

pub struct FileCreate {
    pub message: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub input: String,
    pub cursor_position: usize,
}

pub struct FileRename {
    pub path: PathBuf,
    pub input: String,
    pub cursor_position: usize,
}

#[derive(Clone)]
pub struct NoteItem {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}
