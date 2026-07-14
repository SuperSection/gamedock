use crate::apk::ApkInstaller;
use crate::apkm::ApkmInstaller;
use crate::apks::ApksInstaller;
use crate::xapk::XapkInstaller;
use crate::download::Downloader;
use gamedock_core::{AppConfig, AppInfo, Event, EventBus, PackageFormat, PackageInfo, Result, Error};
use std::path::Path;

pub struct PackageInstaller {
    config: AppConfig,
    event_bus: EventBus,
}

impl PackageInstaller {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Self {
        Self { config, event_bus }
    }

    pub fn parse_package(&self, path: &Path) -> Result<PackageInfo> {
        let format = PackageFormat::from_path(path)
            .ok_or_else(|| Error::Installation(format!("Unsupported package format: {:?}", path)))?;

        match format {
            PackageFormat::Apk => ApkInstaller::parse(path),
            PackageFormat::Xapk => XapkInstaller::parse(path),
            PackageFormat::Apks => ApksInstaller::parse(path),
            PackageFormat::Apkm => ApkmInstaller::parse(path),
        }
    }

    pub async fn install_from_file(
        &self,
        path: &Path,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        runtime_id: &str,
    ) -> Result<AppInfo> {
        let package_info = self.parse_package(path)?;
        tracing::info!(
            "Installing {} package: {:?} ({} bytes)",
            package_info.format.display_name(),
            path,
            package_info.file_size
        );

        self.event_bus.publish(Event::Progress {
            operation: format!("Installing {}", path.file_name().unwrap_or_default().to_string_lossy()),
            current: 0,
            total: package_info.file_size,
        });

        match package_info.format {
            PackageFormat::Apk => {
                runtime_manager.install_app(runtime_id, path).await?;
            }
            PackageFormat::Xapk => {
                let apks = XapkInstaller::list_split_apks(path)?;
                for apk_path in &apks {
                    runtime_manager.install_app(runtime_id, apk_path).await?;
                }
            }
            PackageFormat::Apks => {
                let temp_dir = tempfile::tempdir()?;
                let bytes = std::fs::read(path)?;
                let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
                    .map_err(|e| Error::Zip(format!("Invalid APKS: {}", e)))?;
                for i in 0..archive.len() {
                    let entry = archive.by_index(i)
                        .map_err(|e| Error::Zip(format!("{}", e)))?;
                    let name = entry.name().to_string();
                    let size = entry.size();
                    if name.ends_with(".apk") {
                        let out_path = temp_dir.path().join(&name);
                        let mut out_file = std::fs::File::create(&out_path)?;
                        std::io::copy(&mut std::io::Read::take(entry, size), &mut out_file)?;
                        runtime_manager.install_app(runtime_id, &out_path).await?;
                    }
                }
            }
            PackageFormat::Apkm => {
                let temp_dir = tempfile::tempdir()?;
                let bytes = std::fs::read(path)?;
                let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
                    .map_err(|e| Error::Zip(format!("Invalid APKM: {}", e)))?;
                for i in 0..archive.len() {
                    let entry = archive.by_index(i)
                        .map_err(|e| Error::Zip(format!("{}", e)))?;
                    let name = entry.name().to_string();
                    let size = entry.size();
                    if name.ends_with(".apk") {
                        let out_path = temp_dir.path().join(&name);
                        let mut out_file = std::fs::File::create(&out_path)?;
                        std::io::copy(&mut std::io::Read::take(entry, size), &mut out_file)?;
                        runtime_manager.install_app(runtime_id, &out_path).await?;
                    }
                }
            }
        }

        let mut app_info = AppInfo::new(
            package_info.package_name.as_deref().unwrap_or("unknown"),
            package_info.app_name.as_deref().unwrap_or("Unknown App"),
            package_info.version_name.as_deref().unwrap_or("unknown"),
            package_info.version_code.unwrap_or(0),
        );
        app_info.status = gamedock_core::AppStatus::Installed;
        app_info.apk_path = Some(path.to_path_buf());
        app_info.size_bytes = Some(package_info.file_size);

        self.event_bus.publish(Event::AppInstalled {
            app_id: app_info.id.clone(),
            package_name: app_info.package_name.clone(),
        });

        tracing::info!("Successfully installed: {}", app_info.name);
        Ok(app_info)
    }

    pub async fn download_and_install(
        &self,
        url: &str,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        runtime_id: &str,
    ) -> Result<AppInfo> {
        let downloader = Downloader::new(self.config.clone(), self.event_bus.clone());
        let path = downloader.download(url).await?;
        self.install_from_file(&path, runtime_manager, runtime_id).await
    }

    pub async fn uninstall(
        &self,
        package_name: &str,
        runtime_manager: &gamedock_runtime_manager::RuntimeManager,
        runtime_id: &str,
    ) -> Result<()> {
        runtime_manager.uninstall_app(runtime_id, package_name).await?;
        self.event_bus.publish(Event::AppUninstalled {
            app_id: package_name.to_string(),
        });
        Ok(())
    }
}
