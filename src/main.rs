use std::io;

use walkdir::WalkDir;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, ListItem, List, ListState, Paragraph},
};

enum AppState {
    Menu,
    VaultSelect,
}

struct App {
    state: AppState,
    exit: bool,
    vault_files: Vec<std::path::PathBuf>, 
    list_state: ListState,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Menu,
            exit: false,
            vault_files: Vec::new(),
            list_state: ListState::default(),
        }
    }

    fn view(&mut self, frame: &mut ratatui::Frame) {
        match self.state {
            AppState::Menu => self.menu(frame),
            AppState::VaultSelect => self.vault_select(frame),
        }
    }

    fn update(&mut self, key: KeyCode) {
        match (&self.state, key) {
            (_, KeyCode::Char('q')) => self.exit = true,

            (AppState::Menu, KeyCode::Char('v')) => {
                self.vault_files = WalkDir::new(".")
                    .max_depth(1)
                    .into_iter()
                    .filter_entry(|e| e.file_type().is_dir())
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path() != std::path::Path::new("."))
                    .map(|e| e.into_path())
                    .collect();

                self.state = AppState::VaultSelect;

                if !self.vault_files.is_empty() {
                    self.list_state.select(Some(0));
                } else {
                    self.list_state.select(None);
                }
            }

            (AppState::VaultSelect, KeyCode::Char('m')) => {
                self.state = AppState::Menu;
                self.list_state.select(None);
            }

            (AppState::VaultSelect, KeyCode::Char('j')) => {
                self.list_state.select_next();
            }

            (AppState::VaultSelect, KeyCode::Char('k')) => {
                self.list_state.select_previous();
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
                .title(" Select a Vault ")
                .title_bottom(Line::from(vec![
                        " j/k".bold(),
                        " to move ".into(),
                ]))
                .title_bottom(Line::from(vec![
                        " m".bold(),
                        " to open menu ".into(),
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
