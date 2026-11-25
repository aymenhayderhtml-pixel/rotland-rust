use glam::{IVec2, Vec2, Vec3};

use crate::generator::WorldGenerator;
use crate::poi::{PoiGeometry, PoiInstance, PoiPlacer};
use crate::roads::RoadKind;
use crate::scatter::{PropInstance, ScatterPlacer};
use crate::settlement::SettlementGround;
use crate::towns::BOARDWALK_MODEL;

// data: rivers.json bridge_spans_m
const BRIDGE_WOOD_SPAN_M: f32 = 12.0;
// data: rivers.json bridge_spans_m
const BRIDGE_CONCRETE_SPAN_M: f32 = 16.0;
// data: roads.json highway_width
const HIGHWAY_HALF_WIDTH_M: f32 = 4.0;
// data: roads.json trail_width
const TRAIL_HALF_WIDTH_M: f32 = 2.0;
// new: DECISIONS 2026-11-25
const ROAD_PROP_CLEAR_M: f32 = 4.0;

#[inline]
fn in_chunk(p: Vec2, coord: IVec2) -> bool {
    let cx = (p.x / WorldGenerator::CHUNK_SIZE).floor() as i32;
    let cy = (p.y / WorldGenerator::CHUNK_SIZE).floor() as i32;
    cx == coord.x && cy == coord.y
}

impl WorldGenerator {
    /// POIs and props for one chunk, following BuildChunk rules.
    pub fn chunk_contents(&self, coord: IVec2) -> (Vec<PoiInstance>, Vec<PropInstance>) {
        let placed_pois = PoiPlacer::place(self, &self.poi_config, coord);
        let mut pois = SettlementGround::remove_pois(
            &self.settlements,
            placed_pois,
            &self.poi_config,
            self.config.clear_radius,
        );
        let placed_props =
            ScatterPlacer::place_with_registry(self, &self.scatter_config, &self.biome_registry, coord);

        if let Some(city) = self.city() {
            pois.retain(|poi| !city.contains(Vec2::new(poi.position.x, poi.position.z)));
            for (idx, lot) in city.lots.iter().enumerate() {
                if in_chunk(lot.center, coord) {
                    let y = self.height(lot.center.x, lot.center.y);
                    pois.push(PoiInstance::new(
                        format!("city_{idx}"),
                        &lot.building_id,
                        Vec3::new(lot.center.x, y, lot.center.y),
                        lot.yaw,
                    ));
                }
            }
        }

        if let Some(lake) = self.lake() {
            pois.retain(|poi| {
                let p = Vec2::new(poi.position.x, poi.position.z);
                !lake.contains_region(p) || lake.shore_distance(p) > lake.config.beach_width + 10.0
            });
            for (idx, lot) in lake.village_lots.iter().enumerate() {
                if in_chunk(lot.center, coord) {
                    let y = self.height(lot.center.x, lot.center.y);
                    pois.push(PoiInstance::new(
                        format!("lake_village_{idx}"),
                        &lot.building_id,
                        Vec3::new(lot.center.x, y, lot.center.y),
                        lot.yaw,
                    ));
                }
            }
        }

        let mut town_props = Vec::new();
        if self.has_outer {
            pois.retain(|poi| self.town_at(poi.position.x, poi.position.z).is_none());
            self.append_town_contents(coord, &mut pois, &mut town_props);
        }

        let filtered_props: Vec<PropInstance> = placed_props
            .into_iter()
            .filter(|prop| self.keep_scattered_prop(prop))
            .collect();

        let cleared_props = PoiGeometry::remove_cleared(&pois, filtered_props, &self.poi_config);
        let mut props = SettlementGround::remove_props(&self.settlements, cleared_props, self.config.clear_radius);

        if let Some(city) = self.city() {
            let city_props = crate::city::place::place_street_props(self, city, coord);
            for cp in city_props {
                let clear = !props.iter().any(|p| {
                    let dx = p.position.x - cp.position.x;
                    let dz = p.position.z - cp.position.z;
                    dx * dx + dz * dz < 3.0 * 3.0 - 1e-4
                });
                if clear {
                    props.push(cp);
                }
            }
        }

        if let Some(lake) = self.lake() {
            let lake_props = crate::lake::place::place_lake_props(self, lake, coord);
            for lp in lake_props {
                let clear = !props.iter().any(|p| {
                    let dx = p.position.x - lp.position.x;
                    let dz = p.position.z - lp.position.z;
                    dx * dx + dz * dz < 2.0 * 2.0 - 1e-4
                });
                if clear {
                    props.push(lp);
                }
            }
        }

        if self.has_outer {
            props.extend(town_props);
            self.append_bridge_props(coord, &mut props);
            self.append_signpost_props(coord, &mut props);
        }

        (pois, props)
    }

