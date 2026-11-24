use glam::Vec3;
use physics::PhysicsWorld;
use serde::Deserialize;

use crate::player::Player;

// Probe geometry offsets in metres. // new: DECISIONS 2026-11-24
const LOW_RAY_HEIGHT: f32 = 0.3;
const HIGH_RAY_MARGIN: f32 = 0.05;
const HIGH_RAY_EXTRA: f32 = 0.2;
const TOP_RAY_STEP: f32 = 0.1;
const TOP_RAY_HEIGHT_MARGIN: f32 = 0.1;
const CLEARANCE_START_LIFT: f32 = 0.02;
const FAR_EDGE_DROP: f32 = 0.3;
const STEP_EPSILON: f32 = 1e-4;
const MIN_FORWARD_LEN_SQ: f32 = 0.5;

/// Tuning parameters for vaulting over low walls and fences.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultConfig {
    pub version: u32,
    pub probe_distance: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub step_over_max_height: f32,
    pub max_thickness: f32,
    pub landing_distance: f32,
    pub max_landing_drop: f32,
    pub clearance_above: f32,
    pub top_lift: f32,
    pub min_forward_input: f32,
    pub vault_seconds: f32,
    pub step_over_seconds: f32,
    pub speed_after: f32,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            version: 1,                 // data: vault.json version
            probe_distance: 0.9,        // data: vault.json probeDistance
            min_height: 0.45,           // data: vault.json minHeight
            max_height: 1.3,            // data: vault.json maxHeight
            step_over_max_height: 0.75, // data: vault.json stepOverMaxHeight
            max_thickness: 0.7,         // data: vault.json maxThickness
            landing_distance: 0.6,      // data: vault.json landingDistance
            max_landing_drop: 1.0,      // data: vault.json maxLandingDrop
            clearance_above: 0.9,       // data: vault.json clearanceAbove
            top_lift: 0.15,             // data: vault.json topLift
            min_forward_input: 0.3,     // data: vault.json minForwardInput
            vault_seconds: 0.7,         // data: vault.json vaultSeconds
            step_over_seconds: 0.6,     // data: vault.json stepOverSeconds
            speed_after: 3.0,           // data: vault.json speedAfter
        }
    }
}

impl VaultConfig {
    /// Parses a vault configuration from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("invalid vault.json: {e}"))
    }

    /// Loads `data/vault.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/vault.json");
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }
}

/// Which animation and duration to use for a vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultKind {
    StepOver,
    Vault,
}

impl VaultKind {
    /// Full-body animation clip name for this vault kind.
    pub fn clip(self) -> &'static str {
        match self {
            Self::StepOver => "step_over",
            Self::Vault => "vault_low",
        }
    }
}

/// Scripted vault trajectory computed by `find_vault`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VaultPlan {
    pub kind: VaultKind,
    pub start: Vec3,
    pub top: Vec3,
    pub land: Vec3,
    pub seconds: f32,
}

/// Pure probe: returns a plan if a vaultable obstacle is straight ahead of `feet` along `forward` (unit, flat).
pub fn find_vault(
    physics: &PhysicsWorld,
    feet: Vec3,
    forward: Vec3,
    cfg: &VaultConfig,
) -> Option<VaultPlan> {
    let fwd = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero(); // new: DECISIONS 2026-11-24
    if fwd.length_squared() < MIN_FORWARD_LEN_SQ {
        return None;
    }

    // 1. Low ray finds the front face of the obstacle.
    let low_origin = feet + Vec3::Y * LOW_RAY_HEIGHT;
    let face_hit = physics.raycast(low_origin, fwd, cfg.probe_distance)?;
    let d = face_hit.distance;

    // 2. High ray must miss, otherwise the obstacle is too tall.
    let high_origin = feet + Vec3::Y * (cfg.max_height + HIGH_RAY_MARGIN);
    if physics
        .raycast(high_origin, fwd, d + HIGH_RAY_EXTRA)
        .is_some()
    {
        return None;
    }

    // 3. Down ray finds the top surface height.
    let top_ray_y = feet.y + cfg.max_height + TOP_RAY_HEIGHT_MARGIN;
    let top_origin = Vec3::new(
        feet.x + fwd.x * (d + TOP_RAY_STEP),
        top_ray_y,
        feet.z + fwd.z * (d + TOP_RAY_STEP),
    );
    let top_hit = physics.raycast(
        top_origin,
        Vec3::NEG_Y,
        cfg.max_height + TOP_RAY_HEIGHT_MARGIN + HIGH_RAY_EXTRA,
    )?;
    let h = top_hit.point.y - feet.y;
    if h < cfg.min_height || h > cfg.max_height {
        return None;
    }
    let top_y = top_hit.point.y;

    // 4. Clearance ray above the top must not hit.
    let clearance_origin = top_hit.point + Vec3::Y * CLEARANCE_START_LIFT;
    if physics
        .raycast(clearance_origin, Vec3::Y, cfg.clearance_above)
        .is_some()
    {
        return None;
    }

    // 5. Walk forward in 0.1 m steps to find the far edge within max_thickness.
    let mut far_edge_dist = None;
    let mut offset = TOP_RAY_STEP;
    let thickness_ray_len = h + cfg.max_landing_drop + HIGH_RAY_EXTRA;
    while offset <= cfg.max_thickness + STEP_EPSILON {
        let probe_dist = d + offset;
        let probe_origin = Vec3::new(
            feet.x + fwd.x * probe_dist,
            top_y + TOP_RAY_HEIGHT_MARGIN,
            feet.z + fwd.z * probe_dist,
        );
        let dropped = match physics.raycast(probe_origin, Vec3::NEG_Y, thickness_ray_len) {
            Some(hit) => hit.point.y < top_y - FAR_EDGE_DROP,
            None => true,
        };
        if dropped {
            far_edge_dist = Some(probe_dist);
            break;
        }
        offset += TOP_RAY_STEP;
    }
    let far_edge_dist = far_edge_dist?;

    // 6. Landing ray past the far edge must hit valid ground.
    let land_dist = far_edge_dist + cfg.landing_distance;
    let land_origin = Vec3::new(
        feet.x + fwd.x * land_dist,
        top_y + TOP_RAY_HEIGHT_MARGIN,
        feet.z + fwd.z * land_dist,
    );
    let land_ray_len = (top_y - feet.y) + cfg.max_landing_drop + HIGH_RAY_EXTRA;
    let land_hit = physics.raycast(land_origin, Vec3::NEG_Y, land_ray_len)?;
    if top_y - land_hit.point.y < FAR_EDGE_DROP || feet.y - land_hit.point.y > cfg.max_landing_drop
    {
        return None;
    }

    // 7. Build the vault plan.
    let mid_dist = 0.5 * (d + far_edge_dist); // new: DECISIONS 2026-11-24
    let top = Vec3::new(
        feet.x + fwd.x * mid_dist,
        top_y + cfg.top_lift,
        feet.z + fwd.z * mid_dist,
    );
    let (kind, seconds) = if h < cfg.step_over_max_height {
        (VaultKind::StepOver, cfg.step_over_seconds)
    } else {
        (VaultKind::Vault, cfg.vault_seconds)
    };

    Some(VaultPlan {
        kind,
        start: feet,
        top,
        land: land_hit.point,
        seconds,
    })
}

