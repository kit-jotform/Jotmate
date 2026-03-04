use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

/// Strip ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn apply_sgr(params: &str, fg: &mut Option<Color>, bg: &mut Option<Color>) {
    let nums: Vec<u32> = params.split(';').filter_map(|s| s.parse().ok()).collect();
    let mut i = 0;
    while i < nums.len() {
        match nums[i] {
            0 => { *fg = None; *bg = None; }
            7 => { std::mem::swap(fg, bg); }
            38 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                *fg = Some(Color::Rgb(nums[i+2] as u8, nums[i+3] as u8, nums[i+4] as u8));
                i += 4;
            }
            48 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                *bg = Some(Color::Rgb(nums[i+2] as u8, nums[i+3] as u8, nums[i+4] as u8));
                i += 4;
            }
            _ => {}
        }
        i += 1;
    }
}

/// Parse an ANSI-colored string into ratatui Spans.
pub fn ansi_to_line(line: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut fg: Option<Color> = None;
    let mut bg: Option<Color> = None;
    let mut text = String::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            let mut params = String::new();
            let mut final_byte = 'm';
            for c2 in chars.by_ref() {
                if c2.is_ascii_alphabetic() { final_byte = c2; break; }
                params.push(c2);
            }
            if !text.is_empty() {
                let mut style = Style::default();
                if let Some(f) = fg { style = style.fg(f); }
                if let Some(b) = bg { style = style.bg(b); }
                spans.push(Span::styled(std::mem::take(&mut text), style));
            }
            if final_byte == 'm' {
                apply_sgr(&params, &mut fg, &mut bg);
            }
        } else if c == '\x1b' {
            chars.next();
        } else {
            text.push(c);
        }
    }

    if !text.is_empty() {
        let mut style = Style::default();
        if let Some(f) = fg { style = style.fg(f); }
        if let Some(b) = bg { style = style.bg(b); }
        spans.push(Span::styled(text, style));
    }

    Line::from(spans)
}

// Big block-letter JOTMATE (6 rows tall)
const BIG_TEXT: &[&str] = &[
    "     ██╗ ██████╗ ████████╗███╗   ███╗ █████╗ ████████╗███████╗",
    "     ██║██╔═══██╗╚══██╔══╝████╗ ████║██╔══██╗╚══██╔══╝██╔════╝",
    "     ██║██║   ██║   ██║   ██╔████╔██║███████║   ██║   █████╗  ",
    "██   ██║██║   ██║   ██║   ██║╚██╔╝██║██╔══██║   ██║   ██╔══╝  ",
    "╚█████╔╝╚██████╔╝   ██║   ██║ ╚═╝ ██║██║  ██║   ██║   ███████╗",
    " ╚════╝  ╚═════╝    ╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚══════╝",
];
pub const BIG_TEXT_HEIGHT: u16 = BIG_TEXT.len() as u16;

pub struct BigTextWidget;

impl Widget for BigTextWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, &line) in BIG_TEXT.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height { break; }
            Paragraph::new(Line::from(Span::styled(line, Style::default().fg(Color::White))))
                .render(Rect { x: area.x, y, width: area.width, height: 1 }, buf);
        }
    }
}

/// Renders pre-computed chafa ANSI lines as colored image.
pub struct ChafaImageWidget<'a> {
    pub lines: &'a [String],
}

impl<'a> Widget for ChafaImageWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, line) in self.lines.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height { break; }
            let ratatui_line = ansi_to_line(line);
            Paragraph::new(ratatui_line)
                .render(Rect { x: area.x, y, width: area.width, height: 1 }, buf);
        }
    }
}
