use crate::backup::BackupMetadata;
use gamedock_core::{AppConfig, Result, Error};
use gamedock_plugin_sdk::RuntimePlugin;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

pub struct RestoreManager {
    config: AppConfig,
}

impl RestoreManager {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let backups_dir = self.config.backups_dir();
        if !backups_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in std::fs::read_dir(&backups_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("zip") {
                if let Ok(info) = self.get_backup_info(&path).await {
                    backups.push(info);
                }
            }
        }

        backups.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
        Ok(backups)
    }

    pub async fn get_backup_info(&self, path: &Path) -> Result<BackupInfo> {
        let bytes = std::fs::read(path)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
            .map_err(|e| Error::Backup(format!("Invalid backup archive: {}", e)))?;

        for i in 0..archive.len() {
            let entry = archive.by_index(i)
                .map_err(|e| Error::Zip(format!("{}", e)))?;
            let name = entry.name().to_string();
            let size = entry.size();
            if name == "metadata.json" {
                let mut content = String::new();
                let mut reader = std::io::Read::take(entry, size);
                std::io::Read::read_to_string(&mut reader, &mut content)?;
                let metadata: BackupMetadata = serde_json::from_str(&content)?;

                let file_size = std::fs::metadata(path)?.len();

                return Ok(BackupInfo {
                    path: path.to_path_buf(),
                    metadata,
                    file_size,
                });
            }
        }

        Err(Error::Backup("No metadata found in backup archive".into()))
    }

    pub async fn verify_backup(&self, path: &Path) -> Result<bool> {
        let info = self.get_backup_info(path).await?;
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());

        Ok(hash == info.metadata.sha256)
    }

    pub async fn restore_backup(
        &self,
        path: &Path,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
    ) -> Result<BackupMetadata> {
        let info = self.get_backup_info(path).await?;

        tracing::info!("Restoring backup: {} ({})", info.metadata.app_name, info.metadata.id);

        let temp_dir = tempfile::tempdir()?;
        let bytes = std::fs::read(path)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
            .map_err(|e| Error::Backup(format!("Invalid backup archive: {}", e)))?;

        archive.extract(temp_dir.path())
            .map_err(|e| Error::Backup(format!("Failed to extract backup: {}", e)))?;

        if info.metadata.includes_apk {
            let apk_path = temp_dir.path().join("base.apk");
            if apk_path.exists() {
                runtime_manager.install_app(
                    "waydroid",
                    &apk_path,
                ).await?;
            }
        }

        if info.metadata.includes_data {
            let data_dir = temp_dir.path().join("data");
            if data_dir.exists() {
                self.restore_app_data(
                    runtime_manager,
                    &info.metadata.package_name,
                    &data_dir,
                ).await?;
            }
        }

        tracing::info!("Backup restored successfully: {}", info.metadata.app_name);
        Ok(info.metadata)
    }

    async fn restore_app_data(
        &self,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        package_name: &str,
        data_dir: &Path,
    ) -> Result<()> {
        let runtime = runtime_manager.get_runtime("waydroid").await?;

        if data_dir.exists() {
            for entry in walkdir::WalkDir::new(data_dir) {
                let entry = entry.map_err(|e| Error::Backup(format!("{}", e)))?;
                if entry.file_type().is_file() {
                    let relative = entry.path().strip_prefix(data_dir)
                        .unwrap_or(entry.path());
                    let remote_path = format!("/data/data/{}/{}", package_name, relative.to_string_lossy());
                    let _ = runtime.push_file(entry.path(), &remote_path).await;
                }
            }
        }

        Ok(())
    }

    pub async fn delete_backup(&self, path: &Path) -> Result<()> {
        if path.exists() {
            tokio::fs::remove_file(path).await?;
            tracing::info!("Deleted backup: {:?}", path);
        }
        Ok(())
    }

    pub async fn cleanup_old_backups(&self, keep_count: usize) -> Result<u32> {
        let mut backups = self.list_backups().await?;
        let mut deleted = 0;

        if backups.len() > keep_count {
            backups.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
            for backup in backups.iter().skip(keep_count) {
                self.delete_backup(&backup.path).await?;
                deleted += 1;
            }
        }

        Ok(deleted)
    }
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub metadata: BackupMetadata,
    pub file_size: u64,
}
