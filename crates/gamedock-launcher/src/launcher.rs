use gamedock_core::{AppConfig, Error, Event, EventBus, Result};
use gamedock_game_library::GameLibrary;
use gamedock_plugin_sdk::RuntimePlugin;
use gamedock_runtime_manager::RuntimeManager;
use std::sync::Arc;

pub struct AppLauncher {
    config: AppConfig,
    runtime_manager: Arc<RuntimeManager>,
    library: Arc<GameLibrary>,
    event_bus: EventBus,
}

impl AppLauncher {
    pub fn new(
        config: AppConfig,
        runtime_manager: Arc<RuntimeManager>,
        library: Arc<GameLibrary>,
        event_bus: EventBus,
    ) -> Self {
        Self {
            config,
            runtime_manager,
            library,
            event_bus,
        }
    }

    pub async fn launch(&self, app_id: &str) -> Result<()> {
        let app = self
            .library
            .get_app(app_id)
            .await
            .ok_or_else(|| Error::AppNotFound(app_id.to_string()))?;

        if !app.is_installed() {
            return Err(Error::Runtime(format!(
                "App '{}' is not installed",
                app.name
            )));
        }

        tracing::info!("Launching app: {} ({})", app.name, app.package_name);

        self.library.record_launch(app_id).await?;

        self.event_bus.publish(Event::AppLaunched {
            app_id: app_id.to_string(),
        });

        self.runtime_manager
            .launch_app(&app.runtime_id, &app.package_name)
            .await?;

        Ok(())
    }

    pub async fn launch_with_optimization(
        &self,
        app_id: &str,
        enable_gamemode: bool,
        enable_mangohud: bool,
    ) -> Result<()> {
        if enable_gamemode || enable_mangohud {
            match gamedock_optimizer::Optimizer::new(self.config.clone()) {
                Ok(optimizer) => {
                    if enable_gamemode {
                        let _ = optimizer.enable_gamemode().await;
                    }
                    if enable_mangohud {
                        let _ = optimizer.enable_mangohud(None).await;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to apply optimizations: {}", e);
                }
            }
        }

        self.launch(app_id).await
    }

    pub async fn launch_recent(&self) -> Result<()> {
        let recent = self.library.get_recently_played(1).await;
        match recent.first() {
            Some(app) => self.launch(&app.id).await,
            None => Err(Error::AppNotFound("No recently played apps".into())),
        }
    }

    pub async fn get_launch_estimate(&self, app_id: &str) -> Result<LaunchEstimate> {
        let app = self
            .library
            .get_app(app_id)
            .await
            .ok_or_else(|| Error::AppNotFound(app_id.to_string()))?;

        let runtime = self.runtime_manager.get_runtime(&app.runtime_id).await?;

        let runtime_status = runtime.check_status().await?;

        let needs_startup = matches!(
            runtime_status,
            gamedock_core::RuntimeStatus::Installed | gamedock_core::RuntimeStatus::Stopped
        );

        Ok(LaunchEstimate {
            app_name: app.name.clone(),
            runtime_name: app.runtime_id.clone(),
            needs_runtime_startup: needs_startup,
            estimated_startup_seconds: if needs_startup { Some(15) } else { Some(2) },
        })
    }
}

#[derive(Debug, Clone)]
pub struct LaunchEstimate {
    pub app_name: String,
    pub runtime_name: String,
    pub needs_runtime_startup: bool,
    pub estimated_startup_seconds: Option<u64>,
}
