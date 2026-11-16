//! Camp life director coordinating residents, chats, and guards (port of scripts/Settlement/CampLifeDirector.cs).

use std::path::Path;
use glam::Vec3;
use world::{DotnetRandom, SettlementConfig, SettlementConfigData, SettlementSite};

use crate::camp::chat::CampChat;
use crate::camp::config::{CampLifeConfig, CampLifeData};
use crate::camp::guard::SettlementGuard;
use crate::camp::math::CampMath;
use crate::camp::npc::SettlementNpc;
use crate::camp::planner::CampPlanner;
use crate::camp::resident::{CampResident, ResidentOutput};
use crate::camp::types::{CampClock, CampEvent, CampJob, CampState};

/// Runs the camp's living residents, marker NPCs, and guards (Godot: CampLifeDirector.cs).
pub struct CampLifeDirector {
    pub site: SettlementSite,
    pub data: CampLifeData,
    pub planner: CampPlanner,
    pub rng: DotnetRandom,
    pub residents: Vec<CampResident>,
    pub fixed_npcs: Vec<SettlementNpc>,
    pub guards: Vec<SettlementGuard>,
    pub chats: Vec<CampChat>,
    pub manual_only: bool,
    pub day: i32,
    last_hour: f32,
}

impl CampLifeDirector {
    pub const GROUP_NAME: &'static str = "camp_life";

    pub fn new(site: SettlementSite, data: CampLifeData, cfg: &SettlementConfigData) -> Self {
        let rng_seed = CampMath::hash(&site.id, 7) as i32;
        let mut rng = DotnetRandom::new(rng_seed);
        let planner = CampPlanner::new(&site, &data);
        let fixed_npcs = SettlementNpc::default_settlement_npcs(&site, cfg);

        let mut guards = Vec::new();
        for npc in &fixed_npcs {
            if npc.role == "guard" {
                guards.push(SettlementGuard::new(&npc.id, npc.position, npc.yaw, cfg));
            }
        }

        let mut director = Self {
            site,
            data,
            planner,
            rng,
            residents: Vec::new(),
            fixed_npcs,
            guards,
            chats: Vec::new(),
            manual_only: false,
            day: 1,
            last_hour: 12.0,
        };
        director.spawn();
        director
    }

    /// Loads configs from data/ directory and creates the director.
    pub fn from_data_dir(site: SettlementSite, data_dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = data_dir.as_ref();
        let camp_life_path = dir.join("camp_life.json");
        let settlements_path = dir.join("settlements.json");
        let data = CampLifeConfig::load_from_file(&camp_life_path)
            .map_err(|e| format!("{}: {e}", camp_life_path.display()))?;
        let cfg = SettlementConfig::load_from_file(&settlements_path)
            .map_err(|e| format!("{}: {e}", settlements_path.display()))?;
        Ok(Self::new(site, data, &cfg))
    }

    pub fn residents(&self) -> &[CampResident] { &self.residents }
    pub fn residents_mut(&mut self) -> &mut [CampResident] { &mut self.residents }
    pub fn fixed_npcs(&self) -> &[SettlementNpc] { &self.fixed_npcs }
    pub fn guards(&self) -> &[SettlementGuard] { &self.guards }
    pub fn chats(&self) -> &[CampChat] { &self.chats }

    /// Number of residents currently in the given state.
    // Godot: CampLifeDirector.cs CountIn
    pub fn count_in(&self, state: CampState) -> usize {
        self.residents.iter().filter(|r| r.state == state).count()
    }

    /// Simulates camp life for a given duration in seconds with fixed step.
    // Godot: CampLifeDirector.cs Simulate
    pub fn simulate(&mut self, seconds: f32, step: f32, game_hour: f32) {
        let mut t = 0.0;
        while t < seconds {
            self.tick(step, game_hour, None);
            t += step;
        }
    }

    /// Ticks the camp and returns the render state of all residents for this frame.
    pub fn update(&mut self, dt: f32, game_hour: f32, player_pos: Option<Vec3>) -> Vec<ResidentOutput> {
        if !self.manual_only {
            self.tick(dt, game_hour, player_pos);
        }
        self.resident_outputs(player_pos)
    }

    /// Advances camp residents, chats, and guards by dt seconds.
    // Godot: CampLifeDirector.cs Tick
    pub fn tick(&mut self, dt: f32, game_hour: f32, player_pos: Option<Vec3>) {
        if game_hour < self.last_hour && (self.last_hour - game_hour) > 12.0 {
            self.day += 1;
        }
        self.last_hour = game_hour;

        let night = game_hour >= 20.0 || game_hour < 5.0;
        let clock = CampClock::from_hour(game_hour, night, self.day, self.data.fire_hour_from, self.data.fire_hour_to);

        let resident_count = self.residents.len();
        for r_idx in 0..resident_count {
            let r_pos = self.residents[r_idx].position;
            let dist = player_pos.map(|p| CampMath::flat(p, r_pos)).unwrap_or(0.0);
            if player_pos.is_some() && dist > self.data.far_range {
                continue;
            }

            self.residents[r_idx].bubble.tick(dt);

            if self.residents[r_idx].state == CampState::Hidden {
                if !clock.night {
                    self.planner.wake_up(r_idx, &mut self.residents, &self.data, &mut self.rng);
                }
                continue;
            }

            if let Some(p_pos) = player_pos
                && dist <= self.data.greet_range
                && !self.residents[r_idx].is_greeting()
                && self.residents[r_idx].last_greet_day != clock.day
            {
                self.residents[r_idx].last_greet_day = clock.day;
                self.residents[r_idx].greet(p_pos, self.data.greet_seconds);
            }

            match self.residents[r_idx].update(dt) {
                CampEvent::Arrived => self.planner.arrive(r_idx, &mut self.residents, &self.data, &mut self.rng),
                CampEvent::TimerDone => self.planner.timer_done(r_idx, &mut self.residents, &mut self.chats, clock, &self.data, &mut self.rng),
                CampEvent::None => {}
            }
        }

        let mut ended_chat_ids = Vec::new();
        for chat in &mut self.chats {
            if chat.tick(dt, &mut self.residents, &self.data, &mut self.rng) {
                ended_chat_ids.push(chat.id);
            }
        }
        for chat_id in ended_chat_ids {
            if let Some(pos) = self.chats.iter().position(|c| c.id == chat_id) {
                let chat = self.chats.remove(pos);
                self.planner.end_chat(&chat, &mut self.residents, &self.data, &mut self.rng);
            }
        }
    }

