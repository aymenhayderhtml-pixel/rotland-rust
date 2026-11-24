//! First-person weapon viewmodel matrix calculation, sway bob/kick, and wall-clip pull.
//!
//! Port of first-person weapon positioning from Godot (`WeaponMounts.cs`, `WeaponVisuals.cs`,
//! `GunEffects.cs`, `MeleeHandler.cs`, `data/weapons.json`).
//! In first-person view, held weapons are drawn in front of the camera using `fp_weapon_matrix`
//! rather than the third-person right-hand bone matrix.

use engine::render::Camera;
use game::WeaponDef;
use glam::{EulerRot, Mat4, Vec3};
use physics::PhysicsWorld;

/// Default first-person mount position relative to camera from Godot `WeaponMounts.FirstPersonMount`.
pub const FIRST_PERSON_MOUNT: Vec3 = Vec3::new(0.28, -0.22, -0.45);
/// Default first-person rest pose rotation in degrees from Godot `MeleeHandler.RestPose`.
pub const FIRST_PERSON_REST_POSE_DEG: Vec3 = Vec3::new(-20.0, 0.0, 0.0);
/// Horizontal bob amplitude while walking in metres.
pub const BOB_AMPLITUDE_X: f32 = 0.015;
/// Vertical bob amplitude while walking in metres.
pub const BOB_AMPLITUDE_Y: f32 = 0.010;
/// Bobbing cycle frequency (radians per second at normal walk speed).
pub const BOB_SPEED: f32 = 8.0;
/// Gun shot kick in camera space: moves up 0.01m, back 0.04m (Godot `GunEffects.ViewKick`).
pub const GUN_KICK_OFFSET: Vec3 = Vec3::new(0.0, 0.01, 0.04);
/// Melee swing kick rotation in degrees: winds back pitch by 20 deg, turns yaw by -15 deg.
pub const SWING_KICK_ROT_DEG: Vec3 = Vec3::new(20.0, -15.0, 0.0);
/// Melee swing kick translation offset in camera space.
pub const SWING_KICK_OFFSET: Vec3 = Vec3::new(-0.02, 0.02, 0.05);
/// Minimum camera-space distance along -Z so the model does not penetrate the near clipping plane.
pub const MIN_CAMERA_DISTANCE: f32 = 0.06;

/// First-person weapon sway and animation state.
///
/// Tracks walking bob, firing / swing kick offsets, and wall-pull displacement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FpSway {
    /// Walking bob phase accumulator in radians.
    pub bob_phase: f32,
    /// Movement speed ratio (0.0 = idle, 1.0 = full walk speed).
    pub move_speed: f32,
    /// Kick translation offset in camera space (+X right, +Y up, +Z toward camera).
    pub kick_offset: Vec3,
    /// Kick rotation in degrees (Euler angles in Godot order YXZ).
    pub kick_rot_deg: Vec3,
    /// Distance in metres the weapon is pulled toward the camera (+Z) to avoid clipping into walls.
    pub wall_pull: f32,
    /// Procedural body-coupled translation offset in camera space (+X right, +Y up, +Z toward camera).
    pub motion_offset: Vec3,
    /// Procedural body-coupled extra rotation in degrees (x pitch, y yaw, z roll).
    pub motion_rot_deg: Vec3,
}

impl Default for FpSway {
    fn default() -> Self {
        Self {
            bob_phase: 0.0,
            move_speed: 0.0,
            kick_offset: Vec3::ZERO,
            kick_rot_deg: Vec3::ZERO,
            wall_pull: 0.0,
            motion_offset: Vec3::ZERO,
            motion_rot_deg: Vec3::ZERO,
        }
    }
}

impl FpSway {
    /// Triggers weapon kick for a gun shot (moves up 0.01m, back 0.04m, matching Godot `GunEffects.ViewKick`).
    pub fn trigger_shot(&mut self) {
        self.kick_offset += GUN_KICK_OFFSET;
    }

    /// Triggers weapon kick / swing pose for a melee attack.
    pub fn trigger_swing(&mut self) {
        self.kick_offset += SWING_KICK_OFFSET;
        self.kick_rot_deg += SWING_KICK_ROT_DEG;
    }

