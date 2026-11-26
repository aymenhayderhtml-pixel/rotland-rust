//! The New Halden data the world keeps once it is built: layout, towers, highway, jam, look.

use glam::Vec2;

use crate::color::Color;
use crate::metro::paint::rgb;
use crate::metro::{
    Highway, HighwayHeightField, JamProp, MetroConfig, MetroLayout, MetroLook, TowerSpot,
    metro_ground_color,
};

/// Everything the metro city needs after the world is attached (built once per world).
pub struct MetroWorld {
    pub config: MetroConfig,
    pub layout: MetroLayout,
    pub towers: Vec<TowerSpot>,
    pub highway: Highway,
    pub highway_field: HighwayHeightField,
    pub jam: Vec<JamProp>,
    pub look: MetroLook,
}

impl MetroWorld {
    /// True when `p` is inside one of the tower footprints (squares rotated by each tower's yaw).
    pub fn in_tower_footprint(&self, p: Vec2) -> bool {
        self.towers.iter().any(|t| {
            let d = p - t.pos;
            let (s, c) = t.yaw.sin_cos();
            // Rotate the offset into the tower's own frame (the footprint is a square).
            let local = Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c);
            local.x.abs() <= t.half.x && local.y.abs() <= t.half.y
        })
    }
}

impl MetroWorld {
    /// Ground colour after the metro paint: city streets, sidewalks and lots, and the highway
    /// asphalt. Outside both, `color` comes back unchanged.
    pub fn paint_ground(&self, color: Color, wx: f32, wz: f32) -> Color {
        let p = Vec2::new(wx, wz);
        let city = metro_ground_color(color, wx, wz, &self.layout, &self.look);
        let road = self.highway_field.surface_weight(p);
        if road <= 0.0 {
            return city;
        }
        rgb(self.look.asphalt).lerp(city, 1.0 - road)
    }
}
