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
    if !is_enabled() {
        return;
    }
    let ground_y = world
        .physics
        .ground_height(spawn.x, spawn.z)
        .unwrap_or_else(|| world.height(spawn.x, spawn.z));
    let patch = test_patch(spawn, ground_y);

    let mut guard = match WATER_RENDERER.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let renderer = guard.get_or_insert_with(|| WaterRenderer::new(gpu));
    renderer.draw(gpu, target, encoder, camera, light, time, &[patch]);
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
