use std::collections::HashMap;
use std::sync::Mutex;

pub struct NetworkManager {
    cache: Mutex<HashMap<String, image::RgbaImage>>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn fetch_image(&self, url: &str) -> Option<image::RgbaImage> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(img) = cache.get(url) {
                eprintln!("Cache hit for image: {}", url);
                return Some(img.clone());
            }
        }

        eprintln!("Fetching image: {}", url);

        // Try to fetch from network
        if let Ok(img) = self.download_image(url) {
            // Cache it
            {
                let mut cache = self.cache.lock().unwrap();
                cache.insert(url.to_string(), img.clone());
            }
            return Some(img);
        }

        None
    }

    fn download_image(&self, url: &str) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
        // Use a blocking reqwest client
        let client = reqwest::blocking::Client::new();
        let response = client.get(url).timeout(std::time::Duration::from_secs(5)).send()?;
        let bytes = response.bytes()?;
        
        // Try to decode as image
        let img = image::load_from_memory(&bytes)?;
        Ok(img.to_rgba8())
    }
}
