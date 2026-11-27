//! Chunk upload, collider shape caching and New Halden tower/curb loading for `streaming.rs`.

use glam::{IVec2, Vec3};
use physics::{ColliderHandle, PhysicsWorld};
use world::WorldGenerator;

use crate::metro_render::{
    curb_mesh::curb_chunk_mesh,
    hulls::{aabb_points, tower_bases, tower_hulls_with_climb},
    near::{place_chunk_towers, tower_chunk},
    parts::tower_parts_for_index,
};
use crate::props::{Placed, PropLibrary};

use super::{Built, LoadedChunk, Streamer, TreeSlot};

impl Streamer {
    pub(super) fn shapes_of(&mut self, pack: &pack::ModelPack, id: &str) -> &Vec<Vec<[f32; 3]>> {
        self.shapes.entry(id.to_string()).or_insert_with(|| {
            if is_no_collider_prop(id) {
                return Vec::new();
            }
            let Some(m) = pack.find(id) else {
                return Vec::new();
            };
            let cols: Vec<Vec<[f32; 3]>> = m.colliders.iter().map(|c| c.points.clone()).collect();
            if is_walkable_deck_model(id) && needs_deck_fallback(&cols) {
                return vec![deck_box_points(m.bounds_min, m.bounds_max)];
            }
            cols
        })
    }

    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        library: &mut PropLibrary,
        physics: &mut PhysicsWorld,
        b: Built,
    ) -> LoadedChunk {
        let mut placed = Vec::with_capacity(b.props.len() + b.pois.len() + 1);
        let mut colliders = vec![physics.add_ground_chunk_deferred(b.coord, &b.heights)];
        // (model, position, yaw, scale, is a building)
        let mut models: Vec<(&str, Vec3, f32, f32, bool)> = b
            .props
            .iter()
            .map(|p| (p.model.as_str(), p.position, p.yaw, p.scale, false))
            .collect();
        models.extend(b.pois.iter().map(|p| (p.building_id.as_str(), p.position, p.yaw, 1.0, true)));
        let site = self.world_gen.settlements().first().cloned();
        models.extend(b.crates.iter().filter_map(|c| {
            crate::crates::crate_model(c).map(|m| (m, c.position, c.yaw, 1.0, false))
        }));
        if let (true, Some(site)) = (b.coord == self.camp_chunk, site.as_ref()) {
            models.push(("settlement", site.center, site.yaw, 1.0, true));
        }
        let mut ladders = Vec::new();
        let mut trees = Vec::new();
        let mut static_props = Vec::new();
        for (i, (id, position, yaw, scale, building)) in models.into_iter().enumerate() {
            // Props come first in `models`, so i is the prop index for them.
            // `PropInstance::is_tree` checks `tree*` / `pine_*`; cacti (`cactus_*`) stay non-tree
            // static props and receive compound colliders like rocks.
            let tree = i < b.props.len() && b.props[i].is_tree();
            if tree && self.felled.contains(&(b.coord, i)) {
                continue;
            }
            if building && let Some(m) = pack.find(id) {
                let matrix = glam::Mat4::from_translation(position)
                    * glam::Mat4::from_rotation_y(yaw)
                    * glam::Mat4::from_scale(Vec3::splat(scale));
                ladders.extend(game::ladders_from_markers(&m.markers, matrix));
            }
            let mut placed_at = None;
            // Big damage set pieces (fallen section, leaning tower) use the tower LOD distances,
            // not the small-prop ones, so their full model shows from the street.
            let set_piece_lod0 = id
                .starts_with("tw_")
                .then(|| self.world_gen.metro_world().map(|m| m.config.towers.lod0_distance))
                .flatten();
            let slot = match set_piece_lod0 {
                Some(lod0) => library.get_tower(
                    device,
                    pack,
                    id,
                    lod0,
                    crate::metro_render::parts::tower_load_ring_range(),
                ),
                None => library.get(device, pack, id, building),
            };
            if let Some(slot) = slot {
                placed_at = Some(placed.len());
                placed.push(library.place(slot, position, yaw, scale));
            }
            if tree {
                let shapes = self.shapes_of(pack, id).clone();
                let collider = if !shapes.is_empty() {
                    Some(physics.add_static_deferred(&shapes, position, yaw, scale))
                } else {
                    None
                };
                trees.push(TreeSlot {
                    index: i,
                    model: id.to_string(),
                    position,
                    yaw,
                    scale,
                    placed: placed_at,
                    collider,
                });
            } else {
                self.shapes_of(pack, id);
                static_props.push((id, position, yaw, scale));
            }
        }
        if !static_props.is_empty() {
            let h = physics.add_static_compound_deferred(static_props.iter().filter_map(
                |&(id, pos, yaw, scale)| {
                    self.shapes
                        .get(id)
                        .filter(|s| !s.is_empty())
                        .map(|s| (s.as_slice(), pos, yaw, scale))
                },
            ));
            if h != ColliderHandle::invalid() {
                colliders.push(h);
            }
        }
        let curbs = self.load_metro_chunk(
            device,
            pack,
            library,
            physics,
            b.coord,
            &mut placed,
            &mut colliders,
            &mut ladders,
        );
        let origin = Vec3::new(b.coord.x as f32, 0.0, b.coord.y as f32) * WorldGenerator::CHUNK_SIZE;
        LoadedChunk {
            ground: engine::render::GpuMesh::new(device, "ground", &b.ground),
            curbs,
            origin,
            placed,
            crates: b.crates,
            ladders,
            trees,
            pois: b.pois,
            colliders,
        }
    }

    /// New Halden: instanced tower parts (`placed`), raised curbs, and colliders (walk-in base + upper box).
    #[allow(clippy::too_many_arguments)]
    fn load_metro_chunk(
        &mut self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        library: &mut PropLibrary,
        physics: &mut PhysicsWorld,
        coord: IVec2,
        placed: &mut Vec<Placed>,
        colliders: &mut Vec<ColliderHandle>,
        ladders: &mut Vec<game::LadderRail>,
    ) -> Option<engine::render::GpuMesh> {
        let metro_data = self.world_gen.metro_world().map(|m| {
            let ground_y = m.layout.site.ground_y;
            let styles = world::metro::default_styles();
            let curbs = curb_chunk_mesh(device, &m.layout, &m.look, coord);
            let tower_placed = place_chunk_towers(device, pack, library, m, coord, styles);
            let bases = tower_bases(&m.towers, coord, ground_y, &m.damage);
            let climb_plans = world::metro::climb_towers(m);
            let hulls = tower_hulls_with_climb(&m.towers, coord, ground_y, styles, &m.damage, &climb_plans);
            let mut fe_parts = Vec::new();
            for idx in world::metro::fire_escape_towers(m) {
                if m.towers.get(idx).is_some_and(|t| tower_chunk(t.pos) == coord) {
                    fe_parts.extend(tower_parts_for_index(
                        &m.towers,
                        idx,
                        ground_y,
                        styles,
                        &m.config,
                        &m.damage,
                    ));
                }
            }
            (curbs, tower_placed, bases, hulls, fe_parts)
        });
        let (curbs, tower_placed, bases, hulls, fe_parts) = metro_data?;
        placed.extend(tower_placed);
        for (id, _, _) in &bases {
            self.shapes_of(pack, id);
        }
        for part in &fe_parts {
            if matches!(part.kind, world::metro::PartKind::Base | world::metro::PartKind::Crown)
                && let Some(model) = pack.find(&part.model)
                && !model.markers.is_empty()
            {
                let matrix = glam::Mat4::from_translation(part.pos) * glam::Mat4::from_rotation_y(part.yaw);
                ladders.extend(game::ladders_from_markers(&model.markers, matrix));
            }
            if part.kind != world::metro::PartKind::Base {
                self.shapes_of(pack, &part.model);
            }
        }
        let (climb_ladders, climb_boxes) = self.world_gen.metro_climb_extras(coord);
        for cl in climb_ladders {
            ladders.push(game::LadderRail::new(cl.foot, cl.deck, cl.id));
        }
        type BoxShapeEntry = (Vec<Vec<[f32; 3]>>, Vec3, f32);
        let climb_box_shapes: Vec<BoxShapeEntry> = climb_boxes
            .into_iter()
            .map(|b| (vec![aabb_points(-b.half, b.half)], b.centre, b.yaw))
            .collect();

        let base_items = bases.iter().filter_map(|(id, pos, yaw)| {
            self.shapes
                .get(id.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| (s.as_slice(), *pos, *yaw, 1.0))
        });
        let hull_items = hulls
            .iter()
            .map(|(p, yaw, pts)| (std::slice::from_ref(pts), *p, *yaw, 1.0));
        let fe_items = fe_parts
            .iter()
            .filter(|p| p.kind != world::metro::PartKind::Base)
            .filter_map(|p| {
                self.shapes
                    .get(p.model.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| (s.as_slice(), p.pos, p.yaw, 1.0))
            });
        let climb_box_items = climb_box_shapes
            .iter()
            .map(|(shapes, centre, yaw)| (shapes.as_slice(), *centre, *yaw, 1.0));

        let h = physics.add_static_compound_deferred(
            base_items
                .chain(hull_items)
                .chain(fe_items)
                .chain(climb_box_items),
        );
        if h != ColliderHandle::invalid() {
            colliders.push(h);
        }
        curbs
    }
}

