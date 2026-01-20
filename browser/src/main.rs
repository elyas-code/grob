use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use pixels::{Pixels, SurfaceTexture};
use rusttype::{Font, Scale, point};

use std::collections::HashMap;
use engine::parser::html::tree_builder::HtmlParser;
use engine::style::{Stylesheet, Style, Selector};
use engine::layout::LayoutEngine;

fn main() {
    // --- DOM ---
    let html = r#"<body><p>Hello engine</p><div>More text</div></body>"#;
    let dom = HtmlParser::new(html).parse();

    // --- CSS ---
    let mut stylesheet = Stylesheet::new();
    let mut style = Style { properties: HashMap::new() };
    style.properties.insert("color".to_string(), "black".to_string());
    stylesheet.add_rule(Selector::Tag("p".to_string()), style);

    // --- Layout ---
    let layout_engine = LayoutEngine::new();
    let layout_root = layout_engine.layout(&dom, &stylesheet);

    // --- Window ---
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Grob Browser Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(500, 300))
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        Pixels::new(size.width, size.height, surface_texture).unwrap()
    };

    // --- Load font ---
    let font_data = include_bytes!("DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();
    let scale = Scale::uniform(16.0);

    // --- Render loop ---
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit
            }
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();

                // Clear frame to white
                for px in frame.chunks_exact_mut(4) {
                    px[0] = 255;
                    px[1] = 255;
                    px[2] = 255;
                    px[3] = 255;
                }

                // Draw layout boxes (debug)
                draw_layout_box(frame, &layout_root);

                // Draw text
                draw_text(frame, &layout_root, &dom, &font, scale);

                pixels.render().unwrap();
            }
            _ => {}
        }

        window.request_redraw();
    });
}

// --- Draw layout boxes ---
fn draw_layout_box(frame: &mut [u8], layout: &engine::layout::LayoutBox) {
    let dims = &layout.dimensions;
    let width = dims.width as usize;
    let height = dims.height as usize;
    let screen_width = 500;
    let screen_height = 300;

    for y in dims.y as usize..(dims.y as usize + height).min(screen_height) {
        for x in dims.x as usize..(dims.x as usize + width).min(screen_width) {
            let idx = (y * screen_width + x) * 4;
            if idx + 3 < frame.len() {
                frame[idx] = 200;
                frame[idx + 1] = 230;
                frame[idx + 2] = 255;
                frame[idx + 3] = 255;
            }
        }
    }

    for child in &layout.children {
        draw_layout_box(frame, child);
    }
}

// --- Draw text using rusttype ---
fn draw_text(
    frame: &mut [u8],
    layout: &engine::layout::LayoutBox,
    dom: &engine::dom::Dom,
    font: &Font,
    scale: Scale,
) {
    if let engine::dom::NodeType::Text(text) = &dom.nodes[layout.node_id].node_type {
        let v_metrics = font.v_metrics(scale);
        let start_y = layout.dimensions.y as f32 + v_metrics.ascent;

        let mut x = layout.dimensions.x as f32;
        let y = start_y;

        for c in text.chars() {
            if let Some(glyph) = font.glyph(c).scaled(scale).positioned(point(x, y)).pixel_bounding_box() {
                for gx in 0..glyph.width() {
                    for gy in 0..glyph.height() {
                        let px = glyph.min.x + gx;
                        let py = glyph.min.y + gy;
                        if px >= 0 && py >= 0 && (px as usize) < 500 && (py as usize) < 300 {
                            let idx = ((py as usize) * 500 + (px as usize)) * 4;
                            frame[idx] = 0;
                            frame[idx + 1] = 0;
                            frame[idx + 2] = 0;
                            frame[idx + 3] = 255;
                        }
                    }
                }
            }

            x += font.glyph(c).scaled(scale).h_metrics().advance_width;
        }
    }

    for child in &layout.children {
        draw_text(frame, child, dom, font, scale);
    }
}
