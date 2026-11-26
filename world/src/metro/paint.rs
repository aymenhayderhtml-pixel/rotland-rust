//! New Halden paint: street, sidewalk and lot colours on the ground, and the placeholder look
//! (`look` in `data/metro.json`) used by the tower meshes. Pure: no GPU.

use std::path::Path;

use glam::Vec2;
use serde::Deserialize;

use crate::color::Color;
use crate::metro::layout::{MetroDistrict, MetroLayout, Reserved};

// new: DECISIONS 2026-11-26 (byte colour to 0..1 float, same as city/ground.rs)
const COLOR_SCALE: f32 = 1.0 / 255.0;

/// Placeholder colours and window grid numbers (`look` in data/metro.json). Colours are sRGB 0-255.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MetroLook {
    pub asphalt: [u8; 3],
    pub sidewalk: [u8; 3],
    pub lot_concrete: [u8; 3],
    pub lot_dirt: [u8; 3],
    /// Width of the sidewalk band along each street edge, metres.
    pub sidewalk_width: f32,
    pub base_color: [u8; 3],
    /// Body colours a tower picks from: grey concrete, brownstone, dirty glass blue-grey.
    pub wall_palette: Vec<[u8; 3]>,
    pub glass_color: [u8; 3],
    pub glass_dark: [u8; 3],
    pub broken_color: [u8; 3],
    pub roof_color: [u8; 3],
    /// One flat colour for the far skyline.
    pub far_color: [u8; 3],
    /// Height of the darker ground band on a tower, metres.
    pub base_height: f32,
    /// Nominal window bay width, metres.
    pub bay_width: f32,
    /// Most window bays on one face of one floor (keeps near towers inside their triangle budget).
    pub max_bays_per_face: u32,
    /// Pane width as a share of its bay.
    pub pane_width_share: f32,
    /// Pane bottom as a share of its floor height.
    pub pane_bottom_share: f32,
    /// Pane height as a share of its floor height.
    pub pane_height_share: f32,
    /// How far a pane stands off the wall, metres (stops z-fighting).
    pub pane_offset: f32,
    /// One bay in this many is a broken dark patch.
    pub broken_one_in: u32,
    /// One pane in this many is the darker glass.
    pub dark_one_in: u32,
    /// Rooftop box height, metres.
    pub roof_box_height: f32,
    /// Rooftop box half size as a share of the tower half size.
    pub roof_box_share: f32,
    /// Roof parapet height, metres.
    pub parapet_height: f32,
    /// Landmark stepped top height, metres.
    pub step_height: f32,
    /// Landmark stepped top half size as a share of the tower half size.
    pub step_share: f32,
    /// Far skyline is rebuilt when the player crosses a cell this big (metres).
    pub rebuild_cell_m: f32,
}

impl MetroLook {
    /// Reads the `look` key of a `metro.json` string. A missing key is an error.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let root: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("metro.json: {e}"))?;
        let look = root.get("look").ok_or("metro.json: missing \"look\" key")?;
        let look = Self::deserialize(look).map_err(|e| format!("metro.json look: {e}"))?;
        look.validate()?;
        Ok(look)
    }

    /// Reads the `look` key from a `metro.json` file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    fn validate(&self) -> Result<(), String> {
        let shares = [self.pane_width_share, self.pane_bottom_share, self.pane_height_share, self.roof_box_share, self.step_share];
        let positive = [self.sidewalk_width, self.bay_width, self.rebuild_cell_m, self.pane_offset];
        if self.wall_palette.is_empty()
            || self.max_bays_per_face == 0
            || self.broken_one_in == 0
            || self.dark_one_in == 0
            || self.base_height < 0.0
            || self.roof_box_height < 0.0
            || self.parapet_height < 0.0
            || self.step_height < 0.0
            || shares.iter().any(|s| !(0.0..=1.0).contains(s))
            || positive.iter().any(|v| *v <= 0.0)
        {
            return Err("metro.json look: a value is missing, zero or out of range".into());
        }
        Ok(())
    }
}

/// sRGB bytes to a float colour (alpha 1).
pub fn rgb(c: [u8; 3]) -> Color {
    Color::new(f32::from(c[0]) * COLOR_SCALE, f32::from(c[1]) * COLOR_SCALE, f32::from(c[2]) * COLOR_SCALE, 1.0)
}

/// Ground colour of New Halden at world point `(wx, wz)`.
///
/// Outside the skirt it is `base` untouched (grass stays green). On a street it is asphalt, in
/// the sidewalk band along a street edge it is the sidewalk colour, on a lot it is concrete (Core
/// and Midtown) or dirt (Edge). Reserved Park blocks keep `base`. Inside the skirt the city colour
/// blends into `base` over the skirt width.
pub fn metro_ground_color(base: Color, wx: f32, wz: f32, layout: &MetroLayout, look: &MetroLook) -> Color {
    let p = Vec2::new(wx, wz);
    if !layout.near(p) {
        return base;
    }
    match city_surface(p, layout, look) {
        Some(city) if layout.site.signed_distance(p) <= 0.0 => city,
        Some(city) => {
            // Same blend weight as the terrain height (`MetroLayout::flatten`): 0 at the edge, 1 at the skirt end.
            let ground = layout.site.ground_y;
            let t = layout.flatten(ground + 1.0, p) - ground;
            city.lerp(base, t)
        }
        None => base,
    }
}

