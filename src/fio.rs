use std::{fs, io, path::PathBuf};

use edtui::{EditorState, Lines};
use walkdir::WalkDir;

use crate::app::App;

impl App {
    pub fn load_note_into_editor(&mut self, contents: String) {
        self.note_changed = false;
        self.saved_content = contents.clone();
        self.editor = EditorState::new(Lines::from(contents));
    }

    pub fn save_current_note(&mut self) -> io::Result<()> {
        if self.current_note.as_os_str().is_empty() {
            return Ok(());
        }
        let content = self.editor.lines.to_string();
        fs::write(&self.current_note, &content)?;
        self.note_changed = false;
        self.saved_content = content;
        Ok(())
    }

    pub fn travdir(&mut self, dir_path: PathBuf) {
        let mut files: Vec<PathBuf> = WalkDir::new(&dir_path)
            .max_depth(1)
            .into_iter()
            .filter_entry(|e| e.file_type().is_dir())
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != dir_path.as_path())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|name| !name.starts_with('.'))
                    .unwrap_or(true)
            })
            .map(|e| e.into_path())
            .collect();
        files.sort();

        self.vault_files = files;
        if self.vault_files.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn creation_base_dir(&self) -> PathBuf {
        if let Some(idx) = self.list_state.selected() {
            if let Some(item) = self.note_files.get(idx) {
                if item.is_dir {
                    return item.path.clone();
                } else {
                    return item
                        .path
                        .parent()
                        .unwrap_or(&self.current_vault)
                        .to_path_buf();
                }
            }
        }
        self.current_vault.clone()
    }
}
