use super::{
    CampHudInfo, DamageIndicator, DogHudInfo, DownedHudInfo, HeldKind, HordeIndicator,
    HudNotification, PickupPopup, VehicleHudInfo, VitalsTrailState, WaypointHudInfo,
};

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
    /// Body warmth bar (R32-05); `None` hides it.
    pub warmth: Option<super::warmth::WarmthHudInfo>,
    pub message: Option<String>,
    pub dead: bool,

    // --- R10-04 extensions ---
    /// Kind of weapon or item held in hand (determines crosshair visibility, #4).
    pub held: HeldKind,
    /// Pet dog health and status strip, if the player owns an alive dog.
    pub dog: Option<DogHudInfo>,
    /// Whether the player is currently inside a settlement safe zone.
    pub in_safe_zone: bool,
    /// "Entering: <region>" banner (R31-05).
    pub region_banner: Option<super::region_banner::RegionBannerInfo>,
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
    /// Optional weapon condition fractions (`0.0..=1.0`) for hotbar slots.
    pub hotbar_conditions: Option<Vec<Option<f32>>>,
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
    /// Active medical boosts and night-vision battery (R41-09).
    pub boosts: Option<super::boosts::BoostsHudInfo>,
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
            warmth: None,
            message: None,
            dead: false,
            held: HeldKind::None,
            dog: None,
            in_safe_zone: false,
            region_banner: None,
            camp: None,
            waypoint: None,
            scoped: false,
            key_labels: KeyLabels::default(),
            hotbar_item_ids: None,
            hotbar_conditions: None,
            vehicle: None,
            damage_indicators: None,
            pickup_popups: None,
            notifications: None,
            vitals_trail: None,
            player_pos: None,
            horde_indicators: None,
            downed: None,
            boosts: None,
        }
    }
}
