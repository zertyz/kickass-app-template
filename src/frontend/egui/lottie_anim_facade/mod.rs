//! This façade switches between web & native implementations of lottie animations,
//! demonstrating how differences in the platforms (native vs web) may be addressed
//! NOTE: a similar architecture should be used when accessing "backend" services:
//!       when "native", the services may be accessed directly; when "web", it must
//!       be done through the network -- see [crate::frontend::socket_server]

mod types;
pub use types::*;

/* #[cfg(not(target_arch = "wasm32"))]
mod native_lottie_anim;
#[cfg(not(target_arch = "wasm32"))]
pub use native_lottie_anim::*;

#[cfg(target_arch = "wasm32")] */
mod web_lottie_anim;
/* #[cfg(target_arch = "wasm32")] */
pub use web_lottie_anim::*;


/// contains animation names and their data
pub const LOTTIE_ANIMATIONS: &[(&str, &str)] = &[
    ("3D world illusion",      include_str!("3D world illusion.json")),
    ("Hypnotic",               include_str!("Hypnotic.json")),
    ("Infinity Ball",          include_str!("Infinity Ball.json")),
    ("Psychedelic 3D",         include_str!("Psychedelic 3D.json")),
    ("Swirling Wine",          include_str!("Swirling Wine.json")),
    ("Coder with coffee mug",  include_str!("Coder with coffee mug.json")),
    ("Rectangles and Circles", include_str!("Rectangles and Circles.json")),
];
