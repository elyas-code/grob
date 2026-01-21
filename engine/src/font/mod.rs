use rusttype::Font;
use std::collections::HashMap;

pub struct FontManager {
    fonts: HashMap<String, Font<'static>>,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// Load a system font by family name (e.g., "Times New Roman", "Arial", "DejaVuSans")
    pub fn load_system_font(&mut self, family: &str) -> Option<&Font<'static>> {
        if self.fonts.contains_key(family) {
            return self.fonts.get(family);
        }

        // Try to load from system fonts directories
        let font_data = self.get_system_font_bytes(family)?;

        // We need to store the data somewhere with 'static lifetime
        // For now, we'll use Box::leak which is acceptable for a small number of fonts
        let font_bytes: &'static [u8] = Box::leak(font_data.into_boxed_slice());
        let font = Font::try_from_bytes(font_bytes)?;

        self.fonts.insert(family.to_string(), font);
        self.fonts.get(family)
    }

    /// Get font bytes from system directories
    fn get_system_font_bytes(&self, family: &str) -> Option<Vec<u8>> {
        #[cfg(target_os = "windows")]
        {
            return self.load_windows_font(family);
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
            None
        }
    }

    #[cfg(target_os = "windows")]
    fn load_windows_font(&self, family: &str) -> Option<Vec<u8>> {
        use std::env;
        use std::fs;

        let fonts_dir = env::var("WINDIR")
            .ok()
            .map(|wd| format!("{}\\Fonts", wd))?;

        // Map family names to common Windows font filenames
        let filename = match family.to_lowercase().as_str() {
            "times new roman" | "times" => "times.ttf",
            "arial" => "arial.ttf",
            "georgia" => "georgia.ttf",
            "verdana" => "verdana.ttf",
            "courier new" | "courier" => "cour.ttf",
            "comic sans ms" | "comic sans" => "comic.ttf",
            _ => return None,
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
}
