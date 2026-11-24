//! Humanoid skeletal animator and layered pose pipeline.
//!
//! Evaluates base locomotion -> air -> crouch -> stance -> aim -> action/swing ->
//! additive -> left-hand IK -> full-body override -> skin matrices without per-frame allocations.

use std::collections::HashMap;

use engine::anim::Rig;
use glam::{Mat4, Vec3};
use pack::{Clip, Skeleton};

use crate::action::ActionState;
use crate::aim::{AimController, AimJoints};
use crate::airborne::{AirState, LandingKind, landing_clip};
use crate::anim_config::AnimConfig;
use crate::blend::{
    locomotion_weights, lod_interval, sample_locomotion_into, tempo_for, Locomotion,
    LocomotionClips, RUN_BLEND,
};
use crate::clip_events::{ClipEvent, ClipEventTable, EventQueue, EventSpan};
use crate::foot_ik::{FootIkState, FootJoints};
use crate::full_body::{FullBodyOptions, FullBodyState};
use crate::incline::InclineState;
use crate::layers::{LayerBlend, LayerId, LayerStack};
use crate::mask::{BoneMasks, MaskId};
use crate::motion::{MotionClips, MotionJoints, MotionState};
use crate::pose_buffer::{add_pose_masked, blend_pose_masked, PosePool};
use crate::posture::PostureState;
use crate::car_anim::{car_event_clip, CarAnimState, CarEvent};
use crate::ladder_anim::{
    is_ladder_clip, ladder_event_clip, LadderAnimState, LadderEvent, LADDER_CLIMB_CLIP,
};
use crate::stance::{HoldTransform, StanceController, StanceTable};
use crate::swim::{SwimState, SWIM_CLIMB_OUT_CLIP};

const POSE_POOL_BUFFERS: usize = 3; // new: DECISIONS 2026-11-23
const LAYER_EPSILON: f32 = 0.001; // godot: HumanoidAnimTree.cs:210
const MIN_TEMPO_SCALE: f32 = 0.05; // godot: HumanoidAnimTree.cs:112
const SWING_MAX_TIME: f32 = 0.70; // godot: poses_combat.py:47
const DEFAULT_OFF_HAND_GRIP: Vec3 = Vec3::new(0.0, 0.0, 0.10); // godot: HumanoidVisual.cs:95

/// Per-character humanoid animation state machine.
pub struct HumanoidAnimator {
    config: AnimConfig,
    clips: HashMap<String, Clip>,
    masks: BoneMasks,
    layers: LayerStack,
    pool: PosePool,
    parents: Vec<Option<usize>>,
    order: Vec<usize>,
    weapon_name: Option<String>,
    two_handed: bool,
    off_hand_grip: Option<Vec3>,
    stance: StanceController,
    aim: AimController,
    aim_joints: AimJoints,
    motion: MotionState,
    motion_joints: MotionJoints,
    air: AirState,
    air_driven: bool,
    posture: PostureState,
    swim: SwimState,
    ladder: LadderAnimState,
    car: CarAnimState,
    incline: InclineState,
    foot_ik: FootIkState,
    foot_joints: Option<FootJoints>,
    foot_globals: Vec<Mat4>,
    blend_pos: f32,
    tempo_scale: f32,
    time_idle: f32,
    action: Option<ActionState>,
    full_body: Option<FullBodyState>,
    events: ClipEventTable,
    event_queue: EventQueue,
    dying: bool,
    held_pose: Option<String>,
    pose_time: f32,
    update_interval: f32,
    accumulated_dt: f32,
    dirty: bool,
    cached_skin: Vec<Mat4>,
}

impl HumanoidAnimator {
    /// Creates an animator loading shared tuning from `data/animation.json`.
    pub fn new(skeleton: &Skeleton, clips: &[Clip]) -> Self {
        Self::with_config(skeleton, clips, AnimConfig::load_shared().clone())
    }

