pub mod url;
pub mod cache;
pub mod image;
pub mod rewriter;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use std::thread;

pub use url::{resolve_url, resolve_url_with_base, parse_srcset, select_srcset_image, SrcsetEntry, ParsedUrl, is_data_uri, parse_data_uri};
pub use cache::{AssetCache, CacheHeaders, CacheLookup, CacheEntry};
pub use image::{ImageType, detect_image_type, decode_image, ImageDecodeError};
pub use rewriter::{HtmlRewriter, RewriterConfig, ProcessedImage};

/// Configuration for the NetworkManager
#[derive(Clone)]
pub struct NetworkConfig {
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum concurrent downloads
    pub max_concurrent: usize,
    /// Maximum size for inline data URIs (bytes)
    pub max_inline_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            max_redirects: 5,
            max_retries: 3,
            initial_backoff_ms: 100,
            max_concurrent: 6,
            max_inline_size: 32 * 1024, // 32KB
        }
    }
}

/// Represents a fetched resource
#[derive(Debug, Clone)]
pub struct FetchedResource {
    pub url: String,
    pub data: Vec<u8>,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub from_cache: bool,
}

/// Network manager with caching, retry logic, and concurrency control
pub struct NetworkManager {
    /// Legacy image cache (RgbaImage) for backward compatibility
    image_cache: Mutex<HashMap<String, ::image::RgbaImage>>,
    /// Asset cache for raw bytes with HTTP cache headers
    asset_cache: AssetCache,
    /// Configuration
    config: NetworkConfig,
    /// Semaphore for concurrency limiting
    concurrent_count: Mutex<usize>,
    /// Base URL of the current document
    document_url: Mutex<Option<String>>,
    /// Base href from <base> tag
    base_href: Mutex<Option<String>>,
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManager {
    pub fn new() -> Self {
        Self::with_config(NetworkConfig::default())
    }
    
    pub fn with_config(config: NetworkConfig) -> Self {
        Self {
            image_cache: Mutex::new(HashMap::new()),
            asset_cache: AssetCache::new(),
            config,
            concurrent_count: Mutex::new(0),
            document_url: Mutex::new(None),
            base_href: Mutex::new(None),
        }
    }
    
    /// Set the document URL for resolving relative URLs
    pub fn set_document_url(&self, url: &str) {
        let mut doc_url = self.document_url.lock().unwrap();
        *doc_url = Some(url.to_string());
    }
    
    /// Set the base href from <base> tag
    pub fn set_base_href(&self, href: &str) {
        let mut base = self.base_href.lock().unwrap();
        *base = Some(href.to_string());
    }
    
    /// Resolve a potentially relative URL against the document base
    pub fn resolve_url(&self, relative_url: &str) -> String {
        let doc_url = self.document_url.lock().unwrap();
        let base_href = self.base_href.lock().unwrap();
        
        match (&*doc_url, &*base_href) {
            (Some(doc), Some(base)) => resolve_url_with_base(doc, Some(base), relative_url),
            (Some(doc), None) => resolve_url(doc, relative_url),
            _ => relative_url.to_string(),
        }
    }
    
    /// Fetch an image and return as RgbaImage (legacy API for backward compatibility)
    pub fn fetch_image(&self, url: &str) -> Option<::image::RgbaImage> {
        // Check legacy cache first
        {
            let cache = self.image_cache.lock().unwrap();
            if let Some(img) = cache.get(url) {
                eprintln!("Cache hit for image: {}", url);
                return Some(img.clone());
            }
        }
        
        eprintln!("Fetching image: {}", url);
        
        // Resolve relative URL
        let resolved_url = self.resolve_url(url);
        
        // Fetch the resource
        let resource = self.fetch_resource(&resolved_url)?;
        
        // Detect image type and decode
        let image_type = detect_image_type(Some(&resource.content_type), &resource.data);
        
        match decode_image(&resource.data, image_type, None, None) {
            Ok(img) => {
                // Cache the decoded image
                let mut cache = self.image_cache.lock().unwrap();
                cache.insert(url.to_string(), img.clone());
                Some(img)
            }
            Err(e) => {
                eprintln!("Failed to decode image {}: {}", url, e);
                None
            }
        }
    }
    
    /// Fetch a resource with caching, retries, and redirect handling
    pub fn fetch_resource(&self, url: &str) -> Option<FetchedResource> {
        // Handle data URIs
        if url::is_data_uri(url) {
            return self.handle_data_uri(url);
        }
        
        // Check asset cache
        match self.asset_cache.lookup(url) {
            CacheLookup::Hit(entry) => {
                eprintln!("Asset cache hit for: {}", url);
                return Some(FetchedResource {
                    url: url.to_string(),
                    data: entry.data,
                    content_type: entry.content_type,
                    headers: Vec::new(),
                    from_cache: true,
                });
            }
            CacheLookup::Stale { etag, last_modified } => {
                // Try conditional request
                if let Some(resource) = self.fetch_with_validation(url, etag, last_modified) {
                    return Some(resource);
                }
                // If validation fails, fall through to regular fetch
            }
            CacheLookup::Miss => {}
        }
        
        // Regular fetch with retries
        self.fetch_with_retries(url)
    }
    
    /// Handle data URI
    fn handle_data_uri(&self, uri: &str) -> Option<FetchedResource> {
        let (content_type, data) = url::parse_data_uri(uri)?;
        Some(FetchedResource {
            url: uri.to_string(),
            data,
            content_type,
            headers: Vec::new(),
            from_cache: false,
        })
    }
    
    /// Fetch with conditional validation (If-None-Match / If-Modified-Since)
    fn fetch_with_validation(
        &self,
        url: &str,
        etag: Option<String>,
        last_modified: Option<String>,
    ) -> Option<FetchedResource> {
        self.wait_for_slot();
        
        let result = self.do_conditional_fetch(url, etag.as_deref(), last_modified.as_deref());
        
        self.release_slot();
        
        match result {
            Ok((is_modified, resource)) => {
                if !is_modified {
                    // 304 Not Modified - refresh cache and return cached data
                    self.asset_cache.refresh(url);
                    if let CacheLookup::Hit(entry) = self.asset_cache.lookup(url) {
                        return Some(FetchedResource {
                            url: url.to_string(),
                            data: entry.data,
                            content_type: entry.content_type,
                            headers: Vec::new(),
                            from_cache: true,
                        });
                    }
                }
                resource
            }
            Err(_) => None,
        }
    }
    
    /// Fetch with retry logic
    fn fetch_with_retries(&self, url: &str) -> Option<FetchedResource> {
        let mut attempts = 0;
        let mut backoff = self.config.initial_backoff_ms;
        
        while attempts < self.config.max_retries {
            self.wait_for_slot();
            
            let result = self.do_fetch(url);
            
            self.release_slot();
            
            match result {
                Ok(resource) => {
                    // Cache the resource
                    let headers = cache::extract_cache_headers(&resource.headers);
                    self.asset_cache.store(
                        url,
                        resource.data.clone(),
                        resource.content_type.clone(),
                        headers,
                    );
                    return Some(resource);
                }
                Err(e) => {
                    attempts += 1;
                    eprintln!("Fetch attempt {} failed for {}: {}", attempts, url, e);
                    
                    if attempts < self.config.max_retries {
                        thread::sleep(Duration::from_millis(backoff));
                        backoff *= 2; // Exponential backoff
                    }
                }
            }
        }
        
        None
    }
    
    /// Perform the actual HTTP fetch
    fn do_fetch(&self, url: &str) -> Result<FetchedResource, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(self.config.max_redirects as usize))
            .build()?;
        
