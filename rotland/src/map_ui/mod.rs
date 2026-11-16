//! Full-screen map page (egui 0.37).
//!
//! Port of Godot C# scripts/UI/MapView.cs and scripts/UI/HubMapPage.cs.

mod action;
mod draw;
mod draw_icons;
mod state;
mod view;

#[cfg(test)]
mod tests;

pub use action::MapAction;
pub use draw::draw_map;
pub use state::MapUiState;
pub use view::MapView;
