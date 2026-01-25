use winit::{
    event::{Event, WindowEvent, MouseButton, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use pixels::{Pixels, SurfaceTexture};
use rusttype::{Scale, point};

use engine::parser::html::tree_builder::HtmlParser;
use engine::style::{Stylesheet, Style, Selector, Viewport};
use engine::layout::LayoutEngine;
use engine::dom::{NodeType, Dom, NodeId};
use engine::font::FontManager;
use engine::net::NetworkManager;
use engine::net::url::resolve_url;
use std::sync::{Arc, Mutex};

use std::fs::OpenOptions;
use std::io::Write;

fn log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("grob_debug.log") {
        let _ = writeln!(file, "{}", msg);
    }
}

/// Round a dimension up to the nearest multiple of the scale factor.
/// This is required for Wayland which enforces that buffer sizes must be
/// integer multiples of the buffer_scale.
fn round_to_scale(value: u32, scale: f64) -> u32 {
    let scale_int = scale.ceil() as u32;
    if scale_int <= 1 {
        return value;
    }
    // Round up to nearest multiple of scale
    ((value + scale_int - 1) / scale_int) * scale_int
}

// Helper function to find an anchor element at the given coordinates in the layout tree
fn find_anchor_at_position(
    layout: &engine::layout::LayoutBox,
    dom: &Dom,
    x: f32,
    y: f32,
    scale_factor: f32,
) -> Option<String> {
    // Convert physical pixels to logical coordinates
    let logical_x = x / scale_factor;
    let logical_y = y / scale_factor;
    
    find_anchor_recursive(layout, dom, logical_x, logical_y)
}

fn find_anchor_recursive(
    layout: &engine::layout::LayoutBox,
    dom: &Dom,
    x: f32,
    y: f32,
) -> Option<String> {
    let dims = &layout.dimensions;

    // Check if point is within this box's bounds
    if x >= dims.x && x <= dims.x + dims.width &&
       y >= dims.y && y <= dims.y + dims.height {

        // Check if this element is an anchor tag
        if let NodeType::Element(elem) = &dom.nodes[layout.node_id].node_type {
            if elem.tag_name == "a" {
                // Extract href attribute
                if let Some(href) = elem.attributes.iter().find(|(k, _)| k == "href").map(|(_, v)| v.clone()) {
                    return Some(href);
                }
            }
        }

        // If this is a text node, check if any parent is an anchor
        if let NodeType::Text(_) = &dom.nodes[layout.node_id].node_type {
            // Walk up the DOM tree to find an anchor parent
            let mut current_node_id = layout.node_id;
            loop {
                if let Some(parent_id) = dom.nodes[current_node_id].parent {
                    if let NodeType::Element(elem) = &dom.nodes[parent_id].node_type {
                        if elem.tag_name == "a" {
                            // Found an anchor parent!
                            if let Some(href) = elem.attributes.iter().find(|(k, _)| k == "href").map(|(_, v)| v.clone()) {
                                return Some(href);
                            }
                        }
                    }
                    current_node_id = parent_id;
                } else {
                    break;
                }
            }
        }

        // Check layout children first
        for child in &layout.children {
            if let Some(href) = find_anchor_recursive(child, dom, x, y) {
                return Some(href);
            }
        }
        
        // If layout tree is incomplete, also search the DOM tree for anchors
        // This handles cases where layout engine doesn't create layout boxes for all elements
        for &child_id in &dom.nodes[layout.node_id].children {
            if let Some(href) = find_anchor_in_dom(dom, child_id, x, y) {
                return Some(href);
            }
        }
    }

    None
}

// Search through DOM for anchors, checking if text nodes are at the click position
fn find_anchor_in_dom(dom: &Dom, node_id: NodeId, x: f32, y: f32) -> Option<String> {
    // Check if this node or any parent is an anchor
    if let NodeType::Text(_) = &dom.nodes[node_id].node_type {
        // Walk up to find anchor parent
        let mut current_node_id = node_id;
        loop {
            if let Some(parent_id) = dom.nodes[current_node_id].parent {
                if let NodeType::Element(elem) = &dom.nodes[parent_id].node_type {
                    if elem.tag_name == "a" {
                        if let Some(href) = elem.attributes.iter().find(|(k, _)| k == "href").map(|(_, v)| v.clone()) {
                            return Some(href);
                        }
                    }
                }
                current_node_id = parent_id;
            } else {
                break;
            }
        }
    }
    
    // Recurse into children
    for &child_id in &dom.nodes[node_id].children {
        if let Some(href) = find_anchor_in_dom(dom, child_id, x, y) {
            return Some(href);
        }
    }
    
    None
}

// Helper function to load and parse a page given a URL
fn load_page(url: &str, network_manager: &NetworkManager) -> (Arc<Dom>, Stylesheet) {
    // Set the document URL for resolving relative URLs
    network_manager.set_document_url(url);
    
    // Fetch HTML from a web URL
    let html = match fetch_html(url) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to fetch HTML from {}: {}", url, e);
            eprintln!("Using fallback HTML");
            format!(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Error</title>
                </head>
                <body>
                    <h1>Failed to Load Page</h1>
                    <p>Could not fetch: {}</p>
                    <p>Error: {}</p>
                </body>
                </html>
                "#,
                url, e
            )
        }
    };

    let dom = HtmlParser::new(&html).parse();
    
    // Extract and set the <base href> if present
    if let Some(base_href) = engine::parser::html::extract_base_href(&dom) {
        log(&format!("Found <base href=\"{}\">", base_href));
        network_manager.set_base_href(&base_href);
    }
    
    let dom = Arc::new(dom);

    // Extract CSS from <style> tags in the DOM
    let css = extract_css_from_dom(&*dom, dom.root());
    
    log(&format!("Extracted CSS from <style> tags: {} bytes", css.len()));

    // --- CSS (parse and apply) ---
    let mut stylesheet = Stylesheet::new();
    
    // Parse CSS from style tags and convert to stylesheet rules
    if !css.is_empty() {
        log(&format!("CSS extracted: {} bytes", css.len()));
        let mut css_tokenizer = engine::parser::css::CssTokenizer::new(&css);
        let tokens = css_tokenizer.tokenize();
        let mut css_parser = engine::parser::css::CssParser::new(tokens);
        let css_items = css_parser.parse();
        log(&format!("=== Parsed {} CSS items from <style> tags ===", css_items.len()));
        
        // Convert CSS rules to stylesheet rules
        for item in css_items {
            if let engine::parser::css::parser::CssItem::Rule(rule) = item {
                let selector = convert_css_selector(&rule.selector);
                let mut style = Style::new();
                
                for decl in rule.declarations {
                    style.properties.insert(decl.property.clone(), decl.value.clone());
                }
                
                stylesheet.add_rule(selector, style);
            }
        }
        log(&format!("Stylesheet now has {} rules", stylesheet.rules.len()));
    }
    
    // Note: Default styles are now handled by the engine's apply_default_styles() method
    // in style::Stylesheet::compute_style(), so we don't need to add them here

    (dom, stylesheet)
}

