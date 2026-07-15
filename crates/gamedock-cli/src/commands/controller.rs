use crate::cli::{ControllerAction, ControllerArgs};
use console::style;
use gamedock_controller::ControllerManager;
use gamedock_core::AppConfig;
use gamedock_core::EventBus;

pub async fn controller(args: ControllerArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let event_bus = EventBus::default();
    let manager = ControllerManager::new(config, event_bus);
    manager.initialize().await?;

    match args.action {
        ControllerAction::List => {
            let controllers = manager.list_controllers().await;
            if controllers.is_empty() {
                println!("No controllers detected");
            } else {
                for c in &controllers {
                    println!("{} ({})", c.name, c.controller_type.display_name());
                    if let Some(profile) = &c.active_profile {
                        println!("  profile: {}", profile);
                    }
                }
            }
        }
        ControllerAction::InitProfiles => {
            manager.create_default_profiles().await?;
            println!("{} Default profiles created", style("✓").green());
        }
        ControllerAction::Profiles => {
            let profiles = manager.list_profiles().await;
            for p in &profiles {
                println!("{} - {}", style(&p.name).bold(), p.description);
            }
        }
        ControllerAction::SetProfile {
            controller_id,
            profile_name,
        } => {
            manager.set_profile(&controller_id, &profile_name).await?;
            println!(
                "{} profile '{}' set for '{}'",
                style("✓").green(),
                profile_name,
                controller_id
            );
        }
    }

    Ok(())
}
