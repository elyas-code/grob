// Asset caching module for HTTP resources
//
// This module provides:
// - URL-keyed caching with ETag/Last-Modified support
// - Conditional requests (If-None-Match, If-Modified-Since)
// - 304 Not Modified handling
// - Cache eviction (LRU-based with size limits)

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Maximum cache size in bytes (50 MB default)
const DEFAULT_MAX_CACHE_SIZE: usize = 50 * 1024 * 1024;

/// Maximum age for cached entries (24 hours default)
const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Represents HTTP cache validation headers
#[derive(Debug, Clone, Default)]
pub struct CacheHeaders {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub cache_control: Option<String>,
    pub expires: Option<String>,
}

impl CacheHeaders {
    /// Check if the cache headers indicate the resource should not be cached
    pub fn is_no_cache(&self) -> bool {
        if let Some(ref cc) = self.cache_control {
            let cc = cc.to_lowercase();
            cc.contains("no-cache") || cc.contains("no-store")
        } else {
            false
        }
    }
    
    /// Parse max-age from Cache-Control header
    pub fn max_age(&self) -> Option<Duration> {
        if let Some(ref cc) = self.cache_control {
            for part in cc.split(',') {
                let part = part.trim().to_lowercase();
                if part.starts_with("max-age=") {
                    if let Ok(secs) = part.trim_start_matches("max-age=").parse::<u64>() {
                        return Some(Duration::from_secs(secs));
                    }
                }
            }
        }
        None
    }
}

/// A cached entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached data
    pub data: Vec<u8>,
    /// Content-Type of the resource
    pub content_type: String,
    /// ETag for conditional requests
    pub etag: Option<String>,
    /// Last-Modified for conditional requests
    pub last_modified: Option<String>,
    /// When this entry was cached
    pub cached_at: Instant,
    /// How long this entry is valid
    pub max_age: Duration,
    /// Last time this entry was accessed (for LRU)
    pub last_accessed: Instant,
}

impl CacheEntry {
    /// Check if this entry is still fresh
    pub fn is_fresh(&self) -> bool {
        self.cached_at.elapsed() < self.max_age
    }
    
    /// Get the size of this entry in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// Result of a cache lookup
#[derive(Debug)]
pub enum CacheLookup {
    /// Cache hit - return the cached data
    Hit(CacheEntry),
    /// Cache miss - need to fetch
    Miss,
    /// Stale entry exists - need to revalidate
    Stale {
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

/// Result of a conditional request
#[derive(Debug)]
pub enum ConditionalResult {
    /// Server returned 304 Not Modified
    NotModified,
    /// Server returned new content
    Modified {
        data: Vec<u8>,
        content_type: String,
        headers: CacheHeaders,
    },
}

/// HTTP asset cache
pub struct AssetCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    max_size: usize,
    max_age: Duration,
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetCache {
    /// Create a new cache with default settings
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_size: DEFAULT_MAX_CACHE_SIZE,
            max_age: DEFAULT_MAX_AGE,
        }
    }
    
    /// Create a cache with custom settings
    pub fn with_config(max_size: usize, max_age: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_size,
            max_age,
        }
    }
    
    /// Look up an entry in the cache
    pub fn lookup(&self, url: &str) -> CacheLookup {
        let mut entries = self.entries.write().unwrap();
        
        if let Some(entry) = entries.get_mut(url) {
            entry.touch();
            
            if entry.is_fresh() {
                CacheLookup::Hit(entry.clone())
            } else {
                CacheLookup::Stale {
                    etag: entry.etag.clone(),
                    last_modified: entry.last_modified.clone(),
                }
            }
        } else {
            CacheLookup::Miss
        }
    }
    
    /// Store an entry in the cache
    pub fn store(&self, url: &str, data: Vec<u8>, content_type: String, headers: CacheHeaders) {
        if headers.is_no_cache() {
            return;
        }
        
        let max_age = headers.max_age().unwrap_or(self.max_age);
        let now = Instant::now();
        
        let entry = CacheEntry {
            data,
            content_type,
            etag: headers.etag,
            last_modified: headers.last_modified,
            cached_at: now,
            max_age,
            last_accessed: now,
        };
        
        let entry_size = entry.size();
        
        let mut entries = self.entries.write().unwrap();
        
        // Evict entries if we're over the size limit
        self.evict_if_needed(&mut entries, entry_size);
        
        entries.insert(url.to_string(), entry);
    }
    
    /// Update an entry after receiving a 304 Not Modified
    pub fn refresh(&self, url: &str) {
        let mut entries = self.entries.write().unwrap();
        
        if let Some(entry) = entries.get_mut(url) {
            entry.cached_at = Instant::now();
            entry.touch();
        }
    }
    
    /// Remove an entry from the cache
    pub fn remove(&self, url: &str) {
        let mut entries = self.entries.write().unwrap();
        entries.remove(url);
    }
    
