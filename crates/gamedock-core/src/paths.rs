use crate::config::AppConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GameDockPaths {
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub apps_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub icons_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl GameDockPaths {
    pub fn from_config(config: &AppConfig) -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gamedock");

        Self {
            data_dir: config.data_dir.clone(),
            cache_dir: config.cache_dir.clone(),
            config_dir,
            apps_dir: config.apps_dir(),
            backups_dir: config.backups_dir(),
            icons_dir: config.icons_dir(),
            plugins_dir: config.plugins_dir(),
            runtime_dir: config.data_dir.join("runtime"),
            logs_dir: config.data_dir.join("logs"),
        }
    }

    pub fn ensure_directories(&self) -> anyhow::Result<()> {
        let dirs = [
            &self.data_dir,
            &self.cache_dir,
            &self.apps_dir,
            &self.backups_dir,
            &self.icons_dir,
            &self.plugins_dir,
            &self.runtime_dir,
            &self.logs_dir,
        ];
        for dir in &dirs {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}
