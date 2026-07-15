// NOTE: PRESSING ENTER WHILE FOCUSED ON EDITOR OPENS DIRECTORY

// TODO: IMPROVE STRUCTURE BY MOVING MODULES TO DIFFERENT FILES
// TODO: ADD CACHE SO AppState IS STORED AND PERSISTED

use std::{io, path::PathBuf};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use walkdir::WalkDir;

// TODO: IMPROVE STRUCTURE BY MOVING MODULES TO DIFFERENT FILES

enum AppState {
    Menu,
    VaultSelect,
    DirCreate,
}

enum FocusedTab {
    Explorer,
    Editor,
}

struct ConfirmPrompt {
    message: String,
    pending_vault: PathBuf,
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
    current_dir: PathBuf,
    input: String,
    cursor_position: usize,
    confirm: Option<ConfirmPrompt>,
    note_files: Vec<NoteItem>,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Menu,
            focused_tab: FocusedTab::Explorer,
            exit: false,
            vault_files: Vec::new(),
            list_state: ListState::default(),
            current_dir: PathBuf::default(),
            input: String::new(),
            cursor_position: 0,
            confirm: None,
            note_files: Vec::new(),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        match self.state {
            AppState::Menu => self.menu(frame),
            AppState::VaultSelect => self.vault_select(frame),
            AppState::DirCreate => {
                self.vault_select(frame);
                self.dir_create(frame);
            }
            AppState::Note => self.note(frame),
        }

