use crate::backup::{BackupBuilder, BackupResult};
use crate::restore::RestoreManager;
use gamedock_core::{AppConfig, AppInfo, Event, EventBus, Result};
use std::path::Path;

pub struct BackupManager {
    #[allow(dead_code)]
    config: AppConfig,
    builder: BackupBuilder,
    restorer: RestoreManager,
    event_bus: EventBus,
}

impl BackupManager {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Self {
        let builder = BackupBuilder::new(config.clone());
        let restorer = RestoreManager::new(config.clone());
        Self {
            config,
            builder,
            restorer,
            event_bus,
        }
    }

    pub async fn create_backup(
        &self,
        app: &AppInfo,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        include_data: bool,
    ) -> Result<BackupResult> {
        let result = self
            .builder
            .create_backup(app, runtime_manager, include_data)
            .await?;

        self.event_bus.publish(Event::BackupCreated {
            backup_id: result.backup_id.clone(),
            path: result.archive_path.to_string_lossy().to_string(),
        });

        Ok(result)
    }

    pub async fn restore_backup(
        &self,
        path: &Path,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
    ) -> Result<()> {
        let metadata = self.restorer.restore_backup(path, runtime_manager).await?;

        self.event_bus.publish(Event::BackupRestored {
            backup_id: metadata.id,
        });

        Ok(())
    }

    pub async fn list_backups(&self) -> Result<Vec<crate::restore::BackupInfo>> {
        self.restorer.list_backups().await
    }

    pub async fn verify_backup(&self, path: &Path) -> Result<bool> {
        self.restorer.verify_backup(path).await
    }

    pub async fn delete_backup(&self, path: &Path) -> Result<()> {
        self.restorer.delete_backup(path).await
    }

    pub async fn cleanup_old_backups(&self, keep_count: usize) -> Result<u32> {
        self.restorer.cleanup_old_backups(keep_count).await
    }
}
