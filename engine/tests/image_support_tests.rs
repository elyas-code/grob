// Comprehensive tests for image support functionality
//
// Tests cover:
// - URL resolution (relative, absolute, with base href)
// - srcset parsing and selection
// - CSS url() parsing
// - Image type detection (Content-Type + magic bytes)
// - SVG dimension extraction
// - Caching behavior (store, lookup, 304)

#[cfg(test)]
mod url_resolution_tests {
    use grob_engine::net::url::{resolve_url, resolve_url_with_base, ParsedUrl};
    
    #[test]
    fn test_parse_simple_url() {
        let url = ParsedUrl::parse("https://example.com/path/to/file.html").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.path, "/path/to/file.html");
        assert_eq!(url.port, None);
    }
    
    #[test]
    fn test_parse_url_with_port() {
        let url = ParsedUrl::parse("http://localhost:8080/api/data").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api/data");
    }
    
    #[test]
    fn test_parse_url_with_query_and_fragment() {
        let url = ParsedUrl::parse("https://example.com/search?q=test&page=1#results").unwrap();
        assert_eq!(url.path, "/search");
        assert_eq!(url.query, Some("q=test&page=1".to_string()));
        assert_eq!(url.fragment, Some("results".to_string()));
    }
    
    #[test]
    fn test_resolve_relative_same_directory() {
        let result = resolve_url("https://example.com/path/page.html", "image.png");
        assert_eq!(result, "https://example.com/path/image.png");
    }
    
    #[test]
    fn test_resolve_relative_parent_directory() {
        let result = resolve_url("https://example.com/a/b/c.html", "../img.png");
        assert_eq!(result, "https://example.com/a/img.png");
    }
    
    #[test]
    fn test_resolve_relative_root() {
        let result = resolve_url("https://example.com/deep/nested/page.html", "/assets/image.png");
        assert_eq!(result, "https://example.com/assets/image.png");
    }
    
    #[test]
    fn test_resolve_protocol_relative() {
        let result = resolve_url("https://example.com/page.html", "//cdn.example.com/lib.js");
        assert_eq!(result, "https://cdn.example.com/lib.js");
    }
    
    #[test]
    fn test_resolve_absolute_url_unchanged() {
        let result = resolve_url("https://example.com/page.html", "https://other.com/image.png");
        assert_eq!(result, "https://other.com/image.png");
    }
    
    #[test]
    fn test_resolve_with_base_href() {
        let result = resolve_url_with_base(
            "https://example.com/app/index.html",
            Some("/static/"),
            "logo.png"
        );
        assert_eq!(result, "https://example.com/static/logo.png");
    }
    
    #[test]
    fn test_resolve_with_absolute_base_href() {
        let result = resolve_url_with_base(
            "https://example.com/app/index.html",
            Some("https://cdn.example.com/assets/"),
            "image.jpg"
        );
        assert_eq!(result, "https://cdn.example.com/assets/image.jpg");
    }
    
    #[test]
    fn test_resolve_without_base_href() {
        let result = resolve_url_with_base(
            "https://example.com/app/index.html",
            None,
            "local.png"
        );
        assert_eq!(result, "https://example.com/app/local.png");
    }
    
    #[test]
    fn test_data_uri_unchanged() {
        let data_uri = "data:image/png;base64,iVBORw0KGgo=";
        let result = resolve_url("https://example.com/page.html", data_uri);
        assert_eq!(result, data_uri);
    }
}

#[cfg(test)]
mod srcset_tests {
    use grob_engine::net::url::{parse_srcset, select_srcset_image, SrcsetEntry};
    
    #[test]
    fn test_parse_srcset_width_descriptors() {
        let srcset = "small.jpg 300w, medium.jpg 600w, large.jpg 1200w";
        let entries = parse_srcset(srcset);
        
        assert_eq!(entries.len(), 3);
        
        assert_eq!(entries[0].url, "small.jpg");
        assert_eq!(entries[0].width, Some(300));
        assert_eq!(entries[0].density, None);
        
        assert_eq!(entries[1].url, "medium.jpg");
        assert_eq!(entries[1].width, Some(600));
        
        assert_eq!(entries[2].url, "large.jpg");
        assert_eq!(entries[2].width, Some(1200));
    }
    
