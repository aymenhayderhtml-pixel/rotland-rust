//! New Halden on the big map: the square's corners, discovery and which streets to show (pure).

use glam::Vec2;
use world::metro::{MetroLayout, MetroSite, MetroStreet};

/// Sample points per side of the square used to test discovery.
const DISCOVERY_GRID: usize = 5; // new: DECISIONS 2026-11-26 (5 x 5 discovery sample grid)

/// The four world-space corners of the city square.
pub fn site_corners(site: &MetroSite) -> [Vec2; 4] {
    site.corners()
}

/// True once any sample point over the square is revealed on the explore map.
pub fn discovered(site: &MetroSite, explore: &game::MapExplore) -> bool {
    let h = site.half_size;
    let last = (DISCOVERY_GRID - 1) as f32;
    (0..DISCOVERY_GRID).any(|j| {
        (0..DISCOVERY_GRID).any(|i| {
            let fx = i as f32 / last * 2.0 - 1.0;
            let fz = j as f32 / last * 2.0 - 1.0;
            let world = site.to_world(Vec2::new(fx * h, fz * h));
            explore.is_revealed(game::MapExplore::cell_of(world))
        })
    })
}

/// The streets whose midpoint cell is revealed (so only explored streets are drawn).
pub fn streets_on_map<'a>(
    layout: &'a MetroLayout,
    explore: &game::MapExplore,
) -> Vec<&'a MetroStreet> {
    layout
        .streets
        .iter()
        .filter(|s| {
            let mid = layout.site.to_world((s.a + s.b) * 0.5);
            explore.is_revealed(game::MapExplore::cell_of(mid))
        })
        .collect()
}
