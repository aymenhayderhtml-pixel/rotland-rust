// Helper methods here (count steppers, view getters) are used by the tests and by screens still to come
#![allow(dead_code)]

//! egui 0.37 UI for camp screens: Trader Shop, Bank, Crafting, Bounty Board, and Camp Board.
//! Follows Godot layouts, colors, and styles from scripts/UI/ (Camp Board uses menus::theme).

mod action;
mod draw;
mod draw_bank;
mod draw_bounties;
mod draw_crafting;
mod draw_shop;
mod draw_upgrades;
mod screen;
mod state;
mod view;

#[cfg(test)]
mod tests;

pub use action::CampAction;
pub use draw::draw;
pub use draw_bounties::draw_bounty_tracker;
pub use screen::CampScreen;
pub use state::CampUiState;
pub use view::{
    CampUpgradesView, CampView, NextUpgradeView, UpgradeCostItem, UpgradeInProgressView,
    UpgradeLineView,
};
