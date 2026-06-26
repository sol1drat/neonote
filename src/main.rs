use std::io;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout,
    },
    text::Line,
    style::Stylize,
    widgets::Paragraph,
};

enum AppState {
    Menu,
    VaultSelect,
}

struct App {
    state: AppState,
    exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Menu,
            exit: false,
        }
    }

    fn view(&self, frame: &mut ratatui::Frame) {
        match self.state {
            AppState::Menu => self.menu(frame),
            AppState::VaultSelect => self.note(frame),
        }
    }

    fn update(&mut self, key: KeyCode) {
        match (&self.state, key) {
            (_, KeyCode::Char('q')) => self.exit = true,

            (AppState::Menu, KeyCode::Char('v')) => {
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

        let title = Paragraph::new("NeoNote".bold().blue())
            .alignment(Alignment::Center);

        let description = Paragraph::new(
            "NNote is a keyboard-first note taking app in your terminal\n\
                Local markdown notes, simple, quick and lightweight\n\n\
                Start by opening a vault"
        )
            .alignment(Alignment::Center);

        let vault_option = Paragraph::new(Line::from(vec![
                "v".bold(),
                " to open a vault".into(),
        ])).alignment(Alignment::Center);

        let quit_option = Paragraph::new(Line::from(vec![
                "q".bold(),
                " to quit".into(),
        ])).alignment(Alignment::Center);


        frame.render_widget(title, inner[0]);
        frame.render_widget(description, inner[2]);
        frame.render_widget(vault_option, inner[4]);
        frame.render_widget(quit_option, inner[5]);
    }

    fn note(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(
            Paragraph::new("Vault selection (TODO)"),
            frame.area(),
        );
    }
}


fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();

    while !app.exit {
        terminal.draw(|frame| { app.view(frame); })?;
        if let Event::Key(key) = event::read()? { app.update(key.code); }
    }

    ratatui::restore();
    Ok(())
}
