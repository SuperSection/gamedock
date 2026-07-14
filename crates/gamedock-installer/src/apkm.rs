use gamedock_core::{PackageInfo, PackageFormat, Result, Error};
use std::path::Path;
use sha2::{Digest, Sha256};

pub struct ApkmInstaller;

impl ApkmInstaller {
    pub fn parse(path: &Path) -> Result<PackageInfo> {
        if !path.exists() {
            return Err(Error::Installation(format!("APKM not found: {:?}", path)));
        }

        let metadata = std::fs::metadata(path)?;
        let mut info = PackageInfo::from_path(path)?;
        info.format = PackageFormat::Apkm;
        info.file_size = metadata.len();

        Ok(info)
    }

    pub fn extract_manifest(path: &Path) -> Result<ApkmManifest> {
        let bytes = std::fs::read(path)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
            .map_err(|e| Error::Zip(format!("Invalid APKM: {}", e)))?;

        for i in 0..archive.len() {
            let entry = archive.by_index(i)
                .map_err(|e| Error::Zip(format!("{}", e)))?;
            let name = entry.name().to_string();
            let size = entry.size();
            if name == "manifest.json" {
                let mut content = String::new();
                let mut reader = std::io::Read::take(entry, size);
                std::io::Read::read_to_string(&mut reader, &mut content)?;
                return Ok(serde_json::from_str(&content)?);
            }
        }

        Err(Error::Installation("No manifest found in APKM".into()))
    }

    pub fn compute_hash(path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApkmManifest {
    pub name: String,
    pub package: String,
    pub version_code: i64,
    pub version_name: String,
    #[serde(default)]
    pub splits: Vec<ApkmSplit>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApkmSplit {
    pub name: String,
    pub file: String,
    pub size: u64,
}
