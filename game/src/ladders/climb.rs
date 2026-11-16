use glam::Vec3;

use super::rail::LadderRail;

/// Input for climbing ladders each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClimbInput {
    /// Upward/downward climb intent. Positive is up (W / forward), negative is down (S / backward).
    pub up: f32,
    /// Jump key pressed: release ladder and fall.
    pub jump: bool,
    /// Interact key (E) pressed: grab ladder or release.
    pub interact: bool,
}

impl ClimbInput {
    pub fn new(up: f32, jump: bool, interact: bool) -> Self {
        Self { up, jump, interact }
    }
}

/// Climbing phase: on the rungs or moving through a scripted transition path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClimbPhase {
    Climbing,
    Transition,
}

/// Speed of climbing along the ladder in m/s.
// Godot: LadderClimb.cs ClimbSpeed
pub const CLIMB_SPEED: f32 = 2.5;
/// Time in seconds for the step onto or off the deck.
// Godot: LadderClimb.cs TransitionSeconds
pub const TRANSITION_SECONDS: f32 = 0.3;
/// Release presses are ignored this long, so the E that started the climb cannot end it.
// Godot: LadderClimb.cs StartGrace
pub const START_GRACE: f32 = 0.3;
/// Lift above the deck when the climb ends, so feet settle on the floor.
// Godot: LadderClimb.cs SettleLift
pub const SETTLE_LIFT: f32 = 0.05;
/// Maximum reach distance to grab a ladder end.
pub const GRAB_RANGE: f32 = 2.5;

/// Ladder climbing state machine.
/// Port of `scripts/Player/LadderClimb.cs`.
#[derive(Clone, Debug, PartialEq)]
pub struct LadderClimb {
    active: bool,
    rail: Option<LadderRail>,
    phase: ClimbPhase,
    grace: f32,
    path: Vec<Vec3>,
    path_length: f32,
    path_distance: f32,
    finishing: bool,
}

impl Default for LadderClimb {
    fn default() -> Self {
        Self::new()
    }
}

impl LadderClimb {
    pub fn new() -> Self {
        Self {
            active: false,
            rail: None,
            phase: ClimbPhase::Climbing,
            grace: 0.0,
            path: Vec::new(),
            path_length: 0.0,
            path_distance: 0.0,
            finishing: false,
        }
    }

    /// True from the start of a climb until the player lets go or reaches an end.
    // Godot: LadderClimb.cs Active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// True while hands are on the rungs (not during transition path).
    // Godot: LadderClimb.cs IsOnRungs
    pub fn is_on_rungs(&self) -> bool {
        self.active && self.phase == ClimbPhase::Climbing
    }

    pub fn phase(&self) -> ClimbPhase {
        self.phase
    }

    /// The ladder being climbed, or None when not climbing.
    // Godot: LadderClimb.cs Rail
    pub fn rail(&self) -> Option<&LadderRail> {
        self.rail.as_ref()
    }

    pub fn grace(&self) -> f32 {
        self.grace
    }

    /// Starts a climb from the foot or from the deck (climbing down).
    // Godot: LadderClimb.cs Start
    pub fn start(&mut self, rail: LadderRail, from_deck: bool, current_pos: Vec3) {
        if self.active {
            return;
        }
        self.active = true;
        self.grace = START_GRACE;

        let foot = rail.foot;
        let deck = rail.deck;
        self.rail = Some(rail);

        if from_deck {
            let at_top = Vec3::new(foot.x, deck.y, foot.z);
            let p0 = current_pos;
            let p1 = deck + Vec3::new(0.0, LadderRail::CLEAR_HEIGHT, 0.0);
            let p2 = at_top + Vec3::new(0.0, LadderRail::CLEAR_HEIGHT, 0.0);
            let p3 = at_top;
            self.begin_path(vec![p0, p1, p2, p3], false);
        } else {
            self.phase = ClimbPhase::Climbing;
        }
    }

