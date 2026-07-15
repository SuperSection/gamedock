use gamedock_core::{AppConfig, AppInfo, Error, Result};
use gamedock_game_library::GameLibrary;
use gamedock_runtime_manager::RuntimeManager;
use std::sync::Arc;

pub struct PlayStoreLauncher {
    config: AppConfig,
    runtime_manager: Arc<RuntimeManager>,
    library: Option<Arc<GameLibrary>>,
}

impl PlayStoreLauncher {
    pub fn new(config: AppConfig, runtime_manager: Arc<RuntimeManager>) -> Self {
        Self {
            config,
            runtime_manager,
            library: None,
        }
    }

    pub fn with_library(
        config: AppConfig,
        runtime_manager: Arc<RuntimeManager>,
        library: Arc<GameLibrary>,
    ) -> Self {
        Self {
            config,
            runtime_manager,
            library: Some(library),
        }
    }

    pub async fn launch(&self) -> Result<()> {
        if !self.config.play_store.enabled {
            return Err(Error::Runtime(
                "Google Play Store is disabled in configuration".into(),
            ));
        }

        tracing::info!("Launching Google Play Store...");
        self.runtime_manager
            .launch_play_store(&self.config.default_runtime)
            .await
    }

    pub async fn launch_app_page(&self, package_name: &str) -> Result<()> {
        tracing::info!("Opening Play Store page for: {}", package_name);
        let url = format!(
            "https://play.google.com/store/apps/details?id={}",
            package_name
        );

        tokio::process::Command::new("xdg-open").arg(&url).spawn()?;

        Ok(())
    }

    pub async fn check_for_updates(&self) -> Result<Vec<AppUpdateInfo>> {
        tracing::info!("Checking for app updates via Play Store...");

        let library = match &self.library {
            Some(lib) => lib.clone(),
            None => return Ok(Vec::new()),
        };

        let installed_apps = library.list_installed_apps().await;
        let mut updates = Vec::new();

        for app in &installed_apps {
            if let Some(update) = self.check_app_update(app).await {
                updates.push(update);
            }
        }

        tracing::info!("Found {} app updates available", updates.len());
        Ok(updates)
    }

    async fn check_app_update(&self, app: &AppInfo) -> Option<AppUpdateInfo> {
        let package_name = &app.package_name;

        if let Ok(output) = tokio::process::Command::new("dumpsys")
            .args(["package", package_name])
            .output()
            .await
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let installed_version = Self::parse_version_from_dumpsys(&stdout, "versionName");

            if let Some(web_version) = self.fetch_web_version(package_name).await {
                if installed_version != web_version {
                    return Some(AppUpdateInfo {
                        package_name: package_name.clone(),
                        current_version: installed_version,
                        available_version: web_version,
                    });
                }
            }
        }

        None
    }

    async fn fetch_web_version(&self, package_name: &str) -> Option<String> {
        let url = format!(
            "https://play.google.com/store/apps/details?id={}",
            package_name
        );

        let response = reqwest::get(&url).await.ok()?;
        let body = response.text().await.ok()?;

        for line in body.lines() {
            if line.contains("Current Version") || line.contains("currentVersion") {
                if let Some(version) = Self::extract_version_from_html(line) {
                    return Some(version);
                }
            }
        }

        None
    }

    fn extract_version_from_html(html: &str) -> Option<String> {
        let re = regex::Regex::new(r"(\d+\.\d+(?:\.\d+)*(?:\.\d+)*)").ok()?;
        if let Some(caps) = re.captures(html) {
            if let Some(m) = caps.get(1) {
                let version = m.as_str().to_string();
                if !version.starts_with('0') && version.contains('.') {
                    return Some(version);
                }
            }
        }
        None
    }

    fn parse_version_from_dumpsys(output: &str, field: &str) -> String {
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(&format!("{}=", field)) {
                return trimmed
                    .split('=')
                    .nth(1)
                    .unwrap_or("unknown")
                    .trim()
                    .to_string();
            }
        }
        "unknown".to_string()
    }

    pub fn is_enabled(&self) -> bool {
        self.config.play_store.enabled
    }
}

#[derive(Debug, Clone)]
pub struct AppUpdateInfo {
    pub package_name: String,
    pub current_version: String,
    pub available_version: String,
}
