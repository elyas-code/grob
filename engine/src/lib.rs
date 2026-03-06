pub mod parser;
pub mod dom;
pub mod style;
pub mod layout;
pub mod paint;
pub mod js;
pub mod net;
pub mod font;
pub mod platform;

pub use style::{Viewport, Breakpoint, MediaCondition, MediaRule};
pub use layout::{CSS_PX_SCALE, BASE_FONT_SIZE};

// Re-export commonly used net module items
pub use net::{
    NetworkManager, NetworkConfig, FetchedResource,
    resolve_url, resolve_url_with_base, parse_srcset, select_srcset_image,
    ImageType, detect_image_type,
    HtmlRewriter, RewriterConfig,
};

// Re-export HTML image extraction
pub use parser::html::{
    extract_image_refs, extract_base_href, extract_stylesheets,
    ImageRef, ImageRefType,
};