        if let Some(prompt) = &self.confirm {
            self.draw_confirm(frame, frame.area(), prompt);
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
            .block(Block::default().title("Confirm").borders(Borders::ALL));

        frame.render_widget(widget, popup);
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

    fn load_note_items(&mut self) {
        let mut items = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(&self.current_vault) {
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
            if let Ok(read_dir) = std::fs::read_dir(&path) {
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

    fn update(&mut self, key: KeyCode) {
        if let Some(prompt) = &self.confirm {
            match key {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.current_vault = prompt.pending_vault.clone();
                    self.load_note_items();
                    self.confirm = None;
                    self.state = AppState::Note;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm = None;
                }
                _ => {}
            }
            return;
        }

        match (&self.state, key, &self.focused_tab) {
            (AppState::Note, KeyCode::Char('j'), FocusedTab::Explorer) => {
                self.list_state.select_next();
            }

            (AppState::Note, KeyCode::Char('k'), FocusedTab::Explorer) => {
                self.list_state.select_previous();
            }

            _ => {}
        }

        match (&self.state, key) {
            (AppState::Menu, KeyCode::Char('q')) => self.exit = true,
            (AppState::VaultSelect, KeyCode::Char('q')) => self.exit = true,
            (AppState::Note, KeyCode::Char('q')) => self.exit = true,

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

            (AppState::VaultSelect, KeyCode::Char('j')) => {
                self.list_state.select_next();
            }

            (AppState::VaultSelect, KeyCode::Char('k')) => {
                self.list_state.select_previous();
            }

            (AppState::VaultSelect, KeyCode::Char('l')) => {
                if let Some(selected_index) = self.list_state.selected() {
                    if let Some(dir_path) = self.vault_files.get(selected_index) {
                        if let Ok(full_path) = std::fs::canonicalize(dir_path) {
                            self.current_dir = full_path.clone();
                            self.travdir(self.current_dir.clone());
                            self.list_state.select(Some(0));
                        }
                    }
                }
            }

            (AppState::VaultSelect, KeyCode::Enter) => {
                if let Some(selected_index) = self.list_state.selected() {
                    if let Some(dir_path) = self.vault_files.get(selected_index) {
                        if let Ok(full_path) = std::fs::canonicalize(dir_path) {
                            self.current_dir = full_path.clone();
                            self.confirm = Some(ConfirmPrompt {
                                message: format!("Open {} as a vault?", full_path.display()),
                                pending_vault: full_path,
                            })
                        }
                    }
                }
            }

            (AppState::VaultSelect, KeyCode::Char('c')) => {
                self.state = AppState::DirCreate;
            }

            (AppState::DirCreate, KeyCode::Esc) => {
                self.state = AppState::VaultSelect;
            }

            (AppState::DirCreate, KeyCode::Enter) => {
                // FIX: INPUT IS NOT CHECKED BEFORE CREATING DIRECTORY
                let new_dir = format!(
                    "{}/{}",
                    self.current_dir.to_string_lossy(),
                    self.input.clone()
                );
                let _ = std::fs::create_dir(new_dir);
                self.travdir(self.current_dir.clone());
                self.list_state.select(Some(0));
                self.state = AppState::VaultSelect;
            }

            (AppState::DirCreate, KeyCode::Char(c)) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }

            (AppState::DirCreate, KeyCode::Backspace) => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }

            (AppState::DirCreate, KeyCode::Left) => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }

            (AppState::DirCreate, KeyCode::Right) => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }

            (AppState::Note, KeyCode::Tab) => {
                self.focused_tab = match self.focused_tab {
                    FocusedTab::Explorer => FocusedTab::Editor,
                    FocusedTab::Editor => FocusedTab::Explorer,
                };
            }

            (AppState::Note, KeyCode::Enter) => {
                if let Some(selected_index) = self.list_state.selected() {
                    if let Some(item) = self.note_files.get(selected_index) {
                        if item.is_dir {
                            self.toggle_expand(selected_index);
                        } else {
                            // TODO: OPEN FILE IN EDITOR
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn menu(&self, frame: &mut Frame) {
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(9),
                Constraint::Fill(1),
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

        // TODO: MOVE CONSTANTS TO DIFFERENT FILES

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
                    .title_bottom(Line::from(vec![
                        " c".bold(),
                        " to create new vault ".into(),
                    ]))
                    .title_bottom(Line::from(vec![
                        " Enter".bold(),
                        " to select vault ".into(),
                    ]))
                    .title_bottom(Line::from(vec![" q".bold(), " to quit ".into()]))
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(
                ratatui::style::Style::default()
                    .fg(Color::Black)
                    .bg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" -> ");

        frame.render_stateful_widget(list, outer_padded_area, &mut self.list_state);
    }

    fn dir_create(&mut self, frame: &mut Frame) {
        let height = 3u16;
        let width = 45u16;

        let x = frame.area().x + (frame.area().width.saturating_sub(width)) / 2;
        let y = frame.area().y + (frame.area().height.saturating_sub(height)) / 2;

        let area = Rect::new(
            x,
            y,
            width.min(frame.area().width),
            height.min(frame.area().height),
        );

        frame.render_widget(Clear, area);

        let block = Block::bordered()
            .title(" Create vault ")
            .title_bottom(Line::from(vec![" Esc".bold(), " to close ".into()]))
            .title_bottom(Line::from(vec![" Enter".bold(), " to create ".into()]))
            .title_alignment(Alignment::Center);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let visible_width = inner.width as usize;
        let mut cursor_offset = self.cursor_position.min(self.input.len());

        let display_start = if cursor_offset > visible_width {
            cursor_offset - visible_width
        } else {
            0
        };

        let chars: Vec<char> = self.input.chars().collect();
        let display_end = (display_start + visible_width).min(chars.len());

        let visible_text: String = chars[display_start..display_end].iter().collect();

        cursor_offset -= display_start;
        cursor_offset = cursor_offset.min(visible_width.saturating_sub(1));

        let input = Paragraph::new(visible_text).style(Style::default().fg(Color::Yellow));
        frame.render_widget(input, inner);

        frame.set_cursor_position((inner.x + cursor_offset as u16, inner.y));
    }

    fn note(&mut self, frame: &mut Frame) {
        let outer = frame.area();

        let outer_block = Block::bordered()
            .title(" NeoNote ")
            .title_alignment(Alignment::Center);

        let inner = outer_block.inner(outer);
        frame.render_widget(outer_block, outer);

        let [explorer_area, content_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .areas(inner);

        match self.focused_tab {
            FocusedTab::Explorer => {
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

                        let style = if item.is_dir {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Reset)
                        };

                        ListItem::new(text).style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::bordered()
                            .title(" Explorer ")
                            .title_bottom(Line::from(vec![" j/k".bold(), " to move ".into()]))
                            .title_bottom(Line::from(vec![" Enter".bold(), " to open ".into()]))
                            .title_bottom(Line::from(vec![" q".bold(), " to quit ".into()]))
                            .border_style(Style::default().fg(Color::Reset)),
                    )
                    .highlight_style(
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("  ");

                frame.render_stateful_widget(list, explorer_area, &mut self.list_state);

                let content_block = Block::bordered()
                    .title(" Note ")
                    .border_style(Style::default().fg(Color::DarkGray));

                frame.render_widget(content_block, content_area);
            }
            FocusedTab::Editor => {
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

                        ListItem::new(text).style(Style::default().fg(Color::DarkGray))
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::bordered()
                            .title(" Explorer ")
                            .title_bottom(Line::from(vec![" j/k".bold(), " to move ".into()]))
                            .title_bottom(Line::from(vec![" Enter".bold(), " to open ".into()]))
                            .title_bottom(Line::from(vec![" q".bold(), " to quit ".into()]))
                            .title_alignment(Alignment::Center)
                            .border_style(Style::default().fg(Color::DarkGray)),
                    )
                    .highlight_symbol("  ");

                frame.render_stateful_widget(list, explorer_area, &mut self.list_state);

                let content_block = Block::bordered()
                    .title(" Note ")
                    .border_style(Style::default().fg(Color::Reset));

                frame.render_widget(content_block, content_area);
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();

    while !app.exit {
        terminal.draw(|frame| app.view(frame))?;
        if let Event::Key(key) = event::read()? {
            app.update(key.code);
        }
    }

    ratatui::restore();
    Ok(())
}
