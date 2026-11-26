//! Attaches New Halden to the outer world and exposes its data to the game.

use glam::Vec2;

use crate::generator::WorldGenerator;
use crate::metro::{
    Highway, HighwayHeightField, JamProp, MetroConfig, MetroLayout, MetroLook, MetroSite,
    MetroWorld, TowerSpot, place_metro, plan_highway, plan_jam, plan_towers,
};
use crate::roads::RoadsData;

impl WorldGenerator {
    /// Builds New Halden once: site, layout, towers, highway, jam. Call after `attach_outer_world`.
    /// Sets the city ground height from the raw terrain, and adds the highway's river crossings
    /// to the bridge list. Flattening happens in `apply_outer_height`.
    pub fn attach_metro(&mut self, mut cfg: MetroConfig, look: MetroLook, roads_data: &RoadsData) {
        cfg.reference_dir = self.layout.reference_dir();
        let camp = self.origin;
        let mut site: MetroSite = place_metro(self.seed as u64, camp, &cfg);
        site.ground_y = self.raw_height(site.center.x, site.center.y);
        let layout: MetroLayout = MetroLayout::generate(site, &cfg);
        let towers: Vec<TowerSpot> = plan_towers(&layout, &cfg);
        let highway: Highway = plan_highway(&layout.site, &self.towns, roads_data, &cfg);
        let highway_field = {
            let towns = &self.towns;
            let avg = &self.town_avg_heights;
            HighwayHeightField::build(&highway, layout.site.ground_y, &|x, z| {
                let mut h = self.raw_height(x, z);
                let p = Vec2::new(x, z);
                for (town, &avg_h) in towns.iter().zip(avg) {
                    if p.distance_squared(town.center) <= town.radius * town.radius {
                        h = avg_h;
                    }
                }
                h
            }, &cfg)
        };
        let jam: Vec<JamProp> = plan_jam(&highway, &layout, &cfg);
        let highway_crossings = self
            .rivers
            .crossings_for_polylines(&[highway.points.as_slice()]);
        self.crossings.extend(highway_crossings);
        self.metro = Some(MetroWorld {
            config: cfg,
            layout,
            towers,
            highway,
            highway_field,
            jam,
            look,
        });
    }

    /// The attached New Halden data, None when the world has no metro city.
    pub fn metro_world(&self) -> Option<&MetroWorld> {
        self.metro.as_ref()
    }

    pub fn metro_layout(&self) -> Option<&MetroLayout> {
        self.metro.as_ref().map(|m| &m.layout)
    }

    pub fn metro_highway_field(&self) -> Option<&HighwayHeightField> {
        self.metro.as_ref().map(|m| &m.highway_field)
    }

    /// Name of the metro city when `(x, z)` is inside its square, None otherwise.
    pub fn metro_name_at(&self, x: f32, z: f32) -> Option<&str> {
        let m = self.metro.as_ref()?;
        let p = Vec2::new(x, z);
        m.layout.site.contains_square(p).then_some(m.layout.site.name.as_str())
    }
}
