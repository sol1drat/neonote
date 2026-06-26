use std::io;

use walkdir::WalkDir;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, ListItem, List, Paragraph},
};

enum AppState {
    Menu,
    VaultSelect,
}

struct App {
    state: AppState,
    exit: bool,
    vault_files: Vec<std::path::PathBuf>, 
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Menu,
            exit: false,
            vault_files: Vec::new(),
        }
    }

    fn view(&self, frame: &mut ratatui::Frame) {
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
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter_map(|e| {
                        let name = e.file_name();
                        if name.to_string_lossy().starts_with('.') {
                            return None;
                        }
                        Some(e.into_path())
                    })
                    .collect();
                self.state = AppState::VaultSelect;
            }

            (AppState::VaultSelect, KeyCode::Char('m')) => {
                self.state = AppState::Menu;
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

    fn vault_select(&self, frame: &mut ratatui::Frame) {
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(frame.area())[1];

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area)[1];

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
            .block(Block::bordered().title("Select a Vault"));

        frame.render_widget(list, area);
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