    /// Creates an animator using an explicit `AnimConfig` (preferred for tests).
    pub fn with_config(skeleton: &Skeleton, clips: &[Clip], config: AnimConfig) -> Self {
        let n = skeleton.joints.len();
        let joint_names: Vec<&str> = skeleton.joints.iter().map(|j| j.name.as_str()).collect();
        let masks = BoneMasks::new(&joint_names);
        let aim_joints = AimJoints::from_names(&joint_names);
        let motion_joints = MotionJoints::from_names(&joint_names);
        let foot_joints = FootJoints::from_names(&joint_names);
        let parents: Vec<Option<usize>> = skeleton
            .joints
            .iter()
            .map(|j| j.parent.map(|p| p as usize))
            .collect();
        let order = crate::ik::parent_first_order(&parents);
        let mut map = HashMap::with_capacity(clips.len());
        for c in clips {
            map.insert(c.name.clone(), c.clone());
        }
        let events = ClipEventTable::from_config_filtered(&config, |name| map.contains_key(name));
        let layers = LayerStack::new(&config);
        Self {
            config,
            clips: map,
            masks,
            layers,
            pool: PosePool::new(n, POSE_POOL_BUFFERS),
            parents,
            order,
            weapon_name: None,
            two_handed: false,
            off_hand_grip: None,
            stance: StanceController::new(),
            aim: AimController::new(),
            aim_joints,
            motion: MotionState::new(),
            motion_joints,
            air: AirState::new(),
            air_driven: false,
            posture: PostureState::new(),
            swim: SwimState::new(),
            ladder: LadderAnimState::new(),
            car: CarAnimState::new(),
            incline: InclineState::new(),
            foot_ik: FootIkState::new(),
            foot_joints,
            foot_globals: vec![Mat4::IDENTITY; n],
            blend_pos: 0.0,
            tempo_scale: 1.0, // new: DECISIONS 2026-11-23
            time_idle: 0.0,
            action: None,
            full_body: None,
            events,
            event_queue: EventQueue::default(),
            dying: false,
            held_pose: None,
            pose_time: 0.0,
            update_interval: 0.0,
            accumulated_dt: 0.0,
            dirty: true,
            cached_skin: vec![Mat4::IDENTITY; n],
        }
    }

    /// Sets the equipped weapon model name (e.g. "bat", "knife", "pistol").
    pub fn set_weapon(&mut self, weapon: Option<&str>) {
        let weapon_changed = self.weapon_name.as_deref() != weapon;
        let had_stance_aim = self.stance.aim_clip().is_some();
        let mut aim_class = None;
        if let Some(name) = weapon {
            if weapon_changed {
                self.weapon_name = Some(name.to_string());
            }
            self.two_handed = crate::weapon::is_two_handed(name);
            self.off_hand_grip = crate::weapon::off_hand_grip_for(name);
            let def = StanceTable::load_shared().for_weapon(name);
            aim_class = def.map(|d| d.class);
            self.stance.set_weapon(def);
        } else {
            self.weapon_name = None;
            self.two_handed = false;
            self.off_hand_grip = None;
            self.stance.set_weapon(None);
        }
        if weapon_changed {
            if self.aim.is_reloading() {
                self.action = None;
                self.layers.get_mut(LayerId::Action).weight = 0.0;
                self.layers.set_clip(LayerId::Action, None);
            }
            self.aim.set_class(aim_class);
        }

        let next_ready = self.stance.ready_clip();
        let ready_changed = self.layers.get(LayerId::Stance).clip.as_deref() != next_ready;
        self.layers.set_clip(LayerId::Stance, next_ready);
        let st = self.layers.get_mut(LayerId::Stance);
        st.mask = MaskId::Upper;
        st.weight = self.stance.ready_weight();
        st.target = st.weight;
        if ready_changed {
            st.time = 0.0; // new: DECISIONS 2026-11-23
        }
        if had_stance_aim || self.stance.aim_clip().is_some() {
            self.layers.set_clip(LayerId::Aim, self.stance.aim_clip());
            let aim = self.layers.get_mut(LayerId::Aim);
            aim.mask = MaskId::Upper;
            aim.weight = self.stance.aim_weight();
            aim.target = aim.weight;
        }
        self.dirty = true;
    }

    /// Sets whether the character is aiming the currently equipped weapon.
    pub fn set_aiming(&mut self, aiming: bool) {
        self.stance.set_aiming(aiming);
        self.dirty = true;
    }

    /// Sets camera look pitch in radians (positive looks up).
    pub fn set_look_pitch(&mut self, radians: f32) {
        self.aim.set_look_pitch(radians);
        self.dirty = true;
    }

    /// Triggers weapon recoil kick for a fired shot.
    pub fn on_gun_fired(&mut self) {
        self.aim.fire();
        self.dirty = true;
    }

    /// Starts the active weapon class's reload clip scaled to last `seconds`.
    pub fn on_reload_started(&mut self, seconds: f32) {
        let clips = &self.clips;
        let started = self
            .aim
            .start_reload(seconds, |name| clips.get(name).map(|c| c.duration))
            .map(|(clip, speed)| (clip.to_owned(), speed));
        if let Some((clip, speed)) = started {
            self.play_action(&clip, speed);
        }
    }

