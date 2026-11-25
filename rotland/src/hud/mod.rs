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
pub mod events;
pub mod fps;
pub mod hint;
pub mod hotbar;
pub mod key_bar;
pub mod notification;
pub mod pickup;
pub mod safe_zone;
pub mod scope;
pub mod text;
pub mod vehicle;
pub mod vitals;

#[path = "../icons.rs"]
pub mod icons;

pub use compass::{CampHudInfo, WaypointHudInfo};
pub use config::HudConfig;
pub use damage::DamageIndicator;
pub use downed::DownedHudInfo;
#[allow(unused_imports)]
pub use events::{
    direction_word_from_angle, direction_word_to_target, EventCooldowns, EventKind, HordeIndicator,
};
pub use game::hud_math::HeldKind;
pub use icons::IconCache;
pub use notification::{HudNotification, HudNotifications, NotificationKind};
pub use pickup::PickupPopup;
pub use vehicle::VehicleHudInfo;
pub use vitals::{DogHudInfo, VitalsTrailState};

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
                "[Tab] Bag   [L] Skills   [J] Bounties   [M] Map   [E] Use   [Esc] Menu".to_string(),
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
    /// Driving dashboard data; hidden the moment the player leaves the car.
    pub vehicle: Option<VehicleHudInfo>,

    // --- R16-09 HUD Polish extensions ---
    /// Active damage direction indicators.
    pub damage_indicators: Option<Vec<DamageIndicator>>,
    /// Active item pickup popups stacked on the right.
    pub pickup_popups: Option<Vec<PickupPopup>>,
    /// Event notifications queue displayed top-center under compass.
    pub notifications: Option<Vec<HudNotification>>,
    /// Smooth recent damage trails for health, stamina, and needs.
    pub vitals_trail: Option<VitalsTrailState>,
    /// Player world position for relative directional effects.
    pub player_pos: Option<glam::Vec3>,
    /// Screen-edge arrows pointing toward near hordes.
    pub horde_indicators: Option<Vec<HordeIndicator>>,
    /// Knocked-down bleed-out and revive indicator; `None` while standing.
    pub downed: Option<DownedHudInfo>,
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
            vehicle: None,
            damage_indicators: None,
            pickup_popups: None,
            notifications: None,
            vitals_trail: None,
            player_pos: None,
            horde_indicators: None,
            downed: None,
        }
    }
}

/// The main HUD manager containing persistent UI state, transient effects, and notifications.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Hud {
    pub damage_indicators: Vec<DamageIndicator>,
    pub pickup_popups: Vec<PickupPopup>,
    pub notifications: HudNotifications,
    pub vitals_trail: VitalsTrailState,
    pub config: HudConfig,
    pub player_pos: glam::Vec3,
    pub camera_yaw: f32,
    pub last_time: f64,
    pub car_out_of_fuel_notified: bool,
    pub horde_indicators: Vec<HordeIndicator>,
    pub cooldowns: EventCooldowns,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            damage_indicators: Vec::new(),
            pickup_popups: Vec::new(),
            notifications: HudNotifications::default(),
            vitals_trail: VitalsTrailState::default(),
            config: HudConfig::load_default(),
            player_pos: glam::Vec3::ZERO,
            camera_yaw: 0.0,
            last_time: 0.0,
            car_out_of_fuel_notified: false,
            horde_indicators: Vec::new(),
            cooldowns: EventCooldowns::default(),
        }
    }
}

#[allow(dead_code)]
impl Hud {
    pub fn new() -> Self {
        Self::default()
    }

    /// Triggered when the player takes damage from an attacker at world position `from`.
    /// Adds a directional damage arc that fades over 1.2s.
    pub fn on_damage(&mut self, from: glam::Vec3) {
        self.on_damage_at(from, self.last_time);
    }

    /// Triggered when the player takes damage from an attacker at world position `from` at timestamp `now`.
    pub fn on_damage_at(&mut self, from: glam::Vec3, now: f64) {
        let duration = self.config.damage_direction.fade_duration_s;
        self.damage_indicators.push(DamageIndicator::new(from, now, duration));
    }

