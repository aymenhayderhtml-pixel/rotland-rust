//! First-time tips system: trigger polling, sequential queuing, and display lifetime.
//!
//! Port of Godot C# scripts/UI/HintSystem.cs.
//! Godot save system id: "hints".

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

use super::config::{HintConfig, HintConfigData};
use super::context::HintContext;
use super::def::HintDef;
use super::triggers::{parse_number, split, split_npc};

// Godot: HintSystem.cs PollSeconds
pub const POLL_SECONDS: f32 = 0.25;

// Godot: HintSystem.cs GapSeconds
pub const GAP_SECONDS: f32 = 0.5;

/// First-time tips manager. Each tip shows once per save.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HintSystem {
    config: HintConfigData,
    shown: HashSet<String>,
    times_shown: HashMap<String, u32>,
    #[serde(skip)]
    queue: VecDeque<HintDef>,
    #[serde(skip)]
    queued: HashSet<String>,
    #[serde(skip)]
    done_seen: HashSet<String>,
    #[serde(skip)]
    current: Option<HintDef>,
    #[serde(skip)]
    shown_for: f32,
    #[serde(skip)]
    gap_left: f32,
    #[serde(skip)]
    poll_left: f32,
    #[serde(skip)]
    play_time: f32,
    #[serde(skip)]
    was_in_camp: bool,
    #[serde(skip)]
    left_camp: bool,
    #[serde(skip)]
    outside_safe_zone: bool,
    #[serde(skip)]
    dead: bool,
    #[serde(skip)]
    died: bool,
    #[serde(skip)]
    died_and_respawned: bool,
}

impl Default for HintSystem {
    fn default() -> Self {
        Self::new(HintConfigData::default())
    }
}

impl HintSystem {
    pub fn new(config: HintConfigData) -> Self {
        Self {
            config,
            shown: HashSet::new(),
            times_shown: HashMap::new(),
            queue: VecDeque::new(),
            queued: HashSet::new(),
            done_seen: HashSet::new(),
            current: None,
            shown_for: 0.0,
            gap_left: 0.0,
            poll_left: 0.0,
            play_time: 0.0,
            was_in_camp: false,
            left_camp: false,
            outside_safe_zone: false,
            dead: false,
            died: false,
            died_and_respawned: false,
        }
    }

    /// Loads the hint system with the real data/hints.json config.
    pub fn from_repo() -> Result<Self, String> {
        let config = HintConfig::load_default()?;
        Ok(Self::new(config))
    }

    pub fn config(&self) -> &HintConfigData { &self.config }
    pub fn current(&self) -> Option<&HintDef> { self.current.as_ref() }
    pub fn current_id(&self) -> Option<&str> { self.current.as_ref().map(|h| h.id.as_str()) }
    #[allow(dead_code)]
    fn current_text(&self) -> Option<&str> { self.current.as_ref().map(|h| h.text.as_str()) }
    pub fn is_shown(&self, id: &str) -> bool { self.shown.contains(id) }
    pub fn is_queued(&self, id: &str) -> bool { self.queued.contains(id) }
    pub fn times_shown(&self, id: &str) -> u32 { self.times_shown.get(id).copied().unwrap_or(0) }
    #[allow(dead_code)]
    fn is_visible(&self) -> bool { self.current.is_some() }

    /// Requests a specific tip to be queued next.
    // Godot: HintSystem.cs Request
    pub fn request(&mut self, id: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        if self.shown.contains(id) || self.queued.contains(id) {
            return false;
        }
        if let Some(hint) = self.config.hints.iter().find(|h| h.id == id).cloned() {
            self.enqueue(hint);
            true
        } else {
            false
        }
    }

    /// Advances simulation time, polls triggers, updates display timers and queue.
    /// Returns the active hint to show on screen this frame, if any.
    // Godot: HintSystem.cs _Process
    pub fn tick(&mut self, dt: f32, ctx: &HintContext) -> Option<&HintDef> {
        if !self.config.enabled {
            return None;
        }

        self.play_time += dt;
        self.poll_left -= dt;
        if self.poll_left <= 0.0 {
            self.poll_left = POLL_SECONDS;
            self.queue_triggered(ctx);
        }

        self.update_showing(dt, ctx);

        if !ctx.is_menu_open && !ctx.is_dead && ctx.hints_wanted {
            self.current.as_ref()
        } else {
            None
        }
    }

    /// Records an event such as "npc_talked", "item_used", or a game event.
    pub fn on_event(&mut self, event: &str) {
        self.done_seen.insert(event.to_string());
        match event {
            "safe_zone_left" => self.left_camp = true,
            "died" => {
                self.died = true;
                self.died_and_respawned = true;
            }
            _ => {}
        }
    }

