use crate::config::Config;
use crate::tui::widgets::settings_form::SettingsFormState;

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveCommand {
    Sync,
    Time,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Menu,
    Running {
        command: ActiveCommand,
        finished: bool,
    },
    Settings,
}

pub struct App {
    pub state: AppState,
    pub menu_selection: usize,
    pub output_lines: Vec<String>,
    pub output_scroll: usize,
    pub settings: SettingsFormState,
    pub config: Config,
    pub logo_ansi: Vec<String>,
}

pub struct MenuItem {
    pub name: &'static str,
    pub description: &'static str,
}

pub const MENU_ITEMS: &[MenuItem] = &[
    MenuItem { name: "Sync",         description: "Sync repos to upstream" },
    MenuItem { name: "Time Doctor",  description: "Track your work hours" },
    MenuItem { name: "Settings",     description: "Configure jotmate" },
    MenuItem { name: "Exit",         description: "" },
];

impl App {
    pub fn new(config: Config) -> Self {
        let settings = SettingsFormState::from_config(&config);
        Self {
            state: AppState::Menu,
            menu_selection: 0,
            output_lines: Vec::new(),
            output_scroll: 0,
            settings,
            config,
            logo_ansi: Vec::new(),
        }
    }

    pub fn move_menu_up(&mut self) {
        if self.menu_selection > 0 {
            self.menu_selection -= 1;
        }
    }

    pub fn move_menu_down(&mut self) {
        if self.menu_selection + 1 < MENU_ITEMS.len() {
            self.menu_selection += 1;
        }
    }

    pub fn append_output_line(&mut self, line: String) {
        self.output_lines.push(line);
        // Auto-scroll: if already at bottom, keep scrolling
        // (scroll_offset == 0 means "follow tail")
        // We keep scroll_offset as lines-from-bottom when > 0.
    }

    pub fn clear_output(&mut self) {
        self.output_lines.clear();
        self.output_scroll = 0;
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.output_scroll = self.output_scroll.saturating_add(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.output_scroll = self.output_scroll.saturating_sub(amount);
    }

    pub fn mark_command_finished(&mut self) {
        if let AppState::Running { ref mut finished, .. } = self.state {
            *finished = true;
        }
    }
}
