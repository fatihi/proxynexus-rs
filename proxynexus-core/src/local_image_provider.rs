use crate::ImageProvider;
use std::path::PathBuf;

pub struct LocalImageProvider {
    base_path: PathBuf,
}

impl LocalImageProvider {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl ImageProvider for LocalImageProvider {
    async fn get_image_bytes(&self, key: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let full_path = self.base_path.join(key);
        let bytes = std::fs::read(full_path)?;
        Ok(bytes)
    }
}
