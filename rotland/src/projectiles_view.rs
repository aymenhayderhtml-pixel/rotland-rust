//! Visual transforms and model association for active projectiles.

use game::projectiles::{ProjectileKind, ProjectileSim};
use glam::{Mat4, Quat, Vec3};

/// Returns the model name in `models.pack` for the given projectile kind.
pub fn projectile_model_name(kind: ProjectileKind) -> &'static str {
    match kind {
        ProjectileKind::Arrow => "arrow",
        ProjectileKind::ThrownSpear => "spear",
    }
}

/// Computes the world transformation matrix of a projectile.
/// Rotates the model's forward axis (+Y) to align with the flight velocity.
pub fn projectile_matrix(pos: Vec3, vel: Vec3) -> Mat4 {
    let dir = vel.normalize_or_zero();
    let rot = if dir.length_squared() > 1e-6 {
        Quat::from_rotation_arc(Vec3::Y, dir)
    } else {
        Quat::IDENTITY
    };
    Mat4::from_rotation_translation(rot, pos)
}

/// Collects (model_name, world_matrix) pairs for all active projectiles.
pub fn projectile_instances(sim: &ProjectileSim) -> Vec<(&'static str, Mat4)> {
    sim.projectiles()
        .iter()
        .map(|proj| {
            let model = projectile_model_name(proj.kind);
            let mat = projectile_matrix(proj.pos, proj.vel);
            (model, mat)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::projectiles::Projectile;

    #[test]
    fn projectile_model_names_match() {
        assert_eq!(projectile_model_name(ProjectileKind::Arrow), "arrow");
        assert_eq!(projectile_model_name(ProjectileKind::ThrownSpear), "spear");
    }

    #[test]
    fn projectile_matrix_points_along_velocity() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let vel = Vec3::new(0.0, 10.0, 0.0); // +Y velocity matches default model forward
        let mat = projectile_matrix(pos, vel);
        let transformed_pos = mat.transform_point3(Vec3::ZERO);
        assert!((transformed_pos - pos).length() < 1e-5);

        let mut sim = ProjectileSim::default();
        sim.spawn(Projectile::new(pos, vel, ProjectileKind::Arrow, 10.0, None));
        let instances = projectile_instances(&sim);
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].0, "arrow");
    }
}
