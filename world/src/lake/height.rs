use glam::Vec2;

use crate::color::Color;
use crate::lake::layout::LakeLayout;
use crate::math::{lerp, smooth_step};

impl LakeLayout {
    // moved: world/src/lake/layout.rs
    const COLOR_SCALE: f32 = 1.0 / 255.0;

    pub fn sand_color(&self) -> Color {
        Color::new(
            self.config.sand[0] as f32 * Self::COLOR_SCALE,
            self.config.sand[1] as f32 * Self::COLOR_SCALE,
            self.config.sand[2] as f32 * Self::COLOR_SCALE,
            1.0,
        )
    }

    pub fn wet_sand_color(&self) -> Color {
        Color::new(
            self.config.wet_sand[0] as f32 * Self::COLOR_SCALE,
            self.config.wet_sand[1] as f32 * Self::COLOR_SCALE,
            self.config.wet_sand[2] as f32 * Self::COLOR_SCALE,
            1.0,
        )
    }

    pub fn lake_bed_color(&self) -> Color {
        Color::new(
            self.config.lake_bed[0] as f32 * Self::COLOR_SCALE,
            self.config.lake_bed[1] as f32 * Self::COLOR_SCALE,
            self.config.lake_bed[2] as f32 * Self::COLOR_SCALE,
            1.0,
        )
    }

    /// Carves the lake bed and flat beach into the terrain, returning the adjusted height.
    pub fn ground_height(&self, p: Vec2, raw: f32) -> f32 {
        if !self.contains_region(p) {
            return raw;
        }
        let d = self.shore_distance(p);
        let blend_end = self.config.beach_width + self.config.beach_blend;
        if d >= blend_end {
            return raw;
        }
        if d < 0.0 {
            self.water_level - self.depth_at(p)
        } else if d <= self.config.beach_width {
            let beach_fraction = if self.config.beach_width > 1e-5 {
                d / self.config.beach_width
            } else {
                0.0
            };
            self.water_level + beach_fraction * self.config.water_level_above_shore
        } else {
            let beach_edge_height = self.water_level + self.config.water_level_above_shore;
            let t = smooth_step(self.config.beach_width, blend_end, d);
            lerp(beach_edge_height, raw, t)
        }
    }

    /// Paints the terrain inside the lake region (lake bed, wet sand, dry sand beach).
    pub fn paint(&self, base_color: Color, wx: f32, wz: f32) -> Color {
        let p = Vec2::new(wx, wz);
        if !self.contains_region(p) {
            return base_color;
        }
        let d = self.shore_distance(p);
        let blend_end = self.config.beach_width + self.config.beach_blend;
        if d >= blend_end {
            return base_color;
        }

        if d < 0.0 {
            let t = if self.config.shallow_width > 1e-4 {
                (1.0 + d / self.config.shallow_width).clamp(0.0, 1.0)
            } else {
                0.0
            };
            self.lake_bed_color().lerp(self.wet_sand_color(), t)
        } else if d <= self.config.beach_width {
            // moved: world/src/lake/layout.rs
            let wet_width = (self.config.beach_width * 0.25).max(2.0);
            if d < wet_width {
                let t = d / wet_width;
                self.wet_sand_color().lerp(self.sand_color(), t)
            } else {
                self.sand_color()
            }
        } else {
            let t = smooth_step(self.config.beach_width, blend_end, d);
            self.sand_color().lerp(base_color, t)
        }
    }
}