    /// Returns the blended weapon hold transform when a stance is active, or `None` for legacy hold.
    pub fn weapon_hold(&self) -> Option<HoldTransform> {
        self.stance.ready_clip().map(|_| self.stance.hold())
    }

    /// Explicitly sets two-handed mode and off-hand grip offset.
    pub fn set_two_handed(&mut self, two_handed: bool, off_hand_grip: Option<Vec3>) {
        self.two_handed = two_handed;
        self.off_hand_grip = off_hand_grip;
        self.dirty = true;
    }

    pub fn is_two_handed(&self) -> bool { self.two_handed }
    pub fn off_hand_grip(&self) -> Option<Vec3> { self.off_hand_grip }

    /// Advance blends, one-shots, full-body overrides, events and animation clocks by `dt` seconds.
    pub fn update(&mut self, dt: f32, locomotion: Locomotion) {
        if !dt.is_finite() || dt < 0.0 {
            return;
        }
        if self.update_interval > 0.0 {
            self.accumulated_dt += dt;
            if self.accumulated_dt < self.update_interval {
                return;
            }
        }
        let step = if self.update_interval > 0.0 {
            let s = self.accumulated_dt;
            self.accumulated_dt = 0.0;
            s
        } else {
            dt
        };

        self.dirty = true;
        if self.held_pose.is_some() {
            self.foot_ik.set_active(false);
            self.foot_ik.tick(step, self.blend_pos);
            self.pose_time += step;
            return;
        }
        if self.dying {
            self.foot_ik.set_active(false);
            self.foot_ik.tick(step, self.blend_pos);
            self.advance_full_body(step);
            return;
        }

        let k = 1.0 - (-self.config.blend_rate * step).exp();
        self.blend_pos += (locomotion.speed.clamp(0.0, RUN_BLEND) - self.blend_pos) * k;
        let car_event = self.car.tick(step);
        let in_car = self.car.is_seated();
        let (on_ladder, ladder_event) = if in_car {
            (false, None)
        } else {
            self.car.release_layers(&mut self.layers);
            if let Some(event) = car_event {
                self.start_car_event(event);
            }
            let ev = self.ladder.tick(step);
            self.incline.tick(step);
            (self.ladder.is_on_ladder(), ev)
        };
        if in_car {
            self.ladder.release_air(&mut self.layers);
            self.swim.release_air(&mut self.layers);
            self.incline.release_air(&mut self.layers);
            self.air = AirState::new();
            self.car.apply_car(&mut self.layers);
        } else if on_ladder {
            self.swim.release_air(&mut self.layers);
            self.incline.release_air(&mut self.layers);
            self.air = AirState::new();
            let clip_len = self.clips.get(LADDER_CLIMB_CLIP).map_or(0.0, |c| c.duration);
            self.ladder.apply_ladder(&mut self.layers, clip_len);
            self.ladder.suppress_upper(&mut self.layers);
        } else {
            self.ladder.release_air(&mut self.layers);
            let climb_out = self.swim.tick(step);
            if self.swim.is_swimming() {
                self.incline.release_air(&mut self.layers);
                self.swim.apply_swim(&mut self.layers);
            } else {
                let (air_target, air_clip, landing) = if self.air_driven {
                    let landing = self.air.tick(step, locomotion.on_floor);
                    (self.air.air_target(), self.air.air_clip().unwrap_or("jump"), landing)
                } else {
                    (if locomotion.on_floor { 0.0 } else { 1.0 }, "jump", None)
                };
                let air_free = locomotion.on_floor && air_target <= LAYER_EPSILON;
                let posture_busy = locomotion.crouching
                    || self.posture.crouch_walk_weight() > LAYER_EPSILON
                    || self.posture.sprint_weight() > LAYER_EPSILON;
                if air_free && self.swim.wants_wade() {
                    self.incline.release_air(&mut self.layers);
                    self.swim.apply_wade(&mut self.layers);
                } else if air_free
                    && self.full_body.is_none()
                    && !posture_busy
                    && self.incline.wants_incline()
                {
                    self.swim.release_air(&mut self.layers);
                    self.incline.apply_air(&mut self.layers);
                } else {
                    self.swim.release_air(&mut self.layers);
                    self.incline.release_air(&mut self.layers);
                    let layer = self.layers.get_mut(LayerId::Air);
                    layer.target = air_target;
                    if self.air_driven && layer.clip.as_deref() != Some(air_clip) {
                        layer.clip = Some(air_clip.to_owned());
                        layer.time = 0.0;
                    }
                }
                if let Some(kind) = landing {
                    self.start_landing(kind);
                }
            }
            if climb_out {
                self.start_climb_out();
            }
        }
        if let Some(event) = ladder_event {
            self.start_ladder_event(event);
        }
        if !in_car {
            self.layers.get_mut(LayerId::Crouch).target =
                if !on_ladder && locomotion.crouching { 1.0 } else { 0.0 };
        }
        self.motion.tick(step, locomotion.on_floor && !on_ladder && !in_car);
        let tempo = tempo_for(self.blend_pos) * self.tempo_scale * self.motion.tempo_scale();
        self.layers.get_mut(LayerId::Locomotion).speed = tempo;
        self.layers.advance(step);
        self.time_idle += step;

        self.aim.tick(step);
        self.advance_action(step);
        if !on_ladder && !in_car {
            self.advance_stance(step);
            self.advance_posture(step);
        }
        self.advance_full_body(step);
        if self.full_body.is_some()
            || self.posture.crouch_walk_weight() > LAYER_EPSILON
            || self.posture.sprint_weight() > LAYER_EPSILON
        {
            self.incline.release_air(&mut self.layers);
        }
        self.foot_ik.set_active(
            locomotion.on_floor
                && !in_car
                && !on_ladder
                && !self.swim.is_swimming()
                && self.full_body.is_none(),
        );
        self.foot_ik.tick(step, self.blend_pos);
    }

