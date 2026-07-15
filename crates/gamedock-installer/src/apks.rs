use gamedock_core::{Error, PackageFormat, PackageInfo, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct ApksInstaller;

impl ApksInstaller {
    pub fn parse(path: &Path) -> Result<PackageInfo> {
        if !path.exists() {
            return Err(Error::Installation(format!("APKS not found: {:?}", path)));
        }

        let metadata = std::fs::metadata(path)?;
        let mut info = PackageInfo::from_path(path)?;
        info.format = PackageFormat::Apks;
        info.file_size = metadata.len();

        Ok(info)
    }

    pub fn list_apks(path: &Path) -> Result<Vec<String>> {
        let bytes = std::fs::read(path)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
            .map_err(|e| Error::Zip(format!("Invalid APKS: {}", e)))?;

        let mut apks = Vec::new();
        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| Error::Zip(format!("{}", e)))?;
            if entry.name().ends_with(".apk") {
                apks.push(entry.name().to_string());
            }
        }

        Ok(apks)
    }

    pub fn compute_hash(path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}
