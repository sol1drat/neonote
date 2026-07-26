mod app;
mod cache;
mod clargs;
mod constants;
mod fio;
mod handlers;
mod misc;
mod tree;
mod types;
mod ui;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    app::App::new(clargs::parse_args()).run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}
