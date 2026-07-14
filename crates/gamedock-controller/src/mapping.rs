use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    X,
    Y,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Start,
    Select,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    LeftStick,
    RightStick,
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
    Touchpad,
    Guide,
    Capture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    Space, Enter, Escape, Tab, Backspace, Delete,
    Up, Down, Left, Right,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    LeftShift, RightShift, LeftCtrl, RightCtrl, LeftAlt, RightAlt,
    MouseLeft, MouseRight, MouseMiddle, Mouse4, Mouse5,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Action {
    Tap(Button),
    KeyPress(Vec<Key>),
    Combo(Vec<Button>),
    Swipe { from: (i32, i32), to: (i32, i32) },
    TapAt { x: i32, y: i32 },
    LongPress(Button),
    DoubleTap(Button),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMapping {
    pub name: String,
    pub button_map: HashMap<Button, Action>,
    pub deadzone: f32,
    pub sensitivity: f32,
    pub invert_y: bool,
    pub rumble_enabled: bool,
    pub turbo_enabled: bool,
    pub turbo_rate_ms: u32,
}

impl Default for InputMapping {
    fn default() -> Self {
        let mut button_map = HashMap::new();

        button_map.insert(Button::A, Action::Tap(Button::A));
        button_map.insert(Button::B, Action::Tap(Button::B));
        button_map.insert(Button::X, Action::Tap(Button::X));
        button_map.insert(Button::Y, Action::Tap(Button::Y));

        button_map.insert(Button::DPadUp, Action::Tap(Button::DPadUp));
        button_map.insert(Button::DPadDown, Action::Tap(Button::DPadDown));
        button_map.insert(Button::DPadLeft, Action::Tap(Button::DPadLeft));
        button_map.insert(Button::DPadRight, Action::Tap(Button::DPadRight));

        button_map.insert(Button::Start, Action::Tap(Button::Start));
        button_map.insert(Button::Select, Action::Tap(Button::Select));

        button_map.insert(Button::LeftBumper, Action::Tap(Button::LeftBumper));
        button_map.insert(Button::RightBumper, Action::Tap(Button::RightBumper));
        button_map.insert(Button::LeftTrigger, Action::Tap(Button::LeftTrigger));
        button_map.insert(Button::RightTrigger, Action::Tap(Button::RightTrigger));

        Self {
            name: "Default".to_string(),
            button_map,
            deadzone: 0.15,
            sensitivity: 1.0,
            invert_y: false,
            rumble_enabled: true,
            turbo_enabled: false,
            turbo_rate_ms: 100,
        }
    }
}

impl InputMapping {
    pub fn new(name: impl Into<String>) -> Self {
        let mut mapping = Self::default();
        mapping.name = name.into();
        mapping
    }

    pub fn map_button(&mut self, from: Button, to: Action) {
        self.button_map.insert(from, to);
    }

    pub fn get_action(&self, button: &Button) -> Option<&Action> {
        self.button_map.get(button)
    }

    pub fn create_fps_profile() -> Self {
        let mut mapping = Self::new("FPS Profile");
        mapping.map_button(Button::LeftTrigger, Action::Tap(Button::LeftTrigger));
        mapping.map_button(Button::RightTrigger, Action::Tap(Button::RightTrigger));
        mapping.map_button(Button::LeftBumper, Action::KeyPress(vec![Key::R]));
        mapping.map_button(Button::RightBumper, Action::KeyPress(vec![Key::Q]));
        mapping.map_button(Button::A, Action::Tap(Button::A));
        mapping.map_button(Button::B, Action::Tap(Button::B));
        mapping.map_button(Button::X, Action::KeyPress(vec![Key::E]));
        mapping.map_button(Button::Y, Action::KeyPress(vec![Key::F]));
        mapping.map_button(Button::Start, Action::Tap(Button::Start));
        mapping.map_button(Button::Select, Action::KeyPress(vec![Key::Tab]));
        mapping.map_button(Button::DPadUp, Action::Tap(Button::DPadUp));
        mapping.map_button(Button::DPadDown, Action::Tap(Button::DPadDown));
        mapping.map_button(Button::DPadLeft, Action::KeyPress(vec![Key::A]));
        mapping.map_button(Button::DPadRight, Action::KeyPress(vec![Key::D]));
        mapping.sensitivity = 1.2;
        mapping
    }

    pub fn create_racing_profile() -> Self {
        let mut mapping = Self::new("Racing Profile");
        mapping.map_button(Button::RightTrigger, Action::KeyPress(vec![Key::W]));
        mapping.map_button(Button::LeftTrigger, Action::KeyPress(vec![Key::S]));
        mapping.map_button(Button::DPadLeft, Action::KeyPress(vec![Key::A]));
        mapping.map_button(Button::DPadRight, Action::KeyPress(vec![Key::D]));
        mapping.map_button(Button::A, Action::Tap(Button::A));
        mapping.map_button(Button::B, Action::KeyPress(vec![Key::Space]));
        mapping.map_button(Button::X, Action::KeyPress(vec![Key::E]));
        mapping.map_button(Button::Y, Action::KeyPress(vec![Key::Q]));
        mapping.map_button(Button::Start, Action::Tap(Button::Start));
        mapping.map_button(Button::Select, Action::KeyPress(vec![Key::Tab]));
        mapping.sensitivity = 0.8;
        mapping
    }

    pub fn create_platformer_profile() -> Self {
        let mut mapping = Self::new("Platformer Profile");
        mapping.map_button(Button::A, Action::KeyPress(vec![Key::Space]));
        mapping.map_button(Button::B, Action::KeyPress(vec![Key::X]));
        mapping.map_button(Button::X, Action::KeyPress(vec![Key::Z]));
        mapping.map_button(Button::Y, Action::KeyPress(vec![Key::C]));
        mapping.map_button(Button::DPadUp, Action::KeyPress(vec![Key::Up]));
        mapping.map_button(Button::DPadDown, Action::KeyPress(vec![Key::Down]));
        mapping.map_button(Button::DPadLeft, Action::KeyPress(vec![Key::Left]));
        mapping.map_button(Button::DPadRight, Action::KeyPress(vec![Key::Right]));
        mapping.map_button(Button::Start, Action::Tap(Button::Start));
        mapping.map_button(Button::Select, Action::Tap(Button::Select));
        mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mapping() {
        let mapping = InputMapping::default();
        assert_eq!(mapping.name, "Default");
        assert_eq!(mapping.deadzone, 0.15);
        assert_eq!(mapping.sensitivity, 1.0);
        assert!(!mapping.invert_y);
        assert!(mapping.rumble_enabled);
        assert!(!mapping.turbo_enabled);
    }

    #[test]
    fn test_map_button() {
        let mut mapping = InputMapping::new("Test");
        mapping.map_button(Button::A, Action::KeyPress(vec![Key::Space]));
        let action = mapping.get_action(&Button::A);
        assert!(action.is_some());
        match action.unwrap() {
            Action::KeyPress(keys) => assert_eq!(keys[0], Key::Space),
            _ => panic!("Expected KeyPress action"),
        }
    }

    #[test]
    fn test_get_action_nonexistent() {
        let mapping = InputMapping::default();
        assert!(mapping.get_action(&Button::Touchpad).is_none());
    }

    #[test]
    fn test_fps_profile() {
        let mapping = InputMapping::create_fps_profile();
        assert_eq!(mapping.name, "FPS Profile");
        assert_eq!(mapping.sensitivity, 1.2);
    }

    #[test]
    fn test_racing_profile() {
        let mapping = InputMapping::create_racing_profile();
        assert_eq!(mapping.name, "Racing Profile");
        assert_eq!(mapping.sensitivity, 0.8);
    }

    #[test]
    fn test_platformer_profile() {
        let mapping = InputMapping::create_platformer_profile();
        assert_eq!(mapping.name, "Platformer Profile");
    }

    #[test]
    fn test_mapping_serialization() {
        let mapping = InputMapping::create_fps_profile();
        let json = serde_json::to_string(&mapping).unwrap();
        let deserialized: InputMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(mapping.name, deserialized.name);
        assert_eq!(mapping.sensitivity, deserialized.sensitivity);
    }

    #[test]
    fn test_button_serialization() {
        let buttons = vec![
            Button::A, Button::B, Button::X, Button::Y,
            Button::DPadUp, Button::DPadDown, Button::Start,
        ];
        for button in buttons {
            let json = serde_json::to_string(&button).unwrap();
            let deserialized: Button = serde_json::from_str(&json).unwrap();
            assert_eq!(button, deserialized);
        }
    }
}
