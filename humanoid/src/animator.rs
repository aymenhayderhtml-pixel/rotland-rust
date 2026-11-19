//! Humanoid skeletal animator and state machine.
//!
//! Blends idle/walk/run, air and crouch, layers one-shot actions and aim hold
//! onto the upper body, and evaluates skin matrices via `engine::anim::Rig`.

use std::collections::HashMap;

use engine::anim::Rig;
use glam::{Mat4, Vec3};
use pack::{Clip, Skeleton};

use crate::action::ActionState;
use crate::blend::{
    blend_local_poses, lod_interval, sample_locomotion, tempo_for, Locomotion, BLEND_RATE,
    RUN_BLEND, UPPER_BONES,
};

/// Per-character humanoid animation state machine.
pub struct HumanoidAnimator {
    clips: HashMap<String, Clip>,
    is_upper: Vec<bool>,
    parents: Vec<Option<usize>>,
    order: Vec<usize>,
    weapon_name: Option<String>,
    two_handed: bool,
    off_hand_grip: Option<Vec3>,
    blend_pos: f32,
    air_amount: f32,
    crouch_amount: f32,
    aim_amount: f32,
    aim_target: f32,
    aim_clip: Option<String>,
    tempo_scale: f32,
    time_loco: f32,
    time_idle: f32,
    time_air: f32,
    time_crouch: f32,
    time_aim: f32,
    action: Option<ActionState>,
    dying: bool,
    die_time: f32,
    held_pose: Option<String>,
    pose_time: f32,
    update_interval: f32,
    accumulated_dt: f32,
    dirty: bool,
    cached_skin: Vec<Mat4>,
}

impl HumanoidAnimator {
    pub fn new(skeleton: &Skeleton, clips: &[Clip]) -> Self {
        let n = skeleton.joints.len();
        let is_upper = skeleton
            .joints
            .iter()
            .map(|j| UPPER_BONES.iter().any(|&b| b == j.name))
            .collect();
        let parents: Vec<Option<usize>> = skeleton
            .joints
            .iter()
            .map(|j| j.parent.map(|p| p as usize))
            .collect();
        let order = crate::ik::parent_first_order(&parents);
        let mut map = HashMap::new();
        for c in clips {
            map.insert(c.name.clone(), c.clone());
        }
        Self {
            clips: map, is_upper, parents, order,
            weapon_name: None, two_handed: false, off_hand_grip: None,
            blend_pos: 0.0, air_amount: 0.0, crouch_amount: 0.0,
            aim_amount: 0.0, aim_target: 0.0, aim_clip: None,
            tempo_scale: 1.0, time_loco: 0.0, time_idle: 0.0,
            time_air: 0.0, time_crouch: 0.0, time_aim: 0.0,
            action: None, dying: false, die_time: 0.0,
            held_pose: None, pose_time: 0.0,
            update_interval: 0.0, accumulated_dt: 0.0,
            dirty: true, cached_skin: vec![Mat4::IDENTITY; n],
        }
    }

    /// Sets the equipped weapon model name (e.g. "bat", "knife", "pistol").
    pub fn set_weapon(&mut self, weapon: Option<&str>) {
        if let Some(name) = weapon {
            self.weapon_name = Some(name.to_string());
            self.two_handed = crate::weapon::is_two_handed(name);
            self.off_hand_grip = crate::weapon::off_hand_grip_for(name);
        } else {
            self.weapon_name = None;
            self.two_handed = false;
            self.off_hand_grip = None;
        }
        self.dirty = true;
    }

    /// Explicitly sets two-handed mode and off-hand grip offset.
    pub fn set_two_handed(&mut self, two_handed: bool, off_hand_grip: Option<Vec3>) {
        self.two_handed = two_handed;
        self.off_hand_grip = off_hand_grip;
        self.dirty = true;
    }

    pub fn is_two_handed(&self) -> bool {
        self.two_handed
    }

    pub fn off_hand_grip(&self) -> Option<Vec3> {
        self.off_hand_grip
    }

    /// Advance blends, one-shots and animation clocks by `dt` seconds.
    pub fn update(&mut self, dt: f32, locomotion: Locomotion) {
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
        if self.dying {
            self.die_time += step;
            return;
        }
        if self.held_pose.is_some() {
            self.pose_time += step;
            return;
        }

        let k = 1.0 - (-BLEND_RATE * step).exp();
        self.blend_pos += (locomotion.speed.clamp(0.0, RUN_BLEND) - self.blend_pos) * k;
        let target_air = if locomotion.on_floor { 0.0 } else { 1.0 };
        self.air_amount += (target_air - self.air_amount) * k;
        let target_crouch = if locomotion.crouching { 1.0 } else { 0.0 };
        self.crouch_amount += (target_crouch - self.crouch_amount) * k;
        self.aim_amount += (self.aim_target - self.aim_amount) * k;

        let tempo = tempo_for(self.blend_pos) * self.tempo_scale;
        self.time_idle += step;
        self.time_loco += step * tempo;
        self.time_air += step;
        self.time_crouch += step;
        self.time_aim += step;

        if let Some(act) = &mut self.action {
            act.time += step;
            if act.is_finished() {
                self.action = None;
            }
        }
    }

