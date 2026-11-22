use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Zone classification relative to the lake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LakeZone {
    Land,
    Beach,
    Shallow,
    Deep,
}

/// A pier or dock starting on land and extending into the water.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dock {
    pub start: Vec2,
    pub end: Vec2,
    pub yaw: f32,
    pub deck_y: f32,
}

impl Dock {
    pub fn length(&self) -> f32 {
        self.start.distance(self.end)
    }

    pub fn forward(&self) -> Vec2 {
        Vec2::new(-self.yaw.sin(), -self.yaw.cos())
    }
}

/// A building lot in the fishing village along the shore.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VillageLot {
    pub center: Vec2,
    pub size: Vec2,
    pub yaw: f32,
    pub building_id: String,
    pub floor_y: f32,
}

impl VillageLot {
    pub fn corners(&self) -> [Vec2; 4] {
        let forward = Vec2::new(-self.yaw.sin(), -self.yaw.cos());
        let right = Vec2::new(-forward.y, forward.x);
        let hx = right * (self.size.x * 0.5);
        let hy = forward * (self.size.y * 0.5);
        [
            self.center - hx - hy,
            self.center + hx - hy,
            self.center + hx + hy,
            self.center - hx + hy,
        ]
    }

    pub fn aabb(&self) -> (Vec2, Vec2) {
        let corners = self.corners();
        let mut min = corners[0];
        let mut max = corners[0];
        for &c in &corners[1..] {
            min = min.min(c);
            max = max.max(c);
        }
        (min, max)
    }

    pub fn overlaps(&self, other: &VillageLot) -> bool {
        let corners_a = self.corners();
        let corners_b = other.corners();
        let axes = [
            (corners_a[1] - corners_a[0]).normalize(),
            (corners_a[3] - corners_a[0]).normalize(),
            (corners_b[1] - corners_b[0]).normalize(),
            (corners_b[3] - corners_b[0]).normalize(),
        ];
        for axis in axes {
            let (min_a, max_a) = project_onto_axis(&corners_a, axis);
            let (min_b, max_b) = project_onto_axis(&corners_b, axis);
            if max_a < min_b || max_b < min_a {
                return false;
            }
        }
        true
    }
}

fn project_onto_axis(corners: &[Vec2; 4], axis: Vec2) -> (f32, f32) {
    let mut min = corners[0].dot(axis);
    let mut max = min;
    for &c in &corners[1..] {
        let d = c.dot(axis);
        if d < min {
            min = d;
        }
        if d > max {
            max = d;
        }
    }
    (min, max)
}

/// The kind of fishing spot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FishingSpotKind {
    Dock,
    Beach,
}

/// A placeable fishing spot on a dock or beach.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FishingSpot {
    pub position: Vec2,
    pub kind: FishingSpotKind,
}