    /// Updates sway over time: advances bobbing when moving and recovers kicks toward zero.
    pub fn update(&mut self, dt: f32, horizontal_speed: f32) {
        let target_ratio = (horizontal_speed / 4.0).clamp(0.0, 1.5);
        self.move_speed += (target_ratio - self.move_speed) * (1.0 - (-10.0 * dt).exp());
        if self.move_speed > 0.01 {
            self.bob_phase = (self.bob_phase + dt * BOB_SPEED * self.move_speed) % std::f32::consts::TAU;
        }

        let decay = (-15.0 * dt).exp();
        self.kick_offset *= decay;
        self.kick_rot_deg *= decay;
    }

    /// Computes the current walking bob translation offset in camera space.
    pub fn bob_offset(&self) -> Vec3 {
        if self.move_speed <= 0.001 {
            Vec3::ZERO
        } else {
            let x = self.bob_phase.sin() * BOB_AMPLITUDE_X * self.move_speed;
            let y = (self.bob_phase * 2.0).cos().abs() * BOB_AMPLITUDE_Y * self.move_speed;
            Vec3::new(x, y, 0.0)
        }
    }
}

/// Forward reach of the weapon model from the camera in metres.
pub fn weapon_reach(def: &WeaponDef) -> f32 {
    let hold_z = def
        .fp_hold
        .as_ref()
        .map(|h| h.pos[2])
        .unwrap_or(FIRST_PERSON_MOUNT.z);
    let length = if def.size[2] > 0.0 { def.size[2] } else { 0.85 };
    -hold_z + length
}

/// Computes the wall-pull distance (in metres) to pull the weapon toward the camera
/// so its tip does not penetrate obstacles in front of the player.
///
/// Casts a ray along the camera's forward direction up to `weapon_reach(def)`.
/// If a hit occurs at distance `d < reach`, the weapon would penetrate the wall.
/// We return the penetration depth `(reach - d)`, moving the weapon closer to the camera.
pub fn compute_wall_pull(
    camera: &Camera,
    def: &WeaponDef,
    physics: &PhysicsWorld,
) -> f32 {
    let reach = weapon_reach(def);
    let origin = camera.position;
    let dir = camera.forward();
    if let Some(hit) = physics.raycast(origin, dir, reach)
        && hit.distance < reach
    {
        (reach - hit.distance).max(0.0)
    } else {
        0.0
    }
}

/// Computes the weapon model's world matrix placed in front of the camera from `fpHold`
/// (pos in camera space, rot in degrees, Godot order YXZ), applying sway, bob, and wall pull.
pub fn fp_weapon_matrix(
    camera: &Camera,
    def: &WeaponDef,
    sway: FpSway,
) -> Mat4 {
    let (hold_pos, hold_rot_deg) = match &def.fp_hold {
        Some(hold) => (Vec3::from_array(hold.pos), Vec3::from_array(hold.rot)),
        None => (FIRST_PERSON_MOUNT, FIRST_PERSON_REST_POSE_DEG),
    };

    let bob = sway.bob_offset();
    let x = hold_pos.x + bob.x + sway.kick_offset.x + sway.motion_offset.x;
    let y = hold_pos.y + bob.y + sway.kick_offset.y + sway.motion_offset.y;
    // Positive wall_pull pulls toward the camera along +Z. Clamped so it doesn't cross near plane.
    let z = (hold_pos.z + sway.wall_pull + sway.kick_offset.z + sway.motion_offset.z)
        .min(-MIN_CAMERA_DISTANCE);
    let local_pos = Vec3::new(x, y, z);

    let rot_x = (hold_rot_deg.x + sway.kick_rot_deg.x + sway.motion_rot_deg.x).to_radians();
    let rot_y = (hold_rot_deg.y + sway.kick_rot_deg.y + sway.motion_rot_deg.y).to_radians();
    let rot_z = (hold_rot_deg.z + sway.kick_rot_deg.z + sway.motion_rot_deg.z).to_radians();
    let local_rot = Mat4::from_euler(EulerRot::YXZ, rot_y, rot_x, rot_z);

    let local_transform = Mat4::from_translation(local_pos) * local_rot;

    let cam_world = Mat4::from_translation(camera.position)
        * Mat4::from_rotation_y(camera.yaw)
        * Mat4::from_rotation_x(camera.pitch);

    cam_world * local_transform
}

