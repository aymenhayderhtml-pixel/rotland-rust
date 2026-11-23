//! Timed animation clip event markers and bounded event queue.

use std::collections::{HashMap, VecDeque};

use crate::anim_config::AnimConfig;

/// Maximum number of undrained events buffered per animator to prevent unbounded growth.
pub const MAX_QUEUED_EVENTS: usize = 64; // new: DECISIONS 2026-11-23

/// A fired animation event returned by `HumanoidAnimator::drain_events`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipEvent {
    pub clip: String,
    pub name: String,
    pub time: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct EventEntry {
    name: String,
    time: f32,
}

/// Time interval traversed by a clip during a single update step.
#[derive(Clone, Copy, Debug)]
pub struct EventSpan {
    pub prev: f32,
    pub next: f32,
    pub duration: f32,
    pub just_started: bool,
}

/// Bounded ring queue of fired `ClipEvent`s.
#[derive(Clone, Debug)]
pub struct EventQueue {
    events: VecDeque<ClipEvent>,
    limit: usize,
}

impl EventQueue {
    pub fn new(limit: usize) -> Self {
        let cap = limit.max(1); // new: DECISIONS 2026-11-23
        Self {
            events: VecDeque::with_capacity(cap),
            limit: cap,
        }
    }

    pub fn push(&mut self, event: ClipEvent) {
        if self.events.len() >= self.limit {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn drain_into(&mut self, out: &mut Vec<ClipEvent>) {
        out.extend(self.events.drain(..));
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new(MAX_QUEUED_EVENTS)
    }
}

/// Per-clip table of timed event markers sorted chronologically.
#[derive(Clone, Debug, Default)]
pub struct ClipEventTable {
    by_clip: HashMap<String, Vec<EventEntry>>,
}

impl ClipEventTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a sorted event table from `cfg`, keeping only clips accepted by `keep_clip`.
    pub fn from_config_filtered<F>(cfg: &AnimConfig, mut keep_clip: F) -> Self
    where
        F: FnMut(&str) -> bool,
    {
        let mut table = Self::new();
        for (clip, defs) in &cfg.events {
            if !keep_clip(clip) || defs.is_empty() {
                continue;
            }
            let mut entries: Vec<EventEntry> = defs
                .iter()
                .filter(|d| d.time.is_finite() && d.time >= 0.0)
                .map(|d| EventEntry {
                    name: d.name.clone(),
                    time: d.time,
                })
                .collect();
            entries.sort_by(|a, b| a.time.total_cmp(&b.time));
            if !entries.is_empty() {
                table.by_clip.insert(clip.clone(), entries);
            }
        }
        table
    }

    /// Builds a sorted event table from all entries in `cfg`.
    pub fn from_config(cfg: &AnimConfig) -> Self {
        Self::from_config_filtered(cfg, |_| true)
    }

    /// Inserts a single event marker for `clip` and keeps the list sorted by time.
    pub fn add(&mut self, clip: &str, name: &str, time: f32) {
        if !time.is_finite() || time < 0.0 {
            return;
        }
        let entries = self.by_clip.entry(clip.to_owned()).or_default();
        entries.push(EventEntry {
            name: name.to_owned(),
            time,
        });
        entries.sort_by(|a, b| a.time.total_cmp(&b.time));
    }

    /// Emits events crossed by a non-looping clip over `span`. No allocation if clip has no events.
    pub fn collect_non_looping(&self, clip_name: &str, span: EventSpan, queue: &mut EventQueue) {
        if span.next <= span.prev || span.duration <= 0.0 {
            return;
        }
        let Some(entries) = self.by_clip.get(clip_name) else {
            return;
        };
        if entries.is_empty() {
            return;
        }
        let end = span.next.min(span.duration);
        for entry in entries {
            if entry.time > end {
                break;
            }
            let after_start = if span.just_started {
                entry.time >= span.prev
            } else {
                entry.time > span.prev
            };
            if after_start {
                queue.push(ClipEvent {
                    clip: clip_name.to_owned(),
                    name: entry.name.clone(),
                    time: entry.time,
                });
            }
        }
    }

    /// Emits events crossed by a looping clip over unwrapped `span` (handles multiple wraps).
    pub fn collect_looping(&self, clip_name: &str, span: EventSpan, queue: &mut EventQueue) {
        if span.next <= span.prev || span.duration <= 0.0 {
            return;
        }
        let Some(entries) = self.by_clip.get(clip_name) else {
            return;
        };
        if entries.is_empty() {
            return;
        }
        let mut k_start = (span.prev / span.duration).floor().max(0.0) as u64;
        let k_end = (span.next / span.duration).floor().max(0.0) as u64;
        let max_loops = MAX_QUEUED_EVENTS as u64;
        if k_end.saturating_sub(k_start) > max_loops {
            k_start = k_end - max_loops;
        }
        for k in k_start..=k_end {
            let base = k as f32 * span.duration;
            for entry in entries {
                if entry.time > span.duration {
                    break;
                }
                let occ = base + entry.time;
                let after_start = if span.just_started && k == 0 {
                    occ >= span.prev
                } else {
                    occ > span.prev
                };
                if after_start && occ <= span.next {
                    queue.push(ClipEvent {
                        clip: clip_name.to_owned(),
                        name: entry.name.clone(),
                        time: entry.time,
                    });
                }
            }
        }
    }
}
