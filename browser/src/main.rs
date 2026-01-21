use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use pixels::{Pixels, SurfaceTexture};
use rusttype::{Scale, point};

use engine::parser::html::tree_builder::HtmlParser;
use engine::style::{Stylesheet, Style, Selector};
use engine::layout::LayoutEngine;
use engine::dom::NodeType;
use engine::font::FontManager;
use engine::net::NetworkManager;
use std::sync::Arc;

use std::fs::OpenOptions;
use std::io::Write;

fn log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("c:\\temp\\grob_debug.log") {
        let _ = writeln!(file, "{}", msg);
    }
}

fn main() {
    // Fetch HTML from a web URL
    let url = "https://example.com";
    let html = match fetch_html(url) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to fetch HTML: {}", e);
            eprintln!("Using fallback HTML");
            r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Grob Browser</title>
                </head>
                <body>
                    <h1>Failed to Load Page</h1>
                    <p>Could not fetch the requested URL.</p>
                </body>
                </html>
            "#.to_string()
        }
    };
    let mut tokenizer = engine::parser::html::tokenizer::Tokenizer::new(&html);
    let mut token_count = 0;
    while let Some(token) = tokenizer.next_token() {
        if matches!(token, engine::parser::html::tokenizer::Token::Eof) {
            break;
        }
        eprintln!("Token {}: {:?}", token_count, token);
        token_count += 1;
        if token_count > 50 {
            eprintln!("(stopping after 50 tokens for brevity)");
            break;
        }
    }

    let dom = Arc::new(HtmlParser::new(&html).parse());

    // Extract title from DOM
    let page_title = extract_title(&dom);
    let window_title = format!("Grob Browser - {}", page_title);

    // Extract CSS from <style> tags in the DOM
    let css = extract_css_from_dom(&*dom, dom.root());
    
    log(&format!("Extracted CSS from <style> tags: {} bytes", css.len()));
    if !css.is_empty() {
        log(&format!("CSS content: {}", css));
    }

    // --- CSS (parse and apply) ---
    let mut stylesheet = Stylesheet::new();
    
    // Parse CSS from style tags and convert to stylesheet rules
    if !css.is_empty() {
        log(&format!("CSS extracted: {}", css));
        let mut css_tokenizer = engine::parser::css::CssTokenizer::new(&css);
        let tokens = css_tokenizer.tokenize();
        let mut css_parser = engine::parser::css::CssParser::new(tokens);
        let css_items = css_parser.parse();
        log(&format!("=== Parsed {} CSS items from <style> tags ===", css_items.len()));
        
        // Convert CSS rules to stylesheet rules
        for item in css_items {
            if let engine::parser::css::parser::CssItem::Rule(rule) = item {
                log(&format!("Processing CSS rule: {:?}", rule.selector));
                let selector = convert_css_selector(&rule.selector);
                log(&format!("  -> Converted to: {:?}", selector));
                let mut style = Style::new();
                
                log(&format!("  Declarations: {} items", rule.declarations.len()));
                for decl in rule.declarations {
                    log(&format!("    {} = {}", decl.property, decl.value));
                    style.properties.insert(decl.property, decl.value);
                }
                
                stylesheet.add_rule(selector, style);
            }
        }
        log(&format!("Stylesheet now has {} rules", stylesheet.rules.len()));
    } else {
        log("No <style> tags found in HTML, using fallback CSS");
    }
    
    // Add fallback styles if no CSS was parsed
    if stylesheet.rules.is_empty() {
        let mut style = Style::new();
        style.properties.insert("color".to_string(), "black".to_string());
        style.properties.insert("font-family".to_string(), "Times New Roman".to_string());
        style.properties.insert("font-size".to_string(), "16px".to_string());
        stylesheet.add_rule(Selector::Tag("p".to_string()), style);
    }

    // --- Layout ---
    let layout_engine = LayoutEngine::new();
    
    // --- Font Manager ---
    let mut font_manager = FontManager::new();

    // --- Network Manager ---
    let network_manager = Arc::new(NetworkManager::new());

    // --- Window ---
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(&window_title)
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        Pixels::new(size.width, size.height, surface_texture).unwrap()
    };

    let mut viewport_width = 800.0;
    let mut viewport_height = 600.0;

    // --- Render loop ---
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                viewport_width = new_size.width as f32;
                viewport_height = new_size.height as f32;
                // Recreate pixels buffer with new dimensions
                let surface_texture = SurfaceTexture::new(new_size.width, new_size.height, &window);
                pixels = Pixels::new(new_size.width, new_size.height, surface_texture).unwrap();
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let layout_root = layout_engine.layout_with_viewport(&dom, &stylesheet, viewport_width);
                
                let frame = pixels.frame_mut();

                // Clear frame to white
                for px in frame.chunks_exact_mut(4) {
                    px[0] = 255;
                    px[1] = 255;
                    px[2] = 255;
                    px[3] = 255;
                }

                // Draw layout and text
                draw_layout_and_text(frame, &layout_root, &dom, &mut font_manager, viewport_width as usize, viewport_height as usize);
                draw_images(frame, &layout_root, &dom, &network_manager, viewport_width as usize, viewport_height as usize);

                pixels.render().unwrap();
            }
            _ => {
                window.request_redraw();
            }
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
) {
    draw_box_recursive(frame, layout, dom, font_manager, screen_width, screen_height);
}