    /// Produces frame output for each resident so Claude can draw them with the humanoid animator.
    pub fn resident_outputs(&self, player_pos: Option<Vec3>) -> Vec<ResidentOutput> {
        self.residents
            .iter()
            .map(|r| {
                let near = player_pos.map(|p| CampMath::flat(p, r.position) <= self.data.bubble_range).unwrap_or(true);
                r.render_output(near)
            })
            .collect()
    }

    /// Finds the nearest interactable NPC or board in front of the player (Godot: Range = 2.5m, HalfCone = 35°).
    pub fn nearest_interactable(&self, player_pos: Vec3, facing: Vec3) -> Option<(String, String, String)> {
        let forward = Vec3::new(facing.x, 0.0, facing.z);
        let forward_len2 = forward.length_squared();
        if forward_len2 < 0.0001 {
            return None;
        }
        let forward = forward / forward_len2.sqrt();
        let range2 = 2.5 * 2.5;

        let mut best: Option<(String, String, String)> = None;
        let mut best_angle = f32::MAX;

        for npc in &self.fixed_npcs {
            if npc.can_interact() {
                check_interactable(npc.position, &npc.id, &npc.role, &npc.prompt, player_pos, forward, range2, &mut best, &mut best_angle);
            }
        }
        for r in &self.residents {
            if r.can_interact() {
                let prompt = format!("[E] Talk to {}", r.role);
                check_interactable(r.position, &r.id, &r.role, &prompt, player_pos, forward, range2, &mut best, &mut best_angle);
            }
        }
        best
    }

    fn spawn(&mut self) {
        let mut models = Vec::with_capacity(self.data.resident_count);
        for i in 0..self.data.resident_count {
            models.push(if i % 2 == 0 { "npc_villager" } else { "npc_survivor" });
        }
        self.shuffle(&mut models);

        let mut spots = Vec::with_capacity(self.data.wander_points.len());
        for p in &self.data.wander_points {
            spots.push(self.planner.world_point(*p));
        }
        self.shuffle(&mut spots);

        for i in 0..self.data.resident_count {
            let model = models[i];
            let role = if model == "npc_villager" { "villager" } else { "survivor" };
            let pos = if i < spots.len() { spots[i] } else { self.site.center };
            let yaw = CampMath::yaw_toward(pos, self.site.center);
            let id = format!("camp_resident_{}", i + 1);

            let tempo = if role == "survivor" {
                let h = CampMath::hash(&id, 2);
                SettlementNpc::MIN_SURVIVOR_TEMPO + (SettlementNpc::MAX_SURVIVOR_TEMPO - SettlementNpc::MIN_SURVIVOR_TEMPO) * CampMath::unit(h)
            } else {
                1.0
            };

            let mut resident = CampResident::new(i, id, role, model, pos, yaw, self.data.walk_speed, tempo);
            resident.job = match i {
                0 => CampJob::Woodcutter,
                1 => CampJob::Carrier,
                _ => CampJob::None,
            };

            let initial_idle = CampMath::range(&mut self.rng, 0.0, self.data.idle_max_seconds);
            resident.stand(CampState::Idle, initial_idle, None, None);
            self.residents.push(resident);
        }
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.rng.next(i as i32 + 1) as usize;
            items.swap(i, j);
        }
    }
}

fn check_interactable(
    pos: Vec3,
    id: &str,
    role: &str,
    prompt: &str,
    player_pos: Vec3,
    forward: Vec3,
    range2: f32,
    best: &mut Option<(String, String, String)>,
    best_angle: &mut f32,
) {
    let to_target = pos - player_pos;
    if to_target.length_squared() > range2 {
        return;
    }
    let flat_to = Vec3::new(to_target.x, 0.0, to_target.z);
    if flat_to.dot(forward) < 0.0 {
        return;
    }
    let flat_len2 = flat_to.length_squared();
    let angle = if flat_len2 < 0.0001 {
        0.0
    } else {
        let cos_angle = (flat_to.dot(forward) / flat_len2.sqrt()).clamp(-1.0, 1.0);
        cos_angle.acos().to_degrees()
    };
    if angle <= 35.0 && angle < *best_angle {
        *best_angle = angle;
        *best = Some((id.to_string(), role.to_string(), prompt.to_string()));
    }
}
