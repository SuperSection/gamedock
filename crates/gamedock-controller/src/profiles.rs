use crate::mapping::{Action, Button, InputMapping};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProfile {
    pub name: String,
    pub description: String,
    pub mapping: InputMapping,
    pub controller_type: String,
    pub is_default: bool,
}

impl MappingProfile {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            mapping: InputMapping::default(),
            controller_type: "generic".to_string(),
            is_default: false,
        }
    }

    pub fn default_xbox() -> Self {
        let mut profile = Self::new("Xbox Default", "Default mapping for Xbox controllers");
        profile.controller_type = "xbox".to_string();
        profile.is_default = true;
        profile.mapping = InputMapping::default();
        profile
    }

    pub fn default_dualsense() -> Self {
        let mut profile = Self::new("DualSense Default", "Default mapping for PS5 DualSense");
        profile.controller_type = "dualsense".to_string();
        profile.is_default = true;

        profile
            .mapping
            .map_button(Button::A, Action::Tap(Button::B));
        profile
            .mapping
            .map_button(Button::B, Action::Tap(Button::A));
        profile
            .mapping
            .map_button(Button::X, Action::Tap(Button::Y));
        profile
            .mapping
            .map_button(Button::Y, Action::Tap(Button::X));

        profile
    }

    pub fn default_switch_pro() -> Self {
        let mut profile = Self::new("Switch Pro Default", "Default mapping for Switch Pro");
        profile.controller_type = "switch_pro".to_string();
        profile.is_default = true;
        profile.mapping = InputMapping::default();
        profile
    }

    pub fn default_8bitdo() -> Self {
        let mut profile = Self::new("8BitDo Default", "Default mapping for 8BitDo controllers");
        profile.controller_type = "8bitdo".to_string();
        profile.is_default = true;
        profile.mapping = InputMapping::default();
        profile
    }

    pub fn fps_profile() -> Self {
        let mut profile = Self::new("FPS Profile", "Optimized for first-person shooters");
        profile.mapping = InputMapping::create_fps_profile();
        profile
    }

    pub fn racing_profile() -> Self {
        let mut profile = Self::new("Racing Profile", "Optimized for racing games");
        profile.mapping = InputMapping::create_racing_profile();
        profile
    }

    pub fn platformer_profile() -> Self {
        let mut profile = Self::new("Platformer Profile", "Optimized for platformers");
        profile.mapping = InputMapping::create_platformer_profile();
        profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = MappingProfile::new("Test", "A test profile");
        assert_eq!(profile.name, "Test");
        assert_eq!(profile.description, "A test profile");
        assert_eq!(profile.controller_type, "generic");
        assert!(!profile.is_default);
    }

    #[test]
    fn test_default_xbox_profile() {
        let profile = MappingProfile::default_xbox();
        assert_eq!(profile.name, "Xbox Default");
        assert_eq!(profile.controller_type, "xbox");
        assert!(profile.is_default);
    }

    #[test]
    fn test_default_dualsense_profile() {
        let profile = MappingProfile::default_dualsense();
        assert_eq!(profile.name, "DualSense Default");
        assert_eq!(profile.controller_type, "dualsense");
        assert!(profile.is_default);
    }

    #[test]
    fn test_default_switch_pro_profile() {
        let profile = MappingProfile::default_switch_pro();
        assert_eq!(profile.name, "Switch Pro Default");
        assert_eq!(profile.controller_type, "switch_pro");
        assert!(profile.is_default);
    }

    #[test]
    fn test_default_8bitdo_profile() {
        let profile = MappingProfile::default_8bitdo();
        assert_eq!(profile.name, "8BitDo Default");
        assert_eq!(profile.controller_type, "8bitdo");
        assert!(profile.is_default);
    }

    #[test]
    fn test_fps_profile() {
        let profile = MappingProfile::fps_profile();
        assert_eq!(profile.name, "FPS Profile");
        assert_eq!(profile.mapping.name, "FPS Profile");
    }

    #[test]
    fn test_racing_profile() {
        let profile = MappingProfile::racing_profile();
        assert_eq!(profile.name, "Racing Profile");
    }

    #[test]
    fn test_platformer_profile() {
        let profile = MappingProfile::platformer_profile();
        assert_eq!(profile.name, "Platformer Profile");
    }

    #[test]
    fn test_profile_serialization() {
        let profile = MappingProfile::default_xbox();
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: MappingProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile.name, deserialized.name);
        assert_eq!(profile.controller_type, deserialized.controller_type);
    }
}