    #[test]
    fn test_parse_srcset_density_descriptors() {
        let srcset = "normal.jpg 1x, retina.jpg 2x, super-retina.jpg 3x";
        let entries = parse_srcset(srcset);
        
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].density, Some(1.0));
        assert_eq!(entries[1].density, Some(2.0));
        assert_eq!(entries[2].density, Some(3.0));
    }
    
    #[test]
    fn test_parse_srcset_mixed() {
        let srcset = "img-320.jpg 320w, img-640.jpg 640w";
        let entries = parse_srcset(srcset);
        assert_eq!(entries.len(), 2);
    }
    
    #[test]
    fn test_parse_srcset_with_spaces() {
        let srcset = "  image1.jpg 100w  ,  image2.jpg 200w  ";
        let entries = parse_srcset(srcset);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].url, "image1.jpg");
    }
    
    #[test]
    fn test_select_srcset_width_exact_match() {
        let entries = vec![
            SrcsetEntry { url: "small.jpg".to_string(), width: Some(400), density: None },
            SrcsetEntry { url: "medium.jpg".to_string(), width: Some(800), density: None },
            SrcsetEntry { url: "large.jpg".to_string(), width: Some(1200), density: None },
        ];
        
        // For 800px viewport at 1x, should select 800w exactly
        let result = select_srcset_image(&entries, None, 800, 1.0);
        assert_eq!(result, Some("medium.jpg".to_string()));
    }
    
    #[test]
    fn test_select_srcset_width_next_larger() {
        let entries = vec![
            SrcsetEntry { url: "small.jpg".to_string(), width: Some(400), density: None },
            SrcsetEntry { url: "medium.jpg".to_string(), width: Some(800), density: None },
            SrcsetEntry { url: "large.jpg".to_string(), width: Some(1200), density: None },
        ];
        
        // For 500px viewport at 1x, should select 800w (smallest >= 500)
        let result = select_srcset_image(&entries, None, 500, 1.0);
        assert_eq!(result, Some("medium.jpg".to_string()));
    }
    
    #[test]
    fn test_select_srcset_with_dpr() {
        let entries = vec![
            SrcsetEntry { url: "small.jpg".to_string(), width: Some(400), density: None },
            SrcsetEntry { url: "medium.jpg".to_string(), width: Some(800), density: None },
            SrcsetEntry { url: "large.jpg".to_string(), width: Some(1200), density: None },
        ];
        
        // For 400px viewport at 2x DPR, need 800 effective pixels
        let result = select_srcset_image(&entries, None, 400, 2.0);
        assert_eq!(result, Some("medium.jpg".to_string()));
    }
    
    #[test]
    fn test_select_srcset_density() {
        let entries = vec![
            SrcsetEntry { url: "1x.jpg".to_string(), width: None, density: Some(1.0) },
            SrcsetEntry { url: "2x.jpg".to_string(), width: None, density: Some(2.0) },
        ];
        
        let result = select_srcset_image(&entries, None, 400, 2.0);
        assert_eq!(result, Some("2x.jpg".to_string()));
    }
    
    #[test]
    fn test_select_srcset_fallback() {
        let entries = vec![];
        let result = select_srcset_image(&entries, Some("fallback.jpg"), 400, 1.0);
        assert_eq!(result, Some("fallback.jpg".to_string()));
    }
}

#[cfg(test)]
mod css_url_tests {
    use grob_engine::parser::html::parse_css_urls;
    
    #[test]
    fn test_parse_simple_url() {
        let css = "background-image: url(image.png)";
        let refs = parse_css_urls(css);
        
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "image.png");
    }
    
    #[test]
    fn test_parse_url_with_single_quotes() {
        let css = "background: url('assets/bg.jpg')";
        let refs = parse_css_urls(css);
        
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "assets/bg.jpg");
    }
    
    #[test]
    fn test_parse_url_with_double_quotes() {
        let css = r#"background-image: url("images/hero.png")"#;
        let refs = parse_css_urls(css);
        
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "images/hero.png");
    }
    
    #[test]
    fn test_parse_multiple_urls() {
        let css = "background-image: url(first.png); list-style-image: url(second.gif)";
        let refs = parse_css_urls(css);
        
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].url, "first.png");
        assert_eq!(refs[1].url, "second.gif");
    }
    
    #[test]
    fn test_skip_data_uri() {
        let css = "background: url(data:image/png;base64,abc123)";
        let refs = parse_css_urls(css);
        
        // Data URIs should be skipped (they don't need fetching)
        assert_eq!(refs.len(), 0);
    }
    
    #[test]
    fn test_property_detection() {
        let css = "background-image: url(bg.png)";
        let refs = parse_css_urls(css);
        
        assert_eq!(refs[0].property, "background-image");
    }
}

#[cfg(test)]
mod image_type_tests {
    use grob_engine::net::image::{
        detect_from_content_type, detect_from_magic_bytes, detect_image_type, ImageType
    };
    
    #[test]
    fn test_detect_png_content_type() {
        assert_eq!(detect_from_content_type("image/png"), ImageType::Png);
        assert_eq!(detect_from_content_type("IMAGE/PNG"), ImageType::Png);
        assert_eq!(detect_from_content_type("image/png; charset=utf-8"), ImageType::Png);
    }
    
