pub mod loader;
pub mod registry;
pub mod traits;

pub use loader::PluginLoader;
pub use registry::PluginRegistry;
pub use traits::{InstallerPlugin, OptimizerPlugin, PluginMetadata, PluginType, RuntimePlugin};
