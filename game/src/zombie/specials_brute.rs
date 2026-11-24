//! Brute special zombie charge, windup, and stun state machine.

use glam::Vec3;

use super::specials::{
    opt_dir, turn_toward, BruteConfig, Phase, SpecialDirective, SpecialHit, SpecialPose,
    SpecialState, TickCtx,
};

impl SpecialState {
    pub(super) fn tick_brute(&mut self, cfg: &BruteConfig, ctx: TickCtx) -> SpecialDirective {
        match self.phase {
            Phase::Idle => {
                if ctx.los
                    && self.cooldown <= 0.0
                    && ctx.dist >= cfg.charge_min_range
                    && ctx.dist <= cfg.charge_max_range
                {
                    self.phase = Phase::BruteWindup;
                    self.timer = f64::from(cfg.windup_seconds);
                    return self.step_brute_windup(cfg, ctx);
                }
                SpecialDirective::default()
            }
            Phase::BruteWindup => self.step_brute_windup(cfg, ctx),
            Phase::BruteCharge => self.step_brute_charge(cfg, ctx),
            Phase::BruteStunned => self.step_brute_stunned(ctx),
            _ => SpecialDirective::default(),
        }
    }

    fn step_brute_windup(&mut self, cfg: &BruteConfig, ctx: TickCtx) -> SpecialDirective {
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.dir = ctx.to_player;
            self.hit_done = false;
            self.phase = Phase::BruteCharge;
            self.timer = f64::from(cfg.charge_seconds);
        }
        SpecialDirective {
            velocity: Some(Vec3::ZERO),
            face: opt_dir(ctx.to_player),
            pose: SpecialPose::Windup,
            ..SpecialDirective::default()
        }
    }

    fn step_brute_charge(&mut self, cfg: &BruteConfig, ctx: TickCtx) -> SpecialDirective {
        let max_turn = cfg.turn_rate_deg.to_radians() * ctx.dt;
        self.dir = turn_toward(self.dir, ctx.to_player, max_turn);
        if !self.hit_done && ctx.dist <= cfg.hit_range {
            self.hit_done = true;
            self.phase = Phase::Idle;
            self.timer = 0.0;
            self.cooldown = f64::from(cfg.cooldown_seconds);
            let kb_dir = if ctx.to_player != Vec3::ZERO {
                ctx.to_player
            } else {
                self.dir
            };
            return SpecialDirective {
                velocity: Some(Vec3::ZERO),
                face: opt_dir(self.dir),
                pose: SpecialPose::Charge,
                damage: Some(SpecialHit {
                    amount: cfg.hit_damage,
                    knockback: kb_dir * cfg.knockback_speed,
                }),
                ..SpecialDirective::default()
            };
        }
        if ctx.hit_wall {
            self.phase = Phase::BruteStunned;
            self.timer = f64::from(cfg.stun_seconds);
            self.cooldown = f64::from(cfg.cooldown_seconds);
            return self.step_brute_stunned(ctx);
        }
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::BruteStunned;
            self.timer = f64::from(cfg.stun_seconds);
            self.cooldown = f64::from(cfg.cooldown_seconds);
        }
        SpecialDirective {
            velocity: Some(self.dir * cfg.charge_speed),
            face: opt_dir(self.dir),
            pose: SpecialPose::Charge,
            ..SpecialDirective::default()
        }
    }

    fn step_brute_stunned(&mut self, ctx: TickCtx) -> SpecialDirective {
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::Idle;
            self.timer = 0.0;
        }
        SpecialDirective {
            velocity: Some(Vec3::ZERO),
            pose: SpecialPose::Stunned,
            stunned: true,
            vulnerable: true,
            ..SpecialDirective::default()
        }
    }
}
