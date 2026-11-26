//! Grenade aiming arc preview, throwing, fuse stepping, rendering, and explosion effects.

use engine::Input;
use glam::{Mat4, Vec3};
use winit::event::MouseButton;

use crate::app::state::{ActorsState, PlaySide, RenderState, WorldState};
use crate::props::{Placed, PropLibrary};

// data: data/weapons.json grenade
const THROW_SPEED: f32 = 14.0;
// data: data/weapons.json grenade
const GRAVITY: f32 = 9.8;
// new: DECISIONS 2026-11-25
const BOUNCE_RESTITUTION: f32 = 0.3;
// new: DECISIONS 2026-11-25
const FUSE_SECONDS: f32 = 2.5;
// new: DECISIONS 2026-11-25
const EXPLOSION_RADIUS: f32 = 6.0;
// new: DECISIONS 2026-11-25
const EXPLOSION_MAX_DAMAGE: f32 = 150.0;
// new: DECISIONS 2026-11-25
const EXPLOSION_MIN_EDGE_FRAC: f32 = 0.2;
// new: DECISIONS 2026-11-25
const EXPLOSION_KNOCKBACK: f32 = 8.0;
// data: data/weapons.json grenade
const EXPLOSION_NOISE_RADIUS: f32 = 80.0;
// new: DECISIONS 2026-11-25
const ARC_STEPS: usize = 24;
// new: DECISIONS 2026-11-25
const ARC_STEP_DT: f32 = 0.08;
// new: DECISIONS 2026-11-25
const ARC_DOT_SCALE: f32 = 0.08;

/// Manages grenade aiming arc preview and active thrown grenades in flight.
#[derive(Default)]
pub struct GrenadePlay {
    pub aiming: bool,
    pub arc_points: Vec<Vec3>,
    pub active: Vec<game::grenade::ActiveGrenade>,
}

