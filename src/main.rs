// TODO: IMPROVE STRUCTURE BY MOVING MODULES TO DIFFERENT FILES
// TODO: ADD CACHE SO AppState IS STORED AND PERSISTED
// TODO: ADD DIRECTORY AND FILE CREATION
// TODO: IMPLEMENT COMMAND ARGUMENTS

use std::{fs, io, path::PathBuf};

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
    DirCreate,
}

enum FocusedTab {
    Explorer,
    Editor,
}

enum ConfirmSubject {
    Vault,
    Exit,
}

struct ConfirmPrompt {
    message: String,
    subject: ConfirmSubject,
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
        self.note_changed = false;
        if self.current_note.as_os_str().is_empty() {
            return Ok(());
        }
        let content = self.editor.lines.to_string();
        fs::write(&self.current_note, &content)?;
        self.saved_content = content;
        Ok(())
    }

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
            note_changed: false,
            editor: EditorState::default(),
            editor_handler: EditorEventHandler::default(),
            current_note: PathBuf::default(),
            saved_content: String::new(),
            last_cursor_mode: None,
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

        self.apply_cursor_shape();
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
                    ConfirmSubject::Exit => self.exit = true,
                },
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm = None;
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
            (AppState::VaultSelect, KeyCode::Char('c')) => self.state = AppState::DirCreate,

            (AppState::DirCreate, KeyCode::Esc) => {
                self.state = AppState::VaultSelect;
                self.input.clear();
                self.cursor_position = 0;
            }

            // TODO: INPUT IS NOT CHECKED BEFORE CREATING DIRECTORY AND USER IS NOT WARNED IN CASE
            // OF BAD INPUT
            (AppState::DirCreate, KeyCode::Enter) => {
                // FIX: INPUT IS NOT CHECKED BEFORE CREATING DIRECTORY
                let new_dir = format!(
                    "{}/{}",
                    self.current_dir.to_string_lossy(),
                    self.input.clone()
                );
                let _ = fs::create_dir(new_dir);
                self.travdir(self.current_dir.clone());
                self.list_state.select(Some(0));
                self.state = AppState::VaultSelect;
                self.input.clear();
                self.cursor_position = 0;
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
            _ => {}
        }
    }

    fn menu(&self, frame: &mut Frame) {
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
                    .title_bottom(Line::from(vec![" c".bold(), " to create vault ".into()]))
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

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "Note taking application\n\n\
                     Usage: {} [OPTIONS]\n\n\
                     Options:\n\
                     -h, --help       Print this message\n\
                     -v, --version    Print version information",
                    args[0]
                );
                return Ok(());
            }
            "--version" | "-v" => {
                println!("NeoNote v0.2.0");
                return Ok(());
            }
            _ => {
                eprintln!(
                    "error: no such option or command '{}'\n\
                     use the option '-h' or '--help' for help\n\n\
                     Usage: {} [OPTIONS]",
                    arg, args[0]
                );
                std::process::exit(1);
            }
        }
    }

    let mut terminal = ratatui::init();
    let mut app = App::new();

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