    #[test]
    fn test_detect_jpeg_content_type() {
        assert_eq!(detect_from_content_type("image/jpeg"), ImageType::Jpeg);
        assert_eq!(detect_from_content_type("image/jpg"), ImageType::Jpeg);
    }
    
    #[test]
    fn test_detect_svg_content_type() {
        assert_eq!(detect_from_content_type("image/svg+xml"), ImageType::Svg);
    }
    
    #[test]
    fn test_detect_webp_content_type() {
        assert_eq!(detect_from_content_type("image/webp"), ImageType::WebP);
    }
    
    #[test]
    fn test_detect_png_magic_bytes() {
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(detect_from_magic_bytes(&png_header), ImageType::Png);
    }
    
    #[test]
    fn test_detect_jpeg_magic_bytes() {
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert_eq!(detect_from_magic_bytes(&jpeg_header), ImageType::Jpeg);
    }
    
    #[test]
    fn test_detect_gif_magic_bytes() {
        let gif89a = b"GIF89a\x00\x00\x00\x00";
        assert_eq!(detect_from_magic_bytes(gif89a), ImageType::Gif);
        
        let gif87a = b"GIF87a\x00\x00\x00\x00";
        assert_eq!(detect_from_magic_bytes(gif87a), ImageType::Gif);
    }
    
    #[test]
    fn test_detect_webp_magic_bytes() {
        let webp = b"RIFF\x00\x00\x00\x00WEBPVP8 ";
        assert_eq!(detect_from_magic_bytes(webp), ImageType::WebP);
    }
    
    #[test]
    fn test_detect_svg_magic_bytes() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\">";
        assert_eq!(detect_from_magic_bytes(svg), ImageType::Svg);
        
        let svg_with_xml = b"<?xml version=\"1.0\"?><svg>";
        assert_eq!(detect_from_magic_bytes(svg_with_xml), ImageType::Svg);
    }
    
    #[test]
    fn test_detect_with_content_type_priority() {
        // Even if magic bytes say PNG, Content-Type should be tried first
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let result = detect_image_type(Some("image/png"), &png_data);
        assert_eq!(result, ImageType::Png);
    }
    
    #[test]
    fn test_fallback_to_magic_bytes() {
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let result = detect_image_type(Some("application/octet-stream"), &png_data);
        assert_eq!(result, ImageType::Png);
    }
    
    #[test]
    fn test_image_type_mime_type() {
        assert_eq!(ImageType::Png.mime_type(), "image/png");
        assert_eq!(ImageType::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageType::Gif.mime_type(), "image/gif");
        assert_eq!(ImageType::WebP.mime_type(), "image/webp");
        assert_eq!(ImageType::Svg.mime_type(), "image/svg+xml");
    }
}

#[cfg(test)]
mod svg_tests {
    use grob_engine::net::image::{decode_image, ImageType};
    
    #[test]
    fn test_svg_rasterization_basic() {
        let svg = r#"<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" fill="red"/>
        </svg>"#;
        
        let result = decode_image(svg.as_bytes(), ImageType::Svg, Some(100), Some(100));
        assert!(result.is_ok());
        
        let img = result.unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }
    
