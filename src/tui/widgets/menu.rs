use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::tui::app::MENU_ITEMS;

pub struct MenuWidget {
    pub selected: usize,
    pub active: bool,
}

impl MenuWidget {
    pub fn render_into(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = MENU_ITEMS
            .iter()
            .map(|item| ListItem::new(Line::from(Span::raw(*item))))
            .collect();

        let block = Block::default()
            .title(" jotmate ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let highlight_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