fn main() {
    // Force X11 backend on Linux to avoid Wayland fractional scaling issues
    // This is a workaround for GNOME's scale-monitor-framebuffer feature
    // which causes buffer size validation errors with client-side decorations
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            std::env::remove_var("WAYLAND_DISPLAY");
            eprintln!("Note: Forcing X11 backend due to Wayland scaling compatibility issues");
        }
    }
    
    // Initial URL to load
    let initial_url = "https://info.cern.ch/";
    
    // --- Network Manager (created early so load_page can use it) ---
    let network_manager = Arc::new(NetworkManager::new());
    
    // Load initial page
    let (mut dom, mut stylesheet) = load_page(initial_url, &network_manager);
    let mut current_url = initial_url.to_string();

    // Extract title from DOM
    let page_title = extract_title(&dom);
    let window_title = format!("Grob Browser - {}", page_title);

    // --- Layout ---
    let mut layout_engine = LayoutEngine::new();
    
    // --- Font Manager ---
    let mut font_manager = FontManager::new();

    // State for navigation
    let pending_navigation = Arc::new(Mutex::new(Option::<String>::None));

    // --- Window ---
    let event_loop = EventLoop::new();
    
    // Use a logical size that will result in even physical dimensions at any scale factor
    // 800x600 logical -> 1600x1200 at scale 2, 800x600 at scale 1
    let initial_logical_size = winit::dpi::LogicalSize::new(800.0, 600.0);
    
    let window = WindowBuilder::new()
        .with_title(&window_title)
        .with_inner_size(initial_logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    let scale_factor = window.scale_factor() as f32;
    let physical_size = window.inner_size();
    
    // Ensure physical dimensions are multiples of the scale factor for Wayland compatibility
    let buffer_width = round_to_scale(physical_size.width, window.scale_factor());
    let buffer_height = round_to_scale(physical_size.height, window.scale_factor());
    
    let mut pixels = {
        let surface_texture = SurfaceTexture::new(buffer_width, buffer_height, &window);
        Pixels::new(buffer_width, buffer_height, surface_texture).expect("Failed to create pixels buffer")
    };

    // Use logical size for layout calculations (scale-independent)
    let mut viewport = Viewport::new(initial_logical_size.width as f32, initial_logical_size.height as f32);
    layout_engine.set_viewport(viewport);
    stylesheet.set_viewport(viewport);
    
    // Track mouse position and layout root for click handling
    let mut last_mouse_pos = (0.0, 0.0);
    let mut last_layout_root: Option<engine::layout::LayoutBox> = None;
    let mut needs_layout = true;
    
    // Request an initial redraw
    window.request_redraw();

    // --- Render loop ---
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                // Skip zero-size resizes (can happen during minimize)
                if new_size.width == 0 || new_size.height == 0 {
                    return;
                }
                
                // Round to scale factor multiple for Wayland compatibility
                let buffer_width = round_to_scale(new_size.width, window.scale_factor());
                let buffer_height = round_to_scale(new_size.height, window.scale_factor());
                
                // If dimensions needed rounding, resize window to match
                if buffer_width != new_size.width || buffer_height != new_size.height {
                    window.set_inner_size(winit::dpi::PhysicalSize::new(buffer_width, buffer_height));
                    return;
                }
                
                let logical_size: winit::dpi::LogicalSize<f32> = new_size.to_logical(window.scale_factor());
                viewport = Viewport::new(logical_size.width, logical_size.height);
                layout_engine.set_viewport(viewport);
                stylesheet.set_viewport(viewport);
                needs_layout = true;
                
                // Recreate pixels buffer with new dimensions
                let surface_texture = SurfaceTexture::new(buffer_width, buffer_height, &window);
                pixels = Pixels::new(buffer_width, buffer_height, surface_texture).unwrap();
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                // Update mouse position in physical pixels
                last_mouse_pos = (position.x as f32, position.y as f32);
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. }, .. } => {
                // Handle click on anchor tag
                if let Some(layout) = &last_layout_root {
                    if let Some(href) = find_anchor_at_position(layout, &dom, last_mouse_pos.0, last_mouse_pos.1, scale_factor) {
                        // Resolve relative URL against current page URL
                        // resolve_url(base_url, relative_url) - base is current page, relative is the href
                        let resolved_url = resolve_url(&current_url, &href);
                        log(&format!("SUCCESS: Navigating to {} (resolved from {})", resolved_url, href));
                        if let Ok(mut nav) = pending_navigation.lock() {
                            *nav = Some(resolved_url);
                        }
                    }
                }
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Check if we need to navigate to a new page
                if let Ok(mut nav) = pending_navigation.lock() {
                    if let Some(new_url) = nav.take() {
                        log(&format!("Navigating to: {}", new_url));
                        current_url = new_url.clone();
                        let (new_dom, new_stylesheet) = load_page(&new_url, &network_manager);
                        dom = new_dom;
                        stylesheet = new_stylesheet;
                        stylesheet.set_viewport(viewport);
                        needs_layout = true;
                        
                        // Update window title
                        let new_title = extract_title(&dom);
                        window.set_title(&format!("Grob Browser - {}", new_title));
                    }
                }
                
                // Always recompute layout to ensure it fills current viewport
                let layout_root = layout_engine.layout_with_full_viewport(&dom, &stylesheet, viewport, &mut font_manager);
                last_layout_root = Some(layout_root);
                needs_layout = false;
                
                let frame = pixels.frame_mut();
                let physical_size = window.inner_size();

                // Clear frame to white - fill entire buffer
                for byte in frame.iter_mut() {
                    *byte = 255;
                }

                // Draw layout and text - pass both logical and physical dimensions for proper scaling
                if let Some(ref layout_root) = last_layout_root {
                    draw_layout_and_text(frame, layout_root, &dom, &mut font_manager, physical_size.width as usize, physical_size.height as usize, scale_factor);
                    draw_images(frame, layout_root, &dom, &network_manager, physical_size.width as usize, physical_size.height as usize, scale_factor);
                }

                pixels.render().unwrap();
            }
            Event::MainEventsCleared => {
                // Only request redraw if layout changed
                if needs_layout {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    });
}

