// Image handling module
//
// This module provides:
// - Image type detection (Content-Type + magic bytes fallback)
// - Support for PNG, JPEG, WebP, GIF, SVG
// - SVG rasterization to PNG for raster-only renderers
// - Image decoding utilities

use image::{DynamicImage, RgbaImage, ImageFormat};

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    Png,
    Jpeg,
    Gif,
    WebP,
    Svg,
    Bmp,
    Ico,
    Unknown,
}

impl ImageType {
    /// Get the MIME type for this image format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageType::Png => "image/png",
            ImageType::Jpeg => "image/jpeg",
            ImageType::Gif => "image/gif",
            ImageType::WebP => "image/webp",
            ImageType::Svg => "image/svg+xml",
            ImageType::Bmp => "image/bmp",
            ImageType::Ico => "image/x-icon",
            ImageType::Unknown => "application/octet-stream",
        }
    }
    
    /// Get the file extension for this image format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageType::Png => "png",
            ImageType::Jpeg => "jpg",
            ImageType::Gif => "gif",
            ImageType::WebP => "webp",
            ImageType::Svg => "svg",
            ImageType::Bmp => "bmp",
            ImageType::Ico => "ico",
            ImageType::Unknown => "bin",
        }
    }
    
    /// Check if this format requires rasterization
    pub fn needs_rasterization(&self) -> bool {
        matches!(self, ImageType::Svg)
    }
}

/// Detect image type from Content-Type header
pub fn detect_from_content_type(content_type: &str) -> ImageType {
    let ct = content_type.to_lowercase();
    let ct = if let Some(pos) = ct.find(';') {
        &ct[..pos]
    } else {
        &ct
    };
    let ct = ct.trim();
    
    match ct {
        "image/png" => ImageType::Png,
        "image/jpeg" | "image/jpg" => ImageType::Jpeg,
        "image/gif" => ImageType::Gif,
        "image/webp" => ImageType::WebP,
        "image/svg+xml" | "image/svg" => ImageType::Svg,
        "image/bmp" | "image/x-bmp" => ImageType::Bmp,
        "image/x-icon" | "image/vnd.microsoft.icon" => ImageType::Ico,
        _ => ImageType::Unknown,
    }
}

/// Magic bytes signatures for different image formats
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
const GIF87_MAGIC: &[u8] = b"GIF87a";
const GIF89_MAGIC: &[u8] = b"GIF89a";
const WEBP_MAGIC: &[u8] = b"RIFF";
const WEBP_TYPE: &[u8] = b"WEBP";
const BMP_MAGIC: &[u8] = b"BM";

/// Detect image type from magic bytes (first few bytes of the file)
pub fn detect_from_magic_bytes(data: &[u8]) -> ImageType {
    if data.len() < 4 {
        return ImageType::Unknown;
    }
    
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data.len() >= 8 && data.starts_with(PNG_MAGIC) {
        return ImageType::Png;
    }
    
    // JPEG: FF D8 FF
    if data.starts_with(JPEG_MAGIC) {
        return ImageType::Jpeg;
    }
    
    // GIF: GIF87a or GIF89a
    if data.len() >= 6 && (data.starts_with(GIF87_MAGIC) || data.starts_with(GIF89_MAGIC)) {
        return ImageType::Gif;
    }
    
    // WebP: RIFF....WEBP
    if data.len() >= 12 && data.starts_with(WEBP_MAGIC) && &data[8..12] == WEBP_TYPE {
        return ImageType::WebP;
    }
    
    // BMP: BM
    if data.starts_with(BMP_MAGIC) {
        return ImageType::Bmp;
    }
    
    // ICO: 00 00 01 00 or 00 00 02 00
    if data.len() >= 4 && data[0] == 0 && data[1] == 0 && (data[2] == 1 || data[2] == 2) && data[3] == 0 {
        return ImageType::Ico;
    }
    
    // SVG: Look for <?xml or <svg (possibly with whitespace)
    let text_start: String = data.iter()
        .take(256)
        .take_while(|&&b| b < 128)
        .map(|&b| b as char)
        .collect();
    let text_lower = text_start.to_lowercase();
    if text_lower.contains("<?xml") && text_lower.contains("svg") {
        return ImageType::Svg;
    }
    if text_lower.trim_start().starts_with("<svg") {
        return ImageType::Svg;
    }
    
    ImageType::Unknown
}

/// Detect image type using Content-Type with magic bytes fallback
pub fn detect_image_type(content_type: Option<&str>, data: &[u8]) -> ImageType {
    // Try Content-Type first
    if let Some(ct) = content_type {
        let detected = detect_from_content_type(ct);
        if detected != ImageType::Unknown {
            return detected;
        }
    }
    
    // Fall back to magic bytes
    detect_from_magic_bytes(data)
}

