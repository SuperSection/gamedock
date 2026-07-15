use async_trait::async_trait;
use gamedock_core::{AppInfo, PackageInfo, Result, RuntimeInfo, RuntimeStatus};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    Runtime,
    Installer,
    Optimizer,
    Launcher,
}

#[async_trait]
pub trait RuntimePlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;

    async fn check_status(&self) -> Result<RuntimeStatus>;

    async fn install(&self) -> Result<()>;

    async fn uninstall(&self) -> Result<()>;

    async fn start(&self) -> Result<()>;

    async fn stop(&self) -> Result<()>;

    async fn restart(&self) -> Result<()> {
        self.stop().await?;
        self.start().await
    }

    async fn update(&self) -> Result<()>;

    async fn health_check(&self) -> Result<bool>;

    async fn install_app(&self, package: &Path) -> Result<String>;

    async fn uninstall_app(&self, package_name: &str) -> Result<()>;

    async fn launch_app(&self, package_name: &str) -> Result<()>;

    async fn launch_play_store(&self) -> Result<()>;

    async fn list_installed_apps(&self) -> Result<Vec<AppInfo>>;

    async fn get_runtime_info(&self) -> Result<RuntimeInfo>;

    async fn push_file(&self, local: &Path, remote: &str) -> Result<()>;

    async fn pull_file(&self, remote: &str, local: &Path) -> Result<()>;

    async fn run_command(&self, command: &[&str]) -> Result<String>;
}

#[async_trait]
pub trait InstallerPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;

    fn supported_formats(&self) -> Vec<String>;

    async fn parse_package(&self, path: &Path) -> Result<PackageInfo>;

    async fn install(&self, path: &Path) -> Result<AppInfo>;

    async fn verify_integrity(&self, path: &Path, expected_hash: &str) -> Result<bool>;
}

#[async_trait]
pub trait OptimizerPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;

    async fn apply_profile(&self, profile: &str) -> Result<()>;

    async fn reset(&self) -> Result<()>;

    async fn get_available_profiles(&self) -> Result<Vec<String>>;

    async fn enable_gamemode(&self) -> Result<()>;

    async fn enable_mangohud(&self, fps_limit: Option<u32>) -> Result<()>;

    async fn set_cpu_governor(&self, governor: &str) -> Result<()>;

    async fn get_system_info(&self) -> Result<SystemInfo>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_memory_mb: u64,
    pub gpu_name: Option<String>,
    pub gpu_driver: Option<String>,
    pub display_server: String,
    pub compositor: Option<String>,
    pub kernel_version: String,
}
