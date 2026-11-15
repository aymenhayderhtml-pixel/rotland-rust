//! State execution for ZombieBrain (port of ZombieAI.cs, ZombieAI.Actions.cs and ZombieWounds.cs).

use glam::{Vec2, Vec3};
use physics::PhysicsWorld;

use super::brain::{BrainInput, Step, ZombieBrain};
use super::herd::find_chasing_near;
use super::hideouts::is_under_roof;
use super::senses::hears;
use super::state::ZombieState;
use crate::damage::DamageInfo;

const INVESTIGATE_SPEED_FACTOR: f32 = 0.7;
const LOOK_TURN_RATE: f32 = 1.2;
const MAX_WANDER_TIME: f32 = 15.0;

impl ZombieBrain {
    pub(super) fn think_idle(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        if self.senses.can_see() {
            self.spot_player(ctx.target);
            return Step::default();
        }
        if self.check_herd(ctx, dt) {
            return Step::default();
        }
        self.timer -= dt;
        if self.timer <= 0.0 {
            if self.should_eat(ctx) {
                self.enter_eat();
            } else {
                self.enter_wander(ctx.position);
            }
        }
        Step::default()
    }

    pub(super) fn think_wander(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        if self.senses.can_see() {
            self.spot_player(ctx.target);
            return Step::default();
        }
        if self.check_herd(ctx, dt) {
            return Step::default();
        }
        let speed = if self.crawler {
            self.numbers.crawler_speed
        } else {
            self.numbers.wander_speed
        };
        let (wish, arrived) = self.walk_to(ctx.position, self.goal, speed, ctx.world, dt);
        if arrived || self.time_in_state > MAX_WANDER_TIME {
            if let Some(horde_arc) = &self.horde
                && let Ok(mut horde) = horde_arc.lock()
            {
                horde.wander_target = None;
            }
            self.enter_idle();
        }
        Step { wish, hit: None }
    }

