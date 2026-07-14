use gamedock_core::{AppInfo, Result};
use serde::{Deserialize, Serialize};

pub struct MetadataFetcher {
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayStoreMetadata {
    pub title: String,
    pub developer: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    pub rating: Option<f32>,
}

impl MetadataFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_icon(&self, url: &str, save_path: &std::path::Path) -> Result<()> {
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(save_path, &bytes).await?;
        Ok(())
    }

    pub fn enrich_app_info(&self, app: &mut AppInfo, metadata: &PlayStoreMetadata) {
        if app.description.is_empty() {
            app.description = metadata.description.clone();
        }
        if app.author.is_empty() {
            app.author = metadata.developer.clone();
        }
        if app.rating.is_none() {
            app.rating = metadata.rating;
        }
    }
}

impl Default for MetadataFetcher {
    fn default() -> Self {
        Self::new()
    }
}
