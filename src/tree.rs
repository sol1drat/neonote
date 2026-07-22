use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::app::App;
use crate::types::NoteItem;

impl App {
    pub fn get_selected_note_item(&self) -> Option<&NoteItem> {
        let idx = self.list_state.selected()?;
        self.note_files.get(idx)
    }

    pub fn reload_note_tree(&mut self, force_expand: Option<&Path>) {
        let mut expanded: HashSet<PathBuf> = self
            .note_files
            .iter()
            .filter(|i| i.expanded)
            .map(|i| i.path.clone())
            .collect();

        if let Some(p) = force_expand {
            expanded.insert(p.to_path_buf());
        }

        let mut items = Vec::new();

        let root_expanded = expanded.contains(&self.current_vault);
        items.push(NoteItem {
            path: self.current_vault.clone(),
            depth: 0,
            is_dir: true,
            expanded: root_expanded,
        });

        if root_expanded {
            self.build_tree_level(&self.current_vault, 1, &expanded, &mut items);
        }

        self.note_files = items;

        if self.note_files.is_empty() {
            self.list_state.select(None);
        } else {
            let current = self.list_state.selected().unwrap_or(1);
            self.list_state
                .select(Some(current.min(self.note_files.len() - 1)));
        }
    }

    fn build_tree_level(
        &self,
        dir: &Path,
        depth: usize,
        expanded: &HashSet<PathBuf>,
        items: &mut Vec<NoteItem>,
    ) {
        let mut entries: Vec<NoteItem> = Vec::new();
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .map_or(false, |n| n.to_str().map_or(false, |s| s.starts_with('.')))
                {
                    continue;
                }
                let is_dir = path.is_dir();
                let is_expanded = expanded.contains(&path);
                entries.push(NoteItem {
                    path,
                    depth,
                    is_dir,
                    expanded: is_expanded,
                });
            }
        }
        entries.sort_by_key(|i| {
            (
                !i.is_dir,
                i.path
                    .file_name()
                    .map_or(String::new(), |n| n.to_string_lossy().to_string()),
            )
        });

        for entry in entries {
            let should_expand = entry.is_dir && entry.expanded;
            items.push(entry);
            if should_expand {
                let path = items.last().unwrap().path.clone();
                self.build_tree_level(&path, depth + 1, expanded, items);
            }
        }
    }

    pub fn load_note_items(&mut self) {
        let mut items = vec![NoteItem {
            path: self.current_vault.clone(),
            depth: 0,
            is_dir: true,
            expanded: true,
        }];

        let mut expanded = HashSet::new();
        expanded.insert(self.current_vault.clone());
        self.build_tree_level(&self.current_vault, 1, &expanded, &mut items);

        self.note_files = items;
        self.editor = edtui::EditorState::default();
        self.current_note = PathBuf::default();
        self.saved_content.clear();

        if self.note_files.is_empty() {
            self.list_state.select(None);
        } else {
            let select_idx = if self.note_files.len() > 1 { 1 } else { 0 };
            self.list_state.select(Some(select_idx));
        }
    }

    pub fn toggle_expand(&mut self, index: usize) {
        if index >= self.note_files.len() {
            return;
        }

        let is_expanded = self.note_files[index].expanded;
        if is_expanded {
            let current_depth = self.note_files[index].depth;
            self.note_files[index].expanded = false;

            let i = index + 1;
            while i < self.note_files.len() {
                if self.note_files[i].depth <= current_depth {
                    break;
                }
                self.note_files.remove(i);
            }
        } else {
            self.note_files[index].expanded = true;
            let path = self.note_files[index].path.clone();
            let depth = self.note_files[index].depth + 1;

            let mut new_items = Vec::new();
            if let Ok(read_dir) = fs::read_dir(&path) {
                for entry in read_dir.flatten() {
                    let child_path = entry.path();
                    if child_path
                        .file_name()
                        .map_or(false, |n| n.to_str().map_or(false, |s| s.starts_with('.')))
                    {
                        continue;
                    }

                    let is_dir = child_path.is_dir();

                    new_items.push(NoteItem {
                        path: child_path,
                        depth,
                        is_dir,
                        expanded: false,
                    });
                }
            }
            new_items.sort_by_key(|i| {
                (
                    !i.is_dir,
                    i.path
                        .file_name()
                        .map_or(String::new(), |n| n.to_string_lossy().to_string()),
                )
            });
            for (offset, item) in new_items.into_iter().enumerate() {
                self.note_files.insert(index + 1 + offset, item);
            }
        }
    }
}
