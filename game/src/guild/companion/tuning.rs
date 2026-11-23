//! Internal pacing and distance constants for companion AI (new: DECISIONS 2026-11-23).

pub(super) const EPSILON_SQ: f32 = 1e-6; // new: DECISIONS 2026-11-23
pub(super) const STOP_RADIUS: f32 = 0.25; // new: DECISIONS 2026-11-23
pub(super) const PLAYER_SEPARATION: f32 = 1.1; // new: DECISIONS 2026-11-23
pub(super) const TEAMMATE_SEPARATION: f32 = 0.9; // new: DECISIONS 2026-11-23
pub(super) const CAMP_SLOT_RADIUS: f32 = 1.6; // new: DECISIONS 2026-11-23
pub(super) const FOLLOW_BACK_FRACTION: f32 = 0.3; // new: DECISIONS 2026-11-23
pub(super) const TELEPORT_VIEW_DOT: f32 = 0.5; // new: DECISIONS 2026-11-23
pub(super) const GUARD_PATROL_RADIUS_RATIO: f32 = 0.65; // new: DECISIONS 2026-11-23
pub(super) const GUARD_PATROL_SPEED_RATIO: f32 = 0.7; // new: DECISIONS 2026-11-23
pub(super) const GUARD_FOLLOW_ENGAGE_RATIO: f32 = 0.7; // new: DECISIONS 2026-11-23
pub(super) const CHASE_DISENGAGE_MULT: f32 = 1.25; // new: DECISIONS 2026-11-23
pub(super) const FLEE_ESCAPE_MULT: f32 = 1.25; // new: DECISIONS 2026-11-23
pub(super) const FLEE_SPRINT_RATIO: f32 = 0.6; // new: DECISIONS 2026-11-23
pub(super) const COMBAT_HEAL_DELAY: f32 = 5.0; // new: DECISIONS 2026-11-23
pub(super) const STAGGER_SLOTS: usize = 4; // new: DECISIONS 2026-11-23
pub(super) const PATROL_WAYPOINTS: usize = 4; // new: DECISIONS 2026-11-23