    /// Ends the climb immediately.
    // Godot: LadderClimb.cs Stop
    pub fn stop(&mut self) {
        self.active = false;
        self.phase = ClimbPhase::Climbing;
        self.rail = None;
        self.path.clear();
        self.path_length = 0.0;
        self.path_distance = 0.0;
        self.finishing = false;
    }

    fn begin_path(&mut self, points: Vec<Vec3>, finishing: bool) {
        self.path = points;
        self.path_length = 0.0;
        for i in 0..self.path.len().saturating_sub(1) {
            self.path_length += self.path[i].distance(self.path[i + 1]);
        }
        self.path_distance = 0.0;
        self.finishing = finishing;
        self.phase = ClimbPhase::Transition;
    }

    fn sample_path(&self, distance: f32) -> Vec3 {
        let mut d = distance;
        for i in 0..self.path.len().saturating_sub(1) {
            let a = self.path[i];
            let b = self.path[i + 1];
            let len = a.distance(b);
            if d <= len || i == self.path.len() - 2 {
                let t = if len > 0.0 {
                    (d / len).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                return a.lerp(b, t);
            }
            d -= len;
        }
        *self.path.last().unwrap_or(&Vec3::ZERO)
    }

    fn step_path(&mut self, dt: f32) -> (Vec3, bool) {
        let speed = if TRANSITION_SECONDS > 0.0 {
            self.path_length / TRANSITION_SECONDS
        } else {
            0.0
        };
        self.path_distance += speed * dt;
        if self.path_distance < self.path_length {
            return (self.sample_path(self.path_distance), true);
        }

        let end = *self.path.last().unwrap_or(&Vec3::ZERO);
        if self.finishing {
            let final_pos = end + Vec3::new(0.0, SETTLE_LIFT, 0.0);
            self.stop();
            (final_pos, false)
        } else {
            self.phase = ClimbPhase::Climbing;
            (end, true)
        }
    }

    /// Advances the climb state by `dt` seconds given input.
    /// Modifies `position`, `velocity`, and `yaw`.
    /// Returns whether the player is still on the ladder.
    // Godot: LadderClimb.cs Step
    pub fn step(
        &mut self,
        dt: f32,
        input: &ClimbInput,
        position: &mut Vec3,
        velocity: &mut Vec3,
        yaw: &mut f32,
    ) -> bool {
        if !self.active {
            return false;
        }
        let Some(rail) = self.rail.clone() else {
            self.stop();
            return false;
        };

        self.grace -= dt;

        // Face the ladder; Godot forward is -Z so yaw is atan2(-facing.x, -facing.z)
        let facing = rail.facing();
        *yaw = (-facing.x).atan2(-facing.z);

        if self.phase == ClimbPhase::Transition {
            let (new_pos, still_active) = self.step_path(dt);
            *position = new_pos;
            *velocity = Vec3::ZERO;
            return still_active;
        }

        let release = input.jump || input.interact;
        if release && self.grace <= 0.0 {
            self.stop();
            *velocity = Vec3::ZERO;
            return false;
        }

        let up = input.up;
        let foot = rail.foot;
        let deck_y = rail.deck.y;
        let target_y = position.y + up * CLIMB_SPEED * dt;

        if up > 0.0 && target_y >= deck_y {
            let at_top = Vec3::new(foot.x, deck_y, foot.z);
            let deck = rail.deck;
            let p0 = at_top;
            let p1 = at_top + Vec3::new(0.0, LadderRail::CLEAR_HEIGHT, 0.0);
            let p2 = deck + Vec3::new(0.0, LadderRail::CLEAR_HEIGHT, 0.0);
            let p3 = deck;
            self.begin_path(vec![p0, p1, p2, p3], true);
            *position = at_top;
            *velocity = Vec3::ZERO;
            return true;
        }

        if up < 0.0 && target_y <= foot.y {
            *position = foot;
            *velocity = Vec3::ZERO;
            self.stop();
            return false;
        }

        let clamped_y = target_y.clamp(foot.y, deck_y);
        *position = Vec3::new(foot.x, clamped_y, foot.z);
        *velocity = Vec3::ZERO;
        true
    }
}