    /// Clear the entire cache
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().unwrap();
        let total_size: usize = entries.values().map(|e| e.size()).sum();
        let fresh_count = entries.values().filter(|e| e.is_fresh()).count();
        
        CacheStats {
            entry_count: entries.len(),
            fresh_count,
            total_size,
            max_size: self.max_size,
        }
    }
    
    /// Evict entries using LRU policy if needed
    fn evict_if_needed(&self, entries: &mut HashMap<String, CacheEntry>, needed_space: usize) {
        let current_size: usize = entries.values().map(|e| e.size()).sum();
        
        if current_size + needed_space <= self.max_size {
            return;
        }
        
        // First, remove stale entries
        let stale_urls: Vec<String> = entries
            .iter()
            .filter(|(_, e)| !e.is_fresh())
            .map(|(url, _)| url.clone())
            .collect();
        
        for url in stale_urls {
            entries.remove(&url);
        }
        
        let current_size: usize = entries.values().map(|e| e.size()).sum();
        if current_size + needed_space <= self.max_size {
            return;
        }
        
        // Then, remove LRU entries until we have enough space
        loop {
            let current_size: usize = entries.values().map(|e| e.size()).sum();
            if current_size + needed_space <= self.max_size || entries.is_empty() {
                break;
            }
            
            // Find the LRU entry
            let lru_url = entries
                .iter()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(url, _)| url.clone());
            
            if let Some(url) = lru_url {
                entries.remove(&url);
            } else {
                break;
            }
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub entry_count: usize,
    pub fresh_count: usize,
    pub total_size: usize,
    pub max_size: usize,
}

impl CacheStats {
    pub fn utilization(&self) -> f32 {
        if self.max_size == 0 {
            0.0
        } else {
            self.total_size as f32 / self.max_size as f32
        }
    }
}

/// Build conditional request headers from cache entry
pub fn build_conditional_headers(etag: Option<&str>, last_modified: Option<&str>) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    
    if let Some(etag) = etag {
        headers.push(("If-None-Match".to_string(), etag.to_string()));
    }
    
    if let Some(lm) = last_modified {
        headers.push(("If-Modified-Since".to_string(), lm.to_string()));
    }
    
    headers
}

/// Extract cache-relevant headers from a response
pub fn extract_cache_headers(headers: &[(String, String)]) -> CacheHeaders {
    let mut result = CacheHeaders::default();
    
    for (name, value) in headers {
        match name.to_lowercase().as_str() {
            "etag" => result.etag = Some(value.clone()),
            "last-modified" => result.last_modified = Some(value.clone()),
            "cache-control" => result.cache_control = Some(value.clone()),
            "expires" => result.expires = Some(value.clone()),
            _ => {}
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_store_and_lookup() {
        let cache = AssetCache::new();
        
        let data = b"test image data".to_vec();
        let headers = CacheHeaders {
            etag: Some("\"abc123\"".to_string()),
            ..Default::default()
        };
        
        cache.store("https://example.com/image.png", data.clone(), "image/png".to_string(), headers);
        
        match cache.lookup("https://example.com/image.png") {
            CacheLookup::Hit(entry) => {
                assert_eq!(entry.data, data);
                assert_eq!(entry.content_type, "image/png");
                assert_eq!(entry.etag, Some("\"abc123\"".to_string()));
            }
            _ => panic!("Expected cache hit"),
        }
    }
    
    #[test]
    fn test_cache_miss() {
        let cache = AssetCache::new();
        
        match cache.lookup("https://example.com/nonexistent.png") {
            CacheLookup::Miss => {}
            _ => panic!("Expected cache miss"),
        }
    }
    
    #[test]
    fn test_cache_no_store() {
        let cache = AssetCache::new();
        
        let headers = CacheHeaders {
            cache_control: Some("no-store".to_string()),
            ..Default::default()
        };
        
        cache.store("https://example.com/private.png", vec![1, 2, 3], "image/png".to_string(), headers);
        
        match cache.lookup("https://example.com/private.png") {
            CacheLookup::Miss => {}
            _ => panic!("Expected cache miss for no-store"),
        }
    }
    
    #[test]
    fn test_cache_headers_max_age() {
        let headers = CacheHeaders {
            cache_control: Some("max-age=3600, public".to_string()),
            ..Default::default()
        };
        
        assert_eq!(headers.max_age(), Some(Duration::from_secs(3600)));
    }
    
    #[test]
    fn test_build_conditional_headers() {
        let headers = build_conditional_headers(Some("\"abc\""), Some("Sat, 01 Jan 2000 00:00:00 GMT"));
        
        assert_eq!(headers.len(), 2);
        assert!(headers.iter().any(|(k, v)| k == "If-None-Match" && v == "\"abc\""));
        assert!(headers.iter().any(|(k, v)| k == "If-Modified-Since" && v == "Sat, 01 Jan 2000 00:00:00 GMT"));
    }
}
