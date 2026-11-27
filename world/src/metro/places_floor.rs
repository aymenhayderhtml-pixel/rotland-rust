//! R42 review fix: is a point on the floor of a place's main building? Used to keep grass
//! tufts off the walk-in ground floors (they poked through the hospital/police/fire floors).

use glam::Vec2;

use crate::metro::layout::MetroLayout;
use crate::metro::places::place_sites_cached;
use crate::metro::places_data::place_data;

/// True when `p` (world XZ) is inside the footprint of a hospital, police or fire station building.
pub fn in_place_building(layout: &MetroLayout, p: Vec2) -> bool {
    place_sites_cached(layout).iter().any(|site| {
        let Some(data) = place_data(site.kind) else { return false };
        let d = p - site.pos;
        let (s, c) = site.yaw.sin_cos();
        // Inverse of the lot frame (local x/z -> world) used to place the building.
        let lx = d.x * c - d.y * s;
        let lz = d.x * s + d.y * c;
        lx.abs() <= data.footprint[0] * 0.5 && lz.abs() <= data.footprint[1] * 0.5
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metro::places::place_sites;

    #[test]
    fn building_centre_is_inside_and_far_point_is_not() {
        use crate::metro::config::MetroConfig;
        use crate::metro::place::MetroSite;
        let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json")).unwrap();
        let site = MetroSite {
            key: cfg.key.clone(),
            name: cfg.name.clone(),
            center: Vec2::new(6000.0, 0.0),
            yaw: 0.3,
            half_size: (cfg.size_blocks as f32 * cfg.block_size) * 0.5,
            ground_y: 0.0,
        };
        let layout = MetroLayout::generate(site, &cfg);
        let sites = place_sites(&layout);
        assert!(!sites.is_empty());
        for site in &sites {
            assert!(in_place_building(&layout, site.pos));
            assert!(!in_place_building(&layout, site.pos + site.front() * 60.0));
        }
    }
}
