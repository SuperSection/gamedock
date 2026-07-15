use crate::registry::PluginRegistry;
use crate::traits::{PluginMetadata, PluginType};
use gamedock_core::{AppConfig, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct PluginManifest {
    plugin: ManifestPlugin,
}

#[derive(Debug, Deserialize)]
struct ManifestPlugin {
    name: String,
    version: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    description: String,
    r#type: String,
    #[serde(default)]
    entry: Option<String>,
}

impl ManifestPlugin {
    fn to_metadata(&self) -> PluginMetadata {
        let plugin_type = match self.r#type.as_str() {
            "runtime" => PluginType::Runtime,
            "installer" => PluginType::Installer,
            "optimizer" => PluginType::Optimizer,
            "launcher" => PluginType::Launcher,
            _ => PluginType::Runtime,
        };
        PluginMetadata {
            name: self.name.clone(),
            version: self.version.clone(),
            author: self.author.clone(),
            description: self.description.clone(),
            plugin_type,
        }
    }
}

pub struct PluginLoader {
    config: AppConfig,
}

impl PluginLoader {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn load_plugins(&self, registry: &PluginRegistry) -> Result<()> {
        let plugins_dir = self.config.plugins_dir();
        if !plugins_dir.exists() {
            tracing::debug!("No plugins directory found at {:?}", plugins_dir);
            return Ok(());
        }

        let entries: Vec<_> = std::fs::read_dir(&plugins_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "toml")
                    .unwrap_or(false)
            })
            .collect();

        for entry in entries {
            let path = entry.path();
            match self.load_plugin_manifest(&path, registry).await {
                Ok(()) => {}
                Err(e) => {
                    tracing::warn!("Failed to load plugin from {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    async fn load_plugin_manifest(&self, path: &Path, registry: &PluginRegistry) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let manifest: PluginManifest = toml::from_str(&content)?;
        let metadata = manifest.plugin.to_metadata();

        if let Some(ref entry_name) = manifest.plugin.entry {
            let plugin_dir = path.parent().unwrap_or(path);
            let lib_path = plugin_dir.join(entry_name);

            if lib_path.exists() {
                match self.load_native_plugin(&lib_path, registry).await {
                    Ok(()) => {
                        tracing::info!(
                            "Loaded native plugin '{}' v{} from {:?}",
                            metadata.name,
                            metadata.version,
                            path
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load native library for '{}': {}",
                            metadata.name,
                            e
                        );
                    }
                }
            } else {
                tracing::warn!(
                    "Plugin '{}' entry '{}' not found at {:?}",
                    metadata.name,
                    entry_name,
                    lib_path
                );
            }
        }

        tracing::info!(
            "Registered plugin manifest '{}' v{} (type: {:?}) from {:?}",
            metadata.name,
            metadata.version,
            metadata.plugin_type,
            path
        );

        Ok(())
    }

    #[allow(improper_ctypes_definitions)]
    pub async fn load_native_plugin(&self, path: &Path, _registry: &PluginRegistry) -> Result<()> {
        if !path.exists() {
            return Err(gamedock_core::Error::Plugin(format!(
                "Plugin not found: {:?}",
                path
            )));
        }

        unsafe {
            let lib = libloading::Library::new(path).map_err(|e| {
                gamedock_core::Error::Plugin(format!("Failed to load library: {}", e))
            })?;

            type MetadataFn = unsafe extern "C" fn() -> PluginMetadata;

            if let Ok(meta_fn) = lib.get::<MetadataFn>(b"gamedock_plugin_metadata") {
                let metadata = meta_fn();
                tracing::info!(
                    "Loaded native plugin '{}' v{} from {:?}",
                    metadata.name,
                    metadata.version,
                    path
                );
            } else {
                tracing::info!("Loaded native plugin library from {:?}", path);
            }

            Box::leak(Box::new(lib));
        }

        Ok(())
    }
}
