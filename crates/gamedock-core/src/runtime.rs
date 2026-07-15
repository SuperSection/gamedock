use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeStatus {
    NotInstalled,
    Installing,
    Installed,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

impl std::fmt::Display for RuntimeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeStatus::NotInstalled => write!(f, "Not Installed"),
            RuntimeStatus::Installing => write!(f, "Installing"),
            RuntimeStatus::Installed => write!(f, "Installed"),
            RuntimeStatus::Running => write!(f, "Running"),
            RuntimeStatus::Stopping => write!(f, "Stopping"),
            RuntimeStatus::Stopped => write!(f, "Stopped"),
            RuntimeStatus::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub status: RuntimeStatus,
    pub android_version: Option<String>,
    pub image_type: Option<String>,
    pub supported_architectures: Vec<String>,
}

impl RuntimeInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: None,
            status: RuntimeStatus::NotInstalled,
            android_version: None,
            image_type: None,
            supported_architectures: vec!["x86_64".to_string(), "arm64-v8a".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_status_display() {
        assert_eq!(RuntimeStatus::NotInstalled.to_string(), "Not Installed");
        assert_eq!(RuntimeStatus::Installing.to_string(), "Installing");
        assert_eq!(RuntimeStatus::Installed.to_string(), "Installed");
        assert_eq!(RuntimeStatus::Running.to_string(), "Running");
        assert_eq!(RuntimeStatus::Stopping.to_string(), "Stopping");
        assert_eq!(RuntimeStatus::Stopped.to_string(), "Stopped");
        assert_eq!(
            RuntimeStatus::Error("test".into()).to_string(),
            "Error: test"
        );
    }

    #[test]
    fn test_runtime_status_equality() {
        assert_eq!(RuntimeStatus::Running, RuntimeStatus::Running);
        assert_ne!(RuntimeStatus::Running, RuntimeStatus::Installed);
        assert_ne!(
            RuntimeStatus::Error("a".into()),
            RuntimeStatus::Error("b".into())
        );
    }

    #[test]
    fn test_runtime_info_creation() {
        let info = RuntimeInfo::new("waydroid", "Waydroid");
        assert_eq!(info.id, "waydroid");
        assert_eq!(info.name, "Waydroid");
        assert_eq!(info.status, RuntimeStatus::NotInstalled);
        assert!(info.supported_architectures.contains(&"x86_64".to_string()));
        assert!(info
            .supported_architectures
            .contains(&"arm64-v8a".to_string()));
    }

    #[test]
    fn test_runtime_info_serialization() {
        let info = RuntimeInfo::new("test", "Test Runtime");
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: RuntimeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.id, deserialized.id);
        assert_eq!(info.name, deserialized.name);
    }
}
