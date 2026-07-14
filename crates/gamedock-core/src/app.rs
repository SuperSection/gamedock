use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type AppId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Category {
    Action,
    Adventure,
    Arcade,
    Board,
    Card,
    Casino,
    Casual,
    Educational,
    Music,
    Puzzle,
    Racing,
    RolePlaying,
    Simulation,
    Sports,
    Strategy,
    Trivia,
    Word,
    Other,
    Custom(String),
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Action => write!(f, "Action"),
            Category::Adventure => write!(f, "Adventure"),
            Category::Arcade => write!(f, "Arcade"),
            Category::Board => write!(f, "Board"),
            Category::Card => write!(f, "Card"),
            Category::Casino => write!(f, "Casino"),
            Category::Casual => write!(f, "Casual"),
            Category::Educational => write!(f, "Educational"),
            Category::Music => write!(f, "Music"),
            Category::Puzzle => write!(f, "Puzzle"),
            Category::Racing => write!(f, "Racing"),
            Category::RolePlaying => write!(f, "Role Playing"),
            Category::Simulation => write!(f, "Simulation"),
            Category::Sports => write!(f, "Sports"),
            Category::Strategy => write!(f, "Strategy"),
            Category::Trivia => write!(f, "Trivia"),
            Category::Word => write!(f, "Word"),
            Category::Other => write!(f, "Other"),
            Category::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppStatus {
    NotInstalled,
    Downloading,
    Installing,
    Installed,
    UpdateAvailable,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: AppId,
    pub name: String,
    pub package_name: String,
    pub version_name: String,
    pub version_code: i64,
    pub description: String,
    pub author: String,
    pub icon_path: Option<PathBuf>,
    pub categories: Vec<Category>,
    pub status: AppStatus,
    pub installed_at: Option<DateTime<Utc>>,
    pub last_played: Option<DateTime<Utc>>,
    pub play_time_seconds: u64,
    pub is_favorite: bool,
    pub install_path: Option<PathBuf>,
    pub apk_path: Option<PathBuf>,
    pub size_bytes: Option<u64>,
    pub rating: Option<f32>,
    pub launch_activity: Option<String>,
    pub runtime_id: String,
}

impl AppInfo {
    pub fn new(
        package_name: impl Into<String>,
        name: impl Into<String>,
        version_name: impl Into<String>,
        version_code: i64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            package_name: package_name.into(),
            version_name: version_name.into(),
            version_code,
            description: String::new(),
            author: String::new(),
            icon_path: None,
            categories: vec![Category::Other],
            status: AppStatus::NotInstalled,
            installed_at: None,
            last_played: None,
            play_time_seconds: 0,
            is_favorite: false,
            install_path: None,
            apk_path: None,
            size_bytes: None,
            rating: None,
            launch_activity: None,
            runtime_id: "waydroid".to_string(),
        }
    }

    pub fn display_name(&self) -> &str {
        &self.name
    }

    pub fn is_installed(&self) -> bool {
        self.status == AppStatus::Installed
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.name.to_lowercase().contains(&q)
            || self.package_name.to_lowercase().contains(&q)
            || self.author.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_app() -> AppInfo {
        AppInfo::new("com.test.game", "Test Game", "1.0.0", 1)
    }

    #[test]
    fn test_app_creation() {
        let app = make_test_app();
        assert_eq!(app.package_name, "com.test.game");
        assert_eq!(app.name, "Test Game");
        assert_eq!(app.version_name, "1.0.0");
        assert_eq!(app.version_code, 1);
        assert!(!app.is_installed());
        assert!(!app.is_favorite);
    }

    #[test]
    fn test_display_name() {
        let app = make_test_app();
        assert_eq!(app.display_name(), "Test Game");
    }

    #[test]
    fn test_is_installed() {
        let mut app = make_test_app();
        assert!(!app.is_installed());
        app.status = AppStatus::Installed;
        assert!(app.is_installed());
    }

    #[test]
    fn test_matches_search() {
        let mut app = make_test_app();
        app.description = "An action adventure game".to_string();
        app.author = "Test Studios".to_string();

        assert!(app.matches_search("Test Game"));
        assert!(app.matches_search("test game"));
        assert!(app.matches_search("com.test.game"));
        assert!(app.matches_search("action adventure"));
        assert!(app.matches_search("Test Studios"));
        assert!(!app.matches_search("nonexistent"));
    }

    #[test]
    fn test_category_display() {
        assert_eq!(Category::Action.to_string(), "Action");
        assert_eq!(Category::RolePlaying.to_string(), "Role Playing");
        assert_eq!(Category::Custom("My Category".into()).to_string(), "My Category");
    }

    #[test]
    fn test_app_status_equality() {
        assert_eq!(AppStatus::Installed, AppStatus::Installed);
        assert_ne!(AppStatus::Installed, AppStatus::NotInstalled);
        assert_ne!(
            AppStatus::Error("test".into()),
            AppStatus::Error("other".into())
        );
    }

    #[test]
    fn test_app_serialization_roundtrip() {
        let app = make_test_app();
        let json = serde_json::to_string(&app).unwrap();
        let deserialized: AppInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(app.id, deserialized.id);
        assert_eq!(app.name, deserialized.name);
        assert_eq!(app.package_name, deserialized.package_name);
    }
}
