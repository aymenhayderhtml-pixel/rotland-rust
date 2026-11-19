//! Player HUD: health, stamina, crosshair, hotbar with icons, compass, clock, FPS, ammo,
//! pet dog status, safe zone label, dynamic key help bar, and rifle scope overlay.
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
pub mod crosshair;
pub mod death;
pub mod fps;
pub mod hint;
pub mod hotbar;
pub mod key_bar;
pub mod safe_zone;
pub mod scope;
pub mod text;
pub mod vitals;

#[path = "../icons.rs"]
pub mod icons;

pub use compass::{CampHudInfo, WaypointHudInfo};
pub use game::hud_math::HeldKind;
pub use icons::IconCache;
pub use vitals::DogHudInfo;

use engine::egui::{self, pos2, Align2, Color32, FontId};

/// One slot in the player hotbar (1..6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotbarSlot {
    pub label: String,
    pub count: u32,
    pub selected: bool,
}

/// Dynamic key binding labels used by the HUD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLabels {
    /// Attack key name for weapon bash prompt ("Out of ammo: bash [{attack}]"). Defaults to "LMB".
    pub attack: String,
    /// Formatted key help bar string shown in bottom-right corner.
    pub key_help: Option<String>,
}

impl Default for KeyLabels {
    fn default() -> Self {
        Self {
            attack: "LMB".to_string(),
            key_help: Some(
                "[Tab] Bag   [K] Skills   [B] Bounties   [M] Map   [E] Use   [Esc] Menu".to_string(),
            ),
        }
    }
}

/// The entire HUD state passed in from the game rules each frame.
#[derive(Debug, Clone)]
pub struct HudState {
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub hour: f32,
    pub day: u32,
    pub heading_yaw: f32,
    pub hotbar: Vec<HotbarSlot>,
    pub ammo: Option<(u32, u32)>,
    pub fps: f32,
    /// Food and water bars: fill 0..1 and whether it is low (the bar pulses red).
    pub food: Option<(f32, bool)>,
    pub water: Option<(f32, bool)>,
    pub message: Option<String>,
    pub dead: bool,

    // --- R10-04 extensions ---
    /// Kind of weapon or item held in hand (determines crosshair visibility, #4).
    pub held: HeldKind,
    /// Pet dog health and status strip, if the player owns an alive dog.
    pub dog: Option<DogHudInfo>,
    /// Whether the player is currently inside a settlement safe zone.
    pub in_safe_zone: bool,
    /// Compass camp indicator (bearing angle and distance).
    pub camp: Option<CampHudInfo>,
    /// Compass waypoint diamond indicator.
    pub waypoint: Option<WaypointHudInfo>,
    /// Whether rifle scope overlay is active (hides normal crosshair).
    pub scoped: bool,
    /// Dynamic key bindings for HUD prompts and key help bar (#18 UI-002).
    pub key_labels: KeyLabels,
    /// Optional explicit item IDs for hotbar slots, matching `assets/icons/<id>.png`.
    pub hotbar_item_ids: Option<Vec<Option<String>>>,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            hour: 12.0,
            day: 1,
            heading_yaw: 0.0,
            hotbar: Vec::new(),
            ammo: None,
            fps: 60.0,
            food: None,
            water: None,
            message: None,
            dead: false,
            held: HeldKind::None,
            dog: None,
            in_safe_zone: false,
            camp: None,
            waypoint: None,
            scoped: false,
            key_labels: KeyLabels::default(),
            hotbar_item_ids: None,
        }
    }
}


/// Draws the HUD overlay using egui painter with explicit IconCache.
pub fn draw_hud(ctx: &egui::Context, s: &HudState, icons: &mut IconCache) {
    egui::Area::new(egui::Id::unique("rotland_hud"))
        .fixed_pos(pos2(0.0, 0.0))
        .interactable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            let screen = ctx.content_rect();
            let scale = game::hud_math::hud_scale(screen.height());
            let time = ctx.input(|i| i.time);

            // 1. Rifle scope overlay when aiming scoped weapon (underlay)
            if s.scoped && !s.dead {
                scope::draw_scope_overlay(painter, &screen);
            }

            // 2. FPS counter (below reserved 180px top-left minimap box, #7)
            fps::draw_fps(painter, &screen, s.fps, scale);

            // 3. Clock "Day N · HH:MM" top right
            clock::draw_clock(painter, &screen, s.hour, s.day, scale);

            // 4. Compass strip top centre (with camp indicator and waypoint diamond)
            compass::draw_compass(
                painter,
                &screen,
                s.heading_yaw,
                s.in_safe_zone,
                s.camp.as_ref(),
                s.waypoint.as_ref(),
                scale,
            );

            // 5. "Safe zone" label below compass when inside settlement
            safe_zone::draw_safe_zone(painter, &screen, s.in_safe_zone, scale);

            // 6. Crosshair in screen center (hidden when holding melee, scoped, or dead)
            if game::hud_math::is_crosshair_visible(s.dead, s.scoped, s.held) {
                crosshair::draw_crosshair(painter, screen.center(), scale);
            }

            // 7. Vital bars (Health & Stamina) bottom left
            vitals::draw_vital_bars(
                painter,
                &screen,
                s.health,
                s.max_health,
                s.stamina,
                s.max_stamina,
                scale,
            );

            // 8. Need bars (Food & Water) above health
            vitals::draw_need_bars(painter, &screen, time, s.food, s.water, scale);

            // 9. Pet Dog HUD strip above needs
            vitals::draw_dog_hud(painter, &screen, time, s.dog.as_ref(), scale);

            // 10. Hotbar bottom centre with item icon pictures and counts
            hotbar::draw_hotbar(
                painter,
                ctx,
                &screen,
                &s.hotbar,
                s.hotbar_item_ids.as_deref(),
                icons,
                scale,
            );

            // 11. Ammo readout bottom right (with dynamic key prompt)
            ammo::draw_ammo(painter, &screen, s.ammo, &s.key_labels.attack, scale);

            // 12. Key help strip bottom right above ammo
            key_bar::draw_key_bar(painter, &screen, s.key_labels.key_help.as_deref(), scale);

            // 13. Centered toast/message if active
            if let Some(msg) = &s.message
                && !s.dead
            {
                text::draw_outline_text(
                    painter,
                    pos2(screen.center().x, screen.center().y + 50.0 * scale),
                    Align2::CENTER_CENTER,
                    msg,
                    FontId::proportional(18.0 * scale),
                    Color32::WHITE,
                );
            }

            // 14. Death overlay when dead
            if s.dead {
                death::draw_death_overlay(painter, &screen, s.message.as_deref(), scale);
            }
        });
}

