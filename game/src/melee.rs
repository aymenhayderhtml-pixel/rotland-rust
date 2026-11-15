//! Melee swings (port of scripts/Combat/MeleeHandler.cs and scripts/Combat/GunBash.cs).
//!
//! A swing has a windup, then the hit check happens at the impact moment. Cooldown starts when the
//! swing starts, as in C#. The caller sends input, checks that the player is alive and can act, and
//! passes in only living targets.

use glam::Vec3;
use physics::PhysicsWorld;

use crate::damage::DamageInfo;
use crate::player::Stamina;
use crate::weapons::{WeaponAnim, WeaponDef, WeaponKind};
use crate::zombie::Noise;

/// The swing's sphere starts this high above the feet (C# HitHeight).
const HIT_HEIGHT: f32 = 1.0;
/// Where a hit lands on a target, above its feet (C# ChestHeight). Not the zombie senses value.
const CHEST_HEIGHT: f32 = 0.9;
/// A wall this close to the target still counts as blocking, so grazing contact is not a false pass.
const WALL_MARGIN: f32 = 0.05;
/// Below this length a direction is treated as zero (C# uses 0.001).
const EPS: f32 = 0.001;

// Gun bash numbers (GunBash.cs). Only the damage, windup and clip come from the gun.
const BASH_RANGE: f32 = 1.9;
const BASH_ARC_DEG: f32 = 80.0;
const BASH_COOLDOWN: f32 = 0.7;
const BASH_STAMINA_COST: f32 = 10.0;
const BASH_KNOCKBACK: f32 = 2.0;
const BASH_STAGGER: f32 = 0.3;
const BASH_NOISE_RADIUS: f32 = 6.0;

/// The one who swings. `facing` gives the swing direction; its height is ignored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attacker {
    /// Used as the source of each hit's damage.
    pub id: u64,
    /// Feet position.
    pub position: Vec3,
    pub facing: Vec3,
    /// Where sight starts for the wall check. Use head height, not a third-person camera
    /// behind the player, or a wall behind the player would block the hit.
    pub eye: Vec3,
}

/// A living thing that can be hit. `position` is its feet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeTarget {
    pub id: u64,
    pub position: Vec3,
    /// Half the width of the body. Lets a target at the edge of the range still count.
    pub radius: f32,
}

/// One target struck by a swing, ready to apply.
#[derive(Clone, Debug, PartialEq)]
pub struct MeleeHit {
    pub target: u64,
    pub damage: DamageInfo,
    /// Push away from the attacker: the horizontal direction times the weapon's knockback.
    pub knockback: Vec3,
    pub stagger: f32,
}

/// Where a swing is. A swing can be cooling down while it is winding up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwingState {
    Idle,
    WindingUp,
    CoolingDown,
}

#[derive(Clone, Debug)]
struct Windup {
    weapon: WeaponDef,
    time_left: f32,
    /// Captured on the first update after the swing starts, as C# captures it at the start.
    dir: Option<Vec3>,
}

/// The state of one melee weapon user. Keep one per player.
#[derive(Clone, Debug, Default)]
pub struct MeleeSwing {
    cooldown_left: f32,
    windup: Option<Windup>,
    noise: Option<Noise>,
}

impl MeleeSwing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> SwingState {
        if self.windup.is_some() {
            SwingState::WindingUp
        } else if self.cooldown_left > 0.0 {
            SwingState::CoolingDown
        } else {
            SwingState::Idle
        }
    }

    /// Starts a swing if no swing is winding up or cooling down, and stamina covers the cost.
    /// Spending stamina resets its regen timer, the same as `Stamina::try_use` in C#.
    pub fn try_start(&mut self, weapon: &WeaponDef, stamina: &mut Stamina) -> bool {
        if self.windup.is_some() || self.cooldown_left > 0.0 {
            return false;
        }
        if stamina.current < weapon.stamina_cost {
            return false;
        }
        stamina.current -= weapon.stamina_cost;
        stamina.time_since_use = 0.0;

        self.windup = Some(Windup {
            weapon: weapon.clone(),
            time_left: weapon.windup,
            dir: None,
        });
        self.cooldown_left = weapon.cooldown;
        true
    }

    /// Stops a windup in progress, for example when switching weapon. The cooldown keeps running,
    /// as the C# Cancel does, so a cancelled swing cannot be repeated at once.
    pub fn cancel(&mut self) {
        self.windup = None;
    }

    /// Advances the cooldown and the windup. Returns the hits when the windup ends (once per swing).
    /// Hits happen only at the impact moment, never earlier.
    pub fn update(
        &mut self,
        dt: f32,
        attacker: Attacker,
        targets: &[MeleeTarget],
        physics: &PhysicsWorld,
    ) -> Vec<MeleeHit> {
        self.cooldown_left = (self.cooldown_left - dt).max(0.0);
        let Some(windup) = self.windup.as_mut() else {
            return Vec::new();
        };
        let dir = *windup.dir.get_or_insert_with(|| horizontal_unit(attacker.facing));
        windup.time_left -= dt;
        if windup.time_left > 0.0 {
            return Vec::new();
        }

        let Some(done) = self.windup.take() else {
            return Vec::new();
        };
        let hits = resolve(&done.weapon, dir, attacker, targets, physics);
        // The noise is made even when the swing misses, as in C#, so zombies still hear it.
        self.noise = Some(Noise {
            position: attacker.position,
            radius: done.weapon.noise_radius,
        });
        hits
    }

    /// The noise from the last swing that resolved. Returns it once, then None until the next swing.
    pub fn take_noise(&mut self) -> Option<Noise> {
        self.noise.take()
    }
}

