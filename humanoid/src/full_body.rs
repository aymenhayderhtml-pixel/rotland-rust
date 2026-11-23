//! Full-body animation override with smooth fade-in, fade-out, hold-last, and crossfading.

use crate::anim_config::AnimConfig;

const DEFAULT_SPEED: f32 = 1.0; // new: DECISIONS 2026-11-23
const MIN_PLAYBACK_SPEED: f32 = 0.05; // new: DECISIONS 2026-11-23
const HOLD_END_EPSILON: f32 = 1e-5; // new: DECISIONS 2026-11-23

/// Playback parameters for `HumanoidAnimator::play_full_body`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullBodyOptions {
    pub speed: f32,
    pub looping: bool,
    pub hold_last: bool,
    pub fade_in: f32,
    pub fade_out: f32,
}

impl FullBodyOptions {
    /// Builds default full-body options using timings from `cfg`.
    pub fn from_config(cfg: &AnimConfig) -> Self {
        Self {
            speed: DEFAULT_SPEED,
            looping: false,
            hold_last: false,
            fade_in: cfg.full_body_fade_in, // data: animation.json fullBodyFadeIn
            fade_out: cfg.full_body_fade_out, // data: animation.json fullBodyFadeOut
        }
    }
}

impl Default for FullBodyOptions {
    fn default() -> Self {
        Self::from_config(&AnimConfig::default())
    }
}

/// Lifecycle phase of an active full-body animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FullBodyPhase {
    FadingIn,
    Playing,
    Holding,
    FadingOut,
}

/// Previous full-body clip being crossfaded out into the current full-body clip.
#[derive(Clone, Debug)]
pub struct PrevFullBody {
    clip_name: String,
    clip_time: f32,
    duration: f32,
    speed: f32,
    looping: bool,
    elapsed: f32,
    fade_duration: f32,
}

impl PrevFullBody {
    pub fn clip_name(&self) -> &str {
        &self.clip_name
    }

    /// Crossfade progress in 0..=1 (0 = all previous clip, 1 = all current clip).
    pub fn alpha(&self) -> f32 {
        if self.fade_duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.fade_duration).clamp(0.0, 1.0)
        }
    }

    /// Sample timestamp for `Rig::local_pose_into` without wrapping the final keyframe.
    pub fn sample_time(&self) -> f32 {
        sample_time_for(self.clip_time, self.duration, self.looping)
    }
}

#[derive(Clone, Copy, Debug)]
struct StopFade {
    elapsed: f32,
    duration: f32,
    start_weight: f32,
}

/// Event span and completion flag returned by `FullBodyState::advance`.
#[derive(Clone, Copy, Debug)]
pub struct FullBodyStep {
    pub prev_time: f32,
    pub raw_next_time: f32,
    pub duration: f32,
    pub looping: bool,
    pub just_started: bool,
    pub finished: bool,
}

/// Active full-body animation state, including crossfade from a previous full-body clip.
#[derive(Clone, Debug)]
pub struct FullBodyState {
    clip_name: String,
    clip_time: f32,
    duration: f32,
    elapsed: f32,
    start_weight: f32,
    weight: f32,
    options: FullBodyOptions,
    phase: FullBodyPhase,
    just_started: bool,
    stop_fade: Option<StopFade>,
    prev: Option<PrevFullBody>,
}

impl FullBodyState {
    /// Starts a new full-body clip, optionally crossfading from `previous`.
    pub fn start(
        clip_name: String,
        duration: f32,
        mut options: FullBodyOptions,
        previous: Option<FullBodyState>,
    ) -> Self {
        options.speed = options.speed.max(MIN_PLAYBACK_SPEED);
        options.fade_in = options.fade_in.max(0.0);
        options.fade_out = options.fade_out.max(0.0);

        let (start_weight, prev) = if let Some(old) = previous {
            let prev_fade = if options.fade_in > 0.0 {
                Some(PrevFullBody {
                    clip_name: old.clip_name,
                    clip_time: old.clip_time,
                    duration: old.duration,
                    speed: old.options.speed,
                    looping: old.options.looping,
                    elapsed: 0.0,
                    fade_duration: options.fade_in,
                })
            } else {
                None
            };
            (old.weight.clamp(0.0, 1.0), prev_fade)
        } else {
            (0.0, None)
        };

        let weight = if options.fade_in <= 0.0 { 1.0 } else { start_weight };
        let phase = if options.fade_in > 0.0 && weight < 1.0 {
            FullBodyPhase::FadingIn
        } else {
            FullBodyPhase::Playing
        };

        Self {
            clip_name,
            clip_time: 0.0,
            duration: duration.max(0.0),
            elapsed: 0.0,
            start_weight,
            weight,
            options,
            phase,
            just_started: true,
            stop_fade: None,
            prev,
        }
    }

    pub fn clip_name(&self) -> &str {
        &self.clip_name
    }

    pub fn weight(&self) -> f32 {
        self.weight
    }

    pub fn phase(&self) -> FullBodyPhase {
        self.phase
    }

    pub fn prev(&self) -> Option<&PrevFullBody> {
        self.prev.as_ref()
    }

