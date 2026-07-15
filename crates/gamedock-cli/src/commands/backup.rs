use crate::cli::{BackupArgs, RestoreArgs};
use console::style;
use gamedock_backup::BackupManager;
use gamedock_core::AppConfig;
use gamedock_core::EventBus;
use gamedock_game_library::GameLibrary;
use gamedock_runtime_manager::RuntimeManager;
use std::sync::Arc;

pub async fn backup(args: BackupArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = EventBus::default();
    let manager = Arc::new(RuntimeManager::new(config.clone(), event_bus.clone()));
    manager.initialize().await?;

    let library = GameLibrary::new(config.clone(), event_bus.clone());
    library.load().await?;

    let app = library
        .get_app(&args.app_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("App not found: {}", args.app_id))?;

    let backup_mgr = BackupManager::new(config, event_bus);
    let result = backup_mgr
        .create_backup(&app, &manager, args.include_data)
        .await?;

    println!("{} {}", style("✓").green(), result.archive_path.display());

    if let Some(keep) = args.keep {
        let deleted = backup_mgr.cleanup_old_backups(keep).await?;
        if deleted > 0 {
            println!("Cleaned up {} old backups", deleted);
        }
    }

    Ok(())
}

pub async fn restore(args: RestoreArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = EventBus::default();
    let manager = Arc::new(RuntimeManager::new(config.clone(), event_bus.clone()));
    manager.initialize().await?;

    let backup_mgr = BackupManager::new(config, event_bus);

    if !args.backup_path.exists() {
        anyhow::bail!("File not found: {:?}", args.backup_path);
    }

    backup_mgr
        .restore_backup(&args.backup_path, &manager)
        .await?;
    println!("{} Restored", style("✓").green());

    Ok(())
}