fn draw_box_recursive(
    frame: &mut [u8],
    layout: &engine::layout::LayoutBox,
    dom: &engine::dom::Dom,
    font_manager: &mut FontManager,
    screen_width: usize,
    screen_height: usize,
) {
    let dims = &layout.dimensions;
    
    // Draw background if element has one
    if let Some((bg_r, bg_g, bg_b)) = layout.style.get_background_color() {
        let x = dims.x as usize;
        let y = dims.y as usize;
        let width = dims.width as usize;
        let height = dims.height as usize;
        
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

    // Draw text if this is a text node
    if let engine::dom::NodeType::Text(text) = &dom.nodes[layout.node_id].node_type {
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
            draw_text_glyphs(frame, layout, text, font_manager, screen_width, screen_height);
        }
    }

    // Draw children
    for child in &layout.children {
        draw_box_recursive(frame, child, dom, font_manager, screen_width, screen_height);
    }
}

fn draw_text_glyphs(
    frame: &mut [u8],
    layout: &engine::layout::LayoutBox,
    text: &str,
    font_manager: &mut FontManager,
    screen_width: usize,
    screen_height: usize,
) {
    let font_family = layout.style.get_font_family();
    let font_size = layout.style.get_font_size();
    let (text_r, text_g, text_b) = layout.style.get_color();
    let has_underline = layout.style.has_text_decoration("underline");
    let scale = Scale::uniform(font_size);

    if let Some(font) = font_manager.load_system_font(font_family) {
        let v_metrics = font.v_metrics(scale);
        let mut x = layout.dimensions.x as f32;
        let y = layout.dimensions.y as f32 + v_metrics.ascent;
        let text_start_x = x;

        for c in text.chars() {
            let glyph = font.glyph(c).scaled(scale).positioned(point(x, y));

            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = gx as i32 + bb.min.x;
                    let py = gy as i32 + bb.min.y;

                    if px >= 0 && py >= 0 && px < screen_width as i32 && py < screen_height as i32 {
                        let idx = (py as usize * screen_width + px as usize) * 4;
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
                });
            }

            x += glyph.unpositioned().h_metrics().advance_width;
        }

        // Draw underline if needed
        if has_underline {
            let underline_y = (layout.dimensions.y as f32 + font_size * 1.05) as usize;
            let start_x = text_start_x as usize;
            let end_x = x as usize;

            if underline_y < screen_height {
                for px in start_x..end_x.min(screen_width) {
                    let idx = (underline_y * screen_width + px) * 4;
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
fn draw_images(frame: &mut [u8], layout: &engine::layout::LayoutBox, dom: &Arc<engine::dom::Dom>, network: &Arc<NetworkManager>, screen_width: usize, screen_height: usize) {
    let node = &dom.nodes[layout.node_id];
    
    // Check if this is an img element
    if let NodeType::Element(el) = &node.node_type {
        if el.tag_name == "img" {
            // Get src and alt attributes
            if let Some(src) = el.attributes.iter().find(|(k, _)| k == "src").map(|(_, v)| v) {
                let alt = el.attributes.iter().find(|(k, _)| k == "alt").map(|(_, v)| v.clone()).unwrap_or_else(|| "Image".to_string());
                
                // Try to fetch and draw the real image
                if let Some(img_data) = network.fetch_image(src) {
                    draw_real_image(frame, layout, &img_data, &alt, screen_width, screen_height);
                } else {
                    // Fall back to placeholder
                    draw_image_placeholder(frame, layout, &alt, screen_width, screen_height);
                }
            }
        }
    }
    
    for child in &layout.children {
        draw_images(frame, child, dom, network, screen_width, screen_height);
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
        return Err(format!("HTTP Error: {}", status).into());
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




