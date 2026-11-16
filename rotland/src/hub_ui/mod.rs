// Helper methods and fields here are used by screens, HUD overlays, and tests
#![allow(dead_code)]

//! egui 0.37 UI for player hub screens: Bag, Skills, Bounties, Profile, Trunk, and Dogs.
//! Follows Godot layouts, colors, and styles from scripts/UI/ and scripts/Dogs/.

mod action;
mod draw;
mod draw_bag;
mod draw_bag_details;
mod draw_bounties;
mod draw_dog;
mod draw_overlays;
mod draw_profile;
mod draw_skills;
mod draw_trunk;
mod screen;
mod state;
mod view;

#[cfg(test)]
mod tests;

pub use action::{DogCommand, HubAction};
pub use draw::{draw, draw_overlays, parse_hex_color};
pub use screen::{HubScreen, HubTab};
pub use state::{
    HubUiState, LevelUpBannerState, XpToast, LEVEL_UP_BANNER_DURATION, XP_TOAST_LIFETIME,
    XP_TOAST_MERGE_WINDOW,
};
pub use view::{DogView, HubView};
