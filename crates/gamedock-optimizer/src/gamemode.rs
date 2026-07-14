use gamedock_core::{Result, Error};
use tokio::process::Command;

pub struct GameModeIntegration;

impl GameModeIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn is_available() -> bool {
        which::which("gamemoderun").is_ok() || which::which("gamemode").is_ok()
    }

    pub async fn enable(&self) -> Result<()> {
        if !Self::is_available() {
            tracing::warn!("GameMode not installed. Install with: sudo pacman -S gamemode");
            return Ok(());
        }

        tracing::info!("GameMode integration enabled");
        Ok(())
    }

    pub async fn disable(&self) -> Result<()> {
        tracing::info!("GameMode integration disabled");
        Ok(())
    }

    pub async fn run_with_gamemode(&self, command: &[&str]) -> Result<String> {
        if !Self::is_available() {
            return Err(Error::Optimization("GameMode not available".into()));
        }

        let output = Command::new("gamemoderun")
            .args(command)
            .output()
            .await
            .map_err(|e| Error::Optimization(format!("Failed to run gamemoderun: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Optimization(format!("gamemoderun failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn status(&self) -> Result<GameModeStatus> {
        if !Self::is_available() {
            return Ok(GameModeStatus {
                available: false,
                active: false,
                client_count: 0,
            });
        }

        let output = Command::new("gamemoded")
            .arg("--status")
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let status_str = String::from_utf8_lossy(&out.stdout);
                let active = status_str.contains("is active");
                Ok(GameModeStatus {
                    available: true,
                    active,
                    client_count: 0,
                })
            }
            _ => Ok(GameModeStatus {
                available: true,
                active: false,
                client_count: 0,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameModeStatus {
    pub available: bool,
    pub active: bool,
    pub client_count: u32,
}
