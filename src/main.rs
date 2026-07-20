// TODO: IMPROVE STRUCTURE BY MOVING MODULES TO DIFFERENT FILES
// TODO: ADD CACHE SO AppState IS STORED AND PERSISTED
// TODO: THROW ERROR IF VAULT DIRECTORY DOESN'T EXIST
// TODO: FIX UI INCONSISTENCIES
// TODO: ADD COMMAND HELP SCREEN
// TODO: ADD FILE/DIRECTORY RENAMING, MOVING AND DELETION

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    process::exit,
};

use crossterm::{
    self,
    cursor::SetCursorStyle,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
};
use edtui::{EditorEventHandler, EditorMode, EditorState, EditorTheme, EditorView, Lines};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph},
};
use walkdir::WalkDir;

// TODO: IMPROVE STRUCTURE BY MOVING MODULES TO DIFFERENT FILES

enum AppState {
    Menu,
    VaultSelect,
    Note,
}

enum FocusedTab {
    Explorer,
    Editor,
}

enum ConfirmSubject {
    Vault,
    Exit,
    StartVault,
}

struct ConfirmPrompt {
    message: String,
    subject: ConfirmSubject,
}

struct FileCreate {
    message: String,
    path: PathBuf,
    is_dir: bool,
    input: String,
    cursor_position: usize,
}

#[derive(Clone)]
struct NoteItem {
    path: PathBuf,
    depth: usize,
    is_dir: bool,
    expanded: bool,
}

struct App {
    state: AppState,
    focused_tab: FocusedTab,
    exit: bool,
    vault_files: Vec<PathBuf>,
    list_state: ListState,
    current_vault: PathBuf,
    current_dir: PathBuf,
    confirm: Option<ConfirmPrompt>,
    file_create: Option<FileCreate>,
    note_files: Vec<NoteItem>,
    editor: EditorState,
    editor_handler: EditorEventHandler,
    current_note: PathBuf,
    note_changed: bool,
    saved_content: String,
    last_cursor_mode: Option<edtui::EditorMode>,
}

