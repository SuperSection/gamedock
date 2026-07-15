use crate::waydroid::WaydroidRuntime;
use gamedock_core::{AppConfig, AppInfo, Error, Event, EventBus, Result, RuntimeStatus};
use gamedock_plugin_sdk::RuntimePlugin;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RuntimeManager {
    _config: AppConfig,
    runtimes: RwLock<HashMap<String, Arc<WaydroidRuntime>>>,
    event_bus: EventBus,
}

impl RuntimeManager {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Self {
        Self {
            _config: config,
            runtimes: RwLock::new(HashMap::new()),
            event_bus,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let waydroid = Arc::new(WaydroidRuntime::new()?);

        if waydroid.is_available() {
            waydroid.start_idle_watcher().await;
        }

        self.runtimes
            .write()
            .await
            .insert("waydroid".to_string(), waydroid);
        tracing::info!("RuntimeManager initialized with Waydroid backend");
        Ok(())
    }

    pub async fn get_runtime(&self, name: &str) -> Result<Arc<WaydroidRuntime>> {
        self.runtimes
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| Error::Runtime(format!("Runtime '{}' not found", name)))
    }

    pub async fn ensure_running(&self, runtime_id: &str) -> Result<()> {
        let runtime = self.get_runtime(runtime_id).await?;
        let status = runtime.check_status().await?;

        match status {
            RuntimeStatus::Running => Ok(()),
            RuntimeStatus::Installed => {
                self.event_bus.publish(Event::RuntimeStatusChanged {
                    runtime_id: runtime_id.to_string(),
                    status: "Starting".to_string(),
                });
                runtime.start().await?;
                self.event_bus.publish(Event::RuntimeStatusChanged {
                    runtime_id: runtime_id.to_string(),
                    status: "Running".to_string(),
                });
                Ok(())
            }
            RuntimeStatus::NotInstalled => {
                self.event_bus.publish(Event::RuntimeStatusChanged {
                    runtime_id: runtime_id.to_string(),
                    status: "Installing".to_string(),
                });
                runtime.install().await?;
                runtime.start().await?;
                self.event_bus.publish(Event::RuntimeStatusChanged {
                    runtime_id: runtime_id.to_string(),
                    status: "Running".to_string(),
                });
                Ok(())
            }
            other => Err(Error::Runtime(format!(
                "Cannot start runtime in state: {}",
                other
            ))),
        }
    }

    pub async fn init_with_gapps(&self) -> Result<()> {
        let runtime = self.get_runtime("waydroid").await?;

        if !runtime.is_available() {
            tracing::info!("Waydroid not installed, installing system package...");
            WaydroidRuntime::install_waydroid_system().await?;
        }

        tracing::info!("Initializing Waydroid with Google Play Services (GAPPS)...");

        // waydroid init needs root and a terminal for password input.
        // We can't run it through a subprocess, so tell the user to run it.
        println!();
        println!("Waydroid needs root access to initialize the Android system image.");
        println!("Please run this command in another terminal:");
        println!();
        println!("  sudo waydroid init -s GAPPS");
        println!();
        println!("Then come back and run:  gamedock status");
        println!();

        Ok(())
    }

    pub async fn init_vanilla(&self) -> Result<()> {
        let runtime = self.get_runtime("waydroid").await?;

        if !runtime.is_available() {
            tracing::info!("Waydroid not installed, installing system package...");
            WaydroidRuntime::install_waydroid_system().await?;
        }

        tracing::info!("Initializing Waydroid (vanilla, no Play Store)...");

        println!();
        println!("Waydroid needs root access to initialize the Android system image.");
        println!("Please run this command in another terminal:");
        println!();
        println!("  sudo waydroid init");
        println!();
        println!("Then come back and run:  gamedock status");
        println!();

        self.event_bus.publish(Event::RuntimeStatusChanged {
            runtime_id: "waydroid".to_string(),
            status: "Installed".to_string(),
        });

        Ok(())
    }

    pub async fn check_all_status(&self) -> Result<HashMap<String, RuntimeStatus>> {
        let mut statuses = HashMap::new();
        let runtimes = self.runtimes.read().await;
        for (name, runtime) in runtimes.iter() {
            statuses.insert(name.clone(), runtime.check_status().await?);
        }
        Ok(statuses)
    }

    pub async fn update_all(&self) -> Result<()> {
        let runtimes = self.runtimes.read().await;
        for (name, runtime) in runtimes.iter() {
            tracing::info!("Updating runtime: {}", name);
            runtime.update().await?;
        }
        Ok(())
    }

    pub async fn install_app(&self, runtime_id: &str, package: &Path) -> Result<String> {
        self.ensure_running(runtime_id).await?;
        let runtime = self.get_runtime(runtime_id).await?;
        let result = runtime.install_app(package).await?;

        self.event_bus.publish(Event::AppInstalled {
            app_id: result.clone(),
            package_name: package.to_string_lossy().to_string(),
        });

        Ok(result)
    }

    pub async fn uninstall_app(&self, runtime_id: &str, package_name: &str) -> Result<()> {
        let runtime = self.get_runtime(runtime_id).await?;
        runtime.uninstall_app(package_name).await?;

        self.event_bus.publish(Event::AppUninstalled {
            app_id: package_name.to_string(),
        });

        Ok(())
    }

    pub async fn launch_app(&self, runtime_id: &str, package_name: &str) -> Result<()> {
        self.ensure_running(runtime_id).await?;
        let runtime = self.get_runtime(runtime_id).await?;
        runtime.launch_app(package_name).await?;

        self.event_bus.publish(Event::AppLaunched {
            app_id: package_name.to_string(),
        });

        Ok(())
    }

    pub async fn launch_play_store(&self, runtime_id: &str) -> Result<()> {
        self.ensure_running(runtime_id).await?;
        let runtime = self.get_runtime(runtime_id).await?;
        runtime.launch_play_store().await
    }

    pub async fn list_installed_apps(&self, runtime_id: &str) -> Result<Vec<AppInfo>> {
        let runtime = self.get_runtime(runtime_id).await?;
        runtime.list_installed_apps().await
    }
}
