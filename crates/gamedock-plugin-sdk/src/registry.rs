use crate::traits::PluginMetadata;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PluginEntry {
    pub metadata: PluginMetadata,
    pub runtime: Option<Arc<dyn crate::traits::RuntimePlugin>>,
    pub installer: Option<Arc<dyn crate::traits::InstallerPlugin>>,
    pub optimizer: Option<Arc<dyn crate::traits::OptimizerPlugin>>,
}

pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, PluginEntry>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_runtime(&self, plugin: Arc<dyn crate::traits::RuntimePlugin>) {
        let metadata = plugin.metadata();
        let entry = PluginEntry {
            metadata: metadata.clone(),
            runtime: Some(plugin),
            installer: None,
            optimizer: None,
        };
        self.plugins
            .write()
            .await
            .insert(metadata.name.clone(), entry);
        tracing::info!("Registered runtime plugin: {}", metadata.name);
    }

    pub async fn register_installer(&self, plugin: Arc<dyn crate::traits::InstallerPlugin>) {
        let metadata = plugin.metadata();
        let entry = PluginEntry {
            metadata: metadata.clone(),
            runtime: None,
            installer: Some(plugin),
            optimizer: None,
        };
        self.plugins
            .write()
            .await
            .insert(metadata.name.clone(), entry);
        tracing::info!("Registered installer plugin: {}", metadata.name);
    }

    pub async fn register_optimizer(&self, plugin: Arc<dyn crate::traits::OptimizerPlugin>) {
        let metadata = plugin.metadata();
        let entry = PluginEntry {
            metadata: metadata.clone(),
            runtime: None,
            installer: None,
            optimizer: Some(plugin),
        };
        self.plugins
            .write()
            .await
            .insert(metadata.name.clone(), entry);
        tracing::info!("Registered optimizer plugin: {}", metadata.name);
    }

    pub async fn get_runtime(&self, name: &str) -> Option<Arc<dyn crate::traits::RuntimePlugin>> {
        self.plugins
            .read()
            .await
            .get(name)
            .and_then(|e| e.runtime.clone())
    }

    pub async fn get_installer(
        &self,
        name: &str,
    ) -> Option<Arc<dyn crate::traits::InstallerPlugin>> {
        self.plugins
            .read()
            .await
            .get(name)
            .and_then(|e| e.installer.clone())
    }

    pub async fn get_optimizer(
        &self,
        name: &str,
    ) -> Option<Arc<dyn crate::traits::OptimizerPlugin>> {
        self.plugins
            .read()
            .await
            .get(name)
            .and_then(|e| e.optimizer.clone())
    }

    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins
            .read()
            .await
            .values()
            .map(|e| e.metadata.clone())
            .collect()
    }

    pub async fn list_runtimes(&self) -> Vec<PluginMetadata> {
        self.plugins
            .read()
            .await
            .values()
            .filter(|e| e.runtime.is_some())
            .map(|e| e.metadata.clone())
            .collect()
    }

    pub async fn remove(&self, name: &str) -> bool {
        self.plugins.write().await.remove(name).is_some()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