    /// Triggered when the player picks up items: stacks above the hotbar and merges repeats.
    pub fn on_pickup(&mut self, item: &str, count: u32) {
        self.on_pickup_at(item, count, self.last_time);
    }

    /// Triggered when the player picks up items at a specific timestamp.
    pub fn on_pickup_at(&mut self, item: &str, count: u32, now: f64) {
        let duration = self.config.pickup_popup.fade_duration_s;
        let max_visible = self.config.pickup_popup.max_visible;
        pickup::push_or_merge_pickup(
            &mut self.pickup_popups,
            item,
            None,
            count,
            now,
            duration,
            max_visible,
        );
    }

    /// Triggered when a horde comes near: adds a screen-edge arrow pointing to the horde.
    pub fn on_horde_near(&mut self, horde_pos: glam::Vec3) {
        self.horde_indicators.push(HordeIndicator::new(horde_pos, self.last_time, 4.0)); // new: DECISIONS 2026-11-21
    }

    /// Triggered when an event notification should be displayed (Info, Warning, Danger, Horde).
    pub fn notify(&mut self, text: impl Into<String>, kind: NotificationKind) {
        self.notifications.push_with_duration(
            text,
            kind,
            self.last_time,
            self.config.notifications.duration_s,
        );
    }

    /// Triggered when an event notification should be displayed at a specific timestamp.
    pub fn notify_at(&mut self, text: impl Into<String>, kind: NotificationKind, now: f64) {
        self.notifications.push_with_duration(
            text,
            kind,
            now,
            self.config.notifications.duration_s,
        );
    }

    /// Ticks transient HUD effects, cleans up expired elements, and updates smooth vital trails.
    pub fn update(&mut self, dt: f32, now: f64, player_pos: glam::Vec3, camera_yaw: f32) {
        self.last_time = now;
        self.player_pos = player_pos;
        self.camera_yaw = camera_yaw;

        self.damage_indicators.retain(|d| !d.is_expired(now));
        self.pickup_popups.retain(|p| !p.is_expired(now));
        self.notifications.update(now);
        self.horde_indicators.retain(|h| !h.is_expired(now));
        let _ = dt;
    }

    /// Updates vital bar trails from live vitals data.
    pub fn update_vitals(
        &mut self,
        health: f32,
        stamina: f32,
        food_frac: Option<f32>,
        water_frac: Option<f32>,
        dt: f32,
    ) {
        self.vitals_trail
            .update(health, stamina, food_frac, water_frac, dt, &self.config.bars);
    }

    /// Attaches persistent HUD state onto `HudState` before drawing.
    pub fn populate_state(&self, state: &mut HudState) {
        state.damage_indicators = Some(self.damage_indicators.clone());
        state.pickup_popups = Some(self.pickup_popups.clone());
        state.notifications = Some(self.notifications.items.clone());
        state.vitals_trail = Some(self.vitals_trail.clone());
        state.player_pos = Some(self.player_pos);
        state.horde_indicators = Some(self.horde_indicators.clone());
    }

