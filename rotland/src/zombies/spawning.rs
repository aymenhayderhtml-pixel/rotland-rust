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

/// What one spawn rolls for its body: the look, the walk clip, the facing and the animation phase.
/// Pure, so the picks can be tested without a GPU device.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SpawnPick {
    pub look_name: String,
    pub gait: String,
    pub yaw: f32,
    pub phase: f32,
}

/// Rolls the body of one spawn from the four uniform rolls in `0..1` `spawn` takes from its rng.
///
/// `forced_model` (a POI building's `zombieModels` pick) replaces the roll over the kind's `models`,
/// so a soldier kind can wear the police look; the remaining rolls are consumed the same way, so
/// every other choice a spawn makes stays the same. `None` behaves exactly like a plain spawn.
pub(super) fn pick_spawn(
    kind: &str,
    models: &[String],
    gaits: &[String],
    forced_model: Option<&str>,
    rolls: [f32; 4],
) -> SpawnPick {
    let look_name = match forced_model {
        Some(name) => name.to_string(),
        None => {
            let pick = (rolls[0] * models.len() as f32) as usize;
            models
                .get(pick)
                .cloned()
                .unwrap_or_else(|| default_model_for_kind(kind).to_string())
        }
    };
    let gait = gaits
        .get((rolls[1] * gaits.len() as f32) as usize)
        .cloned()
        .unwrap_or_else(|| "zombie_shamble".into());
    SpawnPick {
        look_name,
        gait,
        yaw: rolls[2] * std::f32::consts::TAU,
        phase: rolls[3] * 2.0,
    }
}

/// Index of a look name among the loaded looks; unknown names draw the first look.
pub(super) fn look_index(look_names: &[&str], name: &str) -> usize {
    look_names.iter().position(|n| *n == name).unwrap_or(0)
}

/// Extra look ids that are only named by POI buildings (`data/poi.json` `zombieModels`), so they are
/// loaded with the zombie looks even though no `data/zombies.json` kind lists them. Read at run time.
pub(super) fn extra_look_names() -> Vec<String> {
    let path = paths::repo_path("data/poi.json");
    let Ok(cfg) = world::PoiConfig::load_from_file(&path) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for def in &cfg.buildings {
        for model in &def.zombie_models {
            if !names.contains(model) {
                names.push(model.clone());
            }
        }
    }
    names
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
            self.spawn(kind, &def, Vec3::new(x, y, z), None, None);
        }
    }

    pub(super) fn spawn(
        &mut self,
        kind: &str,
        def: &ZombieDef,
        at: Vec3,
        forced_model: Option<&str>,
        horde: Option<Arc<Mutex<ZombieHorde>>>,
    ) {
        let look_roll = self.rand();
        let gait_roll = self.rand();
        let yaw_roll = self.rand();
        let phase_roll = self.rand();
        let pick = pick_spawn(
            kind,
            &def.models,
            &self.defs.behaviour.gaits,
            forced_model,
            [look_roll, gait_roll, yaw_roll, phase_roll],
        );
        let look_names: Vec<&str> = self.looks.iter().map(|l| l.name.as_str()).collect();
        let look = look_index(&look_names, &pick.look_name);
        let gait = pick.gait;
        let yaw = pick.yaw;
        let phase = pick.phase;
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
            herd_route: None,
        });
    }

    /// One zombie of type `kind` (e.g. "walker", "runner") at `at`, for the spawner. Returns its id.
    pub fn spawn_kind(&mut self, kind: &str, at: Vec3) -> Option<u64> {
        self.spawn_kind_look(kind, at, None)
    }

    /// One zombie of type `kind` at `at` that draws `model` instead of the kind's own looks.
    ///
    /// The stats, brain and gait roll stay exactly the kind's; only the model is forced, so a
    /// police station can put police looks on its normal walker kinds. `None` is exactly
    /// `spawn_kind` (same stats and the same look for the same seed). Returns the new id.
    pub fn spawn_kind_look(&mut self, kind: &str, at: Vec3, model: Option<&str>) -> Option<u64> {
        let def = self.defs.get(kind).cloned()?;
        let id = self.next_id;
        self.spawn(kind, &def, at, model, None);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn models(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    fn gaits() -> Vec<String> {
        models(&["zombie_shamble", "zombie_lurch"])
    }

    #[test]
    fn spawn_kind_look_without_model_is_a_plain_spawn() {
        // spawn_kind hands the same rolls and `None` to pick_spawn, so the pick must be identical.
        let kinds = models(&["zombie_01", "zombie_02", "zombie_03"]);
        let rolls = [0.42, 0.61, 0.13, 0.87];
        let plain = pick_spawn("walker", &kinds, &gaits(), None, rolls);
        let with_none = pick_spawn("walker", &kinds, &gaits(), None, rolls);
        assert_eq!(plain, with_none);
        assert_eq!(plain.look_name, "zombie_02", "same seed picks the same look as before");
    }

    #[test]
    fn forced_model_only_changes_the_look() {
        let kinds = models(&["zombie_01", "zombie_02", "zombie_03"]);
        let rolls = [0.42, 0.61, 0.13, 0.87];
        let plain = pick_spawn("walker", &kinds, &gaits(), None, rolls);
        let forced = pick_spawn("walker", &kinds, &gaits(), Some("zombie_police"), rolls);
        assert_eq!(forced.look_name, "zombie_police");
        assert_eq!(forced.gait, plain.gait, "the kind's gait roll is untouched");
        assert_eq!(forced.yaw, plain.yaw);
        assert_eq!(forced.phase, plain.phase);
    }

    #[test]
    fn a_forced_look_is_always_used() {
        let kinds = models(&["zombie_01", "zombie_02"]);
        let picks: Vec<String> = (0..10)
            .map(|i| {
                let rolls = [i as f32 / 10.0, 0.0, 0.0, 0.0];
                pick_spawn("walker", &kinds, &gaits(), Some("zombie_soldier"), rolls).look_name
            })
            .collect();
        assert!(picks.iter().all(|p| p == "zombie_soldier"), "a forced look never falls back: {picks:?}");
    }

    #[test]
    fn unknown_model_name_falls_back_to_the_first_look() {
        let names = ["zombie_01", "zombie_police"];
        assert_eq!(look_index(&names, "zombie_police"), 1);
        assert_eq!(look_index(&names, "zombie_not_loaded"), 0);
    }

    #[test]
    fn kind_without_models_uses_the_kind_fallback() {
        let pick = pick_spawn("frozen", &[], &gaits(), None, [0.5, 0.0, 0.0, 0.0]);
        assert_eq!(pick.look_name, "zombie_frozen");
    }
}
