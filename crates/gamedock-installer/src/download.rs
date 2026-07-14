use gamedock_core::{AppConfig, Event, EventBus, Result};
use std::path::PathBuf;

pub struct Downloader {
    config: AppConfig,
    event_bus: EventBus,
}

impl Downloader {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Self {
        Self { config, event_bus }
    }

    pub async fn download(&self, url: &str) -> Result<PathBuf> {
        tracing::info!("Downloading: {}", url);

        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        let total_size = response.content_length().unwrap_or(0);

        let filename = url
            .rsplit('/')
            .next()
            .unwrap_or("download")
            .to_string();

        let download_dir = self.config.cache_dir.join("downloads");
        std::fs::create_dir_all(&download_dir)?;
        let path = download_dir.join(&filename);

        let mut downloaded: u64 = 0;
        let mut file = std::fs::File::create(&path)?;

        let mut response = response;
        while let Some(chunk) = response.chunk().await? {
            std::io::Write::write_all(&mut file, &chunk)?;
            downloaded += chunk.len() as u64;

            self.event_bus.publish(Event::Progress {
                operation: format!("Downloading {}", filename),
                current: downloaded,
                total: total_size,
            });
        }

        tracing::info!("Download complete: {:?}", path);
        Ok(path)
    }

    pub async fn download_to_cache(&self, url: &str, filename: &str) -> Result<PathBuf> {
        let cache_dir = self.config.cache_dir.join("downloads");
        std::fs::create_dir_all(&cache_dir)?;
        let path = cache_dir.join(filename);

        if path.exists() {
            tracing::info!("Using cached download: {:?}", path);
            return Ok(path);
        }

        self.download(url).await
    }
}
