use glam::{Mat4, Vec3, Vec4};

/// The six planes of the camera's view, for skipping meshes that are off screen.
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    /// Planes from a view-projection matrix with wgpu's 0..1 depth range (works for reverse-Z too;
    /// the infinite far plane simply never rejects anything).
    pub fn from_view_proj(m: &Mat4) -> Self {
        let r = |i: usize| m.row(i);
        let planes = [r(3) + r(0), r(3) - r(0), r(3) + r(1), r(3) - r(1), r(2), r(3) - r(2)];
        Self { planes: planes.map(|p| p / p.truncate().length().max(1e-6)) }
    }

    /// False only when the box is fully outside one plane.
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        self.planes.iter().all(|p| {
            let n = p.truncate();
            let corner = Vec3::new(
                if n.x >= 0.0 { max.x } else { min.x },
                if n.y >= 0.0 { max.y } else { min.y },
                if n.z >= 0.0 { max.z } else { min.z },
            );
            n.dot(corner) + p.w >= 0.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::camera::Camera;

    #[test]
    fn box_in_front_is_visible_and_behind_is_not() {
        let cam = Camera { position: Vec3::ZERO, ..Default::default() };
        let f = Frustum::from_view_proj(&cam.view_proj(16.0 / 9.0));
        assert!(f.intersects_aabb(Vec3::new(-1.0, -1.0, -11.0), Vec3::new(1.0, 1.0, -9.0)));
        assert!(!f.intersects_aabb(Vec3::new(-1.0, -1.0, 9.0), Vec3::new(1.0, 1.0, 11.0)));
    }
}
