use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

const LOGO_ANSI: &str = include_str!("../../../assets/logo.ansi");

pub struct LogoWidget;

/// Strip ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Consume the escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Consume until a letter (the final byte of CSI sequence)
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                // Other escape sequences — consume next char
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl Widget for LogoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let stripped = strip_ansi(LOGO_ANSI);
        let lines: Vec<Line> = stripped
            .lines()
            .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Cyan))))
            .collect();

        let para = Paragraph::new(lines).block(Block::default());
        para.render(area, buf);
    }
}
