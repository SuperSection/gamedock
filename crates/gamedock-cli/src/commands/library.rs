use crate::cli::{ListArgs, SearchArgs};
use console::style;
use gamedock_core::AppConfig;
use gamedock_core::EventBus;
use gamedock_game_library::GameLibrary;

pub async fn list(args: ListArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = EventBus::default();
    let library = GameLibrary::new(config, event_bus);
    library.load().await?;

    let apps = if args.favorites {
        library.get_favorites().await
    } else if args.installed {
        library.list_installed_apps().await
    } else {
        library.list_all_apps().await
    };

    if apps.is_empty() {
        println!("{}", style("No apps found in library").dim());
        return Ok(());
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&apps)?);
    } else {
        println!(
            "{}",
            style(format!("Game Library ({} apps)", apps.len()))
                .cyan()
                .bold()
        );
        println!();
        for app in &apps {
            let status = match &app.status {
                gamedock_core::AppStatus::Installed => style("✓").green().to_string(),
                _ => style("○").dim().to_string(),
            };
            let fav = if app.is_favorite { " ★" } else { "" };
            println!("  {} {}{}", status, style(&app.name).bold(), fav);
            println!("    {}", style(&app.package_name).dim());
        }
    }

    Ok(())
}

pub async fn search(args: SearchArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = EventBus::default();
    let library = GameLibrary::new(config, event_bus);
    library.load().await?;

    let results = library.search(&args.query).await;

    if results.is_empty() {
        println!("{}", style("No results found").dim());
        return Ok(());
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!(
            "{}",
            style(format!(
                "Search results for '{}' ({} found)",
                args.query,
                results.len()
            ))
            .cyan()
            .bold()
        );
        println!();
        for app in &results {
            println!(
                "  {} {}",
                style(&app.name).bold(),
                style(&app.package_name).dim()
            );
        }
    }

    Ok(())
}