    /// The body's velocity and facing this frame, for side and backward steps and leaning.
    pub fn set_move_velocity(&mut self, velocity: Vec3, yaw: f32) {
        self.motion.set_input(velocity, yaw);
        self.air.set_velocity(velocity);
        self.air_driven = true;
        self.dirty = true;
    }

    /// Feeds crouch, sprint and stamina so the posture layers can react (third person only).
    pub fn set_posture(&mut self, speed: f32, crouching: bool, sprinting: bool, stamina_frac: f32) {
        self.posture.set_input(speed, crouching, sprinting, stamina_frac);
        self.dirty = true;
    }

    /// Feeds lake depth, deep-water flag, horizontal speed and stamina.
    pub fn set_water(&mut self, depth: f32, swimming: bool, speed: f32, stamina_frac: f32) {
        self.swim.set_input(depth, swimming, speed, stamina_frac);
        self.dirty = true;
    }

    /// Feeds ladder climb activity, on-rungs flag and player height.
    pub fn set_ladder(&mut self, active: bool, on_rungs: bool, y: f32) {
        self.ladder.set_input(active, on_rungs, y);
        self.dirty = true;
    }

    /// Feeds world position and grounded flag for stairs and steep-slope leg blends.
    pub fn set_incline(&mut self, position: Vec3, on_floor: bool) {
        self.incline.set_input(position, on_floor);
        self.dirty = true;
    }

    /// Feeds car driving state: seated flag, steer angle (rad) and flipped flag.
    pub fn set_car(&mut self, seated: bool, steer: f32, flipped: bool) {
        self.car.set_input(seated, steer, flipped);
        self.dirty = true;
    }

    /// Starts a one-shot action on the upper body.
    pub fn play_action(&mut self, clip_name: &str, speed: f32) {
        if let Some(clip) = self.clips.get(clip_name) {
            self.action = Some(ActionState::with_fades(
                clip_name.to_string(),
                clip.duration,
                speed,
                self.config.action_fade_in,
                self.config.action_fade_out,
            ));
            self.layers.set_clip(LayerId::Action, Some(clip_name));
            self.layers.get_mut(LayerId::Action).weight = 0.0;
            self.dirty = true;
        }
    }

    pub fn play_hit(&mut self) {
        self.play_action("hit", 1.0); // godot: HumanoidAnimTree.cs:152
    }

    pub fn set_aim_pose(&mut self, clip_name: Option<&str>) {
        if let Some(name) = clip_name {
            self.layers.set_clip(LayerId::Aim, Some(name));
            self.layers.get_mut(LayerId::Aim).target = 1.0;
        } else {
            self.layers.get_mut(LayerId::Aim).target = 0.0;
        }
        self.dirty = true;
    }

    /// Plays a full-body clip above all other layers; returns `false` if the clip does not exist.
    pub fn play_full_body(&mut self, clip: &str, opts: FullBodyOptions) -> bool {
        let Some(c) = self.clips.get(clip) else {
            return false;
        };
        let duration = c.duration;
        let prev = self.full_body.take();
        let state = FullBodyState::start(clip.to_owned(), duration, opts, prev);
        self.layers.set_clip(LayerId::FullBody, Some(clip));
        self.layers.get_mut(LayerId::FullBody).mask = MaskId::Full;
        self.layers.get_mut(LayerId::FullBody).weight = state.weight();
        self.full_body = Some(state);
        self.dirty = true;
        true
    }

