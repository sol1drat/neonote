use edtui::{EditorTheme, EditorView};
use ratatui::{
    layout::{Constraint, Direction, HorizontalAlignment, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, Paragraph},
};

use crate::constants::{DESCRIPTION, TITLE};
use crate::types::{ConfirmPrompt, FileCreate, FocusedTab};
use crate::{app::App, types::FileRename};

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
        let outer = frame.area();
        let outer_block = Block::bordered()
            .title(format!(" {} ", TITLE))
            .title_alignment(HorizontalAlignment::Center);
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
            .title_bottom(Line::from(vec![
                " Esc (Normal)".bold(),
                " to switch focus ".into(),
            ]))
            .title_bottom(Line::from(vec![" Ctrl+q".bold(), " to quit ".into()]))
            .style(Style::default().bg(Color::Reset))
            .border_style(editor_border_style);

        let theme = EditorTheme::default().block(editor_block).hide_cursor();

        frame.render_widget(EditorView::new(&mut self.editor).theme(theme), content_area);

        if matches!(self.focused_tab, FocusedTab::Editor) {
            if let Some(pos) = self.editor.cursor_screen_position() {
                frame.set_cursor_position(pos);
            }
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

    pub fn draw_help(&self, frame: &mut ratatui::Frame, area: Rect) {
        let popup = self.centered_rect(40, 60, area);
        frame.render_widget(Clear, popup);

        let key_style = Style::new().bold().yellow();
        let section_style = Style::new().bold().cyan().underlined();
        let desc_style = Style::default().dim();

        let sections: &[(&str, &[(&str, &str)])] = &[
            (
                "General",
                &[
                    ("q", "Quit the application"),
                    ("h", "Show this help screen"),
                    ("Esc", "Close this popup"),
                ],
            ),
            (
                "File Explorer",
                &[
                    ("q", "Quit the application"),
                    ("j/k", "Move selection down/up"),
                    ("Enter", "Open file/dir"),
                    ("Esc", "Focus editor"),
                    ("c", "Create directory"),
                    ("f", "Create file"),
                    ("r", "Rename file/dir"),
                    ("d", "Delete file/dir"),
                ],
            ),
            (
                "Editor",
                &[
                    ("Esc (Normal)", "Focus file explorer"),
                    ("Ctrl+s", "Save edits"),
                    ("Ctrl+q", "Quit the application"),
                ],
            ),
        ];

        let max_key_len = sections
            .iter()
            .flat_map(|(_, bindings)| bindings.iter())
            .map(|(keys, _)| keys.len())
            .max()
            .unwrap_or(0);

        let max_desc_len = sections
            .iter()
            .flat_map(|(_, bindings)| bindings.iter())
            .map(|(_, desc)| desc.len())
            .max()
            .unwrap_or(0);

        let mut lines: Vec<Line> = Vec::new();
        for (i, (title, bindings)) in sections.iter().enumerate() {
            if i > 0 {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(*title, section_style)));

            for (keys, desc) in bindings.iter() {
                lines.push(Line::from(vec![
                    Span::styled(format!("{:<width$}", keys, width = max_key_len), key_style),
                    Span::raw("  "),
                    Span::styled(
                        format!("{:<width$}", desc, width = max_desc_len),
                        desc_style,
                    ),
                ]));
            }
        }

        let widget = Paragraph::new(lines)
            .alignment(HorizontalAlignment::Center)
            .block(
                Block::bordered()
                    .title(" Help ")
                    .title_bottom(Line::from(vec![" Esc".bold(), " to close ".into()]))
                    .title_alignment(HorizontalAlignment::Center),
            );

        frame.render_widget(widget, popup);
    }

    pub fn draw_confirm(&self, frame: &mut ratatui::Frame, area: Rect, prompt: &ConfirmPrompt) {
        let popup = self.centered_rect(50, 20, area);

        frame.render_widget(Clear, popup);

        let text = format!("{}\n\n[Y] Yes    [N] No", prompt.message);
        let widget = Paragraph::new(text)
            .alignment(HorizontalAlignment::Center)
            .block(Block::bordered().title(" Confirm "));

        frame.render_widget(widget, popup);
    }

    pub fn draw_file_create(&self, frame: &mut ratatui::Frame, area: Rect, prompt: &FileCreate) {
        let height = 3u16;
        let width = 50u16;

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup = Rect::new(x, y, width.min(area.width), height.min(area.height));

        frame.render_widget(Clear, popup);

        let block = Block::bordered()
            .title(format!(" {} ", prompt.message))
            .title_bottom(Line::from(vec![" Esc".bold(), " to cancel ".into()]))
            .title_bottom(Line::from(vec![" Enter".bold(), " to create ".into()]))
            .title_alignment(HorizontalAlignment::Center);

        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let input_area = inner_layout[0];
        let visible_width = input_area.width as usize;
        let mut cursor_offset = prompt.cursor_position.min(prompt.input.len());

        let display_start = if cursor_offset > visible_width {
            cursor_offset - visible_width
        } else {
            0
        };

        let chars: Vec<char> = prompt.input.chars().collect();
        let display_end = (display_start + visible_width).min(chars.len());
        let visible_text: String = chars[display_start..display_end].iter().collect();

        cursor_offset -= display_start;
        cursor_offset = cursor_offset.min(visible_width.saturating_sub(1));

        let input = Paragraph::new(visible_text).style(Style::default().fg(Color::Yellow));
        frame.render_widget(input, input_area);
        frame.set_cursor_position((input_area.x + cursor_offset as u16, input_area.y));
    }

    pub fn draw_file_rename(&self, frame: &mut ratatui::Frame, area: Rect, prompt: &FileRename) {
        let height = 3u16;
        let width = 50u16;

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup = Rect::new(x, y, width.min(area.width), height.min(area.height));

        frame.render_widget(Clear, popup);

        let block = Block::bordered()
            .title(" Rename ")
            .title_bottom(Line::from(vec![" Esc".bold(), " to cancel ".into()]))
            .title_bottom(Line::from(vec![" Enter".bold(), " to create ".into()]))
            .title_alignment(HorizontalAlignment::Center);

        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let input_area = inner_layout[0];
        let visible_width = input_area.width as usize;
        let mut cursor_offset = prompt.cursor_position.min(prompt.input.len());

        let display_start = if cursor_offset > visible_width {
            cursor_offset - visible_width
        } else {
            0
        };

        let chars: Vec<char> = prompt.input.chars().collect();
        let display_end = (display_start + visible_width).min(chars.len());
        let visible_text: String = chars[display_start..display_end].iter().collect();

        cursor_offset -= display_start;
        cursor_offset = cursor_offset.min(visible_width.saturating_sub(1));

        let input = Paragraph::new(visible_text).style(Style::default().fg(Color::Yellow));
        frame.render_widget(input, input_area);
        frame.set_cursor_position((input_area.x + cursor_offset as u16, input_area.y));
    }
}