impl App {
    fn select_next(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(i.saturating_add(1)));
    }

    fn select_previous(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.list_state.select(Some(i.saturating_sub(1)));
        }
    }

    fn confirm_exit(&mut self) {
        self.confirm = Some(ConfirmPrompt {
            message: "Are you sure you want to quit?".into(),
            subject: ConfirmSubject::Exit,
        });
    }

    fn load_note_into_editor(&mut self, contents: String) {
        self.note_changed = false;
        self.saved_content = contents.clone();
        self.editor = EditorState::new(Lines::from(contents));
    }

    fn save_current_note(&mut self) -> io::Result<()> {
        if self.current_note.as_os_str().is_empty() {
            return Ok(());
        }
        let content = self.editor.lines.to_string();
        fs::write(&self.current_note, &content)?;
        self.note_changed = false;
        self.saved_content = content;
        Ok(())
    }

    fn new(vault: PathBuf) -> Self {
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
            note_files: Vec::new(),
            note_changed: false,
            editor: EditorState::default(),
            editor_handler: EditorEventHandler::default(),
            current_note: PathBuf::new(),
            saved_content: String::new(),
            last_cursor_mode: None,
        }
    }

    fn creation_base_dir(&self) -> PathBuf {
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

    fn reload_note_tree(&mut self, force_expand: Option<&Path>) {
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
        self.build_tree_level(&self.current_vault, 0, &expanded, &mut items);
        self.note_files = items;

        if self.note_files.is_empty() {
            self.list_state.select(None);
        } else {
            let current = self.list_state.selected().unwrap_or(0);
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

    fn view(&mut self, frame: &mut Frame) {
        self.apply_cursor_shape();

        match self.state {
            AppState::Menu => self.menu(frame),
            AppState::VaultSelect => self.vault_select(frame),
            AppState::Note => self.note(frame),
        }

        if let Some(prompt) = &self.confirm {
            self.draw_confirm(frame, frame.area(), prompt);
        }

        if let Some(prompt) = &self.file_create {
            self.draw_file_create(frame, frame.area(), prompt);
        }
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(vertical[1])[1]
    }

    fn draw_confirm(&self, frame: &mut Frame, area: Rect, prompt: &ConfirmPrompt) {
        let popup = self.centered_rect(50, 20, area);

        frame.render_widget(Clear, popup);

        let text = format!("{}\n\n[Y] Yes    [N] No", prompt.message);
        let widget = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered().title(" Confirm "));

        frame.render_widget(widget, popup);
    }

    fn draw_file_create(&self, frame: &mut Frame, area: Rect, prompt: &FileCreate) {
        let height = 3u16;
        let width = 50u16;

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup = Rect::new(x, y, width.min(area.width), height.min(area.height));

        frame.render_widget(Clear, popup);

        let block = Block::bordered()
            .title(format!(" {} ", prompt.message))
            .title_bottom(Line::from(vec![" Esc".bold(), " to cancel ".into()]))
            .title_bottom(Line::from(vec![" Enter".bold(), " to create ".into()]))
            .title_alignment(Alignment::Center);

        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let input_area = inner_layout[0];
        let visible_width = input_area.width as usize;
        let mut cursor_offset = prompt.cursor_position.min(prompt.input.len());

        let display_start = if cursor_offset > visible_width {
            cursor_offset - visible_width
        } else {
            0
        };

        let chars: Vec<char> = prompt.input.chars().collect();
        let display_end = (display_start + visible_width).min(chars.len());
        let visible_text: String = chars[display_start..display_end].iter().collect();

        cursor_offset -= display_start;
        cursor_offset = cursor_offset.min(visible_width.saturating_sub(1));

        let input = Paragraph::new(visible_text).style(Style::default().fg(Color::Yellow));
        frame.render_widget(input, input_area);
        frame.set_cursor_position((input_area.x + cursor_offset as u16, input_area.y));
    }

    fn travdir(&mut self, dir_path: PathBuf) {
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
                Some(edtui::EditorMode::Normal) => SetCursorStyle::SteadyBlock,
                Some(edtui::EditorMode::Insert) => SetCursorStyle::SteadyBar,
                Some(edtui::EditorMode::Visual) => SetCursorStyle::SteadyUnderScore,
                Some(edtui::EditorMode::Search) => SetCursorStyle::SteadyUnderScore,
                None => SetCursorStyle::DefaultUserShape,
            };
            let _ = execute!(io::stdout(), style);
            self.last_cursor_mode = want;
        }
    }

    fn load_note_items(&mut self) {
        let mut items = Vec::new();

        if let Ok(read_dir) = fs::read_dir(&self.current_vault) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .map_or(false, |n| n.to_str().map_or(false, |s| s.starts_with('.')))
                {
                    continue;
                }

                let is_dir = path.is_dir();

                items.push(NoteItem {
                    path,
                    depth: 0,
                    is_dir,
                    expanded: false,
                });
            }
        }

        items.sort_by_key(|i| {
            (
                !i.is_dir,
                i.path
                    .file_name()
                    .map_or(String::new(), |n| n.to_string_lossy().to_string()),
            )
        });

        self.note_files = items;
        self.editor = EditorState::default();
        self.current_note = PathBuf::default();
        self.saved_content.clear();

        if self.note_files.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn toggle_expand(&mut self, index: usize) {
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

    fn update(&mut self, key: KeyEvent) {
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

            if key.code == KeyCode::Esc && self.editor.mode == EditorMode::Normal {
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
                KeyCode::Char('j') => self.select_next(),
                KeyCode::Char('k') => self.select_previous(),
                KeyCode::Tab => self.focused_tab = FocusedTab::Editor,
                KeyCode::Char('q') => self.confirm_exit(),
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

    fn menu(&mut self, frame: &mut Frame) {
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(9),
                Constraint::Min(1),
            ])
            .split(frame.area())[1];

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        let title = Paragraph::new("NeoNote".bold().blue()).alignment(Alignment::Center);
        let description = Paragraph::new(
            "NNote is a keyboard-first note taking app in your terminal\n\
             Local Markdown notes, simple, quick and lightweight\n\n\
             Start by opening a vault",
        )
        .alignment(Alignment::Center);
        let vault_option = Paragraph::new(Line::from(vec!["v".bold(), " to open vault".into()]))
            .alignment(Alignment::Center);
        let quit_option = Paragraph::new(Line::from(vec!["q".bold(), " to quit".into()]))
            .alignment(Alignment::Center);

        frame.render_widget(title, inner[0]);
        frame.render_widget(description, inner[2]);
        frame.render_widget(vault_option, inner[4]);
        frame.render_widget(quit_option, inner[5]);
    }

    fn vault_select(&mut self, frame: &mut Frame) {
        let outer_padded_area = frame.area().inner(Margin {
            horizontal: 30,
            vertical: 6,
        });
        let items: Vec<ListItem> = self
            .vault_files
            .iter()
            .filter_map(|f| {
                f.file_name().map(|name| {
                    ListItem::new(name.to_string_lossy().to_string()).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                })
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(format!(" Path: {} ", self.current_dir.display()))
                    .title_bottom(Line::from(vec![" h/j/k/l".bold(), " to move ".into()]))
                    .title_bottom(Line::from(vec![" c".bold(), " to create dir ".into()]))
                    .title_bottom(Line::from(vec![" Enter".bold(), " to open vault ".into()]))
                    .title_bottom(Line::from(vec![" q".bold(), " to quit ".into()]))
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" -> ");
        frame.render_stateful_widget(list, outer_padded_area, &mut self.list_state);
    }

    fn note(&mut self, frame: &mut Frame) {
        let outer = frame.area();
        let outer_block = Block::bordered()
            .title(" NeoNote ")
            .title_alignment(Alignment::Center);
        let inner = outer_block.inner(outer);
        frame.render_widget(outer_block, outer);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner);
        let explorer_area = layout[0];
        let content_area = layout[1];

        let items: Vec<ListItem> = self
            .note_files
            .iter()
            .map(|item| {
                let indent = "  ".repeat(item.depth);
                let name = item
                    .path
                    .file_name()
                    .map_or(String::new(), |n| n.to_string_lossy().to_string());
                let symbol = if item.is_dir {
                    if item.expanded { "▾ " } else { "▸ " }
                } else {
                    "  "
                };
                let text = format!("{}{}{}", indent, symbol, name);
                let explorer_items_style = match self.focused_tab {
                    FocusedTab::Explorer => {
                        if item.is_dir {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Reset)
                        }
                    }
                    FocusedTab::Editor => Style::default().fg(Color::Gray),
                };
                ListItem::new(text).style(explorer_items_style)
            })
            .collect();

        let explorer_border_style = match self.focused_tab {
            FocusedTab::Explorer => Style::default().fg(Color::Reset),
            FocusedTab::Editor => Style::default().fg(Color::Gray),
        };

        let explorer_highlight_style = match self.focused_tab {
            FocusedTab::Explorer => Style::default()
                .fg(Color::Black)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
            FocusedTab::Editor => Style::default(),
        };

        let explorer_list = List::new(items)
            .block(
                Block::bordered()
                    .title(" Explorer ")
                    .title(format!(
                        " {} ",
                        self.current_vault
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ))
                    .title_bottom(Line::from(vec![" j/k".bold(), " to move ".into()]))
                    .title_bottom(Line::from(vec![" Enter".bold(), " to open ".into()]))
                    .title_bottom(Line::from(vec![" Tab".bold(), " to switch ".into()]))
                    .title_bottom(Line::from(vec![" c".bold(), " new dir ".into()]))
                    .title_bottom(Line::from(vec![" f".bold(), " new file ".into()]))
                    .title_bottom(Line::from(vec![" q".bold(), " to quit ".into()]))
                    .border_style(explorer_border_style),
            )
            .highlight_style(explorer_highlight_style)
            .highlight_symbol(" ");

        frame.render_stateful_widget(explorer_list, explorer_area, &mut self.list_state);

        let editor_border_style = match self.focused_tab {
            FocusedTab::Editor => Style::default().fg(Color::Reset),
            FocusedTab::Explorer => Style::default().fg(Color::Gray),
        };

        let note_file_name = self
            .current_note
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let editor_title = if note_file_name.is_empty() {
            " Editor ".to_string()
        } else if self.note_changed {
            format!(" {}* ", note_file_name)
        } else {
            format!(" {} ", note_file_name)
        };

        let editor_block = Block::bordered()
            .title(editor_title)
            .title_bottom(Line::from(vec![" Esc".bold(), " to exit ".into()]))
            .title_bottom(Line::from(vec![" Ctrl+s".bold(), " to save ".into()]))
            .title_bottom(Line::from(vec![" Ctrl+q".bold(), " to quit ".into()]))
            .border_style(editor_border_style);

        let theme = EditorTheme::default().block(editor_block).hide_cursor();

        frame.render_widget(EditorView::new(&mut self.editor).theme(theme), content_area);

        if matches!(self.focused_tab, FocusedTab::Editor) {
            if let Some(pos) = self.editor.cursor_screen_position() {
                frame.set_cursor_position(pos);
            }
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let vault = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .map(PathBuf::from);

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "Note taking application\n\n\
                     Usage: {} [OPTIONS] VAULT\n\n\
                     Options:\n\
                     -h, --help       Print this message\n\
                     -v, --version    Print version information",
                    args[0]
                );
                return Ok(());
            }
            "--version" | "-v" => {
                println!("NeoNote v0.2.2");
                return Ok(());
            }
            other if other.starts_with('-') => {
                eprintln!(
                    "error: no such option '{other}'\n\
                     use the option '-h' or '--help' for help\n\n\
                     Usage: {} [OPTIONS] VAULT",
                    args[0]
                );
                exit(1);
            }
            _ => {}
        }
    }

    let vault = vault.unwrap_or_default();

    if !vault.as_os_str().is_empty() && !vault.exists() {
        eprintln!(
            "error: path does not exist or is not accessible\n\
             use the option '-h' or '--help' for help\n\n\
             Usage: {} [OPTIONS] VAULT",
            args[0]
        );
        exit(1);
    }

    let mut terminal = ratatui::init();
    let mut app = App::new(vault);

    while !app.exit {
        terminal.draw(|frame| app.view(frame))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.update(key);
            }
        }
    }

    crossterm::execute!(io::stdout(), SetCursorStyle::DefaultUserShape)?;
    ratatui::restore();
    Ok(())
}