    /// Starts the landing clip for `kind` unless dying, held or already in another full-body clip.
    fn start_landing(&mut self, kind: LandingKind) {
        if self.dying || self.held_pose.is_some() || self.full_body.is_some() {
            return;
        }
        let (fade_in, fade_out) = self.air.landing_fades();
        let opts = FullBodyOptions { speed: 1.0, looping: false, hold_last: false, fade_in, fade_out };
        if !self.play_full_body(landing_clip(kind), opts) {
            return;
        }
        // A soft landing only keys legs and hips, so keep the weapon arms on the stance.
        let mask = if kind == LandingKind::Soft { MaskId::Lower } else { MaskId::Full };
        self.layers.get_mut(LayerId::FullBody).mask = mask;
    }

    /// Plays the climb-out clip once on leaving deep water unless dying, held or busy.
    fn start_climb_out(&mut self) {
        if self.dying || self.held_pose.is_some() || self.full_body.is_some() {
            return;
        }
        let speed = self
            .clips
            .get(SWIM_CLIMB_OUT_CLIP)
            .map_or(1.0, |c| self.swim.climb_out_speed(c.duration));
        let (fade_in, fade_out) = self.swim.climb_out_fades();
        let opts = FullBodyOptions { speed, looping: false, hold_last: false, fade_in, fade_out };
        self.play_full_body(SWIM_CLIMB_OUT_CLIP, opts);
    }

    /// Plays a ladder enter/exit clip unless dying, held or blocked by a non-ladder override.
    fn start_ladder_event(&mut self, event: LadderEvent) {
        if self.dying || self.held_pose.is_some() {
            return;
        }
        if let Some(fb) = &self.full_body
            && !is_ladder_clip(fb.clip_name())
        {
            return;
        }
        let clip_name = ladder_event_clip(event);
        let Some(duration) = self.clips.get(clip_name).map(|c| c.duration) else {
            return;
        };
        let speed = self.ladder.event_speed(event, duration);
        let fade = self.ladder.fade();
        let opts = FullBodyOptions {
            speed,
            looping: false,
            hold_last: false,
            fade_in: fade,
            fade_out: fade,
        };
        self.play_full_body(clip_name, opts);
    }

    /// Plays the car exit or thrown-out clip once unless dying or in a held pose.
    fn start_car_event(&mut self, event: CarEvent) {
        if self.dying || self.held_pose.is_some() {
            return;
        }
        let fade = self.car.event_fade(event);
        let opts = FullBodyOptions {
            speed: 1.0, // new: DECISIONS 2026-11-24
            looping: false,
            hold_last: false,
            fade_in: fade,
            fade_out: fade,
        };
        self.play_full_body(car_event_clip(event), opts);
    }

    /// Smoothly stops the active full-body clip using `fade_out` (or the clip's configured default).
    pub fn stop_full_body(&mut self, fade_out: Option<f32>) {
        if let Some(fb) = &mut self.full_body {
            if fb.begin_stop(fade_out) {
                self.full_body = None;
                self.layers.get_mut(LayerId::FullBody).weight = 0.0;
                self.layers.set_clip(LayerId::FullBody, None);
            }
            self.dirty = true;
        }
    }

    pub fn is_full_body_active(&self) -> bool { self.full_body.is_some() }

    pub fn full_body_progress(&self) -> f32 {
        self.full_body.as_ref().map_or(0.0, FullBodyState::progress)
    }

    /// Moves all queued animation events into `out`.
    pub fn drain_events(&mut self, out: &mut Vec<ClipEvent>) {
        self.event_queue.drain_into(out);
    }

    pub fn play_die(&mut self) {
        self.dying = true;
        let opts = FullBodyOptions {
            speed: 1.0, // new: DECISIONS 2026-11-23
            looping: false,
            hold_last: true,
            fade_in: self.config.die_fade_in, // data: animation.json dieFadeIn
            fade_out: 0.0, // new: DECISIONS 2026-11-23
        };
        self.play_full_body("die", opts);
        self.dirty = true;
    }

    pub fn reset_alive(&mut self) {
        self.dying = false;
        if self.full_body.as_ref().is_some_and(|fb| fb.clip_name() == "die") {
            self.full_body = None;
            self.layers.get_mut(LayerId::FullBody).weight = 0.0;
            self.layers.set_clip(LayerId::FullBody, None);
        }
        self.dirty = true;
    }

    pub fn hold_pose(&mut self, clip_name: &str) {
        if self.held_pose.as_deref() != Some(clip_name) {
            self.held_pose = Some(clip_name.to_string());
        }
        self.pose_time = 0.0;
        self.dirty = true;
    }

