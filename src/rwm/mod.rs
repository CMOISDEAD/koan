pub mod config;
pub mod core;
pub mod error;
pub mod events;
pub mod layouts;
pub mod monitors;
pub mod utils;
pub mod window;

pub use core::MiniWM;
pub use window::Window;
pub use error::MiniWMError;
pub use config::{BORDER_WIDTH, KEY_BINDINGS, Action};
pub use utils::get_pixel_from_color;
