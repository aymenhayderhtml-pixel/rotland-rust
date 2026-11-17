//! Loads the world around the player and drops it behind them, like Godot's ChunkManager:
//! chunks within LOAD_RADIUS are built on worker threads (terrain mesh, props, buildings), a few are
//! uploaded to the GPU per frame so there is never a hitch, and chunks beyond UNLOAD_RADIUS are freed.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use engine::render::{ColorVertex, GpuMesh};
use physics::{ColliderHandle, PhysicsWorld};
use glam::{IVec2, Vec3};
use world::{LootSpot, PoiInstance, PropInstance, WorldGenerator};

use crate::props::{Placed, PropLibrary};
use crate::terrain;

pub const LOAD_RADIUS: i32 = 3;
pub const UNLOAD_RADIUS: i32 = 4;
/// More than Godot's 2: our builds are cheap and run on all cores.
const MAX_IN_FLIGHT: usize = 6;
const MAX_APPLIED_PER_FRAME: usize = 3;

struct Built {
    coord: IVec2,
    ground: Vec<ColorVertex>,
    heights: Vec<f32>,
    props: Vec<PropInstance>,
    pois: Vec<PoiInstance>,
    crates: Vec<LootSpot>,
}

pub struct LoadedChunk {
    pub ground: GpuMesh,
    pub origin: Vec3,
    pub placed: Vec<Placed>,
    /// Loot crate spots of this chunk (opened with E; see crates.rs).
    pub crates: Vec<LootSpot>,
    /// Ladders on this chunk's buildings (watchtowers, the camp's towers), from the models' markers.
    pub ladders: Vec<game::LadderRail>,
    /// Standing trees that an axe can fell.
    pub trees: Vec<TreeSlot>,
    /// The POI buildings on this chunk (stray dogs, map notes).
    pub pois: Vec<PoiInstance>,
    colliders: Vec<ColliderHandle>,
}

/// One standing tree: its index among the chunk's props (the id Godot saves), where it is drawn and
/// its solid shape.
pub struct TreeSlot {
    pub index: usize,
    pub model: String,
    pub position: Vec3,
    pub yaw: f32,
    pub scale: f32,
    /// Index into `placed`.
    pub placed: Option<usize>,
    collider: Option<ColliderHandle>,
}

pub struct Streamer {
    world_gen: Arc<WorldGenerator>,
    pub chunks: HashMap<IVec2, LoadedChunk>,
    in_flight: HashSet<IVec2>,
    tx: Sender<Built>,
    rx: Receiver<Built>,
    /// The camp is one model on the chunk that holds its centre.
    camp_chunk: IVec2,
    /// Collision shapes of each model, read from the pack once.
    shapes: HashMap<String, Vec<Vec<[f32; 3]>>>,
    /// Felled trees (chunk, prop index): left out when their chunk loads.
    pub felled: HashSet<(IVec2, usize)>,
}

impl Streamer {
    pub fn new(world_gen: Arc<WorldGenerator>) -> Self {
        let (tx, rx) = channel();
        let camp_chunk = world_gen.settlements().first().map_or(IVec2::new(i32::MAX, i32::MAX), |s| terrain::chunk_of(s.center));
        Self { world_gen, chunks: HashMap::new(), in_flight: HashSet::new(), tx, rx, camp_chunk, shapes: HashMap::new(), felled: HashSet::new() }
    }

    /// True when every chunk within the load radius of `at` is on the GPU.
    pub fn is_ready_around(&self, at: Vec3) -> bool {
        wanted(terrain::chunk_of(at)).iter().all(|c| self.chunks.contains_key(c))
    }

    /// Call once per frame with the player's position.
    pub fn update(&mut self, device: &wgpu::Device, pack: &pack::ModelPack, library: &mut PropLibrary, physics: &mut PhysicsWorld, player: Vec3) {
        let center = terrain::chunk_of(player);
        let far: Vec<IVec2> = self.chunks.keys().copied().filter(|c| (c.x - center.x).abs().max((c.y - center.y).abs()) > UNLOAD_RADIUS).collect();
        let changed = !far.is_empty();
        for c in far {
            if let Some(chunk) = self.chunks.remove(&c) {
                for h in chunk.colliders.into_iter().chain(chunk.trees.into_iter().filter_map(|t| t.collider)) {
                    physics.remove(h);
                }
            }
        }

        // Ask for the missing chunks, nearest first
        let mut missing: Vec<IVec2> = wanted(center).into_iter().filter(|c| !self.chunks.contains_key(c) && !self.in_flight.contains(c)).collect();
        missing.sort_by_key(|c| (*c - center).length_squared());
        for coord in missing.into_iter().take(MAX_IN_FLIGHT.saturating_sub(self.in_flight.len())) {
            self.in_flight.insert(coord);
            let (world_gen, tx) = (self.world_gen.clone(), self.tx.clone());
            rayon::spawn(move || {
                let (ground, heights) = terrain::chunk_vertices(&world_gen, coord);
                let (pois, props) = world_gen.chunk_contents(coord);
                let crates = world_gen.chunk_crates(coord);
                let _ = tx.send(Built { coord, ground, heights, props, pois, crates });
            });
        }

        let mut applied = false;
        for built in self.rx.try_iter().take(MAX_APPLIED_PER_FRAME).collect::<Vec<_>>() {
            self.in_flight.remove(&built.coord);
            let c = built.coord;
            if (c.x - center.x).abs().max((c.y - center.y).abs()) > UNLOAD_RADIUS {
                continue; // the player moved on while it was building
            }
            let chunk = self.upload(device, pack, library, physics, built);
            self.chunks.insert(c, chunk);
            applied = true;
        }
        if changed || applied {
            // Queries (ground height, the player's mover) must see the new shapes this frame
            physics.update_broad_phase();
        }
    }

