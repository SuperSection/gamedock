use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    #[serde(default = "default_runtime")]
    pub default_runtime: String,

    #[serde(default)]
    pub waydroid: WaydroidConfig,

    #[serde(default)]
    pub optimizer: OptimizerConfig,

    #[serde(default)]
    pub controller: ControllerConfig,

    #[serde(default)]
    pub ui: UiConfig,

    #[serde(default)]
    pub play_store: PlayStoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WaydroidConfig {
    #[serde(default = "default_waydroid_image_type")]
    pub image_type: String,

    #[serde(default)]
    pub custom_props: Vec<String>,

    #[serde(default = "default_true")]
    pub auto_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    #[serde(default = "default_true")]
    pub gamemode: bool,

    #[serde(default = "default_true")]
    pub mangohud: bool,

    #[serde(default)]
    pub cpu_governor: Option<String>,

    #[serde(default = "default_true")]
    pub gpu_optimization: bool,

    #[serde(default)]
    pub fps_limit: Option<u32>,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            gamemode: true,
            mangohud: true,
            cpu_governor: None,
            gpu_optimization: true,
            fps_limit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ControllerConfig {
    #[serde(default = "default_true")]
    pub auto_detect: bool,

    #[serde(default)]
    pub custom_profiles: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_true")]
    pub show_fps_overlay: bool,

    #[serde(default)]
    pub window_width: Option<u32>,

    #[serde(default)]
    pub window_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayStoreConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub auto_update_apps: bool,
}

fn default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gamedock")
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gamedock")
}

fn default_runtime() -> String {
    "waydroid".to_string()
}

fn default_waydroid_image_type() -> String {
    "system".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            cache_dir: default_cache_dir(),
            default_runtime: default_runtime(),
            waydroid: WaydroidConfig::default(),
            optimizer: OptimizerConfig::default(),
            controller: ControllerConfig::default(),
            ui: UiConfig::default(),
            play_store: PlayStoreConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("gamedock").join("config.toml"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn apps_dir(&self) -> PathBuf {
        self.data_dir.join("apps")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }

    pub fn icons_dir(&self) -> PathBuf {
        self.cache_dir.join("icons")
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.data_dir.join("plugins")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.default_runtime, "waydroid");
        assert!(config.optimizer.gamemode);
        assert!(config.optimizer.mangohud);
        assert!(config.optimizer.gpu_optimization);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.default_runtime, deserialized.default_runtime);
        assert_eq!(config.optimizer.gamemode, deserialized.optimizer.gamemode);
    }

    #[test]
    fn test_config_directories() {
        let mut config = AppConfig::default();
        config.data_dir = PathBuf::from("/tmp/gamedock-test/data");
        config.cache_dir = PathBuf::from("/tmp/gamedock-test/cache");

        assert_eq!(config.apps_dir(), PathBuf::from("/tmp/gamedock-test/data/apps"));
        assert_eq!(config.backups_dir(), PathBuf::from("/tmp/gamedock-test/data/backups"));
        assert_eq!(config.icons_dir(), PathBuf::from("/tmp/gamedock-test/cache/icons"));
        assert_eq!(config.plugins_dir(), PathBuf::from("/tmp/gamedock-test/data/plugins"));
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempdir().unwrap();
        let mut config = AppConfig::default();
        config.data_dir = dir.path().join("data");
        config.cache_dir = dir.path().join("cache");

        let path = dir.path().join("config.toml");
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&path, &content).unwrap();

        let loaded: AppConfig = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config.default_runtime, loaded.default_runtime);
    }
}
