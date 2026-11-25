//! Spawning, hordes, and despawning logic for Zombies.

use std::sync::{Arc, Mutex};

use game::Health;
use game::zombie::{
    AnimClip, SpecialKind, SpecialState, ZombieBehaviourDef, ZombieBrain, ZombieDef, ZombieHorde,
};
use glam::Vec3;
use physics::PhysicsWorld;

use super::Zombies;
use super::types::Zombie;

/// Distance in front of the player for `--spawn-zombie=<kind>` showcase spawns. // new: DECISIONS 2026-11-25
const SHOWCASE_DISTANCE_M: f32 = 5.0;
/// Idle lock duration in seconds for `--spawn-zombie=<kind>` showcase spawns. // new: DECISIONS 2026-11-25
const SHOWCASE_IDLE_SECONDS: f32 = 3.0;

/// Fallback model asset name for a zombie kind if `data/zombies.json` omits `models`.
fn default_model_for_kind(kind: &str) -> &'static str {
    match kind {
        "brute" => "zombie_brute",
        "screamer" => "zombie_screamer",
        "crawler" => "zombie_crawler",
        "frozen" => "zombie_frozen",
        "husk" => "zombie_husk",
        "bloater" => "zombie_bloater",
        _ => "zombie_01",
    }
}

impl Zombies {
    pub(super) fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    /// `count` zombies of type `kind` on a ring `min..max` metres around `center` (like BenchmarkZombies).
    pub fn spawn_ring(
        &mut self,
        kind: &str,
        center: Vec3,
        count: usize,
        min: f32,
        max: f32,
        physics: &PhysicsWorld,
    ) {
        let Some(def) = self.defs.get(kind).cloned() else {
            return;
        };
        for i in 0..count {
            let angle = std::f32::consts::TAU * i as f32 / count as f32 + (self.rand() - 0.5) * 0.4;
            let dist = min + (max - min) * self.rand();
            let (x, z) = (center.x + angle.cos() * dist, center.z + angle.sin() * dist);
            let y = physics.ground_height(x, z).unwrap_or(center.y);
            self.spawn(kind, &def, Vec3::new(x, y, z), None);
        }
    }

    pub(super) fn spawn(
        &mut self,
        kind: &str,
        def: &ZombieDef,
        at: Vec3,
        horde: Option<Arc<Mutex<ZombieHorde>>>,
    ) {
        let look_names: Vec<&str> = def.models.iter().map(String::as_str).collect();
        let pick = (self.rand() * look_names.len() as f32) as usize;
        let chosen_name = look_names
            .get(pick)
            .copied()
            .unwrap_or_else(|| default_model_for_kind(kind));
        let look = self.looks.iter().position(|l| l.name == chosen_name).unwrap_or(0);
        let roll = self.rand();
        let gaits = &self.defs.behaviour.gaits;
        let gait = gaits
            .get((roll * gaits.len() as f32) as usize)
            .cloned()
            .unwrap_or_else(|| "zombie_shamble".into());
        let yaw = self.rand() * std::f32::consts::TAU;
        let phase = self.rand() * 2.0;
        let id = self.next_id;
        self.next_id += 1;
        let tuning: &ZombieBehaviourDef = &self.defs.behaviour;
        let mut brain = ZombieBrain::new(def, tuning, at, id);
        if kind == "crawler" {
            brain.set_crawler(true);
        }
        // With data/special_zombies.json loaded, a special's own call replaces the brain's once-per-life scream.
        let special = self
            .special_cfg
            .as_ref()
            .and_then(|_| SpecialKind::from_id(kind))
            .map(SpecialState::new);
        if kind == "screamer" {
            brain.set_screamer(special.is_none());
            if special.is_some() {
                brain.set_scream_chance(0.0);
            }
        }
        if let Some(h) = horde {
            brain.set_horde(Some(h));
        }
        self.list.push(Zombie {
            id,
            kind: kind.to_string(),
            look: look.min(self.looks.len().saturating_sub(1)),
            def: def.clone(),
            brain,
            health: Health::new(def.max_health),
            position: at,
            velocity: Vec3::ZERO,
            knock: Vec3::ZERO,
            yaw,
            on_floor: true,
            gait,
            anim: AnimClip::Idle,
            anim_time: phase,
            fade: game::zombie::anim_fade::AnimFade::default(),
            last_knock: Vec3::ZERO,
            dead_for: 0.0,
            pending_hit: None,
            think_accumulator: 0.0,
            last_out: None,
            special,
            special_vulnerable: false,
            special_hit_wall: false,
            special_los: false,
            special_sight_timer: 0.0,
            bloater_burst_done: false,
            idle_lock_timer: 0.0,
        });
    }