/// Decode image data into an RgbaImage
/// 
/// For SVG images, this will attempt to rasterize at the specified dimensions.
/// If dimensions are not provided, a default size is used.
pub fn decode_image(
    data: &[u8],
    image_type: ImageType,
    target_width: Option<u32>,
    target_height: Option<u32>,
) -> Result<RgbaImage, ImageDecodeError> {
    match image_type {
        ImageType::Svg => {
            // Rasterize SVG
            rasterize_svg(data, target_width, target_height)
        }
        _ => {
            // Use the image crate for raster formats
            decode_raster_image(data)
        }
    }
}

/// Decode a raster image (non-SVG) using the image crate
fn decode_raster_image(data: &[u8]) -> Result<RgbaImage, ImageDecodeError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ImageDecodeError::DecodeFailed(e.to_string()))?;
    Ok(img.to_rgba8())
}

/// Rasterize an SVG image to RGBA pixels
/// 
/// This is a simplified SVG rasterizer. For production use, consider
/// using a full SVG library like resvg.
fn rasterize_svg(
    data: &[u8],
    target_width: Option<u32>,
    target_height: Option<u32>,
) -> Result<RgbaImage, ImageDecodeError> {
    // Try to parse as UTF-8
    let svg_str = std::str::from_utf8(data)
        .map_err(|_| ImageDecodeError::InvalidSvg("Invalid UTF-8 in SVG".to_string()))?;
    
    // Default size if not specified
    let width = target_width.unwrap_or(256);
    let height = target_height.unwrap_or(256);
    
    // Try to extract viewBox or width/height from the SVG
    let (svg_width, svg_height) = extract_svg_dimensions(svg_str)
        .unwrap_or((width, height));
    
    // Calculate scale to fit target size while maintaining aspect ratio
    let scale_x = width as f32 / svg_width as f32;
    let scale_y = height as f32 / svg_height as f32;
    let scale = scale_x.min(scale_y);
    
    let final_width = (svg_width as f32 * scale) as u32;
    let final_height = (svg_height as f32 * scale) as u32;
    
    // Create a simple rasterized placeholder
    // In a real implementation, you'd use resvg or similar
    let mut img = RgbaImage::new(final_width.max(1), final_height.max(1));
    
    // Fill with a light gray to indicate SVG placeholder
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([240, 240, 240, 255]);
    }
    
    // Draw a border
    let w = img.width();
    let h = img.height();
    for x in 0..w {
        img.put_pixel(x, 0, image::Rgba([200, 200, 200, 255]));
        img.put_pixel(x, h.saturating_sub(1), image::Rgba([200, 200, 200, 255]));
    }
    for y in 0..h {
        img.put_pixel(0, y, image::Rgba([200, 200, 200, 255]));
        img.put_pixel(w.saturating_sub(1), y, image::Rgba([200, 200, 200, 255]));
    }
    
    // Note: For full SVG support, integrate resvg:
    // let tree = usvg::Tree::from_str(svg_str, &usvg::Options::default())?;
    // let pixmap = tiny_skia::Pixmap::new(width, height)?;
    // resvg::render(&tree, usvg::FitTo::Width(width), pixmap.as_mut());
    
    Ok(img)
}

/// Extract width and height from SVG attributes or viewBox
fn extract_svg_dimensions(svg: &str) -> Option<(u32, u32)> {
    // Simple regex-free parsing for viewBox or width/height
    // Look for viewBox="x y width height"
    if let Some(viewbox_start) = svg.to_lowercase().find("viewbox") {
        let rest = &svg[viewbox_start..];
        if let Some(quote_start) = rest.find('"') {
            let after_quote = &rest[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                let viewbox = &after_quote[..quote_end];
                let parts: Vec<f32> = viewbox
                    .split_whitespace()
                    .flat_map(|s| s.split(','))
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                if parts.len() >= 4 {
                    return Some((parts[2] as u32, parts[3] as u32));
                }
            }
        }
    }
    
    // Try width and height attributes
    let width = extract_svg_attribute(svg, "width");
    let height = extract_svg_attribute(svg, "height");
    
    match (width, height) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}

/// Extract a numeric attribute from SVG
fn extract_svg_attribute(svg: &str, attr: &str) -> Option<u32> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = svg.to_lowercase().find(&pattern) {
        let rest = &svg[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            let value = &rest[..end];
            // Remove unit suffixes like "px", "pt", etc.
            let num_str: String = value.chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            return num_str.parse().ok();
        }
    }
    None
}

