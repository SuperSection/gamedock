use gamedock_core::{Error, PackageFormat, PackageInfo, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub struct XapkInstaller;

impl XapkInstaller {
    pub fn parse(path: &Path) -> Result<PackageInfo> {
        if !path.exists() {
            return Err(Error::Installation(format!("XAPK not found: {:?}", path)));
        }

        let metadata = std::fs::metadata(path)?;
        let mut info = PackageInfo::from_path(path)?;
        info.format = PackageFormat::Xapk;
        info.file_size = metadata.len();

        Ok(info)
    }

    pub fn extract_manifest(path: &Path) -> Result<XapkManifest> {
        let bytes = std::fs::read(path)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
            .map_err(|e| Error::Zip(format!("Invalid XAPK: {}", e)))?;

        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| Error::Zip(format!("{}", e)))?;
            let name = entry.name().to_string();
            let size = entry.size();
            if name == "manifest.json" || name.ends_with(".json") {
                let mut content = String::new();
                let mut reader = std::io::Read::take(entry, size);
                std::io::Read::read_to_string(&mut reader, &mut content)?;
                let manifest: XapkManifest = serde_json::from_str(&content)?;
                return Ok(manifest);
            }
        }

        Err(Error::Installation("No manifest found in XAPK".into()))
    }

    pub fn list_split_apks(path: &Path) -> Result<Vec<PathBuf>> {
        let bytes = std::fs::read(path)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes))
            .map_err(|e| Error::Zip(format!("Invalid XAPK: {}", e)))?;

        let mut apks = Vec::new();
        let temp_dir = tempfile::tempdir()?;

        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| Error::Zip(format!("{}", e)))?;
            let name = entry.name().to_string();
            let size = entry.size();
            if name.ends_with(".apk") {
                let out_path = temp_dir.path().join(&name);
                let mut out_file = std::fs::File::create(&out_path)?;
                std::io::copy(&mut std::io::Read::take(entry, size), &mut out_file)?;
                apks.push(out_path);
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct XapkManifest {
    pub name: String,
    pub package: String,
    pub version_code: i64,
    pub version_name: String,
    #[serde(default)]
    pub split_apks: Vec<SplitApk>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SplitApk {
    pub file: String,
    pub name: Option<String>,
}
