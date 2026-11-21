//! Water view: provides test water patches and draws water surfaces.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use engine::render::{Camera, Lighting, WaterPatch, WaterRenderer};
use engine::{EngineArgs, FrameTarget, Gpu};
use glam::Vec3;

use crate::app::state::WorldState;

static WATER_TEST: AtomicBool = AtomicBool::new(false);
static WATER_RENDERER: Mutex<Option<WaterRenderer>> = Mutex::new(None);

/// Initializes water test flag from CLI options.
pub fn init(args: &EngineArgs) {
    if args.water_test || args.flag("--water-test") {
        WATER_TEST.store(true, Ordering::Relaxed);
    }
}

/// Whether the water test view is active.
pub fn is_enabled() -> bool {
    WATER_TEST.load(Ordering::Relaxed)
}

/// Constructs a 200 x 200 m test patch at the camp ground height + 0.3 m, 60 m in front of spawn.
pub fn test_patch(spawn: Vec3, ground_y: f32) -> WaterPatch {
    // 60 m in front of spawn (facing -Z):
    let center_x = spawn.x;
    let center_z = spawn.z - 60.0;
    WaterPatch::new(
        glam::Vec2::new(center_x - 100.0, center_z - 100.0),
        glam::Vec2::new(center_x + 100.0, center_z + 100.0),
        ground_y + 0.3,
    )
}

/// Draws water surfaces if enabled or if test patches are active.
/// Strips of water that only cover the lake itself: a single rectangle over the lake's bounding box
/// also covered the low ground in its corners, so the shore looked flooded.
/// Rows of `TILE` m cells; a cell is water when any corner is inside the shore line; each run of water
/// cells in a row becomes one patch (about one or two per row, cached for the world's lifetime).
fn lake_patches(lake: &world::LakeLayout) -> Vec<WaterPatch> {
    const TILE: f32 = 8.0;
    static CACHE: Mutex<Option<(u32, Vec<WaterPatch>)>> = Mutex::new(None);
    let key = lake.water_level.to_bits() ^ lake.center.x.to_bits() ^ lake.center.y.to_bits();
    let mut cache = match CACHE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some((k, patches)) = cache.as_ref()
        && *k == key
    {
        return patches.clone();
    }
    let (min, max) = lake.water_bounds();
    let cols = ((max.x - min.x) / TILE).ceil() as i32;
    let rows = ((max.y - min.y) / TILE).ceil() as i32;
    let wet = |x: f32, z: f32| lake.depth_at(glam::Vec2::new(x, z)) > 0.0;
    let mut patches = Vec::new();
    for r in 0..rows {
        let z0 = min.y + r as f32 * TILE;
        let mut run_start: Option<i32> = None;
        for c in 0..=cols {
            let x0 = min.x + c as f32 * TILE;
            let water = c < cols
                && (wet(x0, z0) || wet(x0 + TILE, z0) || wet(x0, z0 + TILE) || wet(x0 + TILE, z0 + TILE));
            match (water, run_start) {
                (true, None) => run_start = Some(c),
                (false, Some(start)) => {
                    let a = glam::Vec2::new(min.x + start as f32 * TILE, z0);
                    patches.push(WaterPatch::new(a, glam::Vec2::new(x0, z0 + TILE), lake.water_level));
                    run_start = None;
                }
                _ => {}
            }
        }
    }
    *cache = Some((key, patches.clone()));
    patches
}

#[allow(clippy::too_many_arguments)]
pub fn draw(
    gpu: &Gpu,
    target: &FrameTarget,
    encoder: &mut wgpu::CommandEncoder,
    camera: &Camera,
    light: &Lighting,
    time: f32,
    world: &WorldState,
    spawn: Vec3,
) {
    let mut patches = Vec::new();

    if is_enabled() {
        let ground_y = world
            .physics
            .ground_height(spawn.x, spawn.z)
            .unwrap_or_else(|| world.height(spawn.x, spawn.z));
        patches.push(test_patch(spawn, ground_y));
    }

    if let Some(lake) = world.world_gen.as_ref().and_then(|w| w.lake()) {
        let (min, max) = lake.water_bounds();
        let cam_pos = glam::Vec2::new(camera.position.x, camera.position.z);
        let dx = (min.x - cam_pos.x).max(0.0).max(cam_pos.x - max.x);
        let dz = (min.y - cam_pos.y).max(0.0).max(cam_pos.y - max.y);
        let dist_to_box = (dx * dx + dz * dz).sqrt();
        if dist_to_box <= 600.0 {
            patches.extend(lake_patches(lake));
        }
    }

    if patches.is_empty() {
        return;
    }

    let mut guard = match WATER_RENDERER.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let renderer = guard.get_or_insert_with(|| WaterRenderer::new(gpu));
    renderer.draw(gpu, target, encoder, camera, light, time, &patches);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_test_patch_dimensions() {
        let spawn = Vec3::new(50.0, 10.0, 100.0);
        let patch = test_patch(spawn, 12.0);
        assert!((patch.level - 12.3).abs() < 1e-4);
        assert_eq!(patch.min.x, -50.0);
        assert_eq!(patch.max.x, 150.0);
        // center_z = 100 - 60 = 40
        assert_eq!(patch.min.y, -60.0);
        assert_eq!(patch.max.y, 140.0);
        assert_eq!(patch.max.x - patch.min.x, 200.0);
        assert_eq!(patch.max.y - patch.min.y, 200.0);
    }

    #[test]
    fn water_test_cli_flag() {
        let args = EngineArgs::parse(vec!["--water-test".to_string()]);
        init(&args);
        assert!(is_enabled());
    }
}