/// The city's own surface at `p` (street, sidewalk or lot), or None for grass and gaps.
fn city_surface(p: Vec2, layout: &MetroLayout, look: &MetroLook) -> Option<Color> {
    let local = layout.site.to_local(p);
    let mut sidewalk = false;
    for s in &layout.streets {
        if !s.contains_point(local) {
            continue;
        }
        let dist = s.closest_point(local).distance(local);
        if dist <= s.width * 0.5 - look.sidewalk_width {
            return Some(rgb(look.asphalt));
        }
        sidewalk = true;
    }
    if sidewalk {
        return Some(rgb(look.sidewalk));
    }
    let block = layout.block_at_point(p)?;
    if block.reserved == Some(Reserved::Park) {
        return None;
    }
    Some(match block.district {
        MetroDistrict::Edge => rgb(look.lot_dirt),
        MetroDistrict::Midtown | MetroDistrict::Core => rgb(look.lot_concrete),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metro::config::MetroConfig;
    use crate::metro::place::MetroSite;

    fn load_cfg() -> MetroConfig {
        MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
    }

    fn load_look() -> MetroLook {
        MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json look loads")
    }

    /// A square city at the origin with no yaw, so local and world points match.
    fn layout(cfg: &MetroConfig) -> MetroLayout {
        let site = MetroSite {
            key: cfg.key.clone(),
            name: cfg.name.clone(),
            center: Vec2::ZERO,
            half_size: cfg.size_blocks as f32 * cfg.block_size * 0.5,
            yaw: 0.0,
            ground_y: 20.0,
        };
        MetroLayout::generate(site, cfg)
    }

    fn base() -> Color {
        Color::new(0.2, 0.5, 0.2, 1.0)
    }

    #[test]
    fn outside_the_skirt_is_the_base_colour_exactly() {
        let cfg = load_cfg();
        let look = load_look();
        let l = layout(&cfg);
        let far = l.site.half_size + cfg.skirt_width + 5.0;
        assert_eq!(metro_ground_color(base(), far, 0.0, &l, &look), base());
    }

    #[test]
    fn a_street_centreline_is_asphalt() {
        let cfg = load_cfg();
        let look = load_look();
        let l = layout(&cfg);
        let s = l.streets.iter().find(|s| s.avenue).expect("an avenue exists");
        let mid = (s.a + s.b) * 0.5;
        assert_eq!(metro_ground_color(base(), mid.x, mid.y, &l, &look), rgb(look.asphalt));
    }

    #[test]
    fn the_sidewalk_band_is_sidewalk() {
        let cfg = load_cfg();
        let look = load_look();
        let l = layout(&cfg);
        let s = l.streets.iter().find(|s| !s.avenue).expect("a street exists");
        let dir = (s.b - s.a).normalize();
        let side = Vec2::new(-dir.y, dir.x);
        let mid = (s.a + s.b) * 0.5 + side * (s.width * 0.5 - look.sidewalk_width * 0.5);
        assert_eq!(metro_ground_color(base(), mid.x, mid.y, &l, &look), rgb(look.sidewalk));
    }

    #[test]
    fn lots_are_concrete_in_the_core_and_dirt_at_the_edge() {
        let cfg = load_cfg();
        let look = load_look();
        let l = layout(&cfg);
        let centre_of = |b: &crate::metro::layout::MetroBlock| (b.rect.0 + b.rect.1) * 0.5;
        let core = l
            .blocks
            .iter()
            .find(|b| b.district == MetroDistrict::Core && b.reserved.is_none())
            .expect("a core lot");
        let c = centre_of(core);
        assert_eq!(metro_ground_color(base(), c.x, c.y, &l, &look), rgb(look.lot_concrete));
        let edge = l
            .blocks
            .iter()
            .find(|b| b.district == MetroDistrict::Edge && b.reserved.is_none())
            .expect("an edge lot");
        let e = centre_of(edge);
        assert_eq!(metro_ground_color(base(), e.x, e.y, &l, &look), rgb(look.lot_dirt));
    }

    #[test]
    fn a_park_block_keeps_the_grass() {
        let cfg = load_cfg();
        let look = load_look();
        let l = layout(&cfg);
        let park = l
            .blocks
            .iter()
            .find(|b| b.reserved == Some(Reserved::Park))
            .expect("a park block");
        let c = (park.rect.0 + park.rect.1) * 0.5;
        assert_eq!(metro_ground_color(base(), c.x, c.y, &l, &look), base());
    }

    #[test]
    fn the_skirt_ends_on_the_base_colour() {
        let cfg = load_cfg();
        let look = load_look();
        let l = layout(&cfg);
        // Just past the square edge, on a point with no street or lot: blends toward the grass.
        let past = l.site.half_size + cfg.skirt_width * 0.99;
        let c = metro_ground_color(base(), past, 0.0, &l, &look);
        let b = base();
        assert!((c.r - b.r).abs() < 0.01 && (c.g - b.g).abs() < 0.01 && (c.b - b.b).abs() < 0.01);
    }

    #[test]
    fn look_parses_and_rejects_a_missing_key() {
        let json = std::fs::read_to_string(paths::repo_path("data/metro.json")).unwrap();
        assert!(MetroLook::from_json(&json).is_ok());
        let without = json.replace("\"look\"", "\"lookx\"");
        assert!(MetroLook::from_json(&without).is_err());
    }

    #[test]
    fn look_rejects_an_empty_wall_palette() {
        let mut look = load_look();
        look.wall_palette.clear();
        assert!(look.validate().is_err());
    }
}