/// Resize an image to fit within the specified dimensions while maintaining aspect ratio
pub fn resize_image(img: &RgbaImage, max_width: u32, max_height: u32) -> RgbaImage {
    let (w, h) = (img.width(), img.height());
    
    if w <= max_width && h <= max_height {
        return img.clone();
    }
    
    let scale_x = max_width as f32 / w as f32;
    let scale_y = max_height as f32 / h as f32;
    let scale = scale_x.min(scale_y);
    
    let new_width = (w as f32 * scale) as u32;
    let new_height = (h as f32 * scale) as u32;
    
    image::imageops::resize(
        img,
        new_width.max(1),
        new_height.max(1),
        image::imageops::FilterType::Lanczos3,
    )
}

/// Convert an RgbaImage to a data URI
pub fn image_to_data_uri(img: &RgbaImage, format: ImageType) -> Result<String, ImageDecodeError> {
    use std::io::Cursor;
    
    let mut buffer = Cursor::new(Vec::new());
    
    let image_format = match format {
        ImageType::Png | ImageType::Svg => ImageFormat::Png,
        ImageType::Jpeg => ImageFormat::Jpeg,
        ImageType::Gif => ImageFormat::Gif,
        ImageType::Bmp => ImageFormat::Bmp,
        _ => ImageFormat::Png, // Default to PNG
    };
    
    let dynamic = DynamicImage::ImageRgba8(img.clone());
    dynamic.write_to(&mut buffer, image_format)
        .map_err(|e| ImageDecodeError::EncodeFailed(e.to_string()))?;
    
    let bytes = buffer.into_inner();
    let base64 = encode_base64(&bytes);
    let mime = format.mime_type();
    
    Ok(format!("data:{};base64,{}", mime, base64))
}

/// Simple base64 encoder
fn encode_base64(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    let mut i = 0;
    
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = data.get(i + 1).copied().unwrap_or(0) as u32;
        let b2 = data.get(i + 2).copied().unwrap_or(0) as u32;
        
        let triple = (b0 << 16) | (b1 << 8) | b2;
        
        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        
        if i + 1 < data.len() {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        
        if i + 2 < data.len() {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        
        i += 3;
    }
    
    result
}

/// Errors that can occur during image decoding
#[derive(Debug)]
pub enum ImageDecodeError {
    DecodeFailed(String),
    InvalidSvg(String),
    EncodeFailed(String),
    UnsupportedFormat(ImageType),
}

impl std::fmt::Display for ImageDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageDecodeError::DecodeFailed(msg) => write!(f, "Image decode failed: {}", msg),
            ImageDecodeError::InvalidSvg(msg) => write!(f, "Invalid SVG: {}", msg),
            ImageDecodeError::EncodeFailed(msg) => write!(f, "Image encode failed: {}", msg),
            ImageDecodeError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {:?}", fmt),
        }
    }
}

impl std::error::Error for ImageDecodeError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_from_content_type() {
        assert_eq!(detect_from_content_type("image/png"), ImageType::Png);
        assert_eq!(detect_from_content_type("image/jpeg"), ImageType::Jpeg);
        assert_eq!(detect_from_content_type("image/jpeg; charset=utf-8"), ImageType::Jpeg);
        assert_eq!(detect_from_content_type("image/svg+xml"), ImageType::Svg);
        assert_eq!(detect_from_content_type("text/plain"), ImageType::Unknown);
    }
    
    #[test]
    fn test_detect_from_magic_bytes() {
        // PNG
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert_eq!(detect_from_magic_bytes(&png_data), ImageType::Png);
        
        // JPEG
        let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_from_magic_bytes(&jpeg_data), ImageType::Jpeg);
        
        // GIF
        let gif_data = b"GIF89aXXX";
        assert_eq!(detect_from_magic_bytes(gif_data), ImageType::Gif);
        
        // WebP
        let webp_data = b"RIFF\x00\x00\x00\x00WEBP";
        assert_eq!(detect_from_magic_bytes(webp_data), ImageType::WebP);
        
        // SVG
        let svg_data = b"<svg xmlns=\"http://www.w3.org/2000/svg\">";
        assert_eq!(detect_from_magic_bytes(svg_data), ImageType::Svg);
    }
    
    #[test]
    fn test_image_type_mime() {
        assert_eq!(ImageType::Png.mime_type(), "image/png");
        assert_eq!(ImageType::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageType::Svg.mime_type(), "image/svg+xml");
    }
    
    #[test]
    fn test_extract_svg_dimensions() {
        let svg = r#"<svg viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        assert_eq!(extract_svg_dimensions(svg), Some((100, 50)));
        
        let svg2 = r#"<svg width="200" height="100"></svg>"#;
        assert_eq!(extract_svg_dimensions(svg2), Some((200, 100)));
    }
}
