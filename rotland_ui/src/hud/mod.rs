//! Player HUD: health, stamina, crosshair, hotbar with icons, compass, clock, FPS, ammo,
//! pet dog status, safe zone label, dynamic key help bar, rifle scope overlay,
//! damage direction indicators, pickup popups, event notifications, and polished vital bars.
//!
//! Layout, sizes, and colours ported from Godot C# HUD:
//! - `scripts/UI/HUD.cs`
//! - `scripts/UI/Compass.cs`, `scripts/UI/CompassMath.cs`
//! - `scripts/UI/Crosshair.cs`
//! - `scripts/UI/ScopeOverlay.cs`
//! - `scripts/UI/HotbarBar.cs`, `scripts/UI/ItemIcons.cs`
//! - `scripts/Dogs/DogHud.cs`
//! - `scripts/UI/SafeZoneLabel.cs`
//! - `scripts/UI/KeyBar.cs`
//! - GitHub issues #2, #4, #18 (UI-001, UI-002)

pub mod ammo;
pub mod clock;
pub mod compass;
pub mod config;
pub mod crosshair;
pub mod damage;
pub mod death;
pub mod downed;
mod draw;
pub mod events;
pub mod fps;
pub mod hint;
pub mod hotbar;
pub mod key_bar;
mod manager;
pub mod notification;
pub mod pickup;
pub mod region_banner;
pub mod safe_zone;
pub mod scope;
mod state;
pub mod text;
pub mod vehicle;
pub mod vitals;

#[path = "../icons.rs"]
pub mod icons;

pub use compass::{CampHudInfo, WaypointHudInfo};
pub use config::HudConfig;
pub use damage::DamageIndicator;
pub use downed::DownedHudInfo;
pub use draw::draw_hud;
#[allow(unused_imports)]
pub use events::{
    direction_word_from_angle, direction_word_to_target, EventCooldowns, EventKind, HordeIndicator,
};
pub use game::hud_math::HeldKind;
pub use icons::IconCache;
#[allow(unused_imports)]
pub use manager::Hud;
pub use notification::{HudNotification, HudNotifications, NotificationKind};
pub use pickup::PickupPopup;
#[allow(unused_imports)]
pub use state::{HotbarSlot, HudState, KeyLabels};
pub use vehicle::VehicleHudInfo;
pub use vitals::{DogHudInfo, VitalsTrailState};
