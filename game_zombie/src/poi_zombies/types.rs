use glam::Vec3;

/// Lightweight description of a POI building site for zombie wake and spawn checks.
// Godot: PoiSite.cs
#[derive(Clone, Debug, PartialEq)]
pub struct PoiZombieSite {
    pub poi_id: String,
    pub building_id: String,
    pub position: Vec3,
    pub yaw: f32,
    pub spawn_points: Vec<Vec3>,
}

impl PoiZombieSite {
    pub fn new(
        poi_id: impl Into<String>,
        building_id: impl Into<String>,
        position: Vec3,
        yaw: f32,
        spawn_points: Vec<Vec3>,
    ) -> Self {
        Self {
            poi_id: poi_id.into(),
            building_id: building_id.into(),
            position,
            yaw,
            spawn_points,
        }
    }

    /// Computes the world position of the spawn marker at index `i` (modulo `spawn_points.len()`).
    pub fn world_spawn_point(&self, index: usize) -> Option<Vec3> {
        if self.spawn_points.is_empty() {
            return None;
        }
        let marker = self.spawn_points[index % self.spawn_points.len()];
        let rot = glam::Quat::from_rotation_y(self.yaw);
        Some(self.position + rot * marker)
    }
}

/// A request to spawn one zombie inside or around a building.
// Godot: PoiManager.cs Wake
#[derive(Clone, Debug, PartialEq)]
pub struct PoiSpawn {
    pub poi_id: String,
    pub position: Vec3,
    pub kind: String,
}

impl PoiSpawn {
    pub fn new(poi_id: impl Into<String>, position: Vec3, kind: impl Into<String>) -> Self {
        Self {
            poi_id: poi_id.into(),
            position,
            kind: kind.into(),
        }
    }
}
