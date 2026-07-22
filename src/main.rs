// TODO: ADD FILE/DIRECTORY RENAMING, MOVING AND DELETION

// TODO: ADD CACHE SO AppState IS STORED AND PERSISTED
// TODO: THROW ERROR IF VAULT DIRECTORY DOESN'T EXIST
// TODO: FIX UI INCONSISTENCIES
// TODO: ADD COMMAND HELP SCREEN

mod app;
mod clargs;
mod constants;
mod fio;
mod handlers;
mod tree;
mod types;
mod ui;

use std::io;

use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyEventKind},
    execute,
};

use crate::app::App;

fn main() -> io::Result<()> {
    let vault = clargs::parse_args();

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

    execute!(io::stdout(), SetCursorStyle::DefaultUserShape)?;
    ratatui::restore();
    Ok(())
}