    /// One zombie of type `kind` (e.g. "walker", "runner") at `at`, for the spawner. Returns its id.
    pub fn spawn_kind(&mut self, kind: &str, at: Vec3) -> Option<u64> {
        let def = self.defs.get(kind).cloned()?;
        let id = self.next_id;
        self.spawn(kind, &def, at, None);
        Some(id)
    }

    /// Spawns one zombie of `kind` at `at`, facing `yaw`, locked idle for `idle_seconds`.
    pub fn spawn_idle_showcase(
        &mut self,
        kind: &str,
        at: Vec3,
        yaw: f32,
        idle_seconds: f32,
    ) -> Option<u64> {
        let id = self.spawn_kind(kind, at)?;
        if let Some(z) = self.list.iter_mut().find(|z| z.id == id) {
            z.yaw = yaw;
            z.anim = AnimClip::Idle;
            z.idle_lock_timer = idle_seconds.max(0.0);
        }
        Some(id)
    }

    /// Spawns one zombie of `kind` 5 m in front of the player, facing them, idle for 3 s (`--spawn-zombie`).
    pub fn spawn_in_front_of_player(
        &mut self,
        kind: &str,
        player_pos: Vec3,
        player_yaw: f32,
        physics: &PhysicsWorld,
    ) -> Option<u64> {
        // Godot yaw convention: forward = (-sin yaw, 0, -cos yaw)
        let forward = Vec3::new(-player_yaw.sin(), 0.0, -player_yaw.cos());
        let target = player_pos + forward * SHOWCASE_DISTANCE_M;
        let y = physics.ground_height(target.x, target.z).unwrap_or(player_pos.y);
        let face_player_yaw = player_yaw + std::f32::consts::PI;
        self.spawn_idle_showcase(
            kind,
            Vec3::new(target.x, y, target.z),
            face_player_yaw,
            SHOWCASE_IDLE_SECONDS,
        )
    }

    /// Spawns a horde of zombies that share one ZombieHorde wander target (Godot ZombieSpawner:186).
    pub fn spawn_horde(&mut self, kind: &str, positions: &[Vec3]) -> Vec<u64> {
        let horde = Arc::new(Mutex::new(ZombieHorde::new()));
        let mut ids = Vec::with_capacity(positions.len());
        let Some(def) = self.defs.get(kind).cloned() else {
            return ids;
        };
        for &at in positions {
            let id = self.next_id;
            self.spawn(kind, &def, at, Some(Arc::clone(&horde)));
            ids.push(id);
        }
        ids
    }

    /// Assigns a shared ZombieHorde to existing zombies by id so they wander together.
    pub fn assign_horde(&mut self, ids: &[u64]) -> Arc<Mutex<ZombieHorde>> {
        let horde = Arc::new(Mutex::new(ZombieHorde::new()));
        for id in ids {
            if let Some(z) = self.list.iter_mut().find(|z| z.id == *id) {
                z.brain.set_horde(Some(Arc::clone(&horde)));
            }
        }
        horde
    }

    /// Removes zombies the spawner says are too far away.
    pub fn despawn(&mut self, ids: &[u64]) {
        self.list.retain(|z| !ids.contains(&z.id));
    }

    /// Living zombies as the spawner sees them (a chasing zombie is kept farther away).
    pub fn active(&self) -> Vec<game::spawner_config::ActiveZombie> {
        use game::zombie::ZombieState;
        self.list
            .iter()
            .filter(|z| !z.health.is_dead())
            .map(|z| {
                let mut a = game::spawner_config::ActiveZombie::new(z.id, z.position);
                a.is_chasing = matches!(z.brain.state(), ZombieState::Chase | ZombieState::Attack);
                a
            })
            .collect()
    }
}