    fn keep_scattered_prop(&self, prop: &PropInstance) -> bool {
        let p = Vec2::new(prop.position.x, prop.position.z);
        if let Some(city) = self.city()
            && city.contains(p)
        {
            let is_park = city.district_at(p) == Some(crate::city::District::Park);
            let on_street = city.street_at(p).is_some();
            return is_park
                && !on_street
                && (prop.is_tree() || crate::city::place::is_ground_cover(&prop.model));
        }
        if let Some(lake) = self.lake()
            && lake.contains_region(p)
            && lake.shore_distance(p) < 0.0
        {
            return false;
        }
        if !self.has_outer || self.in_core_radius(p.x, p.y) {
            return true;
        }
        if let Some((dist, kind)) = self.roads.distance_to_road(p) {
            let half_w = match kind {
                RoadKind::Highway => HIGHWAY_HALF_WIDTH_M,
                RoadKind::Trail => TRAIL_HALF_WIDTH_M,
            };
            if dist < half_w + ROAD_PROP_CLEAR_M {
                return false;
            }
        }
        if self.rivers.water_at(p.x, p.y).is_some() {
            return false;
        }
        if let Some(town) = self.town_at(p.x, p.y) {
            if town.lots.iter().any(|lot| lot.contains_point(p)) {
                return false;
            }
            // Keep the town's main street (painted through the centre) clear too.
            let right = Vec2::new(-town.street_dir.y, town.street_dir.x);
            if (p - town.center).dot(right).abs() < HIGHWAY_HALF_WIDTH_M + ROAD_PROP_CLEAR_M {
                return false;
            }
        }
        true
    }

    fn append_town_contents(
        &self,
        coord: IVec2,
        pois: &mut Vec<PoiInstance>,
        props: &mut Vec<PropInstance>,
    ) {
        for town in &self.towns {
            for (idx, lot) in town.lots.iter().enumerate() {
                if !in_chunk(lot.pos, coord) {
                    continue;
                }
                let y = self.height(lot.pos.x, lot.pos.y);
                if lot.building == BOARDWALK_MODEL {
                    props.push(PropInstance::new(
                        &lot.building,
                        Vec3::new(lot.pos.x, y, lot.pos.y),
                        lot.yaw,
                        1.0,
                    ));
                } else {
                    pois.push(
                        PoiInstance::new(
                            format!("town_{}_{idx}", town.key),
                            &lot.building,
                            Vec3::new(lot.pos.x, y, lot.pos.y),
                            lot.yaw,
                        )
                        .with_loot_table(&lot.loot_table),
                    );
                    props.extend(
                        crate::towns::interior::place(lot, &self.towns_data, self.seed)
                            .into_iter()
                            .map(|mut p| {
                                p.position.y += y;
                                p
                            }),
                    );
                }
            }
            props.extend(crate::towns::clutter::place_in_chunk(
                town,
                coord,
                self.seed,
                |x, z| self.height(x, z),
                |p| self.roads.distance_to_road(p),
            ));
        }
    }

    fn append_bridge_props(&self, coord: IVec2, props: &mut Vec<PropInstance>) {
        for &(pos, road_yaw, span) in &self.crossings {
            if !in_chunk(pos, coord) {
                continue;
            }
            let is_highway = self
                .roads
                .distance_to_road(pos)
                .is_some_and(|(_, kind)| kind == RoadKind::Highway);
            let (model, base_span) = if is_highway {
                ("bridge_concrete", BRIDGE_CONCRETE_SPAN_M)
            } else {
                ("bridge_wood", BRIDGE_WOOD_SPAN_M)
            };
            let y = self
                .road_heights
                .road_height_at(pos.x, pos.y)
                .unwrap_or_else(|| self.raw_height(pos.x, pos.y));
            props.push(PropInstance::new(
                model,
                Vec3::new(pos.x, y, pos.y),
                road_yaw,
                span / base_span,
            ));
        }
    }

    fn append_signpost_props(&self, coord: IVec2, props: &mut Vec<PropInstance>) {
        for &th in self.roads.trailheads() {
            if !in_chunk(th, coord) {
                continue;
            }
            let dir = (th - self.origin).normalize_or_zero();
            let yaw = (-dir.x).atan2(-dir.y);
            let y = self.height(th.x, th.y);
            props.push(PropInstance::new("signpost", Vec3::new(th.x, y, th.y), yaw, 1.0));
        }
    }

    /// Loot crate spots for one chunk, following BuildChunk rules.
    pub fn chunk_crates(&self, coord: IVec2) -> Vec<crate::loot::LootSpot> {
        let (pois, props) = self.chunk_contents(coord);
        self.chunk_crates_from(coord, &pois, &props)
    }

    /// Crate spots for one chunk, reusing already-built POIs and props (R31-03).
    pub fn chunk_crates_from(
        &self,
        coord: IVec2,
        pois: &[PoiInstance],
        props: &[PropInstance],
    ) -> Vec<crate::loot::LootSpot> {
        let placed = crate::loot::LootPlacer::place(
            self,
            &self.scatter_config,
            &self.poi_config,
            coord,
            props,
            pois,
        );
        SettlementGround::remove_crates(&self.settlements, placed, self.config.clear_radius)
    }
}
