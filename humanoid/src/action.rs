//! One-shot action layering and fade management.

use crate::blend::{ACTION_FADE_IN, ACTION_FADE_OUT};

/// An active one-shot action (e.g. melee attack, gun fire, reload, flinch).
#[derive(Clone, Debug, PartialEq)]
pub struct ActionState {
    pub clip_name: String,
    pub time: f32,
    pub duration: f32,
    pub speed: f32,
    pub fade_in: f32,
    pub fade_out: f32,
}

impl ActionState {
    pub fn new(clip_name: String, clip_duration: f32, speed: f32) -> Self {
        let sp = speed.max(0.05);
        let duration = if clip_duration > 0.0 { clip_duration / sp } else { 0.5 };
        Self {
            clip_name,
            time: 0.0,
            duration,
            speed: sp,
            fade_in: ACTION_FADE_IN,
            fade_out: ACTION_FADE_OUT,
        }
    }

    /// Blend weight in 0..=1 accounting for fade in and fade out.
    pub fn weight(&self) -> f32 {
        if self.time < 0.0 || self.time >= self.duration {
            return 0.0;
        }
        let in_w = if self.fade_in > 0.0 { (self.time / self.fade_in).min(1.0) } else { 1.0 };
        let out_w = if self.fade_out > 0.0 {
            ((self.duration - self.time) / self.fade_out).min(1.0)
        } else {
            1.0
        };
        in_w.min(out_w).clamp(0.0, 1.0)
    }

    pub fn is_finished(&self) -> bool {
        self.time >= self.duration
    }
}
