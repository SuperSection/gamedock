use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gamedock",
    about = "Android gaming platform for Linux",
    version,
    long_about = "GameDock makes Android gaming on Linux seamless.\nManage runtimes, install games, and play with full controller support."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

impl Cli {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Init(args) => crate::commands::runtime::init(args).await,
            Commands::Status(args) => crate::commands::runtime::status(args).await,
            Commands::Install(args) => crate::commands::install::install(args).await,
            Commands::Uninstall(args) => crate::commands::install::uninstall(args).await,
            Commands::Launch(args) => crate::commands::launch::launch(args).await,
            Commands::PlayStore => crate::commands::launch::play_store().await,
            Commands::List(args) => crate::commands::library::list(args).await,
            Commands::Search(args) => crate::commands::library::search(args).await,
            Commands::Backup(args) => crate::commands::backup::backup(args).await,
            Commands::Restore(args) => crate::commands::backup::restore(args).await,
            Commands::Controller(args) => crate::commands::controller::controller(args).await,
            Commands::Optimize(args) => crate::commands::optimize::optimize(args).await,
            Commands::SystemInfo => crate::commands::system::system_info().await,
            Commands::Update => crate::commands::runtime::update().await,
            Commands::Completions(args) => crate::commands::completions::completions(args),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize the Android runtime")]
    Init(InitArgs),

    #[command(about = "Show runtime and system status")]
    Status(StatusArgs),

    #[command(about = "Install an APK, XAPK, APKS, or APKM package")]
    Install(InstallArgs),

    #[command(about = "Uninstall an installed app")]
    Uninstall(UninstallArgs),

    #[command(about = "Launch an installed app")]
    Launch(LaunchArgs),

    #[command(about = "Launch the Google Play Store")]
    PlayStore,

    #[command(about = "List installed apps")]
    List(ListArgs),

    #[command(about = "Search the game library")]
    Search(SearchArgs),

    #[command(about = "Create a backup of an app")]
    Backup(BackupArgs),

    #[command(about = "Restore an app from backup")]
    Restore(RestoreArgs),

    #[command(about = "Controller management")]
    Controller(ControllerArgs),

    #[command(about = "System optimization")]
    Optimize(OptimizeArgs),

    #[command(about = "Show system information")]
    SystemInfo,

    #[command(about = "Update the runtime")]
    Update,

    #[command(about = "Generate shell completions")]
    Completions(CompletionsArgs),
}

#[derive(clap::Args)]
pub struct InitArgs {
    #[arg(long, default_value = "system")]
    pub image_type: String,

    #[arg(long)]
    pub gapps: bool,
}

#[derive(clap::Args)]
pub struct StatusArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct InstallArgs {
    pub path: PathBuf,

    #[arg(long)]
    pub runtime: Option<String>,

    #[arg(long)]
    pub url: Option<String>,
}

#[derive(clap::Args)]
pub struct UninstallArgs {
    pub package_name: String,
}

#[derive(clap::Args)]
pub struct LaunchArgs {
    pub app_id: Option<String>,

    #[arg(long)]
    pub package: Option<String>,

    #[arg(long)]
    pub gamemode: bool,

    #[arg(long)]
    pub mangohud: bool,

    #[arg(long)]
    pub recent: bool,
}

#[derive(clap::Args)]
pub struct ListArgs {
    #[arg(long)]
    pub installed: bool,

    #[arg(long)]
    pub favorites: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub category: Option<String>,
}

#[derive(clap::Args)]
pub struct SearchArgs {
    pub query: String,

    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct BackupArgs {
    pub app_id: String,

    #[arg(long)]
    pub include_data: bool,

    #[arg(long)]
    pub keep: Option<usize>,
}

#[derive(clap::Args)]
pub struct RestoreArgs {
    pub backup_path: PathBuf,
}

#[derive(clap::Args)]
pub struct ControllerArgs {
    #[command(subcommand)]
    pub action: ControllerAction,
}

#[derive(Subcommand)]
pub enum ControllerAction {
    #[command(about = "List connected controllers")]
    List,

    #[command(about = "Create default mapping profiles")]
    InitProfiles,

    #[command(about = "Show available profiles")]
    Profiles,

    #[command(about = "Set controller profile")]
    SetProfile {
        controller_id: String,
        profile_name: String,
    },
}

#[derive(clap::Args)]
pub struct OptimizeArgs {
    #[arg(long)]
    pub gamemode: bool,

    #[arg(long)]
    pub mangohud: bool,

    #[arg(long)]
    pub cpu_governor: Option<String>,

    #[arg(long)]
    pub reset: bool,
}

#[derive(clap::Args)]
pub struct CompletionsArgs {
    pub shell: clap_complete::Shell,
}
