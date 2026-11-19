//! Pure HUD math, scaling, compass bearing, crosshair visibility, and non-overlapping layout calculations.
//!
//! Port of Godot C# HUD math from:
//! - `scripts/UI/CompassMath.cs`
//! - `scripts/UI/Compass.cs`
//! - `scripts/UI/Crosshair.cs`
//! - `scripts/UI/ScopeOverlay.cs`
//! - GitHub issues #2, #4, #18 (UI-001, UI-002)


/// Minimum and maximum scale factors for HUD scaling across resolutions.
/// Baseline is 1080p (scale = 1.0). At 600p minimum height, scale is clamped to 0.6.
pub const MIN_HUD_SCALE: f32 = 0.6;
pub const MAX_HUD_SCALE: f32 = 1.5;
pub const BASELINE_HEIGHT: f32 = 1080.0;

/// Half of the compass view field in degrees (from Godot `CompassMath.HalfViewDeg = 90f`).
pub const HALF_VIEW_DEG: f32 = 90.0;

/// Minimum distance in metres where a camp behind the view gets labeled "(behind)"
/// (from Godot `Compass.BehindMinDistance = 25f`).
pub const BEHIND_MIN_DISTANCE: f32 = 25.0;

/// Fraction of screen's shorter side for scope clear circular aperture
/// (from Godot `ScopeOverlay.ClearRadius = 0.36f`).
pub const SCOPE_CLEAR_RADIUS_FRACTION: f32 = 0.36;

/// Reticle gap as a fraction of clear radius (from Godot `ScopeOverlay gap = radius * 0.06f`).
pub const SCOPE_RETICLE_GAP_FRACTION: f32 = 0.06;

/// Reticle length as a fraction of clear radius (from Godot `ScopeOverlay ends = radius * 0.96f`).
pub const SCOPE_RETICLE_ENDS_FRACTION: f32 = 0.96;

/// Computes the proportional HUD scale factor based on screen height, clamped to [0.6, 1.5].
/// At 1080p (height 1080), scale is 1.0. At 800x600, scale is clamped to 0.6.
#[inline]
pub fn hud_scale(screen_height: f32) -> f32 {
    (screen_height / BASELINE_HEIGHT).clamp(MIN_HUD_SCALE, MAX_HUD_SCALE)
}

/// Computes the angle in degrees from the player's view direction to a world target on flat ground (X/Z).
/// Positive angles are to the right of the view; negative are to the left.
/// Yaw follows Godot convention: yaw 0 looks North (-Z), PI/2 looks West (-X).
/// Forward is (-sin yaw, -cos yaw); Right is (cos yaw, -sin yaw).
/// Port of Godot `CompassMath.RelativeAngleDeg`.
pub fn relative_angle_deg(from_x: f32, from_z: f32, target_x: f32, target_z: f32, yaw_rad: f32) -> f32 {
    let dx = target_x - from_x;
    let dz = target_z - from_z;
    let forward_x = -yaw_rad.sin();
    let forward_z = -yaw_rad.cos();
    let right_x = yaw_rad.cos();
    let right_z = -yaw_rad.sin();
    let dot_right = dx * right_x + dz * right_z;
    let dot_forward = dx * forward_x + dz * forward_z;
    dot_right.atan2(dot_forward).to_degrees()
}

/// Maps a bearing in degrees (0 = North, 90 = East, 180 = South, 270 = West)
/// to degrees relative to the player's view (-180 to 180).
/// Port of Godot `CompassMath.BearingToViewDeg`.
pub fn bearing_to_view_deg(bearing_deg: f32, yaw_rad: f32) -> f32 {
    let raw = bearing_deg + yaw_rad.to_degrees();
    ((raw + 180.0).rem_euclid(360.0)) - 180.0
}

/// Horizontal position in the compass strip (0.0 at left edge, `strip_width` at right edge).
/// Angles exceeding `HALF_VIEW_DEG` (90°) clamp to the strip edges.
/// Port of Godot `CompassMath.MarkerX`.
pub fn marker_x(angle_deg: f32, strip_width: f32) -> f32 {
    let t = (angle_deg / HALF_VIEW_DEG).clamp(-1.0, 1.0);
    strip_width * 0.5 + t * strip_width * 0.5
}

const DIRECTION_NAMES: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

/// Short cardinal/intercardinal direction name for multiples of 45°: N, NE, E, SE, S, SW, W, NW.
/// Port of Godot `CompassMath.Name`.
pub fn direction_name(bearing_deg: i32) -> &'static str {
    let idx = (bearing_deg.rem_euclid(360) / 45) as usize % 8;
    DIRECTION_NAMES[idx]
}

/// Determines whether the camp marker should be labeled "(behind)".
/// Matches Godot `Compass.ShowsBehind`: true when camp is outside the view (> 90°)
/// and at least `BEHIND_MIN_DISTANCE` (25m) away.
pub fn shows_behind(marker_visible: bool, camp_angle_deg: f32, distance_metres: f32) -> bool {
    marker_visible && camp_angle_deg.abs() > HALF_VIEW_DEG && distance_metres >= BEHIND_MIN_DISTANCE
}

/// Kind of item or weapon currently held in the player's hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeldKind {
    #[default]
    None,
    Melee,
    Gun,
    Other,
}

/// Determines whether the central crosshair should be drawn on screen.
/// - Dead: hidden (death overlay shown).
/// - Under rifle scope: hidden (scope overlay reticle is active instead).
/// - Holding melee weapon: hidden (per owner's design decision, issue #4).
/// - Holding gun: shown.
/// - Nothing in hand (None): shown (parity with today's behaviour).
/// - Other held items: shown.
pub fn is_crosshair_visible(dead: bool, scoped: bool, held: HeldKind) -> bool {
    if dead || scoped {
        return false;
    }
    match held {
        HeldKind::Melee => false,
        HeldKind::Gun | HeldKind::None | HeldKind::Other => true,
    }
}

