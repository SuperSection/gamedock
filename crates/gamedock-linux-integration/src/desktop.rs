use gamedock_core::{AppConfig, AppInfo, Result, Error};
use crate::desktop_entry::DesktopEntry;
use crate::icons::IconManager;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    GNOME,
    KDE,
    Hyprland,
    XFCE,
    Cinnamon,
    MATE,
    LXDE,
    LXQt,
    Sway,
    I3,
    Other(String),
}

impl DesktopEnvironment {
    pub fn detect() -> Self {
        if let Ok(session) = std::env::var("XDG_CURRENT_DESKTOP") {
            match session.to_lowercase().as_str() {
                "gnome" | "unity" | "budgie" | "pop" => return Self::GNOME,
                "kde" | "plasma" => return Self::KDE,
                "hyprland" => return Self::Hyprland,
                "xfce" => return Self::XFCE,
                "cinnamon" => return Self::Cinnamon,
                "mate" => return Self::MATE,
                "lxde" => return Self::LXDE,
                "lxqt" => return Self::LXQt,
                _ => {}
            }
        }

        if let Ok(session) = std::env::var("DESKTOP_SESSION") {
            match session.to_lowercase().as_str() {
                "sway" => return Self::Sway,
                "i3" => return Self::I3,
                _ => {}
            }
        }

        Self::Other("unknown".to_string())
    }

    pub fn freedesktop_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("applications")
    }

    pub fn icon_theme_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("icons")
            .join("hicolor")
    }
}

pub struct DesktopIntegration {
    config: AppConfig,
    env: DesktopEnvironment,
    icon_manager: IconManager,
}

impl DesktopIntegration {
    pub fn new(config: AppConfig) -> Self {
        let env = DesktopEnvironment::detect();
        let icon_manager = IconManager::new(config.clone());
        tracing::info!("Detected desktop environment: {:?}", env);
        Self { config, env, icon_manager }
    }

    pub fn create_desktop_entry(&self, app: &AppInfo) -> Result<()> {
        let entry = DesktopEntry::from_app_info(app, &self.config)?;
        let dir = self.env.freedesktop_dir();
        std::fs::create_dir_all(&dir)?;

        let filename = format!("gamedock-{}.desktop", app.package_name.replace('.', "-"));
        let path = dir.join(&filename);
        std::fs::write(&path, entry.to_string())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }

        tracing::info!("Created desktop entry: {:?}", path);
        Ok(())
    }

    pub fn remove_desktop_entry(&self, app: &AppInfo) -> Result<()> {
        let dir = self.env.freedesktop_dir();
        let filename = format!("gamedock-{}.desktop", app.package_name.replace('.', "-"));
        let path = dir.join(&filename);
        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::info!("Removed desktop entry: {:?}", path);
        }
        Ok(())
    }

    pub fn install_icon(&self, app: &AppInfo, icon_data: &[u8]) -> Result<PathBuf> {
        self.icon_manager.install_icon(app, icon_data)
    }

    pub fn refresh_desktop_database(&self) -> Result<()> {
        match &self.env {
            DesktopEnvironment::GNOME => {
                let _ = std::process::Command::new("update-desktop-database")
                    .arg(self.env.freedesktop_dir().to_string_lossy().to_string())
                    .status();
            }
            DesktopEnvironment::KDE => {
                let _ = std::process::Command::new("kbuildsycoca5").status();
            }
            _ => {
                let _ = std::process::Command::new("update-desktop-database")
                    .arg(self.env.freedesktop_dir().to_string_lossy().to_string())
                    .status();
            }
        }
        Ok(())
    }

    pub fn register_autostart(&self, app_name: &str, command: &str) -> Result<()> {
        let autostart_dir = dirs::config_dir()
            .ok_or_else(|| Error::DesktopIntegration("Cannot determine config dir".into()))?
            .join("autostart");

        std::fs::create_dir_all(&autostart_dir)?;

        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec={}\nX-GNOME-Autostart-enabled=true\n",
            app_name, command
        );

        let path = autostart_dir.join(format!("gamedock-{}.desktop", app_name.to_lowercase().replace(' ', "-")));
        std::fs::write(&path, entry)?;
        Ok(())
    }

    pub fn get_desktop_environment(&self) -> &DesktopEnvironment {
        &self.env
    }

    pub fn is_wayland(&self) -> bool {
        std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    pub fn is_x11(&self) -> bool {
        std::env::var("DISPLAY").is_ok() && !self.is_wayland()
    }

    pub fn get_display_server(&self) -> &str {
        if self.is_wayland() {
            "wayland"
        } else if self.is_x11() {
            "x11"
        } else {
            "unknown"
        }
    }
}
