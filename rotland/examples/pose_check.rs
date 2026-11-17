//! Verifies that `PlayerVisual` poses correctly and weapon stays attached to hand_R without NaNs.

#[path = "../src/player_visual.rs"]
mod player_visual;

use glam::{Mat4, Vec3};
use humanoid::Locomotion;
use pack::ModelPack;
use player_visual::PlayerVisual;

fn create_headless_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
        ..Default::default()
    }))
    .expect("Failed to find a compatible GPU adapter for headless pose check");
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("Failed to create headless device")
}

fn main() {
    println!("Running pose_check...");
    let (device, _queue) = create_headless_device();
    let pack = ModelPack::load("assets/models.pack")
        .or_else(|_| ModelPack::load("assets/models.pack"))
        .or_else(|_| ModelPack::load("assets/models.pack"))
        .or_else(|_| ModelPack::load("../assets/models.pack"))
        .expect("Failed to load models.pack");

    let mut visual = PlayerVisual::load(&device, &pack, "human_player", Some("bat"));
    let mut bones = vec![Mat4::IDENTITY]; // index 0 is identity

    let speeds = [0.0, 1.0, 2.0, 4.0, 5.5, 7.0, 8.5];
    let pos = Vec3::new(5.0, 1.0, -3.0);
    let yaw = 0.75;
    let char_world = Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw);

    for &speed in &speeds {
        let (body, weapon) = visual.pose(
            0.05,
            pos,
            yaw,
            Locomotion {
                speed,
                on_floor: true,
                crouching: false,
            },
            &mut bones,
        );
        let weapon_batch = weapon.expect("weapon batch must be present");
        assert_eq!(body.instances.len(), 1);
        assert_eq!(weapon_batch.instances.len(), 1);

        let body_inst = &body.instances[0];
        let weapon_inst = &weapon_batch.instances[0];
        assert_eq!(weapon_inst.bone_base, 0, "weapon must use static bone_base 0");

        let weapon_world = Mat4::from_cols_array_2d(&weapon_inst.model);
        let hand_model = visual.rig().joint_model_matrix(&bones[body_inst.bone_base as usize..], visual.hand_joint());
        let hand_world = char_world * hand_model;

        let hand_pos = hand_world.transform_point3(Vec3::ZERO);
        let weapon_pos = weapon_world.transform_point3(Vec3::ZERO);

        assert!(!hand_pos.is_nan(), "hand_pos has NaNs at speed {speed}");
        assert!(!weapon_pos.is_nan(), "weapon_pos has NaNs at speed {speed}");

        let dist = (weapon_pos - hand_pos).length();
        println!(
            "speed={speed:.1} m/s: hand_pos={hand_pos:.3}, weapon_pos={weapon_pos:.3}, distance={dist:.4} m"
        );

        // The grip offset is 0.06m (6cm) in the palm of hand_R
        assert!((dist - 0.06).abs() < 1e-4, "Distance {dist} is not 0.06m at speed {speed}");
    }

    println!("pose_check PASS: all speeds attached, no NaNs, distance tiny (0.06m)");
}
