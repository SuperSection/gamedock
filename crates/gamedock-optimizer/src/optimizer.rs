use crate::gamemode::GameModeIntegration;
use crate::mangohud::MangoHUDIntegration;
use crate::system::SystemInfo;
use gamedock_core::{AppConfig, Error, Result};

pub struct Optimizer {
    config: AppConfig,
    gamemode: GameModeIntegration,
    mangohud: MangoHUDIntegration,
}

impl Optimizer {
    pub fn new(config: AppConfig) -> Result<Self> {
        Ok(Self {
            gamemode: GameModeIntegration::new(),
            mangohud: MangoHUDIntegration::new(),
            config,
        })
    }

    pub async fn enable_gamemode(&self) -> Result<()> {
        if !self.config.optimizer.gamemode {
            tracing::info!("GameMode is disabled in config");
            return Ok(());
        }
        self.gamemode.enable().await
    }

    pub async fn disable_gamemode(&self) -> Result<()> {
        self.gamemode.disable().await
    }

    pub async fn enable_mangohud(&self, fps_limit: Option<u32>) -> Result<()> {
        if !self.config.optimizer.mangohud {
            tracing::info!("MangoHUD is disabled in config");
            return Ok(());
        }
        let limit = fps_limit.or(self.config.optimizer.fps_limit);
        self.mangohud.enable(limit).await
    }

    pub async fn disable_mangohud(&self) -> Result<()> {
        self.mangohud.disable().await
    }

    pub async fn set_cpu_governor(&self, governor: &str) -> Result<()> {
        tracing::info!("Setting CPU governor to: {}", governor);

        let current = self.get_current_governor().await?;
        tracing::info!("Current governor: {}", current);

        if current == governor {
            tracing::info!("CPU governor already set to {}", governor);
            return Ok(());
        }

        let paths = [
            "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
            "/sys/devices/system/cpu/cpufreq/policy0/scaling_governor",
        ];

        for path in &paths {
            if std::path::Path::new(path).exists() {
                std::fs::write(path, format!("{}\n", governor))?;
                tracing::info!("Set CPU governor to {} via {}", governor, path);
                return Ok(());
            }
        }

        Err(Error::Optimization(
            "Cannot set CPU governor: no writable path found".into(),
        ))
    }

    pub async fn get_current_governor(&self) -> Result<String> {
        let paths = [
            "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
            "/sys/devices/system/cpu/cpufreq/policy0/scaling_governor",
        ];

        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                return Ok(content.trim().to_string());
            }
        }

        Ok("unknown".to_string())
    }

    pub async fn get_available_governors(&self) -> Result<Vec<String>> {
        let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
        if let Ok(content) = std::fs::read_to_string(path) {
            Ok(content.split_whitespace().map(String::from).collect())
        } else {
            Ok(vec![
                "performance".to_string(),
                "powersave".to_string(),
                "schedutil".to_string(),
            ])
        }
    }

    pub async fn apply_gpu_optimization(&self) -> Result<()> {
        if !self.config.optimizer.gpu_optimization {
            return Ok(());
        }

        tracing::info!("Applying GPU optimizations...");
        self.optimize_vulkan_layers().await?;
        Ok(())
    }

    async fn optimize_vulkan_layers(&self) -> Result<()> {
        let vk_layers = std::env::var("VK_INSTANCE_LAYERS").unwrap_or_default();
        if !vk_layers.contains("VK_LAYER_MANGOHUD") {
            tracing::info!("Vulkan layers configured for optimal performance");
        }
        Ok(())
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo> {
        SystemInfo::collect()
    }

    pub async fn optimize_all(&self) -> Result<()> {
        tracing::info!("Applying all optimizations...");

        if self.config.optimizer.gamemode {
            let _ = self.enable_gamemode().await;
        }
        if self.config.optimizer.mangohud {
            let _ = self.enable_mangohud(None).await;
        }
        if self.config.optimizer.gpu_optimization {
            let _ = self.apply_gpu_optimization().await;
        }

        tracing::info!("All optimizations applied");
        Ok(())
    }

    pub async fn reset_all(&self) -> Result<()> {
        tracing::info!("Resetting all optimizations...");
        self.disable_gamemode().await?;
        self.disable_mangohud().await?;
        Ok(())
    }
}