    /// Starts a one-shot action on the upper body.
    pub fn play_action(&mut self, clip_name: &str, speed: f32) {
        if let Some(clip) = self.clips.get(clip_name) {
            self.action = Some(ActionState::new(clip_name.to_string(), clip.duration, speed));
            self.dirty = true;
        }
    }

    pub fn play_hit(&mut self) {
        self.play_action("hit", 1.0);
    }

    pub fn set_aim_pose(&mut self, clip_name: Option<&str>) {
        if let Some(name) = clip_name {
            self.aim_clip = Some(name.to_string());
            self.aim_target = 1.0;
        } else {
            self.aim_target = 0.0;
        }
        self.dirty = true;
    }

    pub fn play_die(&mut self) { self.dying = true; self.die_time = 0.0; self.dirty = true; }
    pub fn reset_alive(&mut self) { self.dying = false; self.dirty = true; }
    pub fn hold_pose(&mut self, clip_name: &str) { self.held_pose = Some(clip_name.to_string()); self.pose_time = 0.0; self.dirty = true; }
    pub fn release_pose(&mut self) { self.held_pose = None; self.dirty = true; }
    pub fn set_tempo_scale(&mut self, scale: f32) { self.tempo_scale = scale.max(0.05); }

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

    /// Writes character skin matrices into `out`.
    pub fn write_skin(&mut self, rig: &Rig, out: &mut [Mat4]) {
        if self.dirty {
            self.recompute_skin(rig);
            self.dirty = false;
        }
        let n = self.cached_skin.len().min(out.len());
        out[..n].copy_from_slice(&self.cached_skin[..n]);
    }

    fn recompute_skin(&mut self, rig: &Rig) {
        let n = rig.joint_count();
        if self.cached_skin.len() != n {
            self.cached_skin.resize(n, Mat4::IDENTITY);
        }

        if self.dying && let Some(c) = self.clips.get("die") {
            let pose = rig.local_pose(c, self.die_time.min(c.duration));
            rig.skin(&pose, &mut self.cached_skin);
            return;
        }
        if let Some(pose_name) = &self.held_pose && let Some(c) = self.clips.get(pose_name) {
            let pose = rig.local_pose(c, self.pose_time);
            rig.skin(&pose, &mut self.cached_skin);
            return;
        }

        let mut local = sample_locomotion(
            rig,
            self.clips.get("idle"),
            self.clips.get("walk"),
            self.clips.get("run"),
            self.blend_pos,
            self.time_idle,
            self.time_loco,
        );

        if self.air_amount > 0.001 && let Some(jc) = self.clips.get("jump") {
            let jp = rig.local_pose(jc, self.time_air);
            blend_local_poses(&mut local, &jp, self.air_amount, None);
        }
        if self.crouch_amount > 0.001 && let Some(cc) = self.clips.get("crouch") {
            let cp = rig.local_pose(cc, self.time_crouch);
            blend_local_poses(&mut local, &cp, self.crouch_amount, None);
        }

        if self.aim_amount > 0.001
            && let Some(name) = &self.aim_clip
            && let Some(ac) = self.clips.get(name)
        {
            let ap = rig.local_pose(ac, self.time_aim);
            blend_local_poses(&mut local, &ap, self.aim_amount, Some(&self.is_upper));
        }

        if let Some(act) = &self.action {
            let w = act.weight();
            if act.clip_name == "melee_2h" {
                let t = (act.time * act.speed).clamp(0.0, 0.70);
                let swing_w = if act.time < act.fade_in { 1.0 } else { w };
                if swing_w > 0.001 {
                    crate::swing::apply_two_hand_swing_pose(rig, &mut local, t, swing_w);
                }
            } else if w > 0.001 {
                if let Some(ac) = self.clips.get(&act.clip_name) {
                    let ap = rig.local_pose(ac, act.time * act.speed);
                    blend_local_poses(&mut local, &ap, w, Some(&self.is_upper));
                }
            }
        }

        let is_swinging = self.action.as_ref().is_some_and(|a| a.clip_name == "melee_2h");
        let is_two_handed_active = self.two_handed || (self.weapon_name.is_none() && is_swinging);
        if is_two_handed_active && (is_swinging || self.aim_amount > 0.001) {
            let grip_offset = self.off_hand_grip.unwrap_or(Vec3::new(0.0, 0.0, 0.10));
            let wname = self.weapon_name.as_deref().unwrap_or("bat");
            let (hold_pos, hold_rot) = crate::weapon::weapon_hold_for(wname);
            crate::ik::apply_two_hand_left_grip_ik(
                &self.parents,
                &self.order,
                &mut local,
                rig,
                hold_pos,
                hold_rot,
                grip_offset,
            );
        }

        rig.skin(&local, &mut self.cached_skin);
    }
}
