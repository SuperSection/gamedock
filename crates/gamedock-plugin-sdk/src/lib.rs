pub mod traits;
pub mod registry;
pub mod loader;

pub use traits::{RuntimePlugin, InstallerPlugin, OptimizerPlugin, PluginMetadata, PluginType};
pub use registry::PluginRegistry;
pub use loader::PluginLoader;
