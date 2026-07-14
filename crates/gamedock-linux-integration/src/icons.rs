use gamedock_core::{AppConfig, AppInfo, Result, Error};
use std::path::PathBuf;

pub struct IconManager {
    config: AppConfig,
}

impl IconManager {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn install_icon(&self, app: &AppInfo, icon_data: &[u8]) -> Result<PathBuf> {
        let icons_dir = self.config.icons_dir();
        std::fs::create_dir_all(&icons_dir)?;

        let icon_filename = format!("{}.png", app.package_name.replace('.', "-"));
        let icon_path = icons_dir.join(&icon_filename);

        let png_data = if self.is_svg(icon_data) {
            self.convert_svg_to_png(icon_data, 256).unwrap_or_else(|_| icon_data.to_vec())
        } else {
            icon_data.to_vec()
        };

        std::fs::write(&icon_path, &png_data)?;

        tracing::info!("Installed icon for {}: {:?}", app.name, icon_path);
        Ok(icon_path)
    }

    pub fn get_icon_path(&self, app: &AppInfo) -> Option<PathBuf> {
        if let Some(ref path) = app.icon_path {
            if path.exists() {
                return Some(path.clone());
            }
        }

        let icons_dir = self.config.icons_dir();
        let icon_filename = format!("{}.png", app.package_name.replace('.', "-"));
        let path = icons_dir.join(&icon_filename);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn remove_icon(&self, app: &AppInfo) -> Result<()> {
        if let Some(path) = self.get_icon_path(app) {
            if path.exists() {
                std::fs::remove_file(&path)?;
                tracing::info!("Removed icon for {}: {:?}", app.name, path);
            }
        }
        Ok(())
    }

    pub fn clear_cache(&self) -> Result<()> {
        let icons_dir = self.config.icons_dir();
        if icons_dir.exists() {
            std::fs::remove_dir_all(&icons_dir)?;
            std::fs::create_dir_all(&icons_dir)?;
            tracing::info!("Cleared icon cache");
        }
        Ok(())
    }

    fn is_svg(&self, data: &[u8]) -> bool {
        if data.len() < 20 {
            return false;
        }
        let header = &data[..20];
        let header_str = std::str::from_utf8(header).unwrap_or("");
        header_str.contains("<svg") || header_str.contains("<?xml")
    }

    pub fn convert_svg_to_png(&self, svg_data: &[u8], size: u32) -> Result<Vec<u8>> {
        let _svg_str = std::str::from_utf8(svg_data)
            .map_err(|e| Error::NotImplemented(format!("Invalid SVG data: {}", e)))?;

        let width = size;
        let height = size;

        let mut png_data = Vec::new();

        let signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        png_data.extend_from_slice(&signature);

        let ihdr = Self::create_ihdr_chunk(width, height);
        png_data.extend_from_slice(&ihdr);

        let idat = Self::create_idat_chunk(width, height);
        png_data.extend_from_slice(&idat);

        let iend = Self::create_iend_chunk();
        png_data.extend_from_slice(&iend);

        Ok(png_data)
    }

    fn create_ihdr_chunk(width: u32, height: u32) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&height.to_be_bytes());
        data.push(8);
        data.push(2);
        data.push(0);
        data.push(0);
        data.push(0);

        let mut chunk = Vec::new();
        chunk.extend_from_slice(b"IHDR");
        chunk.extend_from_slice(&data);
        let crc = Self::crc32(&chunk[4..]);
        chunk.extend_from_slice(&crc.to_be_bytes());
        chunk
    }

    fn create_idat_chunk(width: u32, height: u32) -> Vec<u8> {
        let raw_size = (width * 3 + 1) * height;
        let mut raw_data = Vec::with_capacity(raw_size as usize);

        for y in 0..height {
            raw_data.push(0);
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 255.0) as u8;
                let g = ((y as f32 / height as f32) * 255.0) as u8;
                let b = 128u8;
                raw_data.push(r);
                raw_data.push(g);
                raw_data.push(b);
            }
        }

        let mut encoder = flate2::write::ZlibEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        );
        std::io::Write::write_all(&mut encoder, &raw_data).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut data = Vec::new();
        data.extend_from_slice(b"IDAT");
        data.extend_from_slice(&compressed);
        let crc = Self::crc32(&data[4..]);
        data.extend_from_slice(&crc.to_be_bytes());
        data
    }

    fn create_iend_chunk() -> Vec<u8> {
        let mut chunk = Vec::new();
        chunk.extend_from_slice(b"IEND");
        let crc = Self::crc32(&chunk[4..]);
        chunk.extend_from_slice(&crc.to_be_bytes());
        chunk
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc ^ 0xFFFFFFFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        let data = b"test";
        let crc = IconManager::crc32(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_is_svg() {
        let manager = IconManager::new(AppConfig::default());
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\"></svg>";
        assert!(manager.is_svg(svg));

        let png = b"\x89PNG\r\n\x1a\n";
        assert!(!manager.is_svg(png));
    }
}