    fn shapes_of(&mut self, pack: &pack::ModelPack, id: &str) -> &Vec<Vec<[f32; 3]>> {
        self.shapes
            .entry(id.to_string())
            .or_insert_with(|| pack.find(id).map(|m| m.colliders.iter().map(|c| c.points.clone()).collect()).unwrap_or_default())
    }

    fn upload(&mut self, device: &wgpu::Device, pack: &pack::ModelPack, library: &mut PropLibrary, physics: &mut PhysicsWorld, b: Built) -> LoadedChunk {
        let mut placed = Vec::with_capacity(b.props.len() + b.pois.len() + 1);
        let mut colliders = vec![physics.add_ground_chunk(b.coord, &b.heights)];
        // (model, position, yaw, scale, is a building)
        let mut models: Vec<(&str, Vec3, f32, f32, bool)> = b.props.iter().map(|p| (p.model.as_str(), p.position, p.yaw, p.scale, false)).collect();
        models.extend(b.pois.iter().map(|p| (p.building_id.as_str(), p.position, p.yaw, 1.0, true)));
        let site = self.world_gen.settlements().first().cloned();
        models.extend(b.crates.iter().map(|c| (crate::crates::CRATE_MODEL, c.position, c.yaw, 1.0, false)));
        if let (true, Some(site)) = (b.coord == self.camp_chunk, site.as_ref()) {
            models.push(("settlement", site.center, site.yaw, 1.0, true));
        }
        let mut ladders = Vec::new();
        let mut trees = Vec::new();
        for (i, (id, position, yaw, scale, building)) in models.into_iter().enumerate() {
            // Props come first in `models`, so i is the prop index for them
            let tree = i < b.props.len() && b.props[i].is_tree();
            if tree && self.felled.contains(&(b.coord, i)) {
                continue;
            }
            if building && let Some(m) = pack.find(id) {
                let matrix = glam::Mat4::from_translation(position) * glam::Mat4::from_rotation_y(yaw) * glam::Mat4::from_scale(Vec3::splat(scale));
                ladders.extend(game::ladders_from_markers(&m.markers, matrix));
            }
            let mut placed_at = None;
            if let Some(slot) = library.get(device, pack, id, building) {
                placed_at = Some(placed.len());
                placed.push(library.place(slot, position, yaw, scale));
            }
            let shapes = self.shapes_of(pack, id).clone();
            let mut collider = None;
            if !shapes.is_empty() {
                let h = physics.add_static(&shapes, position, yaw, scale);
                if tree {
                    collider = Some(h);
                } else {
                    colliders.push(h);
                }
            }
            if tree {
                trees.push(TreeSlot { index: i, model: id.to_string(), position, yaw, scale, placed: placed_at, collider });
            }
        }
        let origin = Vec3::new(b.coord.x as f32, 0.0, b.coord.y as f32) * WorldGenerator::CHUNK_SIZE;
        LoadedChunk { ground: GpuMesh::new(device, "ground", &b.ground), origin, placed, crates: b.crates, ladders, trees, pois: b.pois, colliders }
    }
}

impl Streamer {
    /// A tree has been felled: it stops being solid and stays out when its chunk loads again.
    /// Returns its slot (taken out of the chunk's standing trees) so its model can fall.
    pub fn fell(&mut self, physics: &mut PhysicsWorld, coord: IVec2, index: usize) -> Option<TreeSlot> {
        self.felled.insert((coord, index));
        let chunk = self.chunks.get_mut(&coord)?;
        let at = chunk.trees.iter().position(|t| t.index == index)?;
        let mut slot = chunk.trees.swap_remove(at);
        if let Some(h) = slot.collider.take() {
            physics.remove(h);
        }
        Some(slot)
    }
}

fn wanted(center: IVec2) -> Vec<IVec2> {
    (-LOAD_RADIUS..=LOAD_RADIUS).flat_map(|dz| (-LOAD_RADIUS..=LOAD_RADIUS).map(move |dx| center + IVec2::new(dx, dz))).collect()
}
