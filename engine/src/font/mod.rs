use rusttype::Font;
use std::collections::HashMap;

pub struct FontManager {
    fonts: HashMap<String, Font<'static>>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// Load a system font by family name with optional bold/italic variants
    pub fn load_system_font(&mut self, family: &str) -> Option<&Font<'static>> {
        self.load_font_variant(family, false, false)
    }

    pub fn load_font_variant(&mut self, family: &str, bold: bool, italic: bool) -> Option<&Font<'static>> {
        let key = format!("{}-{}-{}", family, bold, italic);
        
        if self.fonts.contains_key(&key) {
            return self.fonts.get(&key);
        }

        // Handle font-family lists (e.g., "system-ui,sans-serif")
        let families: Vec<&str> = family.split(',').map(|f| f.trim()).collect();
        
        for font_family in families {
            if let Some(font_data) = self.get_system_font_bytes_variant(font_family, bold, italic) {
                let font_bytes: &'static [u8] = Box::leak(font_data.into_boxed_slice());
                if let Some(font) = Font::try_from_bytes(font_bytes) {
                    self.fonts.insert(key.clone(), font);
                    return self.fonts.get(&key);
                }
            }
        }
        
        None
    }

    /// Get font bytes from system directories
    fn get_system_font_bytes_variant(&self, family: &str, bold: bool, italic: bool) -> Option<Vec<u8>> {
        #[cfg(target_os = "windows")]
        {
            return self.load_windows_font_variant(family, bold, italic);
        }

        #[cfg(target_os = "macos")]
        {
            return self.load_macos_font(family);
        }

        #[cfg(target_os = "linux")]
        {
            return self.load_linux_font(family);
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            let _ = (family, bold, italic);
            None
        }
    }

    #[cfg(target_os = "windows")]
    fn load_windows_font_variant(&self, family: &str, bold: bool, italic: bool) -> Option<Vec<u8>> {
        use std::env;
        use std::fs;

        let fonts_dir = env::var("WINDIR")
            .ok()
            .map(|wd| format!("{}\\Fonts", wd))?;

        let filename = match (family.to_lowercase().as_str(), bold, italic) {
            ("times new roman" | "times", false, false) => "times.ttf",
            ("times new roman" | "times", true, false) => "timesbd.ttf",
            ("times new roman" | "times", false, true) => "timesi.ttf",
            ("times new roman" | "times", true, true) => "timesbi.ttf",
            ("arial" | "system-ui" | "sans-serif", false, false) => "arial.ttf",
            ("arial" | "system-ui" | "sans-serif", true, false) => "arialbd.ttf",
            ("arial" | "system-ui" | "sans-serif", false, true) => "ariali.ttf",
            ("arial" | "system-ui" | "sans-serif", true, true) => "arialbi.ttf",
            ("georgia", false, false) => "georgia.ttf",
            ("georgia", true, false) => "georgiab.ttf",
            ("georgia", false, true) => "georgiai.ttf",
            ("georgia", true, true) => "georgiaz.ttf",
            ("verdana", false, false) => "verdana.ttf",
            ("verdana", true, false) => "verdanab.ttf",
            ("verdana", false, true) => "verdanai.ttf",
            ("verdana", true, true) => "verdanaz.ttf",
            ("courier new" | "courier" | "monospace", false, false) => "cour.ttf",
            ("courier new" | "courier" | "monospace", true, false) => "courbd.ttf",
            ("courier new" | "courier" | "monospace", false, true) => "couri.ttf",
            ("courier new" | "courier" | "monospace", true, true) => "courbi.ttf",
            _ => "arial.ttf",
        };

        let path = format!("{}\\{}", fonts_dir, filename);
        fs::read(&path).ok()
    }

    #[cfg(target_os = "macos")]
    fn load_macos_font(&self, family: &str) -> Option<Vec<u8>> {
        use std::fs;

        let home = std::env::var("HOME").ok()?;
        let fonts_paths = vec![
            format!("{}/Library/Fonts", home),
            "/Library/Fonts".to_string(),
            "/System/Library/Fonts".to_string(),
        ];

        let filename = match family.to_lowercase().as_str() {
            "times new roman" | "times" => "Times New Roman.ttf",
            "arial" => "Arial.ttf",
            "georgia" => "Georgia.ttf",
            "verdana" => "Verdana.ttf",
            "courier new" | "courier" => "Courier New.ttf",
            _ => return None,
        };

        for fonts_path in fonts_paths {
            let path = format!("{}/{}", fonts_path, filename);
            if let Ok(data) = fs::read(&path) {
                return Some(data);
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn load_linux_font(&self, family: &str) -> Option<Vec<u8>> {
        use std::fs;

        let home_fonts = format!("{}/.local/share/fonts", std::env::var("HOME").ok()?);
        let fonts_paths = vec![
            "/usr/share/fonts/truetype",
            "/usr/local/share/fonts/truetype",
            &home_fonts,
        ];

        // Build a list of font filenames to try, with fallbacks
        let filenames = match family.to_lowercase().as_str() {
            "times new roman" | "times" | "serif" => vec![
                "liberation/LiberationSerif-Regular.ttf",
                "dejavu/DejaVuSerif.ttf",
            ],
            "arial" | "sans-serif" | "system-ui" | "sans" => vec![
                "liberation/LiberationSans-Regular.ttf",
                "dejavu/DejaVuSans.ttf",
                "ubuntu/Ubuntu-Regular.ttf",
                "noto/NotoSans-Regular.ttf",
            ],
            "georgia" => vec![
                "liberation/LiberationSerif-Regular.ttf",
                "dejavu/DejaVuSerif.ttf",
            ],
            "verdana" => vec![
                "liberation/LiberationSans-Regular.ttf",
                "dejavu/DejaVuSans.ttf",
            ],
            "courier new" | "courier" | "monospace" => vec![
                "liberation/LiberationMono-Regular.ttf",
                "dejavu/DejaVuSansMono.ttf",
            ],
            // Default fallback for unknown families
            _ => vec![
                "liberation/LiberationSans-Regular.ttf",
                "dejavu/DejaVuSans.ttf",
            ],
        };

        for fonts_path in fonts_paths {
            for filename in &filenames {
                let path = format!("{}/{}", fonts_path, filename);
                if let Ok(data) = fs::read(&path) {
                    return Some(data);
                }
            }
        }
        None
    }

    /// Measure the width of a text string using actual font metrics
    pub fn measure_text(&mut self, text: &str, font_family: &str, font_size: f32, bold: bool, italic: bool) -> f32 {
        if let Some(font) = self.load_font_variant(font_family, bold, italic) {
            let scale = rusttype::Scale::uniform(font_size);
            let mut width = 0.0_f32;
            for c in text.chars() {
                let glyph = font.glyph(c).scaled(scale);
                width += glyph.h_metrics().advance_width;
            }
            width
        } else {
            // Fallback to estimate if font not available
            text.len() as f32 * font_size * 0.5
        }
    }
}