        let response = client.get(url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }
        
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        
        let final_url = response.url().to_string();
        let bytes = response.bytes()?.to_vec();
        
        Ok(FetchedResource {
            url: final_url,
            data: bytes,
            content_type,
            headers,
            from_cache: false,
        })
    }
    
    /// Perform conditional fetch
    fn do_conditional_fetch(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<(bool, Option<FetchedResource>), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(self.config.max_redirects as usize))
            .build()?;
        
        let mut request = client.get(url);
        
        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        if let Some(lm) = last_modified {
            request = request.header("If-Modified-Since", lm);
        }
        
        let response = request.send()?;
        
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok((false, None));
        }
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }
        
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        
        let final_url = response.url().to_string();
        let bytes = response.bytes()?.to_vec();
        
        // Cache the new resource
        let cache_headers = cache::extract_cache_headers(&headers);
        self.asset_cache.store(url, bytes.clone(), content_type.clone(), cache_headers);
        
        Ok((true, Some(FetchedResource {
            url: final_url,
            data: bytes,
            content_type,
            headers,
            from_cache: false,
        })))
    }
    
    /// Wait for a concurrency slot
    fn wait_for_slot(&self) {
        loop {
            {
                let mut count = self.concurrent_count.lock().unwrap();
                if *count < self.config.max_concurrent {
                    *count += 1;
                    return;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    /// Release a concurrency slot
    fn release_slot(&self) {
        let mut count = self.concurrent_count.lock().unwrap();
        if *count > 0 {
            *count -= 1;
        }
    }
    
    /// Clear all caches
    pub fn clear_cache(&self) {
        let mut image_cache = self.image_cache.lock().unwrap();
        image_cache.clear();
        self.asset_cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> cache::CacheStats {
        self.asset_cache.stats()
    }
    
    /// Fetch multiple resources in parallel
    pub fn fetch_resources(&self, urls: &[String]) -> Vec<Option<FetchedResource>> {
        urls.iter()
            .map(|url| self.fetch_resource(url))
            .collect()
    }
}