    /// Playback progress in 0..=1.
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            return if self.phase == FullBodyPhase::Holding { 1.0 } else { 0.0 };
        }
        if self.options.looping {
            (self.clip_time.rem_euclid(self.duration) / self.duration).clamp(0.0, 1.0)
        } else {
            (self.clip_time / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Sample timestamp for `Rig::local_pose_into` without wrapping the final keyframe.
    pub fn sample_time(&self) -> f32 {
        sample_time_for(self.clip_time, self.duration, self.options.looping)
    }

    /// Begins fading out this clip. Returns `true` if it should stop immediately.
    pub fn begin_stop(&mut self, fade_out: Option<f32>) -> bool {
        let dur = fade_out.unwrap_or(self.options.fade_out).max(0.0);
        if dur <= 0.0 {
            self.weight = 0.0;
            return true;
        }
        self.stop_fade = Some(StopFade {
            elapsed: 0.0,
            duration: dur,
            start_weight: self.weight,
        });
        self.phase = FullBodyPhase::FadingOut;
        false
    }

    /// Advances playback and fades by `dt` seconds.
    pub fn advance(&mut self, dt: f32) -> FullBodyStep {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let prev_time = self.clip_time;
        let just_started = self.just_started;

        // Advance previous clip during crossfade
        if let Some(p) = &mut self.prev {
            p.elapsed += dt;
            if p.duration > 0.0 {
                let next = p.clip_time + dt * p.speed;
                p.clip_time = if p.looping {
                    next.rem_euclid(p.duration)
                } else {
                    next.min(p.duration)
                };
            }
            if p.elapsed >= p.fade_duration {
                self.prev = None;
            }
        }

        // Advance current clip time
        let raw_next_time = if dt > 0.0 {
            self.just_started = false;
            self.elapsed += dt;
            if self.phase == FullBodyPhase::Holding {
                self.clip_time
            } else {
                self.clip_time + dt * self.options.speed
            }
        } else {
            self.clip_time
        };

        if self.options.looping && self.duration > 0.0 {
            self.clip_time = raw_next_time.rem_euclid(self.duration);
        } else {
            self.clip_time = raw_next_time.min(self.duration);
        }

        // Explicit stop fade-out takes priority
        if let Some(sf) = &mut self.stop_fade {
            sf.elapsed += dt;
            let frac = (sf.elapsed / sf.duration).clamp(0.0, 1.0);
            self.weight = (sf.start_weight * (1.0 - frac)).clamp(0.0, 1.0);
            self.phase = FullBodyPhase::FadingOut;
            let finished = sf.elapsed >= sf.duration;
            return FullBodyStep {
                prev_time,
                raw_next_time,
                duration: self.duration,
                looping: self.options.looping,
                just_started,
                finished,
            };
        }

        let in_w = if self.options.fade_in > 0.0 {
            let frac = (self.elapsed / self.options.fade_in).clamp(0.0, 1.0);
            (self.start_weight + (1.0 - self.start_weight) * frac).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let finished = if self.options.looping {
            self.weight = in_w;
            self.phase = if self.options.fade_in > 0.0 && self.elapsed < self.options.fade_in && self.start_weight < 1.0 {
                FullBodyPhase::FadingIn
            } else {
                FullBodyPhase::Playing
            };
            false
        } else if self.options.hold_last {
            self.weight = in_w;
            if self.clip_time >= self.duration && self.duration > 0.0 {
                self.phase = FullBodyPhase::Holding;
            } else if self.options.fade_in > 0.0 && self.elapsed < self.options.fade_in && self.start_weight < 1.0 {
                self.phase = FullBodyPhase::FadingIn;
            } else {
                self.phase = FullBodyPhase::Playing;
            }
            false
        } else {
            let remaining = ((self.duration - raw_next_time) / self.options.speed).max(0.0);
            let out_w = if self.options.fade_out > 0.0 {
                (remaining / self.options.fade_out).clamp(0.0, 1.0)
            } else if raw_next_time < self.duration {
                1.0
            } else {
                0.0
            };
            self.weight = in_w.min(out_w).clamp(0.0, 1.0);
            let done = dt > 0.0 && raw_next_time >= self.duration;
            if self.options.fade_out > 0.0 && remaining < self.options.fade_out {
                self.phase = FullBodyPhase::FadingOut;
            } else if self.options.fade_in > 0.0 && self.elapsed < self.options.fade_in && self.start_weight < 1.0 {
                self.phase = FullBodyPhase::FadingIn;
            } else {
                self.phase = FullBodyPhase::Playing;
            }
            done
        };

        FullBodyStep {
            prev_time,
            raw_next_time,
            duration: self.duration,
            looping: self.options.looping,
            just_started,
            finished,
        }
    }
}

fn sample_time_for(clip_time: f32, duration: f32, looping: bool) -> f32 {
    if duration <= 0.0 {
        return 0.0;
    }
    if looping {
        clip_time.rem_euclid(duration)
    } else if clip_time >= duration {
        (duration - HOLD_END_EPSILON).max(0.0)
    } else {
        clip_time.max(0.0)
    }
}