    pub fn release_pose(&mut self) { self.held_pose = None; self.dirty = true; }
    pub fn set_tempo_scale(&mut self, scale: f32) { self.tempo_scale = scale.max(MIN_TEMPO_SCALE); }

    /// Multi-tiered LOD update throttling from `HumanoidVisual.SetLod`.
    pub fn set_lod(&mut self, distance: f32, on_screen: bool) {
        self.set_update_interval(lod_interval(distance, on_screen));
    }

    pub fn set_update_interval(&mut self, interval: f32) {
        self.update_interval = interval.max(0.0);
        if self.update_interval == 0.0 {
            self.accumulated_dt = 0.0;
        }
    }

    pub fn update_interval(&self) -> f32 { self.update_interval }
    pub fn blend_position(&self) -> f32 { self.blend_pos }
    pub fn is_action_active(&self) -> bool { self.action.as_ref().is_some_and(|a| !a.is_finished()) }
    pub fn current_action(&self) -> Option<&str> { self.action.as_ref().map(|a| a.clip_name.as_str()) }
    pub fn action_state(&self) -> Option<&ActionState> { self.action.as_ref() }
    pub fn is_dying(&self) -> bool { self.dying }
    pub fn layers(&self) -> &LayerStack { &self.layers }
    pub fn layers_mut(&mut self) -> &mut LayerStack { self.dirty = true; &mut self.layers }
    pub fn foot_ik_mut(&mut self) -> &mut FootIkState { self.dirty = true; &mut self.foot_ik }
    pub fn event_table_mut(&mut self) -> &mut ClipEventTable { &mut self.events }

    /// Writes character skin matrices into `out`.
    pub fn write_skin(&mut self, rig: &Rig, out: &mut [Mat4]) {
        if self.dirty {
            self.recompute_skin(rig);
            self.dirty = false;
        }
        let n = self.cached_skin.len().min(out.len());
        out[..n].copy_from_slice(&self.cached_skin[..n]);
    }

    fn advance_action(&mut self, step: f32) {
        let Some(act) = &mut self.action else {
            return;
        };
        let clip_dur = self.clips.get(&act.clip_name).map_or(0.0, |c| c.duration);
        let prev = act.time * act.speed;
        let just_started = act.time == 0.0;
        act.time += step;
        let next = act.time * act.speed;
        if step > 0.0 && clip_dur > 0.0 {
            let span = EventSpan { prev, next, duration: clip_dur, just_started };
            self.events.collect_non_looping(&act.clip_name, span, &mut self.event_queue);
        }
        if act.is_finished() {
            self.action = None;
            self.layers.get_mut(LayerId::Action).weight = 0.0;
            self.layers.set_clip(LayerId::Action, None);
        } else {
            self.layers.get_mut(LayerId::Action).weight = act.weight();
        }
    }

    /// Advances the posture weights and writes them into the existing layers.
    fn advance_posture(&mut self, step: f32) {
        let armed = self.weapon_name.is_some();
        self.posture.tick(step);
        self.posture.apply(&mut self.layers, armed);
    }

    fn advance_stance(&mut self, step: f32) {
        let table = StanceTable::load_shared();
        let rates = (table.ready_rate(), table.aim_rate(), table.ik_rate());
        let is_melee_2h = self.action.as_ref().is_some_and(|a| a.clip_name == "melee_2h");
        let action_active = self.is_action_active() && !is_melee_2h;
        self.stance.tick(step, rates, action_active);

        self.layers.set_clip(LayerId::Stance, self.stance.ready_clip());
        let st = self.layers.get_mut(LayerId::Stance);
        st.mask = MaskId::Upper;
        st.weight = self.stance.ready_weight();
        st.target = st.weight;

        if let Some(aim_clip) = self.stance.aim_clip() {
            self.layers.set_clip(LayerId::Aim, Some(aim_clip));
            let aim = self.layers.get_mut(LayerId::Aim);
            aim.mask = MaskId::Upper;
            aim.weight = self.stance.aim_weight();
            aim.target = aim.weight;
        }
    }

    fn advance_full_body(&mut self, step: f32) {
        let Some(fb) = &mut self.full_body else {
            return;
        };
        let res = fb.advance(step);
        if step > 0.0 && res.duration > 0.0 {
            let span = EventSpan {
                prev: res.prev_time,
                next: res.raw_next_time,
                duration: res.duration,
                just_started: res.just_started,
            };
            if res.looping {
                self.events.collect_looping(fb.clip_name(), span, &mut self.event_queue);
            } else {
                self.events.collect_non_looping(fb.clip_name(), span, &mut self.event_queue);
            }
        }
        if res.finished {
            self.full_body = None;
            self.layers.get_mut(LayerId::FullBody).weight = 0.0;
            self.layers.set_clip(LayerId::FullBody, None);
        } else {
            self.layers.get_mut(LayerId::FullBody).weight = fb.weight();
        }
    }

