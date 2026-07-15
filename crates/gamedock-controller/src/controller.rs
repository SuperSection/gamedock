use crate::mapping::InputMapping;
use crate::profiles::MappingProfile;
use gamedock_core::{AppConfig, Error, Event, EventBus, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ControllerType {
    Xbox,
    DualSense,
    DualShock,
    SwitchPro,
    EightBitDo,
    Generic,
}

impl ControllerType {
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            n if n.contains("xbox") || n.contains("microsoft") => Self::Xbox,
            n if n.contains("dualsense") || n.contains("ps5") => Self::DualSense,
            n if n.contains("dualshock") || n.contains("ps4") || n.contains("playstation") => {
                Self::DualShock
            }
            n if n.contains("switch") || n.contains("pro controller") => Self::SwitchPro,
            n if n.contains("8bitdo") || n.contains("8bit") => Self::EightBitDo,
            _ => Self::Generic,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Xbox => "Xbox Controller",
            Self::DualSense => "DualSense (PS5)",
            Self::DualShock => "DualShock (PS4)",
            Self::SwitchPro => "Switch Pro Controller",
            Self::EightBitDo => "8BitDo Controller",
            Self::Generic => "Generic Controller",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerInfo {
    pub id: String,
    pub name: String,
    pub controller_type: ControllerType,
    pub connected: bool,
    pub battery_level: Option<f32>,
    pub active_profile: Option<String>,
}

impl ControllerInfo {
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.controller_type.display_name(), self.name)
    }

    pub fn device_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("/dev/input/{}", self.name))
    }
}

pub struct ControllerManager {
    config: AppConfig,
    controllers: Arc<RwLock<HashMap<String, ControllerInfo>>>,
    profiles: Arc<RwLock<HashMap<String, MappingProfile>>>,
    event_bus: EventBus,
}

impl ControllerManager {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Self {
        Self {
            config,
            controllers: Arc::new(RwLock::new(HashMap::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        self.load_profiles().await?;
        self.scan_controllers().await?;
        tracing::info!("ControllerManager initialized");
        Ok(())
    }

    pub async fn scan_controllers(&self) -> Result<()> {
        tracing::info!("Scanning for controllers...");
        let mut controllers = self.controllers.write().await;

        let evdev_dir = std::path::Path::new("/dev/input");
        if evdev_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(evdev_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("js") {
                        let id = format!("controller_{}", name);
                        let info = ControllerInfo {
                            id: id.clone(),
                            name: name.clone(),
                            controller_type: ControllerType::Generic,
                            connected: true,
                            battery_level: None,
                            active_profile: None,
                        };
                        controllers.insert(id, info.clone());

                        self.event_bus.publish(Event::ControllerConnected {
                            controller_id: info.id,
                            controller_type: info.controller_type.display_name().to_string(),
                        });
                    }
                }
            }
        }

        tracing::info!("Found {} controllers", controllers.len());
        Ok(())
    }

    pub async fn list_controllers(&self) -> Vec<ControllerInfo> {
        self.controllers.read().await.values().cloned().collect()
    }

    pub async fn get_controller(&self, id: &str) -> Option<ControllerInfo> {
        self.controllers.read().await.get(id).cloned()
    }

    pub async fn set_profile(&self, controller_id: &str, profile_name: &str) -> Result<()> {
        let profiles = self.profiles.read().await;
        let _profile = profiles
            .get(profile_name)
            .ok_or_else(|| Error::Controller(format!("Profile '{}' not found", profile_name)))?;

        let mut controllers = self.controllers.write().await;
        if let Some(controller) = controllers.get_mut(controller_id) {
            controller.active_profile = Some(profile_name.to_string());
            tracing::info!(
                "Set profile '{}' for controller '{}'",
                profile_name,
                controller.name
            );
        }

        Ok(())
    }

    pub async fn get_active_mapping(&self, controller_id: &str) -> Option<InputMapping> {
        let controllers = self.controllers.read().await;
        let profiles = self.profiles.read().await;

        if let Some(controller) = controllers.get(controller_id) {
            if let Some(ref profile_name) = controller.active_profile {
                return profiles.get(profile_name).map(|p| p.mapping.clone());
            }
        }

        profiles.get("default").map(|p| p.mapping.clone())
    }

