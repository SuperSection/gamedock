use gamedock_core::{Error, PackageFormat, PackageInfo, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct ApkInstaller;

impl ApkInstaller {
    pub fn parse(path: &Path) -> Result<PackageInfo> {
        if !path.exists() {
            return Err(Error::Installation(format!("APK not found: {:?}", path)));
        }

        let metadata = std::fs::metadata(path)?;
        let mut info = PackageInfo::from_path(path)?;
        info.format = PackageFormat::Apk;
        info.file_size = metadata.len();

        Ok(info)
    }

    pub fn compute_hash(path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(hex::encode(hasher.finalize()))
    }

    pub fn verify_signature(path: &Path) -> Result<bool> {
        let bytes = std::fs::read(path)?;
        let zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
            .map_err(|e| Error::Zip(format!("Invalid APK: {}", e)))?;

        for name in zip.file_names() {
            if name.starts_with("META-INF/") && name.ends_with(".RSA") {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
