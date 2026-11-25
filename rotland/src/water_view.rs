//! Water view: provides test water patches and draws water surfaces.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

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

/// Returns a cached non-indexed triangle list for the lake's watertight water mesh.
fn lake_mesh_vertices(lake: &world::LakeLayout) -> Arc<Vec<Vec3>> {
    static CACHE: Mutex<Option<(u32, Arc<Vec<Vec3>>)>> = Mutex::new(None);
    let key = lake.water_level.to_bits() ^ lake.center.x.to_bits() ^ lake.center.y.to_bits();
    let mut cache = match CACHE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some((k, verts)) = cache.as_ref()
        && *k == key
    {
        return verts.clone();
    }
    let mesh = lake.build_water_mesh();
    log::info!(
        "lake water mesh generated: {} vertices, {} triangles",
        mesh.vertices.len(),
        mesh.triangle_count()
    );
    let tri_list = Arc::new(mesh.to_triangle_list());
    *cache = Some((key, tri_list.clone()));
    tri_list
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

    let mut custom_lake = None;
    if let Some(lake) = world.world_gen.as_ref().and_then(|w| w.lake()) {
        let (min, max) = lake.water_bounds();
        let cam_pos = glam::Vec2::new(camera.position.x, camera.position.z);
        let dx = (min.x - cam_pos.x).max(0.0).max(cam_pos.x - max.x);
        let dz = (min.y - cam_pos.y).max(0.0).max(cam_pos.y - max.y);
        let dist_to_box = (dx * dx + dz * dz).sqrt();
        if dist_to_box <= 600.0 {
            let verts = lake_mesh_vertices(lake);
            let aabb_min = Vec3::new(min.x, lake.water_level, min.y);
            let aabb_max = Vec3::new(max.x, lake.water_level, max.y);
            let key = (lake.water_level.to_bits() as u64)
                ^ ((lake.center.x.to_bits() as u64) << 16)
                ^ ((lake.center.y.to_bits() as u64) << 32);
            custom_lake = Some((key, (aabb_min, aabb_max), verts));
        }
    }

    let river_mesh = crate::river_view::mesh_for_frame(world, spawn, time);

    if patches.is_empty() && custom_lake.is_none() && river_mesh.is_none() {
        return;
    }

    let mut guard = match WATER_RENDERER.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let renderer = guard.get_or_insert_with(|| WaterRenderer::new(gpu));
    if !patches.is_empty() || custom_lake.is_some() {
        let custom_mesh_ref =
            custom_lake.as_ref().map(|(k, aabb, verts)| (*k, *aabb, verts.as_slice()));
        renderer.draw_all(gpu, target, encoder, camera, light, time, &patches, custom_mesh_ref);
    }
    if let Some((key, aabb, verts)) = river_mesh.as_ref() {
        renderer.draw_all_with_flow(
            gpu,
            target,
            encoder,
            camera,
            light,
            time,
            &[],
            Some((*key, *aabb, verts.as_slice())),
        );
    }
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