    fn recompute_skin(&mut self, rig: &Rig) {
        let n = rig.joint_count();
        if self.cached_skin.len() != n {
            self.cached_skin.resize(n, Mat4::IDENTITY);
        }
        if self.foot_globals.len() != n {
            self.foot_globals.resize(n, Mat4::IDENTITY);
        }
        self.pool.resize(n);

        if let Some(pose_name) = &self.held_pose
            && let Some(c) = self.clips.get(pose_name)
        {
            let pose = self.pool.get_mut(0);
            rig.local_pose_into(c, self.pose_time, pose);
            rig.skin(pose, &mut self.cached_skin);
            return;
        }

        // 1. Base locomotion
        {
            let (base, scratch) = self.pool.pair_mut(0, 1);
            sample_locomotion_into(
                rig,
                LocomotionClips {
                    idle: self.clips.get("idle"),
                    walk: self.clips.get("walk"),
                    run: self.clips.get("run"),
                },
                self.blend_pos,
                self.time_idle,
                self.layers.get(LayerId::Locomotion).time,
                base,
                scratch,
            );
        }
        // 1b. Side, backward and turning steps on the legs
        {
            let run_fraction = locomotion_weights(self.blend_pos).run;
            let time = self.layers.get(LayerId::Locomotion).time;
            let clips = MotionClips::from_map(&self.clips);
            let lower = self.masks.weights(MaskId::Lower);
            let (base, scratch) = self.pool.pair_mut(0, 1);
            self.motion.apply_legs(rig, &clips, time, run_fraction, lower, base, scratch);
        }
        // 2. Air -> 3. Crouch -> 4. Stance -> 5. Aim
        for id in [LayerId::Air, LayerId::Crouch, LayerId::Stance, LayerId::Aim] {
            self.apply_layer(rig, id);
        }
        // 6. Action or procedural swing
        self.apply_action_stage(rig);
        // 7. Additive layer
        self.apply_layer(rig, LayerId::Additive);
        // 7b. Aim pitch and recoil (before left-hand IK so off-hand follows the gun)
        if !self.ladder.is_on_ladder() && !self.car.is_seated() {
            self.aim.apply(&self.aim_joints, self.pool.get_mut(0), self.stance.aim_weight());
            self.motion.apply_lean(&self.motion_joints, self.pool.get_mut(0));
        }
        // 7c. Foot IK (before left-hand IK so off-hand follows when hips drop)
        if let Some(fj) = self.foot_joints {
            self.foot_ik.apply(
                &fj,
                &self.parents,
                &self.order,
                self.pool.get_mut(0),
                &mut self.foot_globals,
            );
        }
        // 8. Left-hand IK
        if !self.ladder.is_on_ladder() && !self.car.is_seated() {
            self.apply_ik_stage(rig);
        }
        // 9. Full-body override (including death fade)
        self.apply_full_body_stage(rig);
        // 10. Skin
        rig.skin(self.pool.get_mut(0), &mut self.cached_skin);
    }

    fn apply_layer(&mut self, rig: &Rig, id: LayerId) {
        let layer = self.layers.get(id);
        if layer.weight <= LAYER_EPSILON {
            return;
        }
        let Some(clip_name) = layer.clip.as_deref() else { return };
        let Some(clip) = self.clips.get(clip_name) else { return };
        let (weight, time, blend) = (layer.weight, layer.time, layer.blend);
        let mask = self.masks.weights(layer.mask);
        let (base, scratch) = self.pool.pair_mut(0, 1);
        rig.local_pose_into(clip, time, scratch);
        match blend {
            LayerBlend::Override => blend_pose_masked(base, scratch, weight, mask),
            LayerBlend::Additive => add_pose_masked(base, scratch, rig.rest_pose(), weight, mask),
        }
    }

    fn apply_action_stage(&mut self, rig: &Rig) {
        let Some(act) = &self.action else { return };
        let w = act.weight();
        if act.clip_name == "melee_2h" {
            let t = (act.time * act.speed).clamp(0.0, SWING_MAX_TIME);
            let swing_w = if act.time < act.fade_in { 1.0 } else { w };
            if swing_w > LAYER_EPSILON {
                crate::swing::apply_two_hand_swing_pose(rig, self.pool.get_mut(0), t, swing_w);
            }
        } else if w > LAYER_EPSILON
            && let Some(ac) = self.clips.get(&act.clip_name)
        {
            let mask = self.masks.weights(self.layers.get(LayerId::Action).mask);
            let (base, scratch) = self.pool.pair_mut(0, 1);
            rig.local_pose_into(ac, act.time * act.speed, scratch);
            blend_pose_masked(base, scratch, w, mask);
        }
    }

