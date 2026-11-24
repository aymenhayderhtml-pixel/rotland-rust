//! Data types and animation clip mapping for the live zombies.

use engine::anim::Rig;
use engine::render::GpuMesh;
use game::zombie::specials::SpecialHit;
use game::zombie::anim_fade::{AnimFade, clip_key, death_clip};
use game::zombie::{AnimClip, AttackHit, BrainOutput, Noise, SpecialState, ZombieBrain, ZombieDef};
use game::{DamageInfo, Health};
use glam::Vec3;

pub const GRAVITY: f32 = 9.8;
/// The flow field is rebuilt this often (it costs ~10 ms; zombies walk ~2.6 m/s, so 0.5 s is plenty).
pub const FLOW_EVERY: f32 = 0.5;
/// Dead bodies stay this long, then the slot is freed.
pub const CORPSE_SECONDS: f32 = 12.0;
/// Body half-width for melee and gun hits.
pub const BODY_RADIUS: f32 = 0.35;
/// Zombie ids start here (the player is 1).
pub const FIRST_ID: u64 = 100;
/// Distant zombies farther than this from player have throttled brain updates (Godot Zombie.cs:17).
pub const FAR_DISTANCE: f32 = 60.0;
/// Distant zombies update thinking every 0.5 s instead of every frame (Godot Zombie.cs:18).
pub const FAR_THINK_INTERVAL: f32 = 0.5;

/// One zombie look: mesh, skeleton and clips.
pub struct Look {
    pub name: String,
    pub mesh: GpuMesh,
    pub rig: Rig,
    pub clips: Vec<pack::Clip>,
}

impl Look {
    pub fn clip(&self, name: &str) -> Option<&pack::Clip> {
        self.clips.iter().find(|c| c.name == name)
    }

    pub fn clip_by_key(&self, key: u64) -> Option<&pack::Clip> {
        self.clips.iter().find(|c| clip_key(&c.name) == key)
    }
}

pub struct Zombie {
    pub id: u64,
    /// The zombies.json type (walker, runner...), for bounties.
    pub kind: String,
    pub look: usize,
    pub def: ZombieDef,
    pub brain: ZombieBrain,
    pub health: Health,
    pub position: Vec3,
    pub velocity: Vec3,
    /// Push from hits, fading out over a fraction of a second.
    pub knock: Vec3,
    pub yaw: f32,
    pub on_floor: bool,
    pub gait: String,
    pub anim: AnimClip,
    pub anim_time: f32,
    pub fade: AnimFade,
    pub last_knock: Vec3,
    pub dead_for: f32,
    pub pending_hit: Option<DamageInfo>,
    /// Accumulated delta time for distant throttled thinking (Godot Zombie.cs:285).
    pub think_accumulator: f32,
    pub last_out: Option<BrainOutput>,
    /// Brute, screamer or crawler only (None for walkers and runners, zero cost).
    pub special: Option<SpecialState>,
    /// From the last special directive: open to extra damage (stunned or recovering).
    pub special_vulnerable: bool,
    /// The mover hit a wall last frame (only read by specials).
    pub special_hit_wall: bool,
    /// Last sight check for specials (throttled).
    pub special_los: bool,
    pub special_sight_timer: f32,
}

/// What happened to the world / player this frame.
#[derive(Default)]
pub struct ZombieFrame {
    pub bites: Vec<AttackHit>,
    /// Hits by special zombies this frame (the player takes the damage and the push).
    pub special_hits: Vec<SpecialHit>,
    /// Sounds emitted by zombies (e.g. screams).
    pub noises: Vec<Noise>,
    /// Positions where zombies started screaming this frame, for audio trigger.
    pub screams: Vec<Vec3>,
    /// Alert and pacing events emitted this frame.
    pub events: Vec<ZombieEvent>,
}

/// Events emitted by the zombie simulation during a frame.
#[derive(Debug, Clone, PartialEq)]
pub enum ZombieEvent {
    HordeFormed { size: usize, position: Vec3 },
    HordeNear { size: usize, distance: f32 },
    SpecialSeen { kind: String },
    CampAttacked,
}

/// Tracks alerts for hordes, specials, and camp attacks to prevent spam.
#[derive(Debug, Clone, Default)]
pub struct ZombieEventTracker {
    pub known_hordes: std::collections::HashSet<u64>,
    pub warned_near_hordes: std::collections::HashSet<u64>,
    pub seen_specials: std::collections::HashSet<u64>,
    pub last_near_horde_pos: Option<Vec3>,
}

