use gamedock_core::Result;
use std::path::PathBuf;

pub struct MangoHUDIntegration {
    config_dir: PathBuf,
}

impl MangoHUDIntegration {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("MangoHud");
        Self { config_dir }
    }

    pub fn is_available() -> bool {
        which::which("mangohud").is_ok() || which::which("mangohud").is_ok()
    }

    pub async fn enable(&self, fps_limit: Option<u32>) -> Result<()> {
        if !Self::is_available() {
            tracing::warn!("MangoHUD not installed. Install with: sudo pacman -S mangohud");
            return Ok(());
        }

        let config = self.generate_config(fps_limit)?;
        self.write_config(&config).await?;

        tracing::info!("MangoHUD configuration written");
        Ok(())
    }

    pub async fn disable(&self) -> Result<()> {
        tracing::info!("MangoHUD disabled");
        Ok(())
    }

    pub async fn enable_for_app(&self, package_name: &str, fps_limit: Option<u32>) -> Result<()> {
        let config = self.generate_config(fps_limit)?;
        let config_path = self.config_dir.join(format!("{}.conf", package_name));
        self.write_config_to(&config, &config_path).await
    }

    fn generate_config(&self, fps_limit: Option<u32>) -> Result<String> {
        let mut config = String::new();

        config.push_str("fps\n");
        config.push_str("frame_timing\n");
        config.push_str("gpu_stats\n");
        config.push_str("gpu_temp\n");
        config.push_str("cpu_temp\n");
        config.push_str("ram\n");
        config.push_str("vram\n");
        config.push_str("engine_version\n");
        config.push_str("vulkan_driver\n");
        config.push_str("io_read\n");
        config.push_str("io_write\n");
        config.push_str("procmem\n");

        if let Some(limit) = fps_limit {
            config.push_str(&format!("fps_limit={}\n", limit));
            config.push_str("fps_value=\n");
        }

        config.push_str("position=top-left\n");
        config.push_str("font_size=24\n");
        config.push_str("background_alpha=0.5\n");
        config.push_str("background_color=000000\n");
        config.push_str("text_color=FFFFFF\n");
        config.push_str("alpha=0.8\n");

        Ok(config)
    }

    async fn write_config(&self, config: &str) -> Result<()> {
        self.write_config_to(config, &self.config_dir.join("GameDock.conf"))
            .await
    }

    async fn write_config_to(&self, config: &str, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, config).await?;
        Ok(())
    }

    pub async fn get_config(&self) -> Result<String> {
        let path = self.config_dir.join("GameDock.conf");
        if path.exists() {
            Ok(tokio::fs::read_to_string(&path).await?)
        } else {
            Ok(self.generate_config(None)?)
        }
    }
}