// --- Combined layout and text drawing ---
fn draw_layout_and_text(
    frame: &mut [u8],
    layout: &engine::layout::LayoutBox,
    dom: &engine::dom::Dom,
    font_manager: &mut FontManager,
    screen_width: usize,
    screen_height: usize,
    scale_factor: f32,
) {
    draw_box_recursive(frame, layout, dom, font_manager, screen_width, screen_height, scale_factor);
}

fn draw_box_recursive(
    frame: &mut [u8],
    layout: &engine::layout::LayoutBox,
    dom: &engine::dom::Dom,
    font_manager: &mut FontManager,
    screen_width: usize,
    screen_height: usize,
    scale_factor: f32,
) {
    let dims = &layout.dimensions;
    
    // Scale logical coordinates to physical pixels
    let x = (dims.x * scale_factor) as usize;
    let y = (dims.y * scale_factor) as usize;
    let width = (dims.width * scale_factor) as usize;
    let height = (dims.height * scale_factor) as usize;
    
    // Draw background if element has one
    if let Some((bg_r, bg_g, bg_b)) = layout.style.get_background_color() {
        for py in y..(y + height).min(screen_height) {
            for px in x..(x + width).min(screen_width) {
                let idx = (py * screen_width + px) * 4;
                if idx + 3 < frame.len() {
                    frame[idx] = bg_r;
                    frame[idx + 1] = bg_g;
                    frame[idx + 2] = bg_b;
                    frame[idx + 3] = 255;
                }
            }
        }
    }

    // Draw text if this layout box has text content
    if let Some(text_content) = &layout.text_content {
        let parent_id = dom.nodes[layout.node_id].parent;
        let should_skip = if let Some(pid) = parent_id {
            if let engine::dom::NodeType::Element(elem) = &dom.nodes[pid].node_type {
                matches!(elem.tag_name.as_str(), "style" | "script" | "head" | "title" | "meta" | "link")
            } else {
                false
            }
        } else {
            false
        };

        if !should_skip {
            draw_text_glyphs(frame, layout, text_content, font_manager, screen_width, screen_height, scale_factor);
        }
    }

    // Draw children
    for child in &layout.children {
        draw_box_recursive(frame, child, dom, font_manager, screen_width, screen_height, scale_factor);
    }
}

