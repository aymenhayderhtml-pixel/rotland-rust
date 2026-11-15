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
    colliders: Vec<ColliderHandle>,
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
}

impl Streamer {
    pub fn new(world_gen: Arc<WorldGenerator>) -> Self {
        let (tx, rx) = channel();
        let camp_chunk = world_gen.settlements().first().map_or(IVec2::new(i32::MAX, i32::MAX), |s| terrain::chunk_of(s.center));
        Self { world_gen, chunks: HashMap::new(), in_flight: HashSet::new(), tx, rx, camp_chunk, shapes: HashMap::new() }
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
                for h in chunk.colliders {
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
        for (id, position, yaw, scale, building) in models {
            if let Some(slot) = library.get(device, pack, id, building) {
                placed.push(library.place(slot, position, yaw, scale));
            }
            let shapes = self.shapes_of(pack, id).clone();
            if !shapes.is_empty() {
                colliders.push(physics.add_static(&shapes, position, yaw, scale));
            }
        }
        let origin = Vec3::new(b.coord.x as f32, 0.0, b.coord.y as f32) * WorldGenerator::CHUNK_SIZE;
        LoadedChunk { ground: GpuMesh::new(device, "ground", &b.ground), origin, placed, crates: b.crates, colliders }
    }
}

fn wanted(center: IVec2) -> Vec<IVec2> {
    (-LOAD_RADIUS..=LOAD_RADIUS).flat_map(|dz| (-LOAD_RADIUS..=LOAD_RADIUS).map(move |dx| center + IVec2::new(dx, dz))).collect()
}
