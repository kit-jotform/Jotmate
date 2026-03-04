use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::tui::app::MENU_ITEMS;

pub struct MenuWidget {
    pub selected: usize,
}

impl Widget for MenuWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Header: "SELECT TOOL   (↑↓ navigate • Enter select • Esc exit)"
        let header_y = area.y;
        let header = Line::from(vec![
            Span::styled(
                "SELECT TOOL",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(
                "(↑↓ navigate • Enter select • Esc exit)",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        Paragraph::new(header).render(
            Rect { x: area.x, y: header_y, width: area.width, height: 1 },
            buf,
        );

        // Menu items: two columns
        let items_start_y = header_y + 1;
        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let y = items_start_y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            let is_selected = i == self.selected;
            let row = Rect { x: area.x, y, width: area.width, height: 1 };

            if is_selected {
                // Render selected row: "► Name   — Description"
                let line = Line::from(vec![
                    Span::styled(
                        "► ",
                        Style::default().fg(Color::Rgb(160, 100, 240)).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        item.name,
                        Style::default().fg(Color::Rgb(160, 100, 240)).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(padding_to(item.name.len() + 2, 18)),
                    Span::styled(
                        "— ",
                        Style::default().fg(Color::Rgb(160, 100, 240)),
                    ),
                    Span::styled(
                        item.description,
                        Style::default().fg(Color::Rgb(160, 100, 240)),
                    ),
                ]);
                Paragraph::new(line).render(row, buf);
            } else {
                let line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(item.name),
                    Span::raw(padding_to(item.name.len() + 2, 18)),
                    Span::styled(
                        "— ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        item.description,
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                Paragraph::new(line).render(row, buf);
            }
        }
    }
}

/// Returns spaces to pad `current_len` up to `target_col`.
fn padding_to(current_len: usize, target_col: usize) -> String {
    if current_len < target_col {
        " ".repeat(target_col - current_len)
    } else {
        "  ".to_string()
    }
}
