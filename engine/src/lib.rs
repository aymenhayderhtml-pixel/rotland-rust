//! Rotland's own engine: window, input, GPU (wgpu on Vulkan), frame image, screenshots and stats.
//! Kept small on purpose: the game crates own all game rules; the engine draws, reads input and measures.
//! See rust/README.md for the commands AI workers use (screenshots, stats, fixed-length runs).

pub mod app;
pub mod capture;
pub mod cli;
pub mod gpu;
pub mod input;
pub mod stats;
pub mod target;
pub mod timing;

pub use app::{run, Game, GameStats};
pub use cli::EngineArgs;
pub use gpu::Gpu;
pub use input::Input;
pub use target::FrameTarget;