/// Convenience function to compute wall pull via raycast against `PhysicsWorld`,
/// apply it to `sway`, and return the weapon world matrix.
pub fn fp_weapon_matrix_with_physics(
    camera: &Camera,
    def: &WeaponDef,
    mut sway: FpSway,
    physics: &PhysicsWorld,
) -> Mat4 {
    sway.wall_pull = compute_wall_pull(camera, def, physics);
    fp_weapon_matrix(camera, def, sway)
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::WeaponHold;

    fn test_bat() -> WeaponDef {
        WeaponDef {
            id: "bat".to_string(),
            fp_hold: Some(WeaponHold { pos: [0.0, -0.05, -0.3], rot: [0.0, 0.0, 0.0] }),
            size: [0.08, 0.08, 0.85],
            ..Default::default()
        }
    }

    fn test_camera(yaw: f32) -> Camera {
        Camera { position: Vec3::ZERO, yaw, pitch: 0.0, fov_y_deg: 75.0, near: 0.05 }
    }

    #[test]
    fn camera_at_origin_looking_neg_z_lands_at_fp_hold_point() {
        let camera = test_camera(0.0);
        let bat = test_bat();
        let matrix = fp_weapon_matrix(&camera, &bat, FpSway::default());
        let pos = matrix.transform_point3(Vec3::ZERO);

        let expected = Vec3::new(0.0, -0.05, -0.3);
        assert!((pos - expected).length() < 1e-5, "expected {expected:?}, got {pos:?}");
    }

    #[test]
    fn yaw_90_deg_rotates_weapon_position() {
        let camera = test_camera(std::f32::consts::FRAC_PI_2);
        let bat = test_bat();
        let matrix = fp_weapon_matrix(&camera, &bat, FpSway::default());
        let pos = matrix.transform_point3(Vec3::ZERO);

        // Yaw 90 deg clockwise around +Y turns -Z forward into -X: x = -0.3, y = -0.05, z = 0.0
        let expected = Vec3::new(-0.3, -0.05, 0.0);
        assert!((pos - expected).length() < 1e-5, "expected {expected:?}, got {pos:?}");
    }

    #[test]
    fn wall_pull_moves_weapon_closer_to_camera() {
        let camera = test_camera(0.0);
        let bat = test_bat();

        let m_unpulled = fp_weapon_matrix(&camera, &bat, FpSway::default());
        let pos_unpulled = m_unpulled.transform_point3(Vec3::ZERO);

        let sway_pulled = FpSway { wall_pull: 0.15, ..Default::default() };
        let m_pulled = fp_weapon_matrix(&camera, &bat, sway_pulled);
        let pos_pulled = m_pulled.transform_point3(Vec3::ZERO);

        let dist_unpulled = (pos_unpulled - camera.position).length();
        let dist_pulled = (pos_pulled - camera.position).length();

        assert!(dist_pulled < dist_unpulled, "expected pulled distance {dist_pulled} < unpulled {dist_unpulled}");
        assert!((pos_unpulled.z - (-0.3)).abs() < 1e-5);
        assert!((pos_pulled.z - (-0.15)).abs() < 1e-5);
    }

    #[test]
    fn physics_wall_pull_raycast_moves_weapon_closer() {
        let camera = test_camera(0.0);
        let bat = test_bat();

        let mut world = PhysicsWorld::new();
        // Wall obstacle at z = -0.5 (half-thickness 0.1, front surface at z = -0.4, 0.4m in front).
        // Bat reach = 0.3 + 0.85 = 1.15m > 0.4m, so it penetrates by 0.75m.
        let (wx, wy, wz) = (1.0, 1.0, 0.1);
        let points = vec![
            [-wx, -wy, -wz], [wx, -wy, -wz], [wx, wy, -wz], [-wx, wy, -wz],
            [-wx, -wy, wz], [wx, wy, wz], [wx, wy, wz], [-wx, wy, wz],
        ];
        world.add_static(&[points], Vec3::new(0.0, 0.0, -0.5), 0.0, 1.0);

        let pull = compute_wall_pull(&camera, &bat, &world);
        assert!(pull > 0.0, "expected positive wall pull, got {pull}");

        let m_unpulled = fp_weapon_matrix(&camera, &bat, FpSway::default());
        let pos_unpulled = m_unpulled.transform_point3(Vec3::ZERO);

        let m_pulled = fp_weapon_matrix_with_physics(&camera, &bat, FpSway::default(), &world);
        let pos_pulled = m_pulled.transform_point3(Vec3::ZERO);

        let dist_unpulled = (pos_unpulled - camera.position).length();
        let dist_pulled = (pos_pulled - camera.position).length();

        assert!(dist_pulled < dist_unpulled, "expected physics pulled distance {dist_pulled} < unpulled {dist_unpulled}");
    }
}