/// The melee version of a gun, used when its magazine and spare ammo are both empty (GunBash.cs).
/// The gun's bash clip plays, and its bash damage and bash impact time set the hit.
pub fn gun_bash(gun: &WeaponDef) -> WeaponDef {
    WeaponDef {
        id: gun.id.clone(),
        name: gun.name.clone(),
        kind: WeaponKind::Melee,
        damage: gun.bash_damage,
        range: BASH_RANGE,
        arc_deg: BASH_ARC_DEG,
        // The windup is the impact time, so the clip plays at normal speed and lands on the hit.
        windup: gun.bash_impact,
        cooldown: BASH_COOLDOWN,
        stamina_cost: BASH_STAMINA_COST,
        knockback: BASH_KNOCKBACK,
        stagger: BASH_STAGGER,
        max_targets: 1,
        noise_radius: BASH_NOISE_RADIUS,
        anim: WeaponAnim {
            attack: gun.anim.bash.clone(),
            impact: gun.bash_impact,
            ..WeaponAnim::default()
        },
        color: gun.color.clone(),
        size: gun.size,
        ..WeaponDef::default()
    }
}

/// The hit check at the impact moment: living targets in range and the arc, not behind a wall,
/// nearest first, at most `max_targets`.
fn resolve(
    weapon: &WeaponDef,
    dir: Vec3,
    attacker: Attacker,
    targets: &[MeleeTarget],
    physics: &PhysicsWorld,
) -> Vec<MeleeHit> {
    let origin = attacker.position + Vec3::Y * HIT_HEIGHT;
    let cos_half_arc = (weapon.arc_deg * 0.5).to_radians().cos();

    let mut found: Vec<(f32, MeleeHit)> = Vec::new();
    for target in targets {
        let offset = Vec3::new(
            target.position.x - attacker.position.x,
            0.0,
            target.position.z - attacker.position.z,
        );
        let horizontal = offset.length();
        let centre = target.position + Vec3::Y * CHEST_HEIGHT;

        // In range when the swing's sphere reaches the target's body, not only its centre.
        if origin.distance(centre) - target.radius > weapon.range {
            continue;
        }
        let away = if horizontal > EPS { offset / horizontal } else { dir };
        if horizontal > EPS && dir.dot(away) < cos_half_arc {
            continue;
        }
        if sight_blocked(physics, attacker.eye, centre) {
            continue;
        }

        let hit = MeleeHit {
            target: target.id,
            damage: DamageInfo {
                amount: weapon.damage,
                source: Some(attacker.id),
                hit_point: centre,
                direction: dir,
                damage_type: "melee".to_string(),
                headshot: false,
            },
            knockback: away * weapon.knockback,
            stagger: weapon.stagger,
        };
        found.push((horizontal, hit));
    }

    // A stable sort keeps the input order for equal distances.
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found
        .into_iter()
        .take(weapon.max_targets as usize)
        .map(|(_, hit)| hit)
        .collect()
}

/// True when world geometry sits between `from` and `to`. Characters are not in the world group,
/// so the ray passes through zombies (as it should: they are the targets).
fn sight_blocked(physics: &PhysicsWorld, from: Vec3, to: Vec3) -> bool {
    let delta = to - from;
    let length = delta.length();
    if length <= EPS {
        return false;
    }
    physics
        .raycast(from, delta / length, length)
        .is_some_and(|hit| hit.distance < length - WALL_MARGIN)
}

/// Flattens a direction onto the ground and scales it to length 1 (zero stays zero).
fn horizontal_unit(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z).normalize_or_zero()
}