    #[test]
    fn test_svg_with_viewbox() {
        let svg = r#"<svg viewBox="0 0 200 150" xmlns="http://www.w3.org/2000/svg">
            <circle cx="100" cy="75" r="50"/>
        </svg>"#;
        
        let result = decode_image(svg.as_bytes(), ImageType::Svg, None, None);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod cache_tests {
    use grob_engine::net::cache::{AssetCache, CacheHeaders, CacheLookup};
    use std::time::Duration;
    
    #[test]
    fn test_cache_store_and_hit() {
        let cache = AssetCache::new();
        
        let data = b"image data".to_vec();
        let headers = CacheHeaders::default();
        
        cache.store("https://example.com/img.png", data.clone(), "image/png".to_string(), headers);
        
        match cache.lookup("https://example.com/img.png") {
            CacheLookup::Hit(entry) => {
                assert_eq!(entry.data, data);
                assert_eq!(entry.content_type, "image/png");
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
            _ => panic!("Expected miss for no-store"),
        }
    }
    
    #[test]
    fn test_cache_etag() {
        let cache = AssetCache::new();
        
        let headers = CacheHeaders {
            etag: Some("\"abc123\"".to_string()),
            ..Default::default()
        };
        
        cache.store("https://example.com/img.png", vec![1, 2, 3], "image/png".to_string(), headers);
        
        match cache.lookup("https://example.com/img.png") {
            CacheLookup::Hit(entry) => {
                assert_eq!(entry.etag, Some("\"abc123\"".to_string()));
            }
            _ => panic!("Expected cache hit"),
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
    fn test_cache_refresh() {
        let cache = AssetCache::new();
        
        let headers = CacheHeaders::default();
        cache.store("https://example.com/img.png", vec![1, 2, 3], "image/png".to_string(), headers);
        
        // Refresh should update the cached_at time
        cache.refresh("https://example.com/img.png");
        
        match cache.lookup("https://example.com/img.png") {
            CacheLookup::Hit(_) => {}
            _ => panic!("Expected hit after refresh"),
        }
    }
    
    #[test]
    fn test_cache_stats() {
        let cache = AssetCache::new();
        
        let headers = CacheHeaders::default();
        cache.store("https://example.com/a.png", vec![1, 2, 3, 4, 5], "image/png".to_string(), headers.clone());
        cache.store("https://example.com/b.png", vec![1, 2, 3], "image/png".to_string(), headers);
        
        let stats = cache.stats();
        assert_eq!(stats.entry_count, 2);
        assert_eq!(stats.total_size, 8); // 5 + 3 bytes
    }
}

#[cfg(test)]
mod html_extraction_tests {
    use grob_engine::parser::html::tree_builder::HtmlParser;
    use grob_engine::parser::html::{extract_image_refs, extract_base_href, ImageRefType};
    
    #[test]
    fn test_extract_img_src() {
        let html = r#"<!DOCTYPE html><html><head></head><body><img src="photo.jpg" alt="A photo"></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        let refs = extract_image_refs(&dom);
        
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "photo.jpg");
        assert!(matches!(refs[0].ref_type, ImageRefType::ImgSrc));
    }
    
    #[test]
    fn test_extract_img_srcset() {
        let html = r#"<!DOCTYPE html><html><head></head><body><img srcset="small.jpg 300w, large.jpg 600w" src="fallback.jpg"></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        let refs = extract_image_refs(&dom);
        
        // Should have both src and srcset
        assert!(refs.len() >= 1);
        assert!(refs.iter().any(|r| matches!(r.ref_type, ImageRefType::Srcset { .. })));
    }
    
    #[test]
    fn test_extract_link_icon() {
        let html = r#"<!DOCTYPE html><html><head><link rel="icon" href="favicon.ico"></head><body></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        let refs = extract_image_refs(&dom);
        
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "favicon.ico");
        assert!(matches!(refs[0].ref_type, ImageRefType::Favicon));
    }
    
    #[test]
    fn test_extract_apple_touch_icon() {
        let html = r#"<!DOCTYPE html><html><head><link rel="apple-touch-icon" href="touch-icon.png"></head><body></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        let refs = extract_image_refs(&dom);
        
        assert_eq!(refs.len(), 1);
        assert!(matches!(refs[0].ref_type, ImageRefType::TouchIcon));
    }
    
    #[test]
    fn test_extract_base_href() {
        let html = r#"<!DOCTYPE html><html><head><base href="/assets/"></head><body></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        let base = extract_base_href(&dom);
        
        assert_eq!(base, Some("/assets/".to_string()));
    }
    
    #[test]
    fn test_no_base_href() {
        let html = r#"<!DOCTYPE html><html><head><title>Test</title></head><body></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        let base = extract_base_href(&dom);
        
        assert_eq!(base, None);
    }
    
    #[test]
    fn test_extract_style_background() {
        let html = r#"<!DOCTYPE html><html><head></head><body><div style="background-image: url(bg.png)">Content</div></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        let refs = extract_image_refs(&dom);
        
        assert!(refs.iter().any(|r| r.url == "bg.png"));
    }
}

#[cfg(test)]
mod data_uri_tests {
    use grob_engine::net::url::{is_data_uri, parse_data_uri};
    
    #[test]
    fn test_is_data_uri() {
        assert!(is_data_uri("data:image/png;base64,abc123"));
        assert!(is_data_uri("DATA:text/plain,hello"));
        assert!(is_data_uri("  data:image/gif;base64,R0lGODlh  "));
        assert!(!is_data_uri("https://example.com/image.png"));
        assert!(!is_data_uri("image.png"));
    }
    
    #[test]
    fn test_parse_data_uri_base64() {
        let uri = "data:image/png;base64,SGVsbG8=";
        let result = parse_data_uri(uri);
        
        assert!(result.is_some());
        let (content_type, data) = result.unwrap();
        assert_eq!(content_type, "image/png");
        assert_eq!(data, b"Hello");
    }
    
    #[test]
    fn test_parse_data_uri_text() {
        let uri = "data:text/plain,Hello%20World";
        let result = parse_data_uri(uri);
        
        assert!(result.is_some());
        let (content_type, data) = result.unwrap();
        assert_eq!(content_type, "text/plain");
        assert_eq!(String::from_utf8(data).unwrap(), "Hello World");
    }
}
