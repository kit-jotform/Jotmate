use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jotmate", about = "Jotform developer productivity CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Sync git forks with upstream
    Sync(SyncArgs),
    /// Check TimeDoctor work-hour stats
    Time(TimeArgs),
    /// Edit default flags and credentials
    Settings,
    /// Print the embedded ANSI icon (internal use)
    #[command(hide = true, name = "_icon")]
    Icon,
    /// Print current settings as key=value lines (internal use)
    #[command(hide = true, name = "_settings-get")]
    SettingsGet,
    /// Toggle a boolean setting (internal use)
    #[command(hide = true, name = "_settings-toggle")]
    SettingsToggle(SettingsToggleArgs),
    /// Add an upstream repo (internal use)
    #[command(hide = true, name = "_settings-add-repo")]
    SettingsAddRepo(SettingsAddRepoArgs),
    /// Remove an upstream repo by name (internal use)
    #[command(hide = true, name = "_settings-remove-repo")]
    SettingsRemoveRepo(SettingsRemoveRepoArgs),
    /// Toggle enabled flag on an upstream repo by name (internal use)
    #[command(hide = true, name = "_settings-toggle-repo")]
    SettingsToggleRepo(SettingsRemoveRepoArgs),
}

#[derive(Args, Clone, Debug, Default)]
pub struct SyncArgs {
    /// Only sync specific projects (comma-separated: Jotform3,vendors,core,backend,frontend)
    #[arg(long, value_delimiter = ',')]
    pub only: Option<Vec<String>>,

    /// Force sync all repos regardless of upstream diff
    #[arg(long)]
    pub sync_all: bool,
}

#[derive(Args, Clone, Debug)]
pub struct SettingsToggleArgs {
    /// Field name: sync_all_by_default | use_cache
    pub field: String,
}

#[derive(Args, Clone, Debug)]
pub struct SettingsAddRepoArgs {
    pub url: String,
    pub name: String,
}

#[derive(Args, Clone, Debug)]
pub struct SettingsRemoveRepoArgs {
    pub name: String,
}

#[derive(Args, Clone, Debug, Default)]
pub struct TimeArgs {
    /// Skip reporting for the current (incomplete) week
    #[arg(long)]
    pub skip_current_week: bool,

    /// Bypass local week cache and re-fetch from API
    #[arg(long)]
    pub no_cache: bool,
}
