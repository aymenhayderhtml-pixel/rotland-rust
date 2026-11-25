//! River water surface mesh caching and off-thread rebuilding around the player.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use engine::render::WaterVertex;
use glam::{Vec2, Vec3};
use world::WorldGenerator;

use crate::app::state::WorldState;

/// Radius around the player for river water mesh extraction (m).
const RIVER_MESH_RADIUS_M: f32 = 300.0; // new: DECISIONS 2026-11-25
/// Distance moved before forcing a river mesh rebuild (m).
const REBUILD_MOVE_DIST_M: f32 = 50.0; // new: DECISIONS 2026-11-25
/// Minimum interval between periodic river mesh rebuilds (s).
const REBUILD_INTERVAL_S: f32 = 1.0; // new: DECISIONS 2026-11-25
/// Key namespace bit so river meshes never collide with lake keys in `WaterRenderer::custom_cache`.
const RIVER_KEY_PREFIX: u64 = 0x5249_5645_0000_0000;

struct BuiltRiverMesh {
    verts: Vec<WaterVertex>,
    aabb: (Vec3, Vec3),
    elapsed_ms: f32,
}

struct RiverMeshCache {
    last_center: Option<Vec2>,
    last_rebuild_time: f32,
    in_flight: bool,
    generation: u64,
    verts: Arc<Vec<WaterVertex>>,
    aabb: (Vec3, Vec3),
    tx: Sender<BuiltRiverMesh>,
    rx: Receiver<BuiltRiverMesh>,
}

impl RiverMeshCache {
    fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            last_center: None,
            last_rebuild_time: f32::NEG_INFINITY,
            in_flight: false,
            generation: 0,
            verts: Arc::new(Vec::new()),
            aabb: (Vec3::ZERO, Vec3::ZERO),
            tx,
            rx,
        }
    }
}

static CACHE: Mutex<Option<RiverMeshCache>> = Mutex::new(None);

/// Computes the bounding box of a non-empty triangle list.
#[cfg(test)]
fn compute_aabb(verts: &[Vec3]) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for &v in verts {
        min = min.min(v);
        max = max.max(v);
    }
    (min, max)
}

/// Returns true when the river mesh should be rebuilt for `player_xz` at `time`.
fn should_rebuild(last_center: Option<Vec2>, last_time: f32, player_xz: Vec2, time: f32) -> bool {
    let Some(prev) = last_center else {
        return true;
    };
    player_xz.distance(prev) >= REBUILD_MOVE_DIST_M
        || (time - last_time).abs() >= REBUILD_INTERVAL_S
}

/// `(cache_key, (aabb_min, aabb_max), triangle_vertices)` for river water rendering.
pub type RiverMeshFrame = (u64, (Vec3, Vec3), Arc<Vec<WaterVertex>>);

/// Returns `(cache_key, aabb, vertices)` for river water near `player`, rebuilding off-thread
/// every ~1 s or when the player moves 50 m.
pub fn mesh_for_frame(
    world: &WorldState,
    player: Vec3,
    time: f32,
) -> Option<RiverMeshFrame> {
    let world_gen: Arc<WorldGenerator> = world.world_gen.clone()?;
    if world_gen.rivers().rivers().is_empty() {
        return None;
    }

    let mut guard = match CACHE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let cache = guard.get_or_insert_with(RiverMeshCache::new);

    while let Ok(built) = cache.rx.try_recv() {
        cache.in_flight = false;
        if !(built.verts.is_empty() && cache.verts.is_empty()) {
            cache.generation = cache.generation.wrapping_add(1);
            cache.verts = Arc::new(built.verts);
            cache.aabb = built.aabb;
        }
        log::debug!(
            "river water mesh rebuilt in {:.2} ms ({} verts)",
            built.elapsed_ms,
            cache.verts.len()
        );
    }

    let player_xz = Vec2::new(player.x, player.z);
    if !cache.in_flight
        && should_rebuild(cache.last_center, cache.last_rebuild_time, player_xz, time)
    {
        cache.in_flight = true;
        cache.last_center = Some(player_xz);
        cache.last_rebuild_time = time;
        let tx = cache.tx.clone();
        rayon::spawn(move || {
            let start = Instant::now();
            let raw = world_gen
                .rivers()
                .water_mesh_with_flow_near(player_xz, RIVER_MESH_RADIUS_M);
            let aabb = if raw.is_empty() {
                (Vec3::ZERO, Vec3::ZERO)
            } else {
                let mut min = Vec3::splat(f32::INFINITY);
                let mut max = Vec3::splat(f32::NEG_INFINITY);
                for &(v, _) in &raw {
                    min = min.min(v);
                    max = max.max(v);
                }
                (min, max)
            };
            let verts: Vec<WaterVertex> = raw
                .into_iter()
                .map(|(pos, flow)| WaterVertex::with_flow(pos, flow))
                .collect();
            let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
            let _ = tx.send(BuiltRiverMesh {
                verts,
                aabb,
                elapsed_ms,
            });
        });
    }

    if cache.verts.is_empty() {
        return None;
    }

    let key = RIVER_KEY_PREFIX ^ cache.generation;
    Some((key, cache.aabb, cache.verts.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuild_trigger_respects_distance_and_interval() {
        assert!(should_rebuild(None, 0.0, Vec2::ZERO, 0.0));
        assert!(!should_rebuild(Some(Vec2::ZERO), 10.0, Vec2::new(10.0, 0.0), 10.5));
        assert!(should_rebuild(Some(Vec2::ZERO), 10.0, Vec2::new(50.0, 0.0), 10.5));
        assert!(should_rebuild(Some(Vec2::ZERO), 10.0, Vec2::new(1.0, 0.0), 11.0));
    }

    #[test]
    fn compute_aabb_covers_all_vertices() {
        let verts = vec![
            Vec3::new(-10.0, 2.0, 5.0),
            Vec3::new(15.0, 4.0, -3.0),
            Vec3::new(0.0, 3.0, 12.0),
        ];
        let (min, max) = compute_aabb(&verts);
        assert_eq!(min, Vec3::new(-10.0, 2.0, -3.0));
        assert_eq!(max, Vec3::new(15.0, 4.0, 12.0));
    }
}
