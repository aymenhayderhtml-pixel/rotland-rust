use glam::{Mat4, Vec3, Vec4};

/// A first-person style camera: position plus yaw/pitch. Yaw 0 looks toward -Z (Godot's forward).
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    /// Vertical field of view in degrees (Godot's Camera3D default "keep height" mode).
    pub fov_y_deg: f32,
    pub near: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { position: Vec3::new(0.0, 10.0, 0.0), yaw: 0.0, pitch: 0.0, fov_y_deg: 75.0, near: 0.05 }
    }
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(-sy * cp, sp, -cy * cp)
    }

    /// Points the camera from its position at `target`.
    pub fn look_at(&mut self, target: Vec3) {
        let d = (target - self.position).normalize_or_zero();
        self.pitch = d.y.clamp(-1.0, 1.0).asin();
        self.yaw = (-d.x).atan2(-d.z);
    }

    /// Right-handed view matrix (camera looks down its -Z), written out so it never depends on library naming.
    pub fn view(&self) -> Mat4 {
        view_matrix(self.position, self.forward())
    }

    /// Reverse-Z infinite projection: depth 1 at the near plane, 0 at infinity. Far more depth precision
    /// over a big outdoor world than normal Z, and no far plane to tune.
    pub fn projection(&self, aspect: f32) -> Mat4 {
        let f = 1.0 / (self.fov_y_deg.to_radians() * 0.5).tan();
        Mat4::from_cols(
            Vec4::new(f / aspect, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, -1.0),
            Vec4::new(0.0, 0.0, self.near, 0.0),
        )
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.projection(aspect) * self.view()
    }
}

/// Right-handed view matrix looking from `eye` along `forward` (the shadow camera uses it too).
pub fn view_matrix(eye: Vec3, forward: Vec3) -> Mat4 {
    let f = forward.normalize_or(Vec3::NEG_Z);
    let s = f.cross(Vec3::Y).normalize_or(Vec3::X);
    let u = s.cross(f);
    Mat4::from_cols(
        Vec4::new(s.x, u.x, -f.x, 0.0),
        Vec4::new(s.y, u.y, -f.y, 0.0),
        Vec4::new(s.z, u.z, -f.z, 0.0),
        Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
    )
}

/// Orthographic projection with wgpu's 0..1 depth (0 at `near`, 1 at `far`).
pub fn ortho(half_width: f32, half_height: f32, near: f32, far: f32) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(1.0 / half_width, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0 / half_height, 0.0, 0.0),
        Vec4::new(0.0, 0.0, -1.0 / (far - near), 0.0),
        Vec4::new(0.0, 0.0, -near / (far - near), 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_at_points_forward_at_the_target() {
        let mut c = Camera { position: Vec3::new(1.0, 2.0, 3.0), ..Default::default() };
        let target = Vec3::new(-4.0, 0.0, 10.0);
        c.look_at(target);
        assert!((c.forward() - (target - c.position).normalize()).length() < 1e-5);
    }

    #[test]
    fn near_plane_maps_to_depth_one() {
        let c = Camera { position: Vec3::ZERO, ..Default::default() };
        let p = c.view_proj(1.0) * Vec4::new(0.0, 0.0, -c.near, 1.0);
        assert!((p.z / p.w - 1.0).abs() < 1e-5);
        let far = c.view_proj(1.0) * Vec4::new(0.0, 0.0, -1000.0, 1.0);
        assert!(far.z / far.w > 0.0 && far.z / far.w < 0.001);
    }

    #[test]
    fn yaw_zero_looks_down_negative_z() {
        assert!((Camera::default().forward() - Vec3::NEG_Z).length() < 1e-6);
    }
}
