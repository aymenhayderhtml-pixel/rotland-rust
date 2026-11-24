//! Screamer special zombie windup, horde call, and retreat state machine.

use glam::Vec3;

use super::specials::{
    opt_dir, Phase, ScreamerConfig, SpecialCall, SpecialDirective, SpecialPose, SpecialState,
    TickCtx,
};

impl SpecialState {
    pub(super) fn tick_screamer(
        &mut self,
        cfg: &ScreamerConfig,
        ctx: TickCtx,
    ) -> SpecialDirective {
        match self.phase {
            Phase::Idle => {
                if ctx.los && self.cooldown <= 0.0 && ctx.dist <= cfg.trigger_range {
                    self.phase = Phase::ScreamerWindup;
                    self.timer = f64::from(cfg.windup_seconds);
                    return self.step_screamer_windup(ctx);
                }
                SpecialDirective::default()
            }
            Phase::ScreamerWindup => self.step_screamer_windup(ctx),
            Phase::ScreamerScream => {
                self.cooldown = f64::from(cfg.call_cooldown_seconds);
                self.phase = Phase::ScreamerRetreat;
                self.timer = f64::from(cfg.retreat_seconds);
                SpecialDirective {
                    velocity: Some(Vec3::ZERO),
                    face: opt_dir(ctx.to_player),
                    pose: SpecialPose::Scream,
                    call: Some(SpecialCall {
                        target: ctx.player,
                        radius: cfg.call_radius,
                        max_zombies: cfg.call_max_zombies,
                    }),
                    ..SpecialDirective::default()
                }
            }
            Phase::ScreamerRetreat => {
                let away = if ctx.to_player == Vec3::ZERO {
                    Vec3::ZERO
                } else {
                    -ctx.to_player
                };
                self.timer -= ctx.dt_f64;
                if self.timer <= ctx.eps {
                    self.phase = Phase::Idle;
                    self.timer = 0.0;
                }
                SpecialDirective {
                    velocity: Some(away * cfg.retreat_speed),
                    face: opt_dir(away),
                    pose: SpecialPose::Retreat,
                    ..SpecialDirective::default()
                }
            }
            _ => SpecialDirective::default(),
        }
    }

    fn step_screamer_windup(&mut self, ctx: TickCtx) -> SpecialDirective {
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::ScreamerScream;
            self.timer = 0.0;
        }
        SpecialDirective {
            velocity: Some(Vec3::ZERO),
            face: opt_dir(ctx.to_player),
            pose: SpecialPose::Windup,
            ..SpecialDirective::default()
        }
    }
}
