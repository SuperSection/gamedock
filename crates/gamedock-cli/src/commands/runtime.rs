use crate::cli::{InitArgs, StatusArgs};
use console::style;
use gamedock_core::AppConfig;
use gamedock_plugin_sdk::RuntimePlugin;
use gamedock_runtime_manager::RuntimeManager;

pub async fn init(args: InitArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = gamedock_core::EventBus::default();
    let manager = RuntimeManager::new(config, event_bus);
    manager.initialize().await?;

    let waydroid_available = {
        let runtime = manager.get_runtime("waydroid").await?;
        runtime.is_available()
    };

    if !waydroid_available {
        println!("{}", style("Installing Waydroid...").cyan());
        println!("GameDock will install it using your package manager (requires sudo).");
        println!();
    }

    if args.gapps {
        println!(
            "{}",
            style("Initializing with Google Play Store support...").cyan()
        );
    } else {
        println!(
            "{}",
            style("Initializing Waydroid (no Play Store)...").cyan()
        );
    }

    manager.init_with_gapps().await?;

    println!();
    println!("{}", style("Done!").green().bold());

    if args.gapps {
        println!();
        println!("Next steps:");
        println!("  1. Run:  sudo waydroid init -s GAPPS");
        println!("  2. Run:  gamedock play-store");
        println!("  3. Sign in, install your game, then:  gamedock launch <package>");
    } else {
        println!();
        println!("Next steps:");
        println!("  1. Run:  sudo waydroid init");
        println!("  2. Run:  gamedock launch <package>");
        println!();
        println!("To add Play Store later:  gamedock init --gapps");
    }

    Ok(())
}

pub async fn status(_args: StatusArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let runtime_name = config.default_runtime.clone();
    let event_bus = gamedock_core::EventBus::default();
    let manager = RuntimeManager::new(config, event_bus);
    manager.initialize().await?;

    let statuses = manager.check_all_status().await?;
    let runtime = manager.get_runtime(&runtime_name).await?;
    let info = runtime.get_runtime_info().await?;

    println!("{}", style("GameDock").cyan().bold());
    println!();

    for (name, status) in &statuses {
        let status_str = match status {
            gamedock_core::RuntimeStatus::Running => style("running").green().to_string(),
            gamedock_core::RuntimeStatus::Installed => {
                style("installed (idle)").yellow().to_string()
            }
            gamedock_core::RuntimeStatus::NotInstalled => style("not installed").red().to_string(),
            other => format!("{}", other),
        };
        println!("  {}: {}", name, status_str);
    }

    if let Some(ref version) = info.version {
        println!("  version: {}", version);
    }

    match runtime.check_status().await? {
        gamedock_core::RuntimeStatus::Running => {
            let apps = manager
                .list_installed_apps(&runtime_name)
                .await
                .unwrap_or_default();
            println!("  apps: {}", apps.len());
        }
        gamedock_core::RuntimeStatus::Installed => {}
        gamedock_core::RuntimeStatus::NotInstalled => {
            println!();
            println!("  Run {} to set up.", style("gamedock init --gapps").dim());
        }
        _ => {}
    }

    Ok(())
}

pub async fn update() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = gamedock_core::EventBus::default();
    let manager = RuntimeManager::new(config, event_bus);
    manager.initialize().await?;

    println!("{}", style("Updating runtime...").cyan());
    manager.update_all().await?;
    println!("{}", style("Done!").green().bold());

    Ok(())
}