impl GrenadePlay {
    /// Updates grenade aiming, throwing, flight physics, and explosions.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        input: &Input,
        dt: f32,
        dead: bool,
        downed: bool,
        in_car: bool,
        captured: bool,
        render: &mut RenderState,
        world: &WorldState,
        actors: &mut ActorsState,
        play: &mut PlaySide,
        audio: &mut Option<audio::Audio>,
    ) {
        self.update_aim_and_throw(
            input, dead, downed, in_car, captured, render, actors, play, audio,
        );
        self.step_active(dt, render, world, actors, audio);
    }

    #[allow(clippy::too_many_arguments)]
    fn update_aim_and_throw(
        &mut self,
        input: &Input,
        dead: bool,
        downed: bool,
        in_car: bool,
        captured: bool,
        render: &mut RenderState,
        actors: &mut ActorsState,
        play: &mut PlaySide,
        audio: &mut Option<audio::Audio>,
    ) {
        let holding_grenade = !dead
            && !downed
            && !in_car
            && actors
                .combat
                .as_ref()
                .and_then(|c| c.weapon_def())
                .is_some_and(|w| w.id == "grenade");

        if !holding_grenade {
            self.aiming = false;
            self.arc_points.clear();
            return;
        }

        let count = play
            .play
            .as_ref()
            .map_or(0, |pl| pl.inventory.count("grenade"));
        if let Some(play_state) = play.play.as_mut() {
            play_state.last_shot.ammo = (count, u32::MAX);
            play_state.gun = actors.combat.as_ref().and_then(|c| c.weapon_def().cloned());
        }

        if count == 0 {
            self.aiming = false;
            self.arc_points.clear();
            return;
        }

        let origin = render.camera.position + render.camera.forward() * 0.6;
        let vel = render.camera.forward().normalize_or_zero() * THROW_SPEED;

        if captured && input.mouse_held(MouseButton::Left) {
            self.aiming = true;
            self.arc_points = game::grenade::compute_arc_points(
                origin,
                vel,
                GRAVITY,
                ARC_STEPS,
                ARC_STEP_DT,
            );
        } else if self.aiming {
            self.aiming = false;
            self.arc_points.clear();

            if let Some(play_state) = play.play.as_mut() {
                play_state.inventory.remove("grenade", 1);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
                if play_state.inventory.count("grenade") == 0 {
                    if let Some(c) = actors.combat.as_mut() {
                        c.set_weapon(None);
                    }
                    play_state.gun = None;
                    render.pending_weapon_model = Some(None);
                }
            }

            self.active.push(game::grenade::ActiveGrenade {
                position: origin,
                velocity: vel,
                fuse_remaining: FUSE_SECONDS,
                bounced: false,
            });
            if let Some(pv) = actors.player_visual.as_mut() {
                pv.animator.play_action("grenade_throw", 1.0);
            }
            actors.fp_sway.trigger_swing();
            if let (Some(a), Some(p)) = (audio.as_mut(), actors.player_ctrl.as_ref()) {
                a.event("throw", Some(p.position));
            }
        }
    }

    fn step_active(
        &mut self,
        dt: f32,
        render: &mut RenderState,
        world: &WorldState,
        actors: &mut ActorsState,
        audio: &mut Option<audio::Audio>,
    ) {
        if self.active.is_empty() {
            return;
        }
        let mut exploded = Vec::new();
        let mut kept = Vec::with_capacity(self.active.len());
        for mut g in self.active.drain(..) {
            let boom = game::grenade::step_grenade(
                &mut g,
                dt,
                GRAVITY,
                BOUNCE_RESTITUTION,
                |orig, dir, dist| {
                    world
                        .physics
                        .raycast(orig, dir, dist)
                        .map(|h| (h.point, h.normal))
                },
            );
            if boom {
                exploded.push(g.position);
            } else {
                kept.push(g);
            }
        }
        self.active = kept;

        for pos in exploded {
            self.detonate(pos, render, world, actors, audio);
        }
    }

    fn detonate(
        &self,
        pos: Vec3,
        render: &mut RenderState,
        world: &WorldState,
        actors: &mut ActorsState,
        audio: &mut Option<audio::Audio>,
    ) {
        crate::explosion_view::ExplosionView::spawn(&mut render.explosion_view, pos, EXPLOSION_RADIUS);
        let cfg = game::grenade::GrenadeConfig {
            blast_radius: EXPLOSION_RADIUS,
            max_damage: EXPLOSION_MAX_DAMAGE,
            min_edge_frac: EXPLOSION_MIN_EDGE_FRAC,
            knockback: EXPLOSION_KNOCKBACK,
            ..Default::default()
        };

        let is_blocked = |from: Vec3, to: Vec3| {
            let delta = to - from;
            let dist = delta.length();
            dist > 1e-4 && world.physics.raycast(from, delta / dist, dist).is_some()
        };

        if let (Some(zombies), Some(combat)) = (actors.zombies.as_mut(), actors.combat.as_mut()) {
            let targets: Vec<(u64, Vec3)> = zombies
                .list
                .iter()
                .filter(|z| !z.health.is_dead())
                .map(|z| (z.id, z.position + Vec3::Y))
                .collect();
            let hits = game::grenade::compute_explosion_hits(pos, &cfg, &targets, is_blocked);
            for hit in hits {
                let dir = hit.knockback.normalize_or_zero();
                let dmg = game::DamageInfo {
                    amount: hit.damage,
                    source: Some(1),
                    hit_point: pos,
                    direction: dir,
                    damage_type: "explosion".to_string(),
                    headshot: false,
                    leg_hit: false,
                };
                if zombies.hit(hit.target_id, dmg, hit.knockback) {
                    combat.kills += 1;
                }
            }
            combat.noises.push(game::zombie::Noise {
                position: pos,
                radius: EXPLOSION_NOISE_RADIUS,
            });
        }

        if let (Some(p), Some(combat)) = (actors.player_ctrl.as_ref(), actors.combat.as_mut()) {
            let player_center = p.position + Vec3::Y;
            let dist = player_center.distance(pos);
            if dist <= EXPLOSION_RADIUS && !is_blocked(pos + Vec3::Y * 0.1, player_center) {
                let dmg = game::grenade::compute_explosion_damage(
                    dist,
                    EXPLOSION_RADIUS,
                    EXPLOSION_MAX_DAMAGE,
                    EXPLOSION_MIN_EDGE_FRAC,
                );
                let now = actors.downed.last_play_time();
                let ev = combat.take_damage_with_downed(
                    &[dmg],
                    &mut actors.downed.machine,
                    now,
                    false,
                );
                if ev.died
                    && let Some(pv) = actors.player_visual.as_mut()
                {
                    pv.animator.play_die();
                }
            }
        }

        if let Some(a) = audio.as_mut() {
            a.event("explosion", Some(pos));
        }
    }

    /// Emits `Placed` instances for flying grenades and the dotted aiming arc preview.
    pub fn fill_render_instances(
        &self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        library: &mut PropLibrary,
        out: &mut Vec<Placed>,
    ) {
        if !self.active.is_empty()
            && let Some(slot) = library.get(device, pack, "grenade", false)
        {
            for g in &self.active {
                out.push(Placed {
                    model: slot,
                    matrix: Mat4::from_translation(g.position),
                    centre: g.position,
                });
            }
        }
        if self.aiming && !self.arc_points.is_empty() {
            let dot_slot = library
                .get(device, pack, "toxic_puff", false)
                .or_else(|| library.get(device, pack, "grenade", false));
            if let Some(slot) = dot_slot {
                for &pt in self.arc_points.iter().step_by(2) {
                    out.push(Placed {
                        model: slot,
                        matrix: Mat4::from_scale_rotation_translation(
                            Vec3::splat(ARC_DOT_SCALE),
                            glam::Quat::IDENTITY,
                            pt,
                        ),
                        centre: pt,
                    });
                }
            }
        }
    }
}
