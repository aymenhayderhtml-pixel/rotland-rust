//! egui 0.37 UI for camp screens: Trader Shop, Bank, Crafting, and Bounty Board.
//! Follows Godot layouts, colors, and styles from scripts/UI/.

mod action;
mod draw;
mod draw_bank;
mod draw_bounties;
mod draw_crafting;
mod draw_shop;
mod screen;
mod state;
mod view;

#[cfg(test)]
mod tests;

pub use action::CampAction;
pub use draw::{draw, parse_hex_color, PANEL_HEIGHT, PANEL_WIDTH};
pub use draw_bounties::draw_bounty_tracker;
pub use screen::CampScreen;
pub use state::CampUiState;
pub use view::CampView;
