use crate::cli::{InstallArgs, UninstallArgs};
use gamedock_core::AppConfig;
use gamedock_runtime_manager::RuntimeManager;
use gamedock_installer::PackageInstaller;
use console::style;

pub async fn install(args: InstallArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = gamedock_core::EventBus::default();
    let manager = RuntimeManager::new(config.clone(), event_bus.clone());
    manager.initialize().await?;

    let installer = PackageInstaller::new(config, event_bus);
    let runtime_id = args.runtime.unwrap_or_else(|| "waydroid".to_string());

    if let Some(url) = &args.url {
        let result = installer.download_and_install(url, &manager, &runtime_id).await?;
        println!("{} {}", style("✓").green(), result.name);
    } else {
        let path = &args.path;
        if !path.exists() {
            anyhow::bail!("File not found: {:?}", path);
        }

        let result = installer.install_from_file(path, &manager, &runtime_id).await?;
        println!("{} {}", style("✓").green(), result.name);
    }

    Ok(())
}

pub async fn uninstall(args: UninstallArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = gamedock_core::EventBus::default();
    let manager = RuntimeManager::new(config.clone(), event_bus.clone());
    manager.initialize().await?;

    let installer = PackageInstaller::new(config, event_bus);
    installer.uninstall(&args.package_name, &manager, "waydroid").await?;

    println!("{} {}", style("✓").green(), args.package_name);
    Ok(())
}
