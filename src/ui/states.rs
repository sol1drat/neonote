use edtui::{EditorTheme, EditorView};
use ratatui::{
    layout::{Constraint, Direction, HorizontalAlignment, Layout, Margin},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::{
    app::App,
    constants::{DESCRIPTION, TITLE},
    types::FocusedTab,
};

impl App {
    pub fn menu(&mut self, frame: &mut ratatui::Frame) {
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

        let title = Paragraph::new(TITLE.bold().blue()).alignment(HorizontalAlignment::Center);
        let description = Paragraph::new(DESCRIPTION).alignment(HorizontalAlignment::Center);
        let vault_option = Paragraph::new(Line::from(vec!["v".bold(), " to open vault".into()]))
            .alignment(HorizontalAlignment::Center);
        let quit_option = Paragraph::new(Line::from(vec!["q".bold(), " to quit".into()]))
            .alignment(HorizontalAlignment::Center);

        frame.render_widget(title, inner[0]);
        frame.render_widget(description, inner[2]);
        frame.render_widget(vault_option, inner[4]);
        frame.render_widget(quit_option, inner[5]);
    }

    pub fn vault_select(&mut self, frame: &mut ratatui::Frame) {
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
                    .title_alignment(HorizontalAlignment::Center),
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

    pub fn note(&mut self, frame: &mut ratatui::Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(frame.area());
        let explorer_area = layout[0];
        let content_area = layout[1];

        let items: Vec<ListItem> = self
            .note_files
            .iter()
            .map(|item| {
                let indent = "  ".repeat(item.depth);
                let name = if item.path.extension().is_some_and(|ext| ext == "md") {
                    item.path
                        .file_stem()
                        .map_or(String::new(), |n| n.to_string_lossy().to_string())
                } else {
                    item.path
                        .file_name()
                        .map_or(String::new(), |n| n.to_string_lossy().to_string())
                };
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
                    .title_bottom(Line::from(vec![" h".bold(), " for help ".into()]))
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
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let editor_title = if note_file_name.is_empty() {
            " Editor ".to_string()
        } else {
            format!(" {} ", note_file_name)
        };

        let editor_block = Block::bordered()
            .title(editor_title)
            .title_bottom(Line::from(vec![
                " Esc (Normal)".bold(),
                " to switch focus ".into(),
            ]))
            .title_bottom(Line::from(vec![" Ctrl+q".bold(), " to quit ".into()]))
            .style(Style::default().bg(Color::Reset))
            .border_style(editor_border_style);

        let theme = EditorTheme::default()
            .block(editor_block)
            .base(Style::default().bg(Color::Reset))
            .hide_cursor();

        frame.render_widget(EditorView::new(&mut self.editor).theme(theme), content_area);

        if matches!(self.focused_tab, FocusedTab::Editor)
            && let Some(pos) = self.editor.cursor_screen_position()
        {
            frame.set_cursor_position(pos);
        }
    }
}
