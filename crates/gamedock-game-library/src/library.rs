use gamedock_core::{AppConfig, AppInfo, AppStatus, Category, Event, EventBus, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryData {
    pub apps: HashMap<String, AppInfo>,
    pub categories: Vec<CategoryEntry>,
    pub recently_played: Vec<String>,
    pub favorites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryEntry {
    pub name: String,
    pub app_ids: Vec<String>,
}

impl Default for LibraryData {
    fn default() -> Self {
        Self {
            apps: HashMap::new(),
            categories: Vec::new(),
            recently_played: Vec::new(),
            favorites: Vec::new(),
        }
    }
}

pub struct GameLibrary {
    config: AppConfig,
    data: Arc<RwLock<LibraryData>>,
    event_bus: EventBus,
}

impl GameLibrary {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Self {
        Self {
            config,
            data: Arc::new(RwLock::new(LibraryData::default())),
            event_bus,
        }
    }

    pub async fn load(&self) -> Result<()> {
        let path = self.library_path();
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let data: LibraryData = serde_json::from_str(&content)?;
            *self.data.write().await = data;
            tracing::info!(
                "Loaded library with {} apps",
                self.data.read().await.apps.len()
            );
        } else {
            tracing::info!("No existing library found, starting fresh");
        }
        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        let path = self.library_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let data = self.data.read().await;
        let content = serde_json::to_string_pretty(&*data)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    pub async fn add_app(&self, app: AppInfo) -> Result<()> {
        let mut data = self.data.write().await;
        data.apps.insert(app.id.clone(), app.clone());
        drop(data);
        self.save().await?;

        self.event_bus.publish(Event::AppInstalled {
            app_id: app.id,
            package_name: app.package_name,
        });

        Ok(())
    }

    pub async fn remove_app(&self, app_id: &str) -> Result<()> {
        let mut data = self.data.write().await;
        data.apps.remove(app_id);
        data.favorites.retain(|id| id != app_id);
        data.recently_played.retain(|id| id != app_id);
        for cat in &mut data.categories {
            cat.app_ids.retain(|id| id != app_id);
        }
        drop(data);
        self.save().await?;

        self.event_bus.publish(Event::AppUninstalled {
            app_id: app_id.to_string(),
        });

        Ok(())
    }

    pub async fn get_app(&self, app_id: &str) -> Option<AppInfo> {
        self.data.read().await.apps.get(app_id).cloned()
    }

    pub async fn list_all_apps(&self) -> Vec<AppInfo> {
        self.data.read().await.apps.values().cloned().collect()
    }

    pub async fn list_installed_apps(&self) -> Vec<AppInfo> {
        self.data
            .read()
            .await
            .apps
            .values()
            .filter(|a| a.status == AppStatus::Installed)
            .cloned()
            .collect()
    }

    pub async fn toggle_favorite(&self, app_id: &str) -> Result<()> {
        let mut data = self.data.write().await;
        if let Some(app) = data.apps.get_mut(app_id) {
            app.is_favorite = !app.is_favorite;
            if app.is_favorite {
                if !data.favorites.contains(&app_id.to_string()) {
                    data.favorites.push(app_id.to_string());
                }
            } else {
                data.favorites.retain(|id| id != app_id);
            }
        }
        drop(data);
        self.save().await
    }

    pub async fn get_favorites(&self) -> Vec<AppInfo> {
        let data = self.data.read().await;
        data.favorites
            .iter()
            .filter_map(|id| data.apps.get(id).cloned())
            .collect()
    }

    pub async fn record_launch(&self, app_id: &str) -> Result<()> {
        let mut data = self.data.write().await;

        if let Some(app) = data.apps.get_mut(app_id) {
            app.last_played = Some(chrono::Utc::now());
        }

        data.recently_played.retain(|id| id != app_id);
        data.recently_played.insert(0, app_id.to_string());
        data.recently_played.truncate(20);

        drop(data);
        self.save().await
    }

    pub async fn get_recently_played(&self, limit: usize) -> Vec<AppInfo> {
        let data = self.data.read().await;
        data.recently_played
            .iter()
            .take(limit)
            .filter_map(|id| data.apps.get(id).cloned())
            .collect()
    }

    pub async fn get_by_category(&self, category: &Category) -> Vec<AppInfo> {
        let data = self.data.read().await;
        data.apps
            .values()
            .filter(|a| a.categories.contains(category))
            .cloned()
            .collect()
    }

    pub async fn add_category(&self, name: &str) -> Result<()> {
        let mut data = self.data.write().await;
        if !data.categories.iter().any(|c| c.name == name) {
            data.categories.push(CategoryEntry {
                name: name.to_string(),
                app_ids: Vec::new(),
            });
        }
        drop(data);
        self.save().await
    }

    pub async fn assign_to_category(&self, app_id: &str, category_name: &str) -> Result<()> {
        let mut data = self.data.write().await;
        if let Some(cat) = data.categories.iter_mut().find(|c| c.name == category_name) {
            if !cat.app_ids.contains(&app_id.to_string()) {
                cat.app_ids.push(app_id.to_string());
            }
        }
        drop(data);
        self.save().await
    }

    pub async fn search(&self, query: &str) -> Vec<AppInfo> {
        let data = self.data.read().await;
        data.apps
            .values()
            .filter(|a| a.matches_search(query))
            .cloned()
            .collect()
    }

    pub async fn sync_from_runtime(
        &self,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        runtime_id: &str,
    ) -> Result<()> {
        let apps = runtime_manager.list_installed_apps(runtime_id).await?;
        let mut data = self.data.write().await;

        for app in apps {
            if !data.apps.contains_key(&app.id) {
                data.apps.insert(app.id.clone(), app);
            }
        }

        drop(data);
        self.save().await?;
        Ok(())
    }

    fn library_path(&self) -> PathBuf {
        self.config.data_dir.join("library.json")
    }
}
