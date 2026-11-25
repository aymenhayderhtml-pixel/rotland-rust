//! Math helpers for camp life rules (port of scripts/Settlement/CampMath.cs).

use glam::{Vec2, Vec3};
use world::DotnetRandom;

/// Pure math and hashing helpers for settlement and camp rules.
// Godot: CampMath.cs
pub struct CampMath;

impl CampMath {
    /// The Y rotation that turns a body with forward -Z to face the target.
    /// Godot's forward is -Z, so the yaw is atan2(-dx, -dz).
    // Godot: CampMath.cs YawToward
    pub fn yaw_toward(from: Vec3, to: Vec3) -> f32 {
        (-(to.x - from.x)).atan2(-(to.z - from.z))
    }

    /// Distance on the ground (X and Z only).
    // Godot: CampMath.cs Flat
    pub fn flat(a: Vec3, b: Vec3) -> f32 {
        Vec2::new(a.x - b.x, a.z - b.z).length()
    }

    /// A random number in [min, max] from the camp's seeded generator.
    // Godot: CampMath.cs Range
    pub fn range(rng: &mut DotnetRandom, min: f32, max: f32) -> f32 {
        min + rng.next_double() as f32 * (max - min)
    }

    /// A stable 32-bit number from a text seed (FNV-1a), so the camp is the same on every run.
    // Godot: CampMath.cs Hash
    pub fn hash(text: &str, salt: u32) -> u32 {
        let mut h = 2166136261u32 ^ salt;
        for c in text.chars() {
            h ^= c as u32;
            h = h.wrapping_mul(16777619u32);
        }
        h
    }

    /// A number in [0, 1) from the low 24 bits (Godot: SettlementNpc.cs Unit).
    pub fn unit(h: u32) -> f32 {
        (h & 0xFFFFFF) as f32 / 16777216.0
    }
}
