//! Spawning, hordes, and despawning logic for Zombies.

use std::sync::{Arc, Mutex};

use game::zombie::{ZombieBehaviourDef, ZombieBrain, ZombieDef, ZombieDefs, ZombieHorde};
use game::Health;
use glam::Vec3;
use physics::PhysicsWorld;

use super::types::{Look, Zombie, FIRST_ID};
use super::Zombies;

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
        let look = ["zombie_01", "zombie_02", "zombie_03"]
            .iter()
            .position(|n| Some(n) == look_names.get(pick))
            .unwrap_or(0);
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
            anim: game::zombie::AnimClip::Idle,
            anim_time: phase,
            dead_for: 0.0,
            pending_hit: None,
            think_accumulator: 0.0,
            last_out: None,
        });
    }

    /// One zombie of type `kind` (e.g. "walker", "runner") at `at`, for the spawner. Returns its id.
    pub fn spawn_kind(&mut self, kind: &str, at: Vec3) -> Option<u64> {
        let def = self.defs.get(kind).cloned()?;
        let id = self.next_id;
        self.spawn(kind, &def, at, None);
        Some(id)
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