    pub async fn add_profile(&self, profile: MappingProfile) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        profiles.insert(profile.name.clone(), profile);
        self.save_profiles().await
    }

    pub async fn remove_profile(&self, name: &str) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        profiles.remove(name);
        self.save_profiles().await
    }

    pub async fn list_profiles(&self) -> Vec<MappingProfile> {
        self.profiles.read().await.values().cloned().collect()
    }

    pub async fn create_default_profiles(&self) -> Result<()> {
        let defaults = vec![
            MappingProfile::default_xbox(),
            MappingProfile::default_dualsense(),
            MappingProfile::default_switch_pro(),
            MappingProfile::default_8bitdo(),
        ];

        for profile in defaults {
            self.add_profile(profile).await?;
        }

        Ok(())
    }

    fn profiles_path(&self) -> PathBuf {
        self.config.data_dir.join("controller_profiles.json")
    }

    async fn load_profiles(&self) -> Result<()> {
        let path = self.profiles_path();
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let profiles: HashMap<String, MappingProfile> = serde_json::from_str(&content)?;
            *self.profiles.write().await = profiles;
        }
        Ok(())
    }

    async fn save_profiles(&self) -> Result<()> {
        let path = self.profiles_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let profiles = self.profiles.read().await;
        let content = serde_json::to_string_pretty(&*profiles)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_type_from_name() {
        assert_eq!(
            ControllerType::from_name("Xbox Wireless Controller"),
            ControllerType::Xbox
        );
        assert_eq!(
            ControllerType::from_name("Sony DualSense"),
            ControllerType::DualSense
        );
        assert_eq!(
            ControllerType::from_name("DualShock 4"),
            ControllerType::DualShock
        );
        assert_eq!(
            ControllerType::from_name("Pro Controller"),
            ControllerType::SwitchPro
        );
        assert_eq!(
            ControllerType::from_name("8BitDo Pro 2"),
            ControllerType::EightBitDo
        );
        assert_eq!(
            ControllerType::from_name("Generic Pad"),
            ControllerType::Generic
        );
    }

    #[test]
    fn test_controller_type_display_name() {
        assert_eq!(ControllerType::Xbox.display_name(), "Xbox Controller");
        assert_eq!(ControllerType::DualSense.display_name(), "DualSense (PS5)");
        assert_eq!(ControllerType::DualShock.display_name(), "DualShock (PS4)");
        assert_eq!(
            ControllerType::SwitchPro.display_name(),
            "Switch Pro Controller"
        );
        assert_eq!(
            ControllerType::EightBitDo.display_name(),
            "8BitDo Controller"
        );
        assert_eq!(ControllerType::Generic.display_name(), "Generic Controller");
    }

    #[test]
    fn test_controller_type_case_insensitive() {
        assert_eq!(
            ControllerType::from_name("xbox controller"),
            ControllerType::Xbox
        );
        assert_eq!(ControllerType::from_name("XBOX"), ControllerType::Xbox);
        assert_eq!(
            ControllerType::from_name("PS5 DualSense"),
            ControllerType::DualSense
        );
    }

    #[test]
    fn test_controller_type_serialization() {
        let types = vec![
            ControllerType::Xbox,
            ControllerType::DualSense,
            ControllerType::DualShock,
            ControllerType::SwitchPro,
            ControllerType::EightBitDo,
            ControllerType::Generic,
        ];
        for ct in types {
            let json = serde_json::to_string(&ct).unwrap();
            let deserialized: ControllerType = serde_json::from_str(&json).unwrap();
            assert_eq!(ct, deserialized);
        }
    }

    #[tokio::test]
    async fn test_controller_manager_init() {
        let config = AppConfig::default();
        let event_bus = EventBus::default();
        let manager = ControllerManager::new(config, event_bus);
        manager.initialize().await.unwrap();
        // Controller count depends on the system
        let _controllers = manager.list_controllers().await;
    }
}