    fn apply_ik_stage(&mut self, rig: &Rig) {
        let is_swinging = self.action.as_ref().is_some_and(|a| a.clip_name == "melee_2h");
        let is_two_handed_active = self.two_handed
            || self.stance.off_hand_grip().is_some()
            || (self.weapon_name.is_none() && is_swinging);
        if is_two_handed_active && is_swinging {
            let grip_offset = self
                .off_hand_grip
                .or_else(|| self.stance.off_hand_grip())
                .unwrap_or(DEFAULT_OFF_HAND_GRIP);
            let (hold_pos, hold_rot) = if let Some(hold) = self.weapon_hold() {
                (hold.pos, hold.rot_deg)
            } else {
                let wname = self.weapon_name.as_deref().unwrap_or("bat");
                crate::weapon::weapon_hold_for(wname)
            };
            crate::ik::apply_two_hand_left_grip_ik(
                &self.parents,
                &self.order,
                self.pool.get_mut(0),
                rig,
                hold_pos,
                hold_rot,
                grip_offset,
            );
            return;
        }

        let ik_weight = if self.aim.is_reloading() {
            0.0 // new: DECISIONS 2026-11-23
        } else {
            self.stance.left_ik_weight()
        };
        if ik_weight > LAYER_EPSILON
            && let Some(grip_offset) = self.stance.off_hand_grip()
        {
            let hold = self.stance.hold();
            if ik_weight >= 1.0 - LAYER_EPSILON {
                // new: DECISIONS 2026-11-23
                crate::ik::apply_two_hand_left_grip_ik(
                    &self.parents,
                    &self.order,
                    self.pool.get_mut(0),
                    rig,
                    hold.pos,
                    hold.rot_deg,
                    grip_offset,
                );
            } else {
                let mask = self.masks.weights(MaskId::LeftArm);
                let (base, scratch) = self.pool.pair_mut(0, 1);
                scratch.copy_from_slice(base);
                crate::ik::apply_two_hand_left_grip_ik(
                    &self.parents,
                    &self.order,
                    scratch,
                    rig,
                    hold.pos,
                    hold.rot_deg,
                    grip_offset,
                );
                blend_pose_masked(base, scratch, ik_weight, mask);
            }
            return;
        }

        let aim_amount = self.layers.get(LayerId::Aim).weight;
        if self.two_handed
            && !self.aim.is_reloading()
            && self.stance.off_hand_grip().is_none()
            && aim_amount > LAYER_EPSILON
        {
            let grip_offset = self.off_hand_grip.unwrap_or(DEFAULT_OFF_HAND_GRIP);
            let wname = self.weapon_name.as_deref().unwrap_or("bat");
            let (hold_pos, hold_rot) = crate::weapon::weapon_hold_for(wname);
            crate::ik::apply_two_hand_left_grip_ik(
                &self.parents,
                &self.order,
                self.pool.get_mut(0),
                rig,
                hold_pos,
                hold_rot,
                grip_offset,
            );
        }
    }

    fn apply_full_body_stage(&mut self, rig: &Rig) {
        let Some(fb) = &self.full_body else { return };
        let weight = fb.weight();
        if weight <= LAYER_EPSILON {
            return;
        }
        let Some(curr_clip) = self.clips.get(fb.clip_name()) else { return };
        let mask = self.masks.weights(self.layers.get(LayerId::FullBody).mask);
        let curr_t = fb.sample_time();
        if let Some(prev) = fb.prev()
            && let Some(prev_clip) = self.clips.get(prev.clip_name())
        {
            let (alpha, prev_t) = (prev.alpha(), prev.sample_time());
            let (base, curr_buf, prev_buf) = self.pool.triple_mut();
            rig.local_pose_into(curr_clip, curr_t, curr_buf);
            rig.local_pose_into(prev_clip, prev_t, prev_buf);
            blend_pose_masked(prev_buf, curr_buf, alpha, mask);
            blend_pose_masked(base, prev_buf, weight, mask);
        } else {
            let (base, fb_buf) = self.pool.pair_mut(0, 1);
            rig.local_pose_into(curr_clip, curr_t, fb_buf);
            blend_pose_masked(base, fb_buf, weight, mask);
        }
    }
}
