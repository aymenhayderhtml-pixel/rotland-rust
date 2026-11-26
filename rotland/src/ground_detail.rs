//! Short-range ground detail ring around the player (R35-01).

use std::collections::HashMap;

use glam::{IVec2, Vec2, Vec3};
use world::ground_detail::{GroundDetailConfig, GroundDetailConfigData};
use world::{District, RoadKind, SettlementGround, WorldGenerator};

use crate::props::{Placed, PropLibrary};

// data: roads.json highway_width
const HIGHWAY_HALF_WIDTH_M: f32 = 4.0;
// data: roads.json trail_width
const TRAIL_HALF_WIDTH_M: f32 = 2.0;
// new: DECISIONS 2026-11-25
const ROAD_CLEAR_M: f32 = 2.0;
// new: DECISIONS 2026-11-25
const TOWN_STREET_CLEAR_M: f32 = 6.0;
// new: DECISIONS 2026-11-25
const MIN_FADE_SCALE: f32 = 0.05;

struct DetailItem {
    slot: usize,
    position: Vec3,
    yaw: f32,
    scale: f32,
    radius_m: f32,
    placed: Placed,
}

#[derive(Default)]
pub struct GroundDetailLayer {
    loaded: bool,
    config: Option<GroundDetailConfigData>,
    cells: HashMap<IVec2, Vec<DetailItem>>,
    active_placed: Vec<Placed>,
}

impl GroundDetailLayer {
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        library: &mut PropLibrary,
        world_gen: &WorldGenerator,
        player_pos: Vec3,
    ) {
        if !self.loaded {
            self.loaded = true;
            match GroundDetailConfig::load_from_file(paths::repo_path("data/ground_detail.json")) {
                Ok(cfg) => self.config = Some(cfg),
                Err(e) => log::warn!("ground_detail: failed to load config: {e}"),
            }
        }
        let Some(cfg) = self.config.as_ref() else {
            return;
        };

        let cell_m = cfg.cell_m.max(1.0); // new: DECISIONS 2026-11-25
        let radius_m = cfg.radius_m.max(cell_m);
        let center_cell = IVec2::new(
            (player_pos.x / cell_m).floor() as i32,
            (player_pos.z / cell_m).floor() as i32,
        );
        let cell_range = (radius_m / cell_m).ceil() as i32 + 1;
        let max_cell_dist_sq = (radius_m + cell_m * std::f32::consts::SQRT_2).powi(2);

        self.cells.retain(|&cell, _| {
            let cx = (cell.x as f32 + 0.5) * cell_m;
            let cz = (cell.y as f32 + 0.5) * cell_m;
            let dx = cx - player_pos.x;
            let dz = cz - player_pos.z;
            dx * dx + dz * dz <= max_cell_dist_sq
        });

        let biome_at = |p: Vec2| world_gen.biome(p.x, p.y);
        let height_at = |p: Vec2| world_gen.height(p.x, p.y);
        let blocked = |p: Vec2| is_blocked(world_gen, p);

        for dz in -cell_range..=cell_range {
            for dx in -cell_range..=cell_range {
                let cell = IVec2::new(center_cell.x + dx, center_cell.y + dz);
                if self.cells.contains_key(&cell) {
                    continue;
                }
                if !cell_in_radius(cell, cell_m, player_pos, radius_m) {
                    continue;
                }
                let raw = world::ground_detail::place(
                    cell,
                    cfg,
                    &biome_at,
                    &height_at,
                    &blocked,
                    world_gen.seed(),
                );
                let mut items = Vec::with_capacity(raw.len());
                for inst in raw {
                    let Some(slot) = library.get(device, pack, &inst.model, false) else {
                        continue;
                    };
                    let placed = library.place(slot, inst.position, inst.yaw, inst.scale);
                    items.push(DetailItem {
                        slot,
                        position: inst.position,
                        yaw: inst.yaw,
                        scale: inst.scale,
                        radius_m: inst.radius_m,
                        placed,
                    });
                }
                self.cells.insert(cell, items);
            }
        }

        self.active_placed.clear();
        let fade_m = cfg.fade_m;
        let fade_sq = fade_m * fade_m;
        'outer: for items in self.cells.values() {
            for item in items {
                if self.active_placed.len() >= cfg.max_instances {
                    break 'outer;
                }
                let eff_radius = item.radius_m.min(radius_m);
                let dx = item.position.x - player_pos.x;
                let dz = item.position.z - player_pos.z;
                let dist_sq = dx * dx + dz * dz;
                if dist_sq >= eff_radius * eff_radius {
                    continue;
                }
                if dist_sq > fade_sq && eff_radius > fade_m {
                    let dist = dist_sq.sqrt();
                    let t = ((eff_radius - dist) / (eff_radius - fade_m)).clamp(0.0, 1.0);
                    if t <= MIN_FADE_SCALE {
                        continue;
                    }
                    self.active_placed.push(library.place(
                        item.slot,
                        item.position,
                        item.yaw,
                        item.scale * t,
                    ));
                } else {
                    self.active_placed.push(item.placed);
                }
            }
        }
    }

    pub fn placed(&self) -> &[Placed] {
        &self.active_placed
    }
}

fn cell_in_radius(cell: IVec2, cell_m: f32, player_pos: Vec3, radius_m: f32) -> bool {
    let min_x = cell.x as f32 * cell_m;
    let max_x = min_x + cell_m;
    let min_z = cell.y as f32 * cell_m;
    let max_z = min_z + cell_m;
    let cx = player_pos.x.clamp(min_x, max_x);
    let cz = player_pos.z.clamp(min_z, max_z);
    let dx = cx - player_pos.x;
    let dz = cz - player_pos.z;
    dx * dx + dz * dz <= radius_m * radius_m
}

fn is_blocked(world_gen: &WorldGenerator, p: Vec2) -> bool {
    if SettlementGround::in_clear_zone(
        world_gen.settlements(),
        p.x,
        p.y,
        0.0,
        world_gen.config().clear_radius,
    ) {
        return true;
    }
    if let Some(city) = world_gen.city()
        && city.contains(p)
    {
        let is_park = city.district_at(p) == Some(District::Park);
        let on_street = city.street_at(p).is_some();
        if !is_park || on_street {
            return true;
        }
    }
    // New Halden: no tufts on its streets or on the highway into it (lots keep them: overgrowth)
    if let Some(metro) = world_gen.metro_layout()
        && metro.contains(p)
        && metro.street_at(p).is_some()
    {
        return true;
    }
    if let Some(highway) = world_gen.metro_highway_field()
        && highway.height_at(p).is_some()
    {
        return true;
    }
    if let Some(lake) = world_gen.lake()
        && lake.contains_region(p)
        && lake.shore_distance(p) < 0.0
    {
        return true;
    }
    if let Some((dist, kind)) = world_gen.roads().distance_to_road(p) {
        let half_w = match kind {
            RoadKind::Highway => HIGHWAY_HALF_WIDTH_M,
            RoadKind::Trail => TRAIL_HALF_WIDTH_M,
        };
        if dist < half_w + ROAD_CLEAR_M {
            return true;
        }
    }
    if world_gen.rivers().water_at(p.x, p.y).is_some() {
        return true;
    }
    if let Some(town) = world_gen.town_at(p.x, p.y) {
        if town.lots.iter().any(|lot| lot.contains_point(p)) {
            return true;
        }
        let right = Vec2::new(-town.street_dir.y, town.street_dir.x);
        if (p - town.center).dot(right).abs() < TOWN_STREET_CLEAR_M {
            return true;
        }
    }
    false
}
