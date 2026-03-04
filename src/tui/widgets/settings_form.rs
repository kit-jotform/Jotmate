use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::config::{parse_contract_periods, Config};

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub label: &'static str,
    pub key: &'static str,
    pub value: String,
    pub field_type: FieldType,
    pub hint: &'static str,
}

#[derive(Debug, Clone)]
pub struct SettingsFormState {
    pub fields: Vec<FormField>,
    pub selected_idx: usize,
    pub editing: bool,
    pub edit_buffer: String,
}

impl SettingsFormState {
    pub fn from_config(config: &Config) -> Self {
        let t = &config.time;
        let s = &config.sync;

        let contract_periods_str = t
            .contract_periods
            .as_ref()
            .map(|periods| {
                periods
                    .iter()
                    .map(|p| format!("{}:{}", p.from, p.weekly_hours))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default();

        let fields = vec![
            FormField {
                label: "Email",
                key: "time.email",
                value: t.email.clone().unwrap_or_default(),
                field_type: FieldType::Text,
                hint: "TimeDoctor account email",
            },
            FormField {
                label: "Company ID",
                key: "time.company_id",
                value: t.company_id.clone().unwrap_or_default(),
                field_type: FieldType::Text,
                hint: "TimeDoctor company ID",
            },
            FormField {
                label: "Timezone",
                key: "time.timezone",
                value: t.timezone.clone().unwrap_or_else(|| "Europe/Istanbul".to_string()),
                field_type: FieldType::Text,
                hint: "e.g. Europe/Istanbul, America/New_York",
            },
            FormField {
                label: "Start Date",
                key: "time.start_date",
                value: t.start_date.map(|d| d.to_string()).unwrap_or_default(),
                field_type: FieldType::Text,
                hint: "YYYY-MM-DD — first week to track",
            },
            FormField {
                label: "Contract Periods",
                key: "time.contract_periods",
                value: contract_periods_str,
                field_type: FieldType::Text,
                hint: "e.g. 2025-11-17:20,2026-02-02:28",
            },
            FormField {
                label: "Skip Current Week",
                key: "time.skip_current_week",
                value: t.skip_current_week.to_string(),
                field_type: FieldType::Boolean,
                hint: "Skip the current (incomplete) week in reports",
            },
            FormField {
                label: "Reset Cumul. From",
                key: "time.reset_cumulative_from_date",
                value: t
                    .reset_cumulative_from_date
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                field_type: FieldType::Text,
                hint: "YYYY-MM-DD — reset cumulative balance from this date (optional)",
            },
            FormField {
                label: "Sync Default --only",
                key: "sync.default_only",
                value: s
                    .default_only
                    .as_ref()
                    .map(|v| v.join(","))
                    .unwrap_or_default(),
                field_type: FieldType::Text,
                hint: "comma-separated project names, empty = all",
            },
            FormField {
                label: "Sync Default --sync-all",
                key: "sync.default_sync_all",
                value: s.default_sync_all.to_string(),
                field_type: FieldType::Boolean,
                hint: "Always run with --sync-all by default",
            },
        ];

        Self {
            fields,
            selected_idx: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    pub fn apply_to_config(&self, config: &mut Config) {
        for field in &self.fields {
            match field.key {
                "time.email" => {
                    config.time.email = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    }
                }
                "time.company_id" => {
                    config.time.company_id = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    }
                }
                "time.timezone" => {
                    config.time.timezone = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    }
                }
                "time.start_date" => {
                    config.time.start_date =
                        chrono::NaiveDate::parse_from_str(&field.value, "%Y-%m-%d").ok();
                }
                "time.contract_periods" => {
                    if !field.value.is_empty() {
                        config.time.contract_periods =
                            parse_contract_periods(&field.value).ok();
                    }
                }
                "time.skip_current_week" => {
                    config.time.skip_current_week = field.value == "true";
                }
                "time.reset_cumulative_from_date" => {
                    config.time.reset_cumulative_from_date =
                        chrono::NaiveDate::parse_from_str(&field.value, "%Y-%m-%d").ok();
                }
                "sync.default_only" => {
                    config.sync.default_only = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.split(',').map(|s| s.trim().to_string()).collect())
                    };
                }
                "sync.default_sync_all" => {
                    config.sync.default_sync_all = field.value == "true";
                }
                _ => {}
            }
        }
    }

    pub fn move_next(&mut self) {
        self.commit_edit();
        if self.selected_idx + 1 < self.fields.len() {
            self.selected_idx += 1;
        }
    }

    pub fn move_prev(&mut self) {
        self.commit_edit();
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn toggle_or_start_edit(&mut self) {
        let field = &mut self.fields[self.selected_idx];
        if field.field_type == FieldType::Boolean {
            // Toggle
            field.value = if field.value == "true" {
                "false".to_string()
            } else {
                "true".to_string()
            };
        } else if self.editing {
            self.commit_edit();
        } else {
            self.editing = true;
            self.edit_buffer = self.fields[self.selected_idx].value.clone();
        }
    }

    pub fn commit_edit(&mut self) {
        if self.editing {
            self.fields[self.selected_idx].value = self.edit_buffer.clone();
            self.editing = false;
            self.edit_buffer.clear();
        }
    }

    pub fn cancel_edit(&mut self) {
        if self.editing {
            self.editing = false;
            self.edit_buffer.clear();
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if self.editing {
            self.edit_buffer.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.editing {
            self.edit_buffer.pop();
        }
    }
}

pub struct SettingsFormWidget<'a> {
    pub state: &'a SettingsFormState,
}

impl<'a> Widget for SettingsFormWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear background
        Clear.render(area, buf);

        let block = Block::default()
            .title(" Settings (Ctrl-S: save  Esc: cancel) ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let field_height = 2u16;
        let footer_height = 2u16;
        let max_fields = ((inner.height.saturating_sub(footer_height)) / field_height) as usize;
        let visible_start = if self.state.selected_idx >= max_fields {
            self.state.selected_idx - max_fields + 1
        } else {
            0
        };
        let visible_end = (visible_start + max_fields).min(self.state.fields.len());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..(visible_end - visible_start))
                    .map(|_| Constraint::Length(field_height))
                    .chain(std::iter::once(Constraint::Min(0)))
                    .chain(std::iter::once(Constraint::Length(footer_height)))
                    .collect::<Vec<_>>(),
            )
            .split(inner);

        for (idx, field_idx) in (visible_start..visible_end).enumerate() {
            let field = &self.state.fields[field_idx];
            let is_selected = field_idx == self.state.selected_idx;
            let is_editing = is_selected && self.state.editing;

            let label_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let display_value = if is_editing {
                format!("{}_", self.state.edit_buffer)
            } else {
                let v = &field.value;
                if field.field_type == FieldType::Boolean {
                    if v == "true" { "✓ yes".to_string() } else { "  no".to_string() }
                } else if v.is_empty() {
                    "(not set)".to_string()
                } else {
                    v.clone()
                }
            };

            let value_style = if is_editing {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::styled(format!("{:<22}", field.label), label_style),
                Span::raw(" "),
                Span::styled(&display_value, value_style),
            ]);

            let hint_style = Style::default().fg(Color::DarkGray);
            let hint_line = Line::from(Span::styled(
                format!("  {}", field.hint),
                hint_style,
            ));

            if idx < chunks.len().saturating_sub(2) {
                Paragraph::new(vec![line, hint_line]).render(chunks[idx], buf);
            }
        }

        // Footer
        if let Some(footer_chunk) = chunks.last() {
            let footer = Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": next  "),
                Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": edit  "),
                Span::styled("Ctrl-S", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(": save  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": cancel"),
            ]);
            Paragraph::new(footer).render(*footer_chunk, buf);
        }
    }
}
