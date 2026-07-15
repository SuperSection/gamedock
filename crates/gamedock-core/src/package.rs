use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageFormat {
    Apk,
    Xapk,
    Apks,
    Apkm,
}

impl PackageFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext.to_lowercase().as_str() {
            "apk" => Some(Self::Apk),
            "xapk" => Some(Self::Xapk),
            "apks" => Some(Self::Apks),
            "apkm" => Some(Self::Apkm),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Apk => "apk",
            Self::Xapk => "xapk",
            Self::Apks => "apks",
            Self::Apkm => "apkm",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Apk => "APK",
            Self::Xapk => "XAPK",
            Self::Apks => "APKS",
            Self::Apkm => "APKM",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub file_path: PathBuf,
    pub format: PackageFormat,
    pub file_size: u64,
    pub sha256: Option<String>,
    pub package_name: Option<String>,
    pub app_name: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<i64>,
}

impl PackageInfo {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let format = PackageFormat::from_path(path)
            .ok_or_else(|| anyhow::anyhow!("Unsupported package format"))?;
        let metadata = std::fs::metadata(path)?;
        Ok(Self {
            file_path: path.to_path_buf(),
            format,
            file_size: metadata.len(),
            sha256: None,
            package_name: None,
            app_name: None,
            version_name: None,
            version_code: None,
        })
    }

    pub fn compute_sha256(&mut self) -> anyhow::Result<()> {
        use sha2::{Digest, Sha256};
        let bytes = std::fs::read(&self.file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        self.sha256 = Some(hex::encode(result));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_package_format_from_path() {
        assert_eq!(
            PackageFormat::from_path(Path::new("game.apk")),
            Some(PackageFormat::Apk)
        );
        assert_eq!(
            PackageFormat::from_path(Path::new("game.xapk")),
            Some(PackageFormat::Xapk)
        );
        assert_eq!(
            PackageFormat::from_path(Path::new("game.apks")),
            Some(PackageFormat::Apks)
        );
        assert_eq!(
            PackageFormat::from_path(Path::new("game.apkm")),
            Some(PackageFormat::Apkm)
        );
        assert_eq!(PackageFormat::from_path(Path::new("game.zip")), None);
        assert_eq!(PackageFormat::from_path(Path::new("game.tar.gz")), None);
    }

    #[test]
    fn test_package_format_extension() {
        assert_eq!(PackageFormat::Apk.extension(), "apk");
        assert_eq!(PackageFormat::Xapk.extension(), "xapk");
        assert_eq!(PackageFormat::Apks.extension(), "apks");
        assert_eq!(PackageFormat::Apkm.extension(), "apkm");
    }

    #[test]
    fn test_package_format_display_name() {
        assert_eq!(PackageFormat::Apk.display_name(), "APK");
        assert_eq!(PackageFormat::Xapk.display_name(), "XAPK");
        assert_eq!(PackageFormat::Apks.display_name(), "APKS");
        assert_eq!(PackageFormat::Apkm.display_name(), "APKM");
    }

    #[test]
    fn test_package_format_case_insensitive() {
        assert_eq!(
            PackageFormat::from_path(Path::new("GAME.APK")),
            Some(PackageFormat::Apk)
        );
        assert_eq!(
            PackageFormat::from_path(Path::new("game.Xapk")),
            Some(PackageFormat::Xapk)
        );
    }

    #[test]
    fn test_package_format_serialization() {
        let formats = vec![
            PackageFormat::Apk,
            PackageFormat::Xapk,
            PackageFormat::Apks,
            PackageFormat::Apkm,
        ];
        for format in formats {
            let json = serde_json::to_string(&format).unwrap();
            let deserialized: PackageFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(format, deserialized);
        }
    }
}
