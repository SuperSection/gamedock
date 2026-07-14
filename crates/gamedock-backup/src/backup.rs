use gamedock_core::{AppConfig, AppInfo, Result, Error};
use gamedock_plugin_sdk::RuntimePlugin;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub app_id: String,
    pub app_name: String,
    pub package_name: String,
    pub version_name: String,
    pub version_code: i64,
    pub created_at: DateTime<Utc>,
    pub file_size: u64,
    pub sha256: String,
    pub includes_data: bool,
    pub includes_apk: bool,
}

pub struct BackupBuilder {
    config: AppConfig,
}

impl BackupBuilder {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub async fn create_backup(
        &self,
        app: &AppInfo,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        include_data: bool,
    ) -> Result<BackupResult> {
        let backup_id = uuid::Uuid::new_v4().to_string();
        let backup_dir = self.config.backups_dir().join(&backup_id);
        tokio::fs::create_dir_all(&backup_dir).await?;

        tracing::info!("Creating backup {} for app: {}", backup_id, app.name);

        let apk_path = if let Some(ref apk) = app.apk_path {
            if apk.exists() {
                let dest = backup_dir.join("base.apk");
                tokio::fs::copy(apk, &dest).await?;
                Some(dest)
            } else {
                None
            }
        } else {
            None
        };

        let _data_path = if include_data {
            let data_dir = backup_dir.join("data");
            tokio::fs::create_dir_all(&data_dir).await?;
            self.backup_app_data(runtime_manager, &app.runtime_id, &app.package_name, &data_dir).await?;
            Some(data_dir)
        } else {
            None
        };

        let metadata = BackupMetadata {
            id: backup_id.clone(),
            app_id: app.id.clone(),
            app_name: app.name.clone(),
            package_name: app.package_name.clone(),
            version_name: app.version_name.clone(),
            version_code: app.version_code,
            created_at: Utc::now(),
            file_size: 0,
            sha256: String::new(),
            includes_data: include_data,
            includes_apk: apk_path.is_some(),
        };

        let metadata_path = backup_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        tokio::fs::write(&metadata_path, &metadata_json).await?;

        let archive_path = self.create_archive(&backup_dir, &backup_id).await?;

        let hash = self.compute_file_hash(&archive_path).await?;
        let file_size = tokio::fs::metadata(&archive_path).await?.len();

        let mut final_metadata = metadata;
        final_metadata.sha256 = hash;
        final_metadata.file_size = file_size;

        let final_metadata_json = serde_json::to_string_pretty(&final_metadata)?;
        tokio::fs::write(&metadata_path, &final_metadata_json).await?;

        tokio::fs::remove_dir_all(&backup_dir).await.ok();

        tracing::info!("Backup created: {:?} ({} bytes)", archive_path, file_size);

        Ok(BackupResult {
            backup_id,
            archive_path,
            metadata: final_metadata,
        })
    }

    async fn backup_app_data(
        &self,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        runtime_id: &str,
        package_name: &str,
        dest_dir: &Path,
    ) -> Result<()> {
        let runtime = runtime_manager.get_runtime(runtime_id).await?;

        let data_dirs = [
            format!("/data/data/{}", package_name),
            format!("/sdcard/Android/data/{}", package_name),
        ];

        for remote_dir in &data_dirs {
            let local_dir = dest_dir.join(remote_dir.trim_start_matches('/'));
            if let Some(parent) = local_dir.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            let _ = runtime.pull_file(remote_dir, &local_dir).await;
        }

        Ok(())
    }

    async fn create_archive(&self, source_dir: &Path, backup_id: &str) -> Result<PathBuf> {
        let archive_path = self.config.backups_dir().join(format!("{}.zip", backup_id));

        let zip_file = std::fs::File::create(&archive_path)?;
        let mut zip = zip::ZipWriter::new(zip_file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        self.add_dir_to_zip(&mut zip, source_dir, source_dir, &options)?;

        zip.finish().map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(archive_path)
    }

    fn add_dir_to_zip(
        &self,
        zip: &mut zip::ZipWriter<std::fs::File>,
        base: &Path,
        current: &Path,
        options: &zip::write::SimpleFileOptions,
    ) -> Result<()> {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path.strip_prefix(base).unwrap_or(&path);

            if path.is_dir() {
                zip.add_directory(relative.to_string_lossy(), options.clone())
                    .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                self.add_dir_to_zip(zip, base, &path, options)?;
            } else {
                zip.start_file(relative.to_string_lossy(), options.clone())
                    .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                let mut file = std::fs::File::open(&path)?;
                std::io::copy(&mut file, zip)?;
            }
        }
        Ok(())
    }

    async fn compute_file_hash(&self, path: &Path) -> Result<String> {
        let bytes = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}

#[derive(Debug, Clone)]
pub struct BackupResult {
    pub backup_id: String,
    pub archive_path: PathBuf,
    pub metadata: BackupMetadata,
}