/// Active vault movement state for the player.
#[derive(Clone, Debug, Default)]
pub struct VaultState {
    plan: Option<VaultPlan>,
    elapsed: f32,
    forward: Vec3,
    speed_after: f32,
}

impl VaultState {
    /// True while a scripted vault is in progress.
    pub fn is_active(&self) -> bool {
        self.plan.is_some()
    }

    /// Starts a vault if allowed: on_floor, not crouching, not in deep water,
    /// forward input >= minForwardInput, a plan is found along `player.yaw`,
    /// and `stamina.try_jump()` succeeds (checked last).
    /// Returns the clip name to play when it starts.
    pub fn try_start(
        &mut self,
        player: &mut Player,
        physics: &PhysicsWorld,
        forward_input: f32,
        cfg: &VaultConfig,
    ) -> Option<&'static str> {
        if self.is_active()
            || !player.on_floor
            || player.crouching
            || player.in_deep_water
            || forward_input < cfg.min_forward_input
        {
            return None;
        }

        let (sy, cy) = player.yaw.sin_cos();
        let forward = Vec3::new(-sy, 0.0, -cy); // new: DECISIONS 2026-11-24
        let plan = find_vault(physics, player.position, forward, cfg)?;

        if !player.stamina.try_jump() {
            return None;
        }

        self.plan = Some(plan);
        self.elapsed = 0.0; // new: DECISIONS 2026-11-24
        self.forward = forward;
        self.speed_after = cfg.speed_after;
        player.on_floor = false;
        player.velocity = Vec3::ZERO;
        Some(plan.kind.clip())
    }

    /// Moves the player along the path. Returns true while still vaulting.
    /// On the last frame puts the feet on `land`, sets velocity to `forward * speed_after` (y = 0),
    /// and sets `on_floor = true`.
    pub fn step(&mut self, player: &mut Player, dt: f32) -> bool {
        let Some(plan) = self.plan else {
            return false;
        };
        if dt <= 0.0 || !dt.is_finite() {
            return true;
        }

        self.elapsed += dt;
        if self.elapsed >= plan.seconds || plan.seconds <= 0.0 {
            player.position = plan.land;
            player.velocity = self.forward * self.speed_after;
            player.on_floor = true;
            self.plan = None;
            return false;
        }

        // Smoothstep normalized time, then two-phase curve (start->top, top->land). // new: DECISIONS 2026-11-24
        let raw_t = (self.elapsed / plan.seconds).clamp(0.0, 1.0);
        let t = raw_t * raw_t * (3.0 - 2.0 * raw_t);
        let pos = if t <= 0.5 {
            let u = t * 2.0;
            let xz = plan.start.lerp(plan.top, u);
            let ease_y = 1.0 - (1.0 - u).powi(3);
            let y = plan.start.y + (plan.top.y - plan.start.y) * ease_y;
            Vec3::new(xz.x, y, xz.z)
        } else {
            let u = (t - 0.5) * 2.0;
            let xz = plan.top.lerp(plan.land, u);
            let ease_y = u.powi(3);
            let y = plan.top.y + (plan.land.y - plan.top.y) * ease_y;
            Vec3::new(xz.x, y, xz.z)
        };

        player.position = pos;
        player.velocity = Vec3::ZERO;
        player.on_floor = false;
        true
    }

    /// Aborts any active vault immediately.
    pub fn cancel(&mut self) {
        self.plan = None;
        self.elapsed = 0.0; // new: DECISIONS 2026-11-24
    }
}