    // Godot: HintSystem.cs OnSafeZoneChanged
    pub fn on_safe_zone_changed(&mut self, inside: bool) {
        self.outside_safe_zone = !inside;
        if inside {
            self.was_in_camp = true;
        } else {
            if self.was_in_camp {
                self.left_camp = true;
            }
            if self.current.as_ref().is_some_and(|h| h.id == "start") {
                self.current = None;
                self.gap_left = GAP_SECONDS;
            }
        }
    }

    // Godot: HintSystem.cs OnDied
    #[allow(dead_code)]
    fn on_player_died(&mut self) {
        self.dead = true;
        self.died = true;
    }

    // Godot: HintSystem.cs OnRespawned
    #[allow(dead_code)]
    fn on_player_respawned(&mut self) {
        self.dead = false;
        if self.died {
            self.died_and_respawned = true;
        }
    }

    // Godot: HintSystem.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        let shown_list: Vec<serde_json::Value> = self
            .shown
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        serde_json::json!({ "shown": shown_list })
    }

    // Godot: HintSystem.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        self.shown.clear();
        if let Some(list) = data.get("shown").and_then(|v| v.as_array()) {
            for entry in list {
                if let Some(id) = entry.as_str() {
                    self.shown.insert(id.to_string());
                }
            }
        }
        self.queue.clear();
        self.queued.clear();
        self.done_seen.clear();
        self.current = None;
    }

    fn queue_triggered(&mut self, ctx: &HintContext) {
        if !ctx.hints_wanted {
            return;
        }
        let hints = self.config.hints.clone();
        for hint in hints {
            if !self.shown.contains(&hint.id)
                && !self.queued.contains(&hint.id)
                && self.is_triggered(&hint.trigger, ctx)
            {
                self.enqueue(hint);
            }
        }
    }

    fn is_triggered(&self, trigger: &str, ctx: &HintContext) -> bool {
        let (name, arg) = split(trigger);
        match name {
            "start" => !self.outside_safe_zone,
            "after" => self.play_time >= parse_number(arg),
            "safe_zone_left" => self.left_camp,
            "zombie_near" => ctx.nearest_zombie_dist.is_some_and(|d| d <= parse_number(arg)),
            "crate_near" => ctx.nearest_crate_dist.is_some_and(|d| d <= parse_number(arg)),
            "board_near" => ctx.nearest_board_dist.is_some_and(|d| d <= parse_number(arg)),
            "near_npc" => {
                let (role, metres) = split_npc(arg);
                ctx.npc_distances.get(role).copied().unwrap_or(f32::INFINITY) <= metres
            }
            "need_below" => {
                let limit = parse_number(arg);
                ctx.needs.values().any(|&v| v < limit)
            }
            "hour_from" => ctx.hour >= parse_number(arg),
            "died" => self.died_and_respawned,
            "bag_over" => ctx.bag_weight_ratio.is_some_and(|r| r >= parse_number(arg)),
            "has_item" => ctx.inventory_items.contains(arg),
            "temp_cold" => ctx.temp_cold,
            "temp_hot" => ctx.temp_hot,
            _ => false,
        }
    }

    fn update_showing(&mut self, dt: f32, ctx: &HintContext) {
        let menu = ctx.is_menu_open || ctx.is_dead;
        if self.current.is_none() {
            self.gap_left -= dt;
            if !ctx.hints_wanted {
                self.queue.clear();
                self.queued.clear();
            } else if self.gap_left <= 0.0
                && let Some(next) = self.queue.pop_front()
            {
                self.show(next);
            }
        } else {
            if !menu {
                self.shown_for += dt;
            }
            let done = self
                .current
                .as_ref()
                .and_then(|c| c.done_when.as_ref())
                .is_some_and(|dw| self.done_seen.contains(dw));

            if done || self.shown_for >= self.config.show_seconds {
                self.current = None;
                self.gap_left = GAP_SECONDS;
            }
        }
    }

    fn show(&mut self, hint: HintDef) {
        self.queued.remove(&hint.id);
        self.shown.insert(hint.id.clone());
        let count = self.times_shown.entry(hint.id.clone()).or_insert(0);
        *count += 1;
        self.done_seen.clear();
        self.shown_for = 0.0;
        self.current = Some(hint);
    }

    fn enqueue(&mut self, hint: HintDef) {
        self.queued.insert(hint.id.clone());
        self.queue.push_back(hint);
    }
}