    pub(super) fn think_investigate(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        if self.senses.can_see() && !self.bang_on_arrive && !ctx.target_unreachable {
            self.spot_player(ctx.target);
            return Step::default();
        }
        if !self.looking_around {
            let speed = if self.crawler {
                self.numbers.crawler_speed
            } else {
                self.numbers.chase_speed * INVESTIGATE_SPEED_FACTOR
            };
            let (wish, mut arrived) = self.walk_to(ctx.position, self.goal, speed, ctx.world, dt);
            if self.bang_on_arrive && horizontal_distance(ctx.position, self.goal) <= 1.5 {
                arrived = true;
            }
            if arrived {
                if self.bang_on_arrive {
                    self.enter_bang();
                    return Step::default();
                }
                self.looking_around = true;
                self.timer = self.numbers.look_around_seconds;
            }
            return Step { wish, hit: None };
        }
        let turned = (self.numbers.look_around_seconds - self.timer) * LOOK_TURN_RATE;
        self.look_point = Some(ctx.position + Vec3::new(turned.sin(), 0.0, turned.cos()));
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.enter_wander(ctx.position);
        }
        Step::default()
    }

    pub(super) fn think_chase(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        let Some(target) = ctx.target else {
            return Step::default();
        };
        let tracking = self.senses.can_see() && !ctx.target_unreachable;
        if !tracking && self.senses.memory_expired() {
            let hidden = self.hideout_override.unwrap_or_else(|| {
                ctx.target_unreachable || is_under_roof(target, ctx.world)
            });
            let spot = self.senses.last_seen();
            self.enter_investigate(spot, hidden);
            return Step::default();
        }
        if tracking && horizontal_distance(ctx.position, target) <= self.numbers.attack_range {
            self.enter_attack();
            return Step::default();
        }

        let speed = if self.crawler {
            self.numbers.crawler_speed
        } else {
            self.numbers.chase_speed
        };
        let wish = match (tracking, ctx.flow_dir) {
            (true, Some(dir)) => Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero() * speed,
            (true, None) => self.walk_to(ctx.position, target, speed, ctx.world, dt).0,
            (false, _) => {
                let spot = self.senses.last_seen();
                self.walk_to(ctx.position, spot, speed, ctx.world, dt).0
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
        if !self.attack.is_active() {
            self.enter_chase();
        }
        Step {
            wish: Vec3::ZERO,
            hit,
        }
    }

    pub(super) fn think_scream(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        self.look_point = ctx.target;
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.enter_chase();
        }
        Step::default()
    }

    pub(super) fn think_eat(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        if self.senses.can_see() {
            self.spot_player(ctx.target);
            return Step::default();
        }
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.enter_idle();
        }
        Step::default()
    }

    pub(super) fn think_bang(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        if self.senses.can_see() && !ctx.target_unreachable {
            self.spot_player(ctx.target);
            return Step::default();
        }
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.enter_wander(ctx.position);
            return self.think_wander(ctx, dt);
        }
        Step::default()
    }

    pub(super) fn on_hit(
        &mut self,
        info: &DamageInfo,
        target: Option<Vec3>,
        feet_y: f32,
        ctx_leg_hit: bool,
    ) -> bool {
        let strong = info.amount >= self.numbers.strong_hit_damage;
        if strong {
            self.stagger_timer = self.stagger_timer.max(self.numbers.stagger_seconds);
        }
        if let Some(spot) = target {
            self.senses.notice_at(spot);
        }

        let is_leg = ctx_leg_hit
            || info.leg_hit
            || (info.hit_point != Vec3::ZERO && (info.hit_point.y - feet_y) < self.numbers.leg_hit_height);
        if is_leg && !self.crawler {
            self.leg_hits += 1;
            if self.leg_hits >= self.numbers.leg_hits_to_crawl || info.amount >= self.numbers.strong_hit_damage {
                self.crawler = true;
            }
        }

        self.alert();
        !strong
    }

    fn alert(&mut self) {
        if matches!(
            self.state,
            ZombieState::Chase | ZombieState::Attack | ZombieState::Dead
        ) {
            return;
        }
        self.enter_chase();
    }

    pub(super) fn last_audible(&self, ctx: &BrainInput<'_>) -> Option<Vec3> {
        ctx.heard
            .iter()
            .rev()
            .find(|n| hears(ctx.position, n, self.numbers.hearing_multiplier))
            .map(|n| n.position)
    }

    pub(super) fn hear_noise(&mut self, spot: Vec3) {
        if self.state.ignores_noise() {
            return;
        }
        self.enter_investigate(spot, false);
    }

    fn spot_player(&mut self, target: Option<Vec3>) {
        if self.rng.range(0.0, 1.0) < self.scream_chance {
            self.enter_scream(target);
        } else {
            self.enter_chase();
        }
    }

    fn check_herd(&mut self, ctx: &BrainInput<'_>, dt: f32) -> bool {
        self.herd_timer -= dt;
        if self.herd_timer > 0.0 {
            return false;
        }
        self.herd_timer = self.numbers.herd_check_seconds;
        if let Some(leader) = find_chasing_near(ctx.position, ctx.nearby_zombies, self.numbers.herd_radius) {
            self.senses.notice_at(leader.last_seen);
            self.enter_chase();
            return true;
        }
        false
    }

    fn should_eat(&mut self, ctx: &BrainInput<'_>) -> bool {
        if let Some(forced) = self.eat_chance_override {
            return self.rng.range(0.0, 1.0) < forced;
        }
        let near_corpse = ctx
            .corpses
            .iter()
            .any(|c| Vec2::new(c.x - ctx.position.x, c.z - ctx.position.z).length() <= self.numbers.corpse_spot_radius);
        let chance = if near_corpse {
            self.numbers.eat_near_corpse_chance
        } else {
            self.numbers.eat_random_chance
        };
        self.rng.range(0.0, 1.0) < chance
    }

    pub(super) fn walk_to(
        &mut self,
        from: Vec3,
        goal: Vec3,
        speed: f32,
        world: &PhysicsWorld,
        dt: f32,
    ) -> (Vec3, bool) {
        self.steering.set_target(goal);
        self.steering.follow(from, speed, dt, world)
    }
}

fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length()
}