fn draw_text_glyphs(
    frame: &mut [u8],
    layout: &engine::layout::LayoutBox,
    text: &str,
    font_manager: &mut FontManager,
    screen_width: usize,
    screen_height: usize,
    scale_factor: f32,
) {
    let font_family = layout.style.get_font_family();
    let font_size = layout.style.get_font_size() * scale_factor;
    let (text_r, text_g, text_b) = layout.style.get_color();
    let has_underline = layout.style.has_text_decoration("underline");
    let is_bold = layout.style.is_bold();
    let is_italic = layout.style.is_italic();
    let scale = Scale::uniform(font_size);

    if let Some(font) = font_manager.load_font_variant(font_family, is_bold, is_italic) {
        let v_metrics = font.v_metrics(scale);
        let mut x = layout.dimensions.x * scale_factor;
        let y = layout.dimensions.y * scale_factor + v_metrics.ascent;
        let text_start_x = x;

        for c in text.chars() {
            let glyph = font.glyph(c).scaled(scale).positioned(point(x, y));

            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = gx as i32 + bb.min.x;
                    let py = gy as i32 + bb.min.y;

                    if px >= 0 && py >= 0 && px < screen_width as i32 && py < screen_height as i32 {
                        let idx = (py as usize * screen_width + px as usize) * 4;
                        if idx + 3 < frame.len() {
                            let coverage = (v * 255.0) as u8;
                            let bg_r = frame[idx] as u32;
                            let bg_g = frame[idx + 1] as u32;
                            let bg_b = frame[idx + 2] as u32;
                            let cov = coverage as u32;

                            frame[idx] = ((bg_r * (255 - cov) + text_r as u32 * cov) / 255) as u8;
                            frame[idx + 1] = ((bg_g * (255 - cov) + text_g as u32 * cov) / 255) as u8;
                            frame[idx + 2] = ((bg_b * (255 - cov) + text_b as u32 * cov) / 255) as u8;
                            frame[idx + 3] = 255;
                        }
                    }
                });
            }

            x += glyph.unpositioned().h_metrics().advance_width;
        }

        // Draw underline if needed
        if has_underline {
            let underline_y = (layout.dimensions.y * scale_factor + font_size * 1.1) as usize;
            let start_x = text_start_x as usize;
            let end_x = x as usize;
            let thickness = (font_size / 16.0).max(1.0) as usize;

            for t in 0..thickness {
                let uy = underline_y + t;
                if uy < screen_height {
                    for px in start_x..end_x.min(screen_width) {
                        let idx = (uy * screen_width + px) * 4;
                        if idx + 3 < frame.len() {
                            frame[idx] = text_r;
                            frame[idx + 1] = text_g;
                            frame[idx + 2] = text_b;
                            frame[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }
}

fn extract_title(dom: &engine::dom::Dom) -> String {
    // Find the title element
    let title_node = find_title_element(dom, dom.root());
    if let Some(title_id) = title_node {
        let title_el = &dom.nodes[title_id];
        // Get first text child
        for &child_id in &title_el.children {
            let child = &dom.nodes[child_id];
            if let NodeType::Text(text) = &child.node_type {
                return text.clone();
            }
        }
    }
    "Grob Browser".to_string()
}

fn find_title_element(dom: &engine::dom::Dom, node_id: engine::dom::NodeId) -> Option<engine::dom::NodeId> {
    let node = &dom.nodes[node_id];
    if let NodeType::Element(el) = &node.node_type {
        if el.tag_name == "title" {
            return Some(node_id);
        }
    }
    for &child_id in &node.children {
        if let Some(found) = find_title_element(dom, child_id) {
            return Some(found);
        }
    }
    None
}

// Draw images from img tags
fn draw_images(frame: &mut [u8], layout: &engine::layout::LayoutBox, dom: &Arc<engine::dom::Dom>, network: &Arc<NetworkManager>, screen_width: usize, screen_height: usize, scale_factor: f32) {
    let node = &dom.nodes[layout.node_id];
    
    // Check if this is an img element
    if let NodeType::Element(el) = &node.node_type {
        if el.tag_name == "img" {
            // Check for srcset first, then fallback to src
            let srcset = el.attributes.iter().find(|(k, _)| k == "srcset").map(|(_, v)| v.clone());
            let src = el.attributes.iter().find(|(k, _)| k == "src").map(|(_, v)| v.clone());
            let alt = el.attributes.iter().find(|(k, _)| k == "alt").map(|(_, v)| v.clone()).unwrap_or_else(|| "Image".to_string());
            
            // Select the best image URL
            let image_url = if let Some(srcset_attr) = srcset {
                // Parse srcset and select the best image for the current viewport
                let viewport_width = layout.dimensions.width as u32;
                let srcset_entries = engine::net::parse_srcset(&srcset_attr);
                engine::net::select_srcset_image(&srcset_entries, src.as_deref(), viewport_width, scale_factor)
            } else {
                src
            };
            
            if let Some(url) = image_url {
                // The NetworkManager will handle URL resolution internally
                if let Some(img_data) = network.fetch_image(&url) {
                    draw_real_image(frame, layout, &img_data, &alt, screen_width, screen_height);
                } else {
                    // Fall back to placeholder
                    draw_image_placeholder(frame, layout, &alt, screen_width, screen_height);
                }
            }
        }
    }
    
    // Also check for CSS background images
    if let Some(bg) = layout.style.get("background-image").or(layout.style.get("background")) {
        if let Some(url) = extract_url_from_css_value(bg) {
            if let Some(img_data) = network.fetch_image(&url) {
                draw_background_image(frame, layout, &img_data, screen_width, screen_height);
            }
        }
    }
    
    for child in &layout.children {
        draw_images(frame, child, dom, network, screen_width, screen_height, scale_factor);
    }
}

// Extract URL from CSS url(...) value
fn extract_url_from_css_value(value: &str) -> Option<String> {
    let value = value.trim().to_lowercase();
    if let Some(start) = value.find("url(") {
        let rest = &value[start + 4..];
        if let Some(end) = rest.find(')') {
            let url = rest[..end].trim();
            // Remove quotes if present
            let url = url.trim_matches(|c| c == '"' || c == '\'');
            if !url.is_empty() && !url.starts_with("data:") {
                return Some(url.to_string());
            }
        }
    }
    None
}

// Draw a background image
fn draw_background_image(frame: &mut [u8], layout: &engine::layout::LayoutBox, img: &image::RgbaImage, screen_width: usize, screen_height: usize) {
    let dims = &layout.dimensions;
    let x = dims.x as usize;
    let y = dims.y as usize;
    let width = dims.width as usize;
    let height = dims.height as usize;
    
    // Tile or stretch the background image
    for py in 0..height {
        if y + py >= screen_height {
            break;
        }
        for px in 0..width {
            if x + px >= screen_width {
                break;
            }
            
            // Tile the image
            let src_x = (px as u32) % img.width();
            let src_y = (py as u32) % img.height();
            
            if let Some(pixel) = img.get_pixel_checked(src_x, src_y) {
                let screen_idx = ((y + py) * screen_width + (x + px)) * 4;
                if screen_idx + 3 < frame.len() && pixel[3] > 0 {
                    // Alpha blending
                    let alpha = pixel[3] as u32;
                    let inv_alpha = 255 - alpha;
                    frame[screen_idx] = ((frame[screen_idx] as u32 * inv_alpha + pixel[0] as u32 * alpha) / 255) as u8;
                    frame[screen_idx + 1] = ((frame[screen_idx + 1] as u32 * inv_alpha + pixel[1] as u32 * alpha) / 255) as u8;
                    frame[screen_idx + 2] = ((frame[screen_idx + 2] as u32 * inv_alpha + pixel[2] as u32 * alpha) / 255) as u8;
                    frame[screen_idx + 3] = 255;
                }
            }
        }
    }
}

fn draw_image_placeholder(frame: &mut [u8], layout: &engine::layout::LayoutBox, alt: &str, screen_width: usize, screen_height: usize) {
    let dims = &layout.dimensions;
    let x = dims.x as usize;
    let y = dims.y as usize;
    let width = dims.width as usize;
    let height = dims.height as usize;
    
    // Draw a light gray placeholder with border
    for py in y..(y + height).min(screen_height) {
        for px in x..(x + width).min(screen_width) {
            let idx = (py * screen_width + px) * 4;
            if idx + 3 < frame.len() {
                // Light gray background
                frame[idx] = 200;     // R
                frame[idx + 1] = 200; // G
                frame[idx + 2] = 200; // B
                frame[idx + 3] = 255; // A
                
                // Draw border (dark gray)
                if py == y || py == y + height - 1 || px == x || px == x + width - 1 {
                    frame[idx] = 100;
                    frame[idx + 1] = 100;
                    frame[idx + 2] = 100;
                }
            }
        }
    }
    
    eprintln!("Drew image placeholder for '{}' ({}x{}) at ({},{})", 
        alt, width, height, x, y);
}

fn draw_real_image(frame: &mut [u8], layout: &engine::layout::LayoutBox, img: &image::RgbaImage, alt: &str, screen_width: usize, screen_height: usize) {
    let dims = &layout.dimensions;
    let x = dims.x as usize;
    let y = dims.y as usize;
    let width = dims.width.min(img.width() as f32) as usize;
    let height = dims.height.min(img.height() as f32) as usize;
    
    // Draw the image, scaling if necessary
    for py in 0..height {
        if y + py >= screen_height {
            break;
        }
        for px in 0..width {
            if x + px >= screen_width {
                break;
            }
            
            // Sample from source image (scaled)
            let src_x = (px as f32 * img.width() as f32 / width as f32) as u32;
            let src_y = (py as f32 * img.height() as f32 / height as f32) as u32;
            
            if let Some(pixel) = img.get_pixel_checked(src_x, src_y) {
                let screen_idx = ((y + py) * screen_width + (x + px)) * 4;
                if screen_idx + 3 < frame.len() {
                    frame[screen_idx] = pixel[0];     // R
                    frame[screen_idx + 1] = pixel[1]; // G
                    frame[screen_idx + 2] = pixel[2]; // B
                    frame[screen_idx + 3] = 255;      // A (opaque)
                }
            }
        }
    }
    
    eprintln!("Drew real image '{}' ({}x{}) at ({},{})", alt, width, height, x, y);
}

fn fetch_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    use reqwest::blocking::Client;
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    eprintln!("Fetching HTML from: {}", url);
    
    let response = client.get(url).send()?;
    let status = response.status();
    
    if !status.is_success() {
        return Err(format!("{}", status).into());
    }
    
    let html = response.text()?;
    eprintln!("Successfully fetched {} bytes from {}", html.len(), url);
    Ok(html)
}

fn extract_css_from_dom(dom: &engine::dom::Dom, node_id: engine::dom::NodeId) -> String {
    log("extract_css_from_dom called");
    let mut css_content = String::new();
    
    fn traverse(dom: &engine::dom::Dom, node_id: engine::dom::NodeId, css: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        log(&format!("{}traverse node_id: {}", indent, node_id));
        let node = &dom.nodes[node_id];
        
        match &node.node_type {
            engine::dom::NodeType::Element(elem) => {
                if elem.tag_name == "style" {
                    log(&format!("{}FOUND STYLE TAG!", indent));
                    // Get text content of style tag
                    for &child_id in &node.children {
                        let child = &dom.nodes[child_id];
                        if let engine::dom::NodeType::Text(text) = &child.node_type {
                            log(&format!("{}  extracting CSS: {}", indent, text));
                            css.push_str(text);
                            css.push('\n');
                        }
                    }
                }
                // Recursively search children
                for &child_id in &node.children {
                    traverse(dom, child_id, css, depth + 1);
                }
            }
            _ => {
                // Recursively search children
                for &child_id in &node.children {
                    traverse(dom, child_id, css, depth + 1);
                }
            }
        }
    }
    
    traverse(dom, node_id, &mut css_content, 0);
    log(&format!("extract_css_from_dom done: {} bytes", css_content.len()));
    css_content
}

fn convert_css_selector(css_selector: &engine::parser::css::parser::Selector) -> Selector {
    use engine::parser::css::parser::Selector as CssSelector;
    
    fn extract_tag_and_pseudo(sel: &CssSelector) -> (Option<String>, Option<String>) {
        match sel {
            CssSelector::Element(tag) => (Some(tag.clone()), None),
            CssSelector::PseudoClass(pseudo) => (None, Some(pseudo.clone())),
            CssSelector::Descendant(parent, child) => {
                let (p_tag, p_pseudo) = extract_tag_and_pseudo(parent);
                let (c_tag, c_pseudo) = extract_tag_and_pseudo(child);
                (c_tag.or(p_tag), c_pseudo.or(p_pseudo))
            },
            CssSelector::Child(parent, child) => {
                let (p_tag, p_pseudo) = extract_tag_and_pseudo(parent);
                let (c_tag, c_pseudo) = extract_tag_and_pseudo(child);
                (c_tag.or(p_tag), c_pseudo.or(p_pseudo))
            },
            _ => (None, None),
        }
    }
    
    match css_selector {
        CssSelector::Element(tag) => Selector::Tag(tag.clone()),
        CssSelector::Id(id) => Selector::Id(id.clone()),
        CssSelector::Class(class) => Selector::Class(class.clone()),
        CssSelector::Descendant(_, _) | CssSelector::Child(_, _) => {
            let (tag, pseudo) = extract_tag_and_pseudo(css_selector);
            match (tag, pseudo) {
                (Some(t), Some(p)) => Selector::TagWithPseudo(t, p),
                (Some(t), None) => Selector::Tag(t),
                (None, Some(p)) => Selector::Tag(p),
                (None, None) => Selector::Any,
            }
        },
        CssSelector::Adjacent(_, child) => convert_css_selector(child),
        CssSelector::GeneralSibling(_, child) => convert_css_selector(child),
        CssSelector::Universal => Selector::Any,
        _ => Selector::Any,
    }
}