    /// Draws the complete HUD using this instance's persistent effects and state.
    pub fn draw(&mut self, ctx: &egui::Context, s: &HudState, icons: &mut IconCache, now: f64) {
        self.last_time = now;
        let mut state_clone = s.clone();
        self.populate_state(&mut state_clone);
        draw_hud(ctx, &state_clone, icons);
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
            let config = HudConfig::load_default();

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

            // 6. Crosshair in screen center (hidden when holding melee, scoped, driving, downed, or dead)
            if s.vehicle.is_none()
                && s.downed.is_none()
                && game::hud_math::is_crosshair_visible(s.dead, s.scoped, s.held)
            {
                crosshair::draw_crosshair(painter, screen.center(), scale);
            }

            // 7. Vital bars (Health & Stamina) bottom left (polished with rounded ends, darker back, recent damage trail, low pulse)
            vitals::draw_vital_bars_polished(
                painter,
                &screen,
                s.health,
                s.max_health,
                s.stamina,
                s.max_stamina,
                s.vitals_trail.as_ref(),
                time,
                scale,
                &config,
            );

            // 8. Need bars (Food & Water) above health (polished with rounded ends, darker back, recent damage trail, low pulse)
            vitals::draw_need_bars_polished(
                painter,
                &screen,
                time,
                s.food,
                s.water,
                s.vitals_trail.as_ref(),
                scale,
                &config,
            );

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

            // 11. Ammo readout bottom right (with dynamic key prompt), or the driving dashboard
            if let Some(info) = &s.vehicle {
                if !s.dead {
                    vehicle::draw_vehicle_hud(painter, &screen, time, info, scale);
                }
            } else {
                ammo::draw_ammo(painter, &screen, s.ammo, &s.key_labels.attack, scale);
            }

            // 12. Key help strip bottom right above ammo
            key_bar::draw_key_bar(painter, &screen, s.key_labels.key_help.as_deref(), scale);

            // 13. Centered toast/message if active
            if let Some(msg) = &s.message
                && !s.dead
                && s.downed.is_none()
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

            // 14. Event notifications queue top-center under compass
            if let Some(notifs) = &s.notifications
                && !notifs.is_empty()
                && !s.dead
            {
                notification::draw_notifications(
                    painter,
                    &screen,
                    notifs,
                    time,
                    scale,
                    &config,
                );
            }

            // 15. Pickup popups stacked above hotbar on the right
            if let Some(popups) = &s.pickup_popups
                && !popups.is_empty()
                && !s.dead
            {
                pickup::draw_pickup_popups(
                    painter,
                    ctx,
                    &screen,
                    popups,
                    icons,
                    time,
                    scale,
                    &config,
                );
            }

            // 16. Damage direction indicators (red arcs on screen edge pointing to attackers)
            if let Some(indicators) = &s.damage_indicators
                && !indicators.is_empty()
                && !s.dead
            {
                let ppos = s.player_pos.unwrap_or(glam::Vec3::ZERO);
                damage::draw_damage_indicators(
                    painter,
                    &screen,
                    indicators,
                    ppos,
                    s.heading_yaw,
                    time,
                    scale,
                    &config,
                );
            }

            // 17. Horde near direction arrows
            if let Some(horde_arrows) = &s.horde_indicators
                && !horde_arrows.is_empty()
                && !s.dead
            {
                let ppos = s.player_pos.unwrap_or(glam::Vec3::ZERO);
                events::draw_horde_arrows(
                    painter,
                    &screen,
                    horde_arrows,
                    ppos,
                    s.heading_yaw,
                    time,
                    scale,
                    &config,
                );
            }

            // 18. Downed bar when knocked down, or death overlay when dead
            if s.dead {
                death::draw_death_overlay(painter, &screen, s.message.as_deref(), scale);
            } else if let Some(info) = &s.downed {
                downed::draw_downed_bar(painter, &screen, info, scale);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_hud_lifecycle_and_events() {
        let mut hud = Hud::new();
        assert_eq!(hud.damage_indicators.len(), 0);
        assert_eq!(hud.pickup_popups.len(), 0);
        assert_eq!(hud.notifications.len(), 0);

        // 1. Damage direction event
        hud.on_damage_at(Vec3::new(10.0, 0.0, 5.0), 10.0);
        assert_eq!(hud.damage_indicators.len(), 1);
        assert!(!hud.damage_indicators[0].is_expired(10.5));
        assert!(hud.damage_indicators[0].alpha(10.5) > 0.0);
        assert!(hud.damage_indicators[0].is_expired(11.3));

        // Convenient 1-arg API hud.on_damage uses hud.last_time
        hud.last_time = 10.2;
        hud.on_damage(Vec3::new(0.0, 0.0, -5.0));
        assert_eq!(hud.damage_indicators.len(), 2);
        assert_eq!(hud.damage_indicators[1].spawn_time, 10.2);

        // 2. Pickup popups with repeat merging
        hud.on_pickup_at("canned_beans", 2, 10.0);
        assert_eq!(hud.pickup_popups.len(), 1);
        assert_eq!(hud.pickup_popups[0].count, 2);
        assert_eq!(hud.pickup_popups[0].label, "Canned Beans");

        // Merging repeat: +1 more canned beans should become +3 and refresh timer
        hud.on_pickup_at("canned_beans", 1, 11.0);
        assert_eq!(hud.pickup_popups.len(), 1);
        assert_eq!(hud.pickup_popups[0].count, 3);
        assert_eq!(hud.pickup_popups[0].spawn_time, 11.0);

        // Different item adds another popup
        hud.on_pickup_at("water_bottle", 1, 11.5);
        assert_eq!(hud.pickup_popups.len(), 2);

        // Convenient API hud.on_pickup
        hud.last_time = 12.0;
        hud.on_pickup("bandage", 1);
        assert_eq!(hud.pickup_popups.len(), 3);

        // 3. Notifications queue
        hud.notify("Upgrade finished", NotificationKind::Info);
        hud.notify_at("Horde incoming", NotificationKind::Danger, 12.0);
        hud.notify_at("Bounty complete", NotificationKind::Warning, 12.0);
        hud.notifications.push_at("Push at test", NotificationKind::Info, 10.0);
        assert_eq!(hud.notifications.len(), 3);
        assert!(!hud.notifications.is_empty());

        // 4. Update vitals trail via Hud API
        hud.update_vitals(75.0, 85.0, Some(0.7), Some(0.6), 0.016);
        assert_eq!(hud.vitals_trail.health_trail, 75.0);

        // 5. Update ticks and cleans up expired elements
        // At t = 20.0, all 10.0s events (1.2s damage, 2.5s pickup, 4.0s notifs) should be expired
        hud.update(0.016, 20.0, Vec3::new(0.0, 0.0, 0.0), 0.0);
        assert_eq!(hud.damage_indicators.len(), 0);
        assert_eq!(hud.pickup_popups.len(), 0);
        assert_eq!(hud.notifications.len(), 0);
    }

    #[test]
    fn test_vitals_trail_state() {
        let mut trail = VitalsTrailState::new();
        let cfg = HudConfig::default();

        // First update initializes trails to current values
        trail.update(80.0, 90.0, Some(0.8), Some(0.7), 0.016, &cfg.bars);
        assert_eq!(trail.health_trail, 80.0);
        assert_eq!(trail.stamina_trail, 90.0);
        assert_eq!(trail.food_trail, 0.8);
        assert_eq!(trail.water_trail, 0.7);

        // Take damage: health drops from 80 to 50
        trail.update(50.0, 90.0, Some(0.8), Some(0.7), 0.1, &cfg.bars);
        // Trail should be shrinking toward 50.0, but still above 50.0
        assert!(trail.health_trail < 80.0 && trail.health_trail > 50.0);

        // Heal to 95: trail snaps up to 95.0
        trail.update(95.0, 90.0, Some(0.8), Some(0.7), 0.1, &cfg.bars);
        assert_eq!(trail.health_trail, 95.0);
    }

    #[test]
    fn test_hud_config_loading_and_colors() {
        let cfg = HudConfig::load_default();
        assert_eq!(cfg.damage_direction.fade_duration_s, 1.2);
        assert_eq!(cfg.pickup_popup.fade_duration_s, 2.5);
        assert_eq!(cfg.notifications.duration_s, 4.0);
        assert_eq!(cfg.bars.corner_radius_px, 6.0);

        let col = cfg.damage_direction.color32(1.0);
        assert!(col.a() > 0);

        let bg = cfg.pickup_popup.bg_color32(1.0);
        assert!(bg.a() > 0);
        let txt = cfg.pickup_popup.text_color32(1.0);
        assert!(txt.a() > 0);

        let notif_col = notification::notification_color(NotificationKind::Info, &cfg);
        assert!(notif_col.a() > 0);

        let bars = &cfg.bars;
        assert!(bars.bg_color32().a() > 0);
        assert!(bars.trail_color32().a() > 0);
        assert!(bars.health_color32().a() > 0);
        assert!(bars.stamina_color32().a() > 0);
        assert!(bars.food_color32().a() > 0);
        assert!(bars.water_color32().a() > 0);
    }

    #[test]
    fn test_hud_state_populate() {
        let mut hud = Hud::new();
        hud.on_damage_at(Vec3::new(5.0, 0.0, 5.0), 1.0);
        hud.on_pickup_at("beans", 1, 1.0);
        hud.notify("Bounty complete", NotificationKind::Warning);

        let mut state = HudState::default();
        hud.populate_state(&mut state);

        assert_eq!(state.damage_indicators.as_ref().map(|v| v.len()), Some(1));
        assert_eq!(state.pickup_popups.as_ref().map(|v| v.len()), Some(1));
        assert_eq!(state.notifications.as_ref().map(|v| v.len()), Some(1));
        assert!(state.vitals_trail.is_some());
    }

    #[test]
    fn test_hud_wired_notifications() {
        let mut hud = Hud::new();
        hud.last_time = 5.0;

        // Verify the notification types supported by the HUD wiring:
        hud.notify("Bounty complete", NotificationKind::Info);
        hud.notify("Skill level up: Survival (2)", NotificationKind::Info);
        hud.notify("Car out of fuel", NotificationKind::Warning);

        assert_eq!(hud.notifications.len(), 3);
        assert_eq!(hud.notifications.items[0].text, "Bounty complete");
        assert_eq!(hud.notifications.items[1].text, "Skill level up: Survival (2)");
        assert_eq!(hud.notifications.items[2].text, "Car out of fuel");
        assert_eq!(hud.notifications.items[2].kind, NotificationKind::Warning);

        // Pushing a 4th notification evicts the oldest (queue capacity 3)
        hud.notify("Quick save done", NotificationKind::Info);
        assert_eq!(hud.notifications.len(), 3);
        assert_eq!(hud.notifications.items[0].text, "Skill level up: Survival (2)");
        assert_eq!(hud.notifications.items[2].text, "Quick save done");
    }

    #[test]
    fn test_serialization_roundtrip_and_defaults() {
        // HudNotifications JSON roundtrip
        let mut notifs = HudNotifications::new();
        notifs.push("Test", NotificationKind::Danger);
        let json = serde_json::to_string(&notifs).expect("serialize");
        let decoded: HudNotifications = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.items.len(), 1);
        assert_eq!(decoded.items[0].text, "Test");
        assert_eq!(decoded.items[0].kind, NotificationKind::Danger);

        // Empty json {} loads as default
        let empty_notifs: HudNotifications = serde_json::from_str("{}").expect("empty notifs");
        assert_eq!(empty_notifs.items.len(), 0);

        // VitalsTrailState JSON roundtrip
        let trail = VitalsTrailState {
            health_trail: 75.0,
            stamina_trail: 80.0,
            food_trail: 0.9,
            water_trail: 0.6,
            initialized: true,
        };
        let t_json = serde_json::to_string(&trail).expect("serialize trail");
        let t_decoded: VitalsTrailState = serde_json::from_str(&t_json).expect("deserialize trail");
        assert_eq!(t_decoded, trail);

        let empty_trail: VitalsTrailState = serde_json::from_str("{}").expect("empty trail");
        assert_eq!(empty_trail.health_trail, 100.0);
        assert_eq!(empty_trail.stamina_trail, 100.0);
        assert_eq!(empty_trail.food_trail, 1.0);
        assert_eq!(empty_trail.water_trail, 1.0);
    }
}