/// Radius of clear circle aperture for rifle scope overlay in pixels.
#[inline]
pub fn scope_clear_radius(screen_w: f32, screen_h: f32) -> f32 {
    screen_w.min(screen_h) * SCOPE_CLEAR_RADIUS_FRACTION
}

/// Center reticle gap for scope overlay in pixels.
#[inline]
pub fn scope_reticle_gap(radius: f32) -> f32 {
    radius * SCOPE_RETICLE_GAP_FRACTION
}

/// Center reticle outer length for scope overlay in pixels.
#[inline]
pub fn scope_reticle_ends(radius: f32) -> f32 {
    radius * SCOPE_RETICLE_ENDS_FRACTION
}

/// A 2D bounding rectangle for layout collision testing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HudRect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl HudRect {
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub fn from_min_size(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x: min_x + width,
            max_y: min_y + height,
        }
    }

    /// Returns true if two rectangles overlap with non-zero area.
    pub fn overlaps(&self, other: &HudRect) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
    }
}

/// Layout rectangles of all main persistent HUD widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct HudLayoutRects {
    pub minimap_reserved: HudRect,
    pub fps: HudRect,
    pub clock: HudRect,
    pub compass: HudRect,
    pub vitals: HudRect,
    pub hotbar: HudRect,
    pub ammo: HudRect,
}

/// Computes the bounding rectangles of HUD widgets for a given screen dimension.
/// Ensures that at all supported resolutions (800x600, 1280x720, 1920x1080, 2560x1440):
/// - Compass does not overlap FPS or Clock.
/// - Minimap reserved area (top-left 180x180 box) does not collide with FPS or Compass.
/// - Hotbar does not overlap Vitals (health/stamina bars) on the left.
/// - Hotbar does not overlap Ammo readout on the right.
pub fn compute_hud_layout(screen_w: f32, screen_h: f32) -> HudLayoutRects {
    let scale = hud_scale(screen_h);

    // 1. Reserved top-left 180px box for minimap (#7), scaled with HUD scale
    let minimap_size = 180.0 * scale;
    let minimap_reserved =
        HudRect::from_min_size(16.0 * scale, 16.0 * scale, minimap_size, minimap_size);

    // 2. FPS counter: positioned below the minimap box to avoid collision at every resolution
    let fps_w = 80.0 * scale;
    let fps_h = 20.0 * scale;
    let fps_y = (16.0 + 180.0 + 8.0) * scale;
    let fps = HudRect::from_min_size(16.0 * scale, fps_y, fps_w, fps_h);

    // 3. Compass strip: top center
    let compass_w = (600.0 * scale).min(screen_w - 32.0 * scale);
    let compass_h = 100.0 * scale; // strip (40) + camp/waypoint labels underneath (60)
    let compass_x = screen_w * 0.5 - compass_w * 0.5;
    let compass_y = 10.0 * scale;
    let compass = HudRect::from_min_size(compass_x, compass_y, compass_w, compass_h);

    // 4. Clock: top right ("Day N · HH:MM")
    let clock_w = 150.0 * scale;
    let clock_h = 24.0 * scale;
    let clock_x = screen_w - 16.0 * scale - clock_w;
    let clock_y = 16.0 * scale;
    let clock = HudRect::from_min_size(clock_x, clock_y, clock_w, clock_h);

    // 5. Vital bars (Health, Stamina, Needs, Dog): bottom left
    let vitals_w = 260.0 * scale;
    let vitals_h = 125.0 * scale; // spans dog hud, needs, health, stamina
    let vitals_x = 16.0 * scale;
    let vitals_y = screen_h - 130.0 * scale;
    let vitals = HudRect::from_min_size(vitals_x, vitals_y, vitals_w, vitals_h);

    // 6. Hotbar: bottom center (6 slots of 56px + 5 gaps of 6px = 366px base)
    let hotbar_w = (6.0 * 56.0 + 5.0 * 6.0) * scale;
    let hotbar_h = 56.0 * scale;
    let hotbar_x = screen_w * 0.5 - hotbar_w * 0.5;
    let hotbar_y = screen_h - 72.0 * scale;
    let hotbar = HudRect::from_min_size(hotbar_x, hotbar_y, hotbar_w, hotbar_h);

    // 7. Ammo readout: bottom right
    let ammo_w = 140.0 * scale;
    let ammo_h = 36.0 * scale;
    let ammo_x = screen_w - 24.0 * scale - ammo_w;
    let ammo_y = screen_h - 40.0 * scale;
    let ammo = HudRect::from_min_size(ammo_x, ammo_y, ammo_w, ammo_h);

    HudLayoutRects {
        minimap_reserved,
        fps,
        clock,
        compass,
        vitals,
        hotbar,
        ammo,
    }
}

/// Checks if any persistent HUD elements overlap at the given screen size.
/// Returns true if an overlap was detected.
pub fn hud_elements_overlap(screen_w: f32, screen_h: f32) -> bool {
    let l = compute_hud_layout(screen_w, screen_h);
    // Test pairs that could potentially collide:
    l.minimap_reserved.overlaps(&l.fps)
        || l.minimap_reserved.overlaps(&l.compass)
        || l.compass.overlaps(&l.clock)
        || l.compass.overlaps(&l.fps)
        || l.vitals.overlaps(&l.hotbar)
        || l.hotbar.overlaps(&l.ammo)
}

