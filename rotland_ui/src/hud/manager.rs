use engine::egui;

use super::{
    draw_hud, pickup, DamageIndicator, EventCooldowns, HordeIndicator, HudConfig, HudNotifications,
    HudState, IconCache, NotificationKind, PickupPopup, VitalsTrailState,
};

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

#[cfg(test)]
mod tests {
    use super::super::notification;
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
