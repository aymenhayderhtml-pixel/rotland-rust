//! Loads the world around the player and drops it behind them, like Godot's ChunkManager:
//! chunks within LOAD_RADIUS are built on worker threads (terrain mesh, props, buildings), a few are
//! uploaded to the GPU per frame so there is never a hitch, and chunks beyond UNLOAD_RADIUS are freed.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use engine::render::{ColorVertex, GpuMesh};
use glam::{IVec2, Vec3};
use world::{PoiInstance, PropInstance, WorldGenerator};

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
    props: Vec<PropInstance>,
    pois: Vec<PoiInstance>,
}

pub struct LoadedChunk {
    pub ground: GpuMesh,
    pub origin: Vec3,
    pub placed: Vec<Placed>,
}

pub struct Streamer {
    world_gen: Arc<WorldGenerator>,
    pub chunks: HashMap<IVec2, LoadedChunk>,
    in_flight: HashSet<IVec2>,
    tx: Sender<Built>,
    rx: Receiver<Built>,
    /// The camp is one model on the chunk that holds its centre.
    camp_chunk: IVec2,
}

impl Streamer {
    pub fn new(world_gen: Arc<WorldGenerator>) -> Self {
        let (tx, rx) = channel();
        let camp_chunk = world_gen.settlements().first().map_or(IVec2::new(i32::MAX, i32::MAX), |s| terrain::chunk_of(s.center));
        Self { world_gen, chunks: HashMap::new(), in_flight: HashSet::new(), tx, rx, camp_chunk }
    }

    /// True when every chunk within the load radius of `at` is on the GPU.
    pub fn is_ready_around(&self, at: Vec3) -> bool {
        wanted(terrain::chunk_of(at)).iter().all(|c| self.chunks.contains_key(c))
    }

    /// Call once per frame with the player's position.
    pub fn update(&mut self, device: &wgpu::Device, pack: &pack::ModelPack, library: &mut PropLibrary, player: Vec3) {
        let center = terrain::chunk_of(player);
        self.chunks.retain(|c, _| (c.x - center.x).abs().max((c.y - center.y).abs()) <= UNLOAD_RADIUS);

        // Ask for the missing chunks, nearest first
        let mut missing: Vec<IVec2> = wanted(center).into_iter().filter(|c| !self.chunks.contains_key(c) && !self.in_flight.contains(c)).collect();
        missing.sort_by_key(|c| (*c - center).length_squared());
        for coord in missing.into_iter().take(MAX_IN_FLIGHT.saturating_sub(self.in_flight.len())) {
            self.in_flight.insert(coord);
            let (world_gen, tx) = (self.world_gen.clone(), self.tx.clone());
            rayon::spawn(move || {
                let ground = terrain::chunk_vertices(&world_gen, coord);
                let (pois, props) = world_gen.chunk_contents(coord);
                let _ = tx.send(Built { coord, ground, props, pois });
            });
        }

        for built in self.rx.try_iter().take(MAX_APPLIED_PER_FRAME).collect::<Vec<_>>() {
            self.in_flight.remove(&built.coord);
            let c = built.coord;
            if (c.x - center.x).abs().max((c.y - center.y).abs()) > UNLOAD_RADIUS {
                continue; // the player moved on while it was building
            }
            self.chunks.insert(c, self.upload(device, pack, library, built));
        }
    }

    fn upload(&self, device: &wgpu::Device, pack: &pack::ModelPack, library: &mut PropLibrary, b: Built) -> LoadedChunk {
        let mut placed = Vec::with_capacity(b.props.len() + b.pois.len() + 1);
        for p in &b.props {
            if let Some(slot) = library.get(device, pack, &p.model, false) {
                placed.push(library.place(slot, p.position, p.yaw, p.scale));
            }
        }
        for poi in &b.pois {
            if let Some(slot) = library.get(device, pack, &poi.building_id, true) {
                placed.push(library.place(slot, poi.position, poi.yaw, 1.0));
            }
        }
        if b.coord == self.camp_chunk {
            let site = &self.world_gen.settlements()[0];
            if let Some(slot) = library.get(device, pack, "settlement", true) {
                placed.push(library.place(slot, site.center, site.yaw, 1.0));
            }
        }
        let origin = Vec3::new(b.coord.x as f32, 0.0, b.coord.y as f32) * WorldGenerator::CHUNK_SIZE;
        LoadedChunk { ground: GpuMesh::new(device, "ground", &b.ground), origin, placed }
    }
}

fn wanted(center: IVec2) -> Vec<IVec2> {
    (-LOAD_RADIUS..=LOAD_RADIUS).flat_map(|dz| (-LOAD_RADIUS..=LOAD_RADIUS).map(move |dx| center + IVec2::new(dx, dz))).collect()
}
