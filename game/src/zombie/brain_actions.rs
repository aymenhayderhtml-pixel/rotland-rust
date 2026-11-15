//! The thinking of each state (ZombieAI.Think*): Idle, Wander, Investigate, Chase and Attack. Called from
//! `ZombieBrain::think` in brain.rs. Scream, Eat and Bang are TODO(R3-07).

use glam::Vec3;

use super::brain::{BrainInput, Step, ZombieBrain};
use super::senses::hears;
use super::state::ZombieState;
use crate::damage::DamageInfo;

/// A walk stops this close to its goal (ZombieSteering.ArriveDistance).
const ARRIVE_DISTANCE: f32 = 1.0;
/// Investigating walks at this share of the chase speed (ZombieAI.InvestigateSpeedFactor).
const INVESTIGATE_SPEED_FACTOR: f32 = 0.7;
/// How fast a zombie turns its head while looking about, in rad/s (ZombieAI.LookTurnRate).
const LOOK_TURN_RATE: f32 = 1.2;
/// A wander this long is given up even if it has not arrived (ZombieAI.MaxWanderTime).
const MAX_WANDER_TIME: f32 = 15.0;

impl ZombieBrain {
    pub(super) fn think_idle(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        if self.senses.can_see() {
            self.spot_player();
            return Step::default();
        }
        // TODO(R3-07): CheckHerd (join a nearby chasing zombie) goes here, before the timer.
        self.timer -= dt;
        if self.timer <= 0.0 {
            // TODO(R3-07): ShouldEat picks Eat near a corpse. Until then every pause ends in a wander.
            self.enter_wander(ctx.position);
        }
        Step::default()
    }

    pub(super) fn think_wander(&mut self, ctx: &BrainInput<'_>) -> Step {
        if self.senses.can_see() {
            self.spot_player();
            return Step::default();
        }
        // TODO(R3-07): CheckHerd here, as in Idle. Horde.WanderTarget is cleared when the walk ends.
        let (wish, arrived) = self.walk_to(ctx.position, self.goal, self.numbers.wander_speed);
        if arrived || self.time_in_state > MAX_WANDER_TIME {
            self.enter_idle();
        }
        Step { wish, hit: None }
    }

    pub(super) fn think_investigate(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        if self.senses.can_see() {
            self.spot_player();
            return Step::default();
        }
        if !self.looking_around {
            let speed = self.numbers.chase_speed * INVESTIGATE_SPEED_FACTOR;
            let (wish, arrived) = self.walk_to(ctx.position, self.goal, speed);
            if arrived {
                // TODO(R3-07): a player who went in or up is banged for here (bang_on_arrive), not looked for.
                self.looking_around = true;
                self.timer = self.numbers.look_around_seconds;
            }
            return Step { wish, hit: None };
        }
        // Stands at the spot and turns the head, looking about
        let turned = (self.numbers.look_around_seconds - self.timer) * LOOK_TURN_RATE;
        self.look_point = Some(ctx.position + Vec3::new(turned.sin(), 0.0, turned.cos()));
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.enter_wander(ctx.position);
        }
        Step::default()
    }

    pub(super) fn think_chase(&mut self, ctx: &BrainInput<'_>) -> Step {
        // think() gives up a chase with no target before getting here
        let Some(target) = ctx.target else {
            return Step::default();
        };
        let tracking = self.senses.can_see();
        if !tracking && self.senses.memory_expired() {
            // Walks to the last seen spot and looks about there, then wanders.
            // TODO(R3-07): a player hidden on a ladder or roof is banged for here instead (ZombieHideouts).
            let spot = self.senses.last_seen();
            self.enter_investigate(spot);
            return Step::default();
        }
        if tracking && horizontal_distance(ctx.position, target) <= self.numbers.attack_range {
            self.enter_attack();
            return Step::default();
        }

        let speed = self.numbers.chase_speed;
        let wish = match (tracking, ctx.flow_dir) {
            // The flow field leads to the player. It knows nothing of a last-seen spot, so that walk is straight.
            (true, Some(dir)) => Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero() * speed,
            (true, None) => self.walk_to(ctx.position, target, speed).0,
            (false, _) => {
                let spot = self.senses.last_seen();
                self.walk_to(ctx.position, spot, speed).0
            }
        };
        Step { wish, hit: None }
    }

    pub(super) fn think_attack(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        let Some(target) = ctx.target else {
            return Step::default();
        };
        self.look_point = Some(target);
        let hit = self
            .attack
            .update(dt, horizontal_distance(ctx.position, target));
        // The attack ends once its cooldown has run out (ZombieAttack.Think returns true)
        if !self.attack.is_active() {
            self.enter_chase();
        }
        Step {
            wish: Vec3::ZERO,
            hit,
        }
    }

    /// A strong hit staggers, a light one flinches. Either way the zombie knows where its attacker
    /// was and is alerted (Zombie.OnWounded). Returns true for a light flinch this frame.
    pub(super) fn on_hit(&mut self, info: &DamageInfo, target: Option<Vec3>) -> bool {
        let strong = info.amount >= self.numbers.strong_hit_damage;
        if strong {
            self.stagger_timer = self.stagger_timer.max(self.numbers.stagger_seconds);
        }
        if let Some(spot) = target {
            self.senses.notice_at(spot);
        }
        self.alert();
        !strong
    }

    /// A hit always alerts. A zombie already chasing or attacking keeps its target (C# Alert).
    fn alert(&mut self) {
        if matches!(
            self.state,
            ZombieState::Chase | ZombieState::Attack | ZombieState::Dead
        ) {
            return;
        }
        self.enter_chase();
    }

    /// The last sound this frame that this zombie can hear. Later sounds re-aim the walk, so the last one wins.
    pub(super) fn last_audible(&self, ctx: &BrainInput<'_>) -> Option<Vec3> {
        ctx.heard
            .iter()
            .rev()
            .find(|n| hears(ctx.position, n, self.numbers.hearing_multiplier))
            .map(|n| n.position)
    }

    /// Chase, Attack and Dead ignore sounds (ZombieAI.HearNoise).
    pub(super) fn hear_noise(&mut self, spot: Vec3) {
        if self.state.ignores_noise() {
            return;
        }
        self.enter_investigate(spot);
    }

    /// The player is in view. TODO(R3-07): roll ScreamChance first; a scream comes before the chase.
    fn spot_player(&mut self) {
        self.enter_chase();
    }

    /// Straight-line walk toward `goal` (ZombieSteering.Follow without its feeler rays). Returns the
    /// flat wish velocity and whether the walker is already within arrival distance.
    fn walk_to(&self, from: Vec3, goal: Vec3, speed: f32) -> (Vec3, bool) {
        let to = Vec3::new(goal.x - from.x, 0.0, goal.z - from.z);
        let dist = to.length();
        if dist <= ARRIVE_DISTANCE {
            return (Vec3::ZERO, true);
        }
        (to / dist * speed, false)
    }
}

/// Distance on the ground between two points (height ignored, as in the C# HorizontalDistance).
fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length()
}
