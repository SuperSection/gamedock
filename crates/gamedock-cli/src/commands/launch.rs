use crate::cli::LaunchArgs;
use console::style;
use gamedock_core::AppConfig;
use gamedock_game_library::GameLibrary;
use gamedock_launcher::AppLauncher;
use gamedock_launcher::PlayStoreLauncher;
use gamedock_runtime_manager::RuntimeManager;
use std::sync::Arc;

pub async fn launch(args: LaunchArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = gamedock_core::EventBus::default();
    let manager = Arc::new(RuntimeManager::new(config.clone(), event_bus.clone()));
    manager.initialize().await?;

    let library = Arc::new(GameLibrary::new(config.clone(), event_bus.clone()));
    library.load().await?;

    let launcher = AppLauncher::new(config, manager, library, event_bus);

    if args.recent {
        launcher.launch_recent().await?;
        println!("{}", style("Launched!").green().bold());
        return Ok(());
    }

    let app_id = args
        .app_id
        .as_deref()
        .or_else(|| args.package.as_deref())
        .ok_or_else(|| anyhow::anyhow!("Provide an app ID or package name"))?;

    launcher
        .launch_with_optimization(app_id, args.gamemode, args.mangohud)
        .await?;

    println!("{}", style("Launched!").green().bold());
    Ok(())
}

pub async fn play_store() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = gamedock_core::EventBus::default();
    let manager = Arc::new(RuntimeManager::new(config.clone(), event_bus.clone()));
    manager.initialize().await?;

    let launcher = PlayStoreLauncher::new(config, manager);
    launcher.launch().await?;

    println!("{}", style("Play Store opened").green().bold());
    Ok(())
}
