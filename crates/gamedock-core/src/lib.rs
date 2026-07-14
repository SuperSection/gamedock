pub mod error;
pub mod config;
pub mod app;
pub mod paths;
pub mod events;
pub mod package;
pub mod runtime;

pub use error::{Error, Result};
pub use config::AppConfig;
pub use app::{AppInfo, AppId, Category, AppStatus};
pub use paths::GameDockPaths;
pub use events::{Event, EventBus};
pub use package::{PackageFormat, PackageInfo};
pub use runtime::{RuntimeStatus, RuntimeInfo};