fn is_no_collider_prop(id: &str) -> bool {
    [
        "weeds_",
        "moss_",
        "leaves_",
        "graffiti_",
        "poster_",
        "vine_",
        "grass_tall_patch",
        "roof_bush",
        "roof_tree",
        "stair_digit_",
        "roof_ladder_seg",
        "stair_sign_",
        "stair_arrow_",
    ]
    .iter()
    .any(|p| id.starts_with(p))
}

fn is_walkable_deck_model(id: &str) -> bool {
    matches!(id, "bridge_wood" | "bridge_concrete" | "boardwalk_section")
}

/// Returns true if the model has no colliders or only a single tall hull that would block walking on the deck.
fn needs_deck_fallback(colliders: &[Vec<[f32; 3]>]) -> bool {
    if colliders.is_empty() {
        return true;
    }
    if colliders.len() == 1 && !colliders[0].is_empty() {
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for &[_, y, _] in &colliders[0] {
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        return (max_y - min_y) > 0.6; // new: DECISIONS 2026-11-25
    }
    false
}

/// Builds an 8-corner thin box covering the horizontal bounds around the deck surface (`y = 0`).
fn deck_box_points(bounds_min: [f32; 3], bounds_max: [f32; 3]) -> Vec<[f32; 3]> {
    let [min_x, min_y, min_z] = bounds_min;
    let [max_x, max_y, max_z] = bounds_max;
    let (y0, y1) = if min_y <= 0.05 && max_y >= 0.0 {
        (-0.25_f32.max(min_y), max_y.min(0.05)) // new: DECISIONS 2026-11-25
    } else {
        (min_y, (min_y + 0.25).min(max_y)) // new: DECISIONS 2026-11-25
    };
    vec![
        [min_x, y0, min_z],
        [max_x, y0, min_z],
        [max_x, y0, max_z],
        [min_x, y0, max_z],
        [min_x, y1, min_z],
        [max_x, y1, min_z],
        [max_x, y1, max_z],
        [min_x, y1, max_z],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_deck_fallback_triggers_for_empty_or_tall_single_hull() {
        assert!(needs_deck_fallback(&[]));
        let tall = vec![vec![[0.0, -0.5, 0.0], [1.0, 1.2, 1.0]]];
        assert!(needs_deck_fallback(&tall));
        let flat = vec![vec![[0.0, -0.2, 0.0], [1.0, 0.05, 1.0]]];
        assert!(!needs_deck_fallback(&flat));
        let multi = vec![
            vec![[0.0, -0.2, 0.0], [1.0, 0.05, 1.0]],
            vec![[0.0, 0.0, 0.0], [0.1, 1.0, 1.0]],
        ];
        assert!(!needs_deck_fallback(&multi));
    }

    #[test]
    fn deck_box_covers_horizontal_bounds_with_thin_slab() {
        let pts = deck_box_points([-6.0, -1.5, -2.5], [6.0, 1.2, 2.5]);
        assert_eq!(pts.len(), 8);
        let min_y = pts.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        let max_y = pts.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!(min_y >= -0.25 - 1e-5);
        assert!(max_y <= 0.05 + 1e-5);
    }
}
