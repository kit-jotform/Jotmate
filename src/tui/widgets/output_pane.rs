use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
};

use crate::tui::widgets::logo::strip_ansi;

pub struct OutputPane<'a> {
    pub lines: &'a [String],
    pub scroll_offset: usize,
    pub title: &'a str,
    pub active: bool,
}

impl<'a> Widget for OutputPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let inner = block.inner(area);
        let visible_height = inner.height as usize;

        if self.lines.is_empty() {
            let placeholder = Paragraph::new(Span::styled(
                "Select a command from the menu",
                Style::default().fg(Color::DarkGray),
            ))
            .block(block);
            placeholder.render(area, buf);
            return;
        }

        // scroll_offset == 0: follow tail (auto-scroll)
        let total = self.lines.len();
        let start = if self.scroll_offset == 0 {
            total.saturating_sub(visible_height)
        } else {
            total.saturating_sub(visible_height + self.scroll_offset)
        };
        let end = (start + visible_height).min(total);
        let visible_lines = &self.lines[start..end];

        // Strip ANSI for display (TUI renders its own styling)
        let text_lines: Vec<Line> = visible_lines
            .iter()
            .map(|raw| {
                let clean = strip_ansi(raw);
                Line::from(Span::raw(clean))
            })
            .collect();

        let para = Paragraph::new(text_lines)
            .block(block)
            .wrap(Wrap { trim: false });

        para.render(area, buf);
    }
}
