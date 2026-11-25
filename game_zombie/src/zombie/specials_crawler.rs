//! Crawler special zombie lurk, lunge, and recovery state machine.

use glam::Vec3;

use super::specials::{
    opt_dir, CrawlerConfig, Phase, SpecialDirective, SpecialHit, SpecialPose, SpecialState,
    TickCtx,
};

impl SpecialState {
    pub(super) fn tick_crawler(
        &mut self,
        cfg: &CrawlerConfig,
        ctx: TickCtx,
        roll: f32,
    ) -> SpecialDirective {
        match self.phase {
            Phase::Idle => {
                if !ctx.los || ctx.dist > cfg.lurk_range {
                    return SpecialDirective {
                        velocity: Some(Vec3::ZERO),
                        pose: SpecialPose::Lurk,
                        ..SpecialDirective::default()
                    };
                }
                if ctx.dist <= cfg.lunge_trigger_range && self.cooldown <= 0.0 {
                    if roll < cfg.lunge_chance {
                        self.dir = ctx.to_player;
                        self.hit_done = false;
                        self.phase = Phase::CrawlerLunge;
                        self.timer = f64::from(cfg.lunge_seconds);
                        return self.step_crawler_lunge(cfg, ctx);
                    }
                    self.cooldown = f64::from(cfg.retry_seconds);
                }
                SpecialDirective::default()
            }
            Phase::CrawlerLunge => self.step_crawler_lunge(cfg, ctx),
            Phase::CrawlerRecover => {
                self.timer -= ctx.dt_f64;
                if self.timer <= ctx.eps {
                    self.phase = Phase::Idle;
                    self.timer = 0.0;
                    self.cooldown = f64::from(cfg.retry_seconds);
                }
                SpecialDirective {
                    velocity: Some(Vec3::ZERO),
                    pose: SpecialPose::Recover,
                    vulnerable: true,
                    ..SpecialDirective::default()
                }
            }
            _ => SpecialDirective::default(),
        }
    }

    fn step_crawler_lunge(&mut self, cfg: &CrawlerConfig, ctx: TickCtx) -> SpecialDirective {
        if !self.hit_done && ctx.dist <= cfg.hit_range {
            self.hit_done = true;
            self.phase = Phase::CrawlerRecover;
            self.timer = f64::from(cfg.recover_seconds);
            return SpecialDirective {
                velocity: Some(Vec3::ZERO),
                face: opt_dir(self.dir),
                pose: SpecialPose::Lunge,
                damage: Some(SpecialHit {
                    amount: cfg.lunge_damage,
                    knockback: Vec3::ZERO,
                }),
                ..SpecialDirective::default()
            };
        }
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::CrawlerRecover;
            self.timer = f64::from(cfg.recover_seconds);
        }
        SpecialDirective {
            velocity: Some(self.dir * cfg.lunge_speed),
            face: opt_dir(self.dir),
            pose: SpecialPose::Lunge,
            ..SpecialDirective::default()
        }
    }
}
