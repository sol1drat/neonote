use ratatui::{
    layout::{Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    app::App,
    types::{ConfirmPrompt, FileCreate, FileRename},
};

impl App {
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

        let display_start = cursor_offset.saturating_sub(visible_width);

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

        let display_start = cursor_offset.saturating_sub(visible_width);

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
