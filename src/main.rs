use std::{io, path::PathBuf};

use walkdir::WalkDir;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, ListItem, List, ListState, Paragraph, Clear},
};

enum AppState {
    Menu,
    VaultSelect,
}

struct App {
    state: AppState,
    exit: bool,
    vault_files: Vec<PathBuf>, 
    list_state: ListState,
    current_dir: PathBuf,
    creating_dir: bool,
    input: String,         
    cursor_position: usize,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Menu,
            exit: false,
            vault_files: Vec::new(),
            list_state: ListState::default(),
            current_dir: PathBuf::default(),
            creating_dir: false,
            input: String::new(),
            cursor_position: 0,
        }
    }

    fn view(&mut self, frame: &mut ratatui::Frame) {
        match self.state {
            AppState::Menu => self.menu(frame),
            AppState::VaultSelect => self.vault_select(frame),
        }
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

    fn update(&mut self, key: KeyCode) {
        match (&self.state, key) {
            (_, KeyCode::Char('q')) => self.exit = true,

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
                            self.current_dir = full_path;
                            self.travdir(self.current_dir.clone());
                            self.list_state.select(Some(0));
                        }
                    }
                }
            }

            (AppState::VaultSelect, KeyCode::Char('c')) if !self.creating_dir => {
                self.creating_dir = true;
            }

            (AppState::VaultSelect, KeyCode::Esc) if self.creating_dir => {
                self.creating_dir = false;
            }

            (AppState::VaultSelect, KeyCode::Char(c)) if self.creating_dir => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }

            (AppState::VaultSelect, KeyCode::Backspace) if self.creating_dir => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }

            (AppState::VaultSelect, KeyCode::Left) => if self.creating_dir {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }

            (AppState::VaultSelect, KeyCode::Right) =>  if self.creating_dir {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }

            _ => {}
        }
    }

    fn menu(&self, frame: &mut ratatui::Frame) {
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

        let title = Paragraph::new("NeoNote".bold().blue()).alignment(Alignment::Center);

        let description = Paragraph::new(
            "NNote is a keyboard-first note taking app in your terminal\n\
             Local Markdown notes, simple, quick and lightweight\n\n\
             Start by opening a vault"
        )
            .alignment(Alignment::Center);

        let vault_option = Paragraph::new(Line::from(vec![
                "v".bold(),
                " to open a vault".into(),
        ]))
            .alignment(Alignment::Center);

        let quit_option = Paragraph::new(Line::from(vec![
                "q".bold(),
                " to quit".into(),
        ]))
            .alignment(Alignment::Center);

        frame.render_widget(title, inner[0]);
        frame.render_widget(description, inner[2]);
        frame.render_widget(vault_option, inner[4]);
        frame.render_widget(quit_option, inner[5]);
    }

    fn vault_select(&mut self, frame: &mut ratatui::Frame) {
        let outer_padded_area = frame.area().inner(Margin {
            horizontal: 30,
            vertical: 6,
        });

        let items: Vec<ListItem> = self
            .vault_files
            .iter()
            .filter_map(|f| {
                f.file_name().map(|name| {
                    ListItem::new(name.to_string_lossy().to_string())
                })
            })
        .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                .title(format!(" Select a vault: {} ", self.current_dir.display()))
                .title_bottom(Line::from(vec![
                        " h/j/k/l".bold(),
                        " to move ".into(),
                ]))
                .title_bottom(Line::from(vec![
                        " q".bold(),
                        " to quit ".into(),
                ]))
                .title_alignment(Alignment::Center)
            )
            .highlight_style(
                ratatui::style::Style::default()
                .fg(Color::Black)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("-> ");

        frame.render_stateful_widget(list, outer_padded_area, &mut self.list_state);

        if self.creating_dir {
            let padded_area = frame.area().inner(Margin {
                horizontal: 45,
                vertical: 16,
            });

            frame.render_widget(Clear, padded_area);

            let input_paragraph = Paragraph::new(self.input.as_str())
                .block(Block::bordered()
                    .title("Create a directory")
                    .title_bottom("Esc to close")
                    .title_bottom("Enter to create a directory"));

            frame.render_widget(input_paragraph, padded_area);

            let cursor_x = padded_area.x + self.cursor_position as u16 + 1; 
            let cursor_y = padded_area.y + 1;

            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();

    while !app.exit {
        terminal.draw(|frame| app.view(frame))?;
        if let Event::Key(key) = event::read()? { app.update(key.code); }
    }

    ratatui::restore();
    Ok(())
}
