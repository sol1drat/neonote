use std::io;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal,
};

pub enum AppState {
    Menu,
    Note,
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|frame| {
            let hello = Paragraph::new("Hello, World!")
                .block(Block::default().title("Hello").borders(Borders::ALL));

            frame.render_widget(hello, frame.area());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    ratatui::restore();
    Ok(())
}
