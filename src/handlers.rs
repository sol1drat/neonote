use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use edtui::{EditorMode, EditorState};

use crate::app::App;
use crate::types::{AppState, ConfirmPrompt, ConfirmSubject, FileCreate, FileRename, FocusedTab};

impl App {
    pub fn update(&mut self, key: KeyEvent) {
        if self.need_help {
            match key.code {
                KeyCode::Esc => {
                    self.need_help = false;
                }
                _ => {}
            }
            return;
        }
        if let Some(prompt) = &self.confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => match prompt.subject {
                    ConfirmSubject::Vault => {
                        self.current_vault = self.current_dir.clone();
                        self.load_note_items();
                        self.confirm = None;
                        self.state = AppState::Note;
                    }
                    ConfirmSubject::StartVault => {
                        self.load_note_items();
                        self.confirm = None;
                        self.state = AppState::Note;
                    }
                    ConfirmSubject::Exit => self.exit = true,
                    ConfirmSubject::Delete => {
                        if let Some(item) = self.get_selected_note_item() {
                            let path = item.path.clone();
                            let is_dir = item.is_dir;
                            let parent = path.parent().unwrap_or(&self.current_vault).to_path_buf();

                            if is_dir {
                                let _ = fs::remove_dir_all(&path);
                            } else {
                                let _ = fs::remove_file(&path);
                                if self.current_note == path {
                                    self.editor = EditorState::default();
                                    self.current_note = PathBuf::new();
                                    self.note_changed = false;
                                    self.saved_content = String::new();
                                }
                            }

                            self.reload_note_tree(Some(&parent));

                            let current = self.list_state.selected().unwrap_or(0);
                            self.list_state
                                .select(Some(current.min(self.note_files.len().saturating_sub(1))));
                        }
                        self.confirm = None;
                    }
                },
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm = None;
                }
                _ => {}
            }
            return;
        }

        if self.file_create.is_some() {
            match key.code {
                KeyCode::Esc => {
                    self.file_create = None;
                }
                KeyCode::Enter => {
                    let (name, base, is_dir) = {
                        let p = self.file_create.as_ref().unwrap();
                        (p.input.clone(), p.path.clone(), p.is_dir)
                    };
                    let in_note = matches!(self.state, AppState::Note);
                    let in_vault = matches!(self.state, AppState::VaultSelect);

                    self.file_create = None;

                    if !name.is_empty() {
                        let new_path = base.join(&name);
                        if is_dir {
                            let _ = fs::create_dir(&new_path);
                        } else {
                            let _ = fs::write(&new_path, "");
                        }

                        if in_vault {
                            self.travdir(self.current_dir.clone());
                            let current = self.list_state.selected().unwrap_or(0);
                            self.list_state.select(Some(
                                current.min(self.vault_files.len().saturating_sub(1)),
                            ));
                        } else if in_note {
                            let parent = new_path
                                .parent()
                                .unwrap_or(&self.current_vault)
                                .to_path_buf();
                            self.reload_note_tree(Some(&parent));
                            if let Some(idx) =
                                self.note_files.iter().position(|i| i.path == new_path)
                            {
                                self.list_state.select(Some(idx));
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(p) = self.file_create.as_mut() {
                        p.input.insert(p.cursor_position, c);
                        p.cursor_position += 1;
                    }
                }
                KeyCode::Backspace => {
                    if let Some(p) = self.file_create.as_mut() {
                        if p.cursor_position > 0 {
                            p.input.remove(p.cursor_position - 1);
                            p.cursor_position -= 1;
                        }
                    }
                }
                KeyCode::Left => {
                    if let Some(p) = self.file_create.as_mut() {
                        if p.cursor_position > 0 {
                            p.cursor_position -= 1;
                        }
                    }
                }
                KeyCode::Right => {
                    if let Some(p) = self.file_create.as_mut() {
                        if p.cursor_position < p.input.len() {
                            p.cursor_position += 1;
                        }
                    }
                }
                _ => {}
            }
            return;
        } else if self.file_rename.is_some() {
            match key.code {
                KeyCode::Esc => {
                    self.file_rename = None;
                }
                KeyCode::Enter => {
                    let (name, base) = {
                        let p = self.file_rename.as_ref().unwrap();
                        (p.input.clone(), p.path.clone())
                    };

                    self.file_rename = None;

                    if !name.is_empty() {
                        let new_path = base.parent().unwrap().join(&name);
                        let _ = fs::rename(base, &new_path);

                        let parent = new_path
                            .parent()
                            .unwrap_or(&self.current_vault)
                            .to_path_buf();
                        self.reload_note_tree(Some(&parent));
                        if let Some(idx) = self.note_files.iter().position(|i| i.path == new_path) {
                            self.list_state.select(Some(idx));
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(p) = self.file_rename.as_mut() {
                        p.input.insert(p.cursor_position, c);
                        p.cursor_position += 1;
                    }
                }
                KeyCode::Backspace => {
                    if let Some(p) = self.file_rename.as_mut() {
                        if p.cursor_position > 0 {
                            p.input.remove(p.cursor_position - 1);
                            p.cursor_position -= 1;
                        }
                    }
                }
                KeyCode::Left => {
                    if let Some(p) = self.file_rename.as_mut() {
                        if p.cursor_position > 0 {
                            p.cursor_position -= 1;
                        }
                    }
                }
                KeyCode::Right => {
                    if let Some(p) = self.file_rename.as_mut() {
                        if p.cursor_position < p.input.len() {
                            p.cursor_position += 1;
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        if matches!(self.state, AppState::Note) && matches!(self.focused_tab, FocusedTab::Editor) {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                let _ = self.save_current_note();
                return;
            }

            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                self.confirm_exit();
                return;
            }

            if matches!(self.editor.mode, EditorMode::Normal) && key.code == KeyCode::Esc {
                self.focused_tab = FocusedTab::Explorer;
                return;
            }

            self.editor_handler.on_key_event(key, &mut self.editor);

            let editor_content = self.editor.lines.to_string();
            self.note_changed =
                !self.current_note.as_os_str().is_empty() && editor_content != self.saved_content;
            return;
        }

        if matches!(self.state, AppState::Note) && matches!(self.focused_tab, FocusedTab::Explorer)
        {
            match key.code {
                KeyCode::Char('q') => self.confirm_exit(),
                KeyCode::Esc => self.focused_tab = FocusedTab::Editor,
                KeyCode::Char('j') => self.select_next(),
                KeyCode::Char('k') => self.select_previous(),
                KeyCode::Char('c') => {
                    let base = self.creation_base_dir();
                    self.file_create = Some(FileCreate {
                        message: "Create Directory".into(),
                        path: base,
                        is_dir: true,
                        input: String::new(),
                        cursor_position: 0,
                    });
                }
                KeyCode::Char('f') => {
                    let base = self.creation_base_dir();
                    self.file_create = Some(FileCreate {
                        message: "Create File".into(),
                        path: base,
                        is_dir: false,
                        input: String::new(),
                        cursor_position: 0,
                    });
                }
                KeyCode::Char('r') => {
                    if let Some(item) = self.get_selected_note_item() {
                        self.file_rename = Some(FileRename {
                            path: item.path.clone(),
                            input: String::new(),
                            cursor_position: 0,
                        });
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(item) = self.get_selected_note_item() {
                        self.confirm = Some(ConfirmPrompt {
                            message: format!(
                                "Delete {}?",
                                item.path.file_name().unwrap().display()
                            ),
                            subject: ConfirmSubject::Delete,
                        });
                    }
                }
                KeyCode::Char('h') => {
                    self.need_help = true;
                }
                KeyCode::Enter => {
                    if let Some(idx) = self.list_state.selected() {
                        if let Some(item) = self.note_files.get(idx) {
                            if item.is_dir {
                                self.toggle_expand(idx);
                            } else if let Ok(contents) = fs::read_to_string(&item.path) {
                                self.current_note = item.path.clone();
                                self.load_note_into_editor(contents);
                                self.focused_tab = FocusedTab::Editor;
                            }
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        match (&self.state, key.code) {
            (AppState::Menu, KeyCode::Char('q')) => self.exit = true,
            (AppState::VaultSelect, KeyCode::Char('q')) => self.confirm_exit(),

            (AppState::Menu, KeyCode::Char('v')) => {
                if let Ok(path) = std::env::current_dir() {
                    self.current_dir = path;
                }
                self.travdir(self.current_dir.clone());
                self.state = AppState::VaultSelect;
            }

            (AppState::VaultSelect, KeyCode::Char('h')) => {
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.travdir(self.current_dir.clone());
                    self.list_state.select(Some(0));
                }
            }
            (AppState::VaultSelect, KeyCode::Char('j')) => self.select_next(),
            (AppState::VaultSelect, KeyCode::Char('k')) => self.select_previous(),
            (AppState::VaultSelect, KeyCode::Char('l')) => {
                if let Some(idx) = self.list_state.selected() {
                    if let Some(dir_path) = self.vault_files.get(idx) {
                        if let Ok(full_path) = fs::canonicalize(dir_path) {
                            self.current_dir = full_path.clone();
                            self.travdir(self.current_dir.clone());
                            self.list_state.select(Some(0));
                        }
                    }
                }
            }
            (AppState::VaultSelect, KeyCode::Enter) => {
                if let Some(idx) = self.list_state.selected() {
                    if let Some(dir_path) = self.vault_files.get(idx) {
                        if let Ok(full_path) = fs::canonicalize(dir_path) {
                            self.current_dir = full_path.clone();
                            self.confirm = Some(ConfirmPrompt {
                                message: format!("Open {} as a vault?", full_path.display()),
                                subject: ConfirmSubject::Vault,
                            });
                        }
                    }
                }
            }
            (AppState::VaultSelect, KeyCode::Char('c')) => {
                self.file_create = Some(FileCreate {
                    message: "Create Directory".into(),
                    path: self.current_dir.clone(),
                    is_dir: true,
                    input: String::new(),
                    cursor_position: 0,
                });
            }
            _ => {}
        }
    }
}