impl ZombieEventTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates horde formations, hordes near player, special sightings, and camp attacks.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate(
        &mut self,
        hordes: &[game::horde::Horde],
        zombies: &[Zombie],
        player: Option<Vec3>,
        player_in_safe_zone: bool,
        camp_center: Option<Vec3>,
        camp_wall_radius: f32,
        near_distance: f32,
        special_distance: f32,
        horde_min_size: usize,
        events: &mut Vec<ZombieEvent>,
    ) {
        // 1. HordeFormed: first time a horde leader is seen
        for horde in hordes {
            if self.known_hordes.insert(horde.leader) {
                events.push(ZombieEvent::HordeFormed {
                    size: horde.member_count(),
                    position: horde.leader_pos,
                });
            }
        }

        // 2. HordeNear: first time a horde of horde_min_size comes within near_distance
        if let Some(p) = player {
            for horde in hordes {
                let size = horde.member_count();
                if size >= horde_min_size {
                    let dist = horde.leader_pos.distance(p);
                    if dist <= near_distance && self.warned_near_hordes.insert(horde.leader) {
                        self.last_near_horde_pos = Some(horde.leader_pos);
                        events.push(ZombieEvent::HordeNear {
                            size,
                            distance: dist,
                        });
                    }
                }
            }
        }

        // 3. SpecialSeen: first sighting of brute, screamer, or crawler within special_distance
        if let Some(p) = player {
            for z in zombies {
                if !z.health.is_dead()
                    && matches!(z.kind.as_str(), "brute" | "screamer" | "crawler")
                    && z.position.distance(p) <= special_distance
                    && self.seen_specials.insert(z.id)
                {
                    events.push(ZombieEvent::SpecialSeen {
                        kind: z.kind.clone(),
                    });
                }
            }
        }

        // 4. CampAttacked: zombie inside camp wall radius while player is in camp
        if player_in_safe_zone {
            let center = camp_center.or(player);
            if let Some(c) = center {
                let inside_camp = zombies.iter().any(|z| {
                    !z.health.is_dead() && z.position.distance(c) <= camp_wall_radius
                });
                if inside_camp {
                    events.push(ZombieEvent::CampAttacked);
                }
            }
        }
    }
}

/// The clip name for a zombie's current animation (ZombieVisual: gait for walking, the type's run clip).
pub fn clip_name(z: &Zombie) -> &str {
    let anim_key = match z.anim {
        AnimClip::Idle => "idle",
        AnimClip::Walk => "walk",
        AnimClip::Run => "run",
        AnimClip::Attack => "attack",
        AnimClip::Hit => "hit",
        AnimClip::Die => "die",
        AnimClip::Scream => "scream",
        AnimClip::Eat => "eat",
        AnimClip::Bang => "bang",
        AnimClip::Crawl => "crawl",
    };
    if let Some(c) = z.def.clips.get(anim_key) {
        return c.as_str();
    }
    let default_name = match z.anim {
        AnimClip::Idle => "idle",
        AnimClip::Walk => z.gait.as_str(),
        AnimClip::Run => z.def.run_clip.as_deref().unwrap_or(z.gait.as_str()),
        AnimClip::Attack => "zombie_attack",
        AnimClip::Hit => "zombie_stagger",
        AnimClip::Die => "die",
        AnimClip::Scream => "zombie_scream",
        AnimClip::Eat => "zombie_eat",
        AnimClip::Bang => "zombie_bang",
        AnimClip::Crawl => "zombie_crawl",
    };
    if let Some(c) = z.def.clips.get(default_name) {
        return c.as_str();
    }
    default_name
}

/// Resolves the clip name including directional death clips when the look provides them.
pub(super) fn resolved_clip_name<'a>(z: &'a Zombie, look: &Look) -> &'a str {
    let base = clip_name(z);
    if z.anim == AnimClip::Die && base == "die" {
        let directional = death_clip(z.last_knock, z.yaw);
        if look.clip(directional).is_some() {
            return directional;
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::horde::Horde;

    #[test]
    fn test_hud_horde_near_fires_once_per_horde() {
        let mut tracker = ZombieEventTracker::new();
        let player = Vec3::new(0.0, 0.0, 0.0);

        // Horde of 6 zombies at 45m distance (<= 60m, size >= 5)
        let mut horde1 = Horde::new(101, vec![102, 103, 104, 105, 106]);
        horde1.set_leader_pos(Vec3::new(45.0, 0.0, 0.0));

        let mut events = Vec::new();
        tracker.evaluate(
            &[horde1.clone()],
            &[],
            Some(player),
            false,
            None,
            20.0,
            60.0,
            30.0,
            5,
            &mut events,
        );

        // Both HordeFormed and HordeNear should be emitted
        assert!(events.iter().any(|e| matches!(e, ZombieEvent::HordeFormed { size: 6, .. })));
        assert!(events.iter().any(|e| matches!(e, ZombieEvent::HordeNear { size: 6, distance } if (distance - 45.0).abs() < 1e-4)));

        // Second evaluation with the exact same horde: neither HordeNear nor HordeFormed should fire again!
        let mut events2 = Vec::new();
        tracker.evaluate(
            &[horde1.clone()],
            &[],
            Some(player),
            false,
            None,
            20.0,
            60.0,
            30.0,
            5,
            &mut events2,
        );
        assert!(events2.is_empty(), "HordeNear must fire only once per horde");

        // Small horde (size 4 < 5) at 30m: should NOT trigger HordeNear
        let mut horde_small = Horde::new(201, vec![202, 203, 204]);
        horde_small.set_leader_pos(Vec3::new(30.0, 0.0, 0.0));
        let mut events3 = Vec::new();
        tracker.evaluate(
            &[horde1.clone(), horde_small.clone()],
            &[],
            Some(player),
            false,
            None,
            20.0,
            60.0,
            30.0,
            5,
            &mut events3,
        );
        assert!(!events3.iter().any(|e| matches!(e, ZombieEvent::HordeNear { .. })));

        // Far horde (size 6, distance 75m > 60m): should NOT trigger HordeNear
        let mut horde_far = Horde::new(301, vec![302, 303, 304, 305, 306]);
        horde_far.set_leader_pos(Vec3::new(75.0, 0.0, 0.0));
        let mut events4 = Vec::new();
        tracker.evaluate(
            &[horde1, horde_small, horde_far],
            &[],
            Some(player),
            false,
            None,
            20.0,
            60.0,
            30.0,
            5,
            &mut events4,
        );
        assert!(!events4.iter().any(|e| matches!(e, ZombieEvent::HordeNear { .. })));
    }
}
