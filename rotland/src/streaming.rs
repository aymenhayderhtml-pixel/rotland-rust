//! Loads the world around the player and drops it behind them, like Godot's ChunkManager:
//! chunks within LOAD_RADIUS are built on worker threads (terrain mesh, props, buildings), a few are
//! uploaded to the GPU per frame so there is never a hitch, and chunks beyond UNLOAD_RADIUS are freed.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use engine::render::{ColorVertex, GpuMesh};
use game::{DogPoiSite, LadderRail};
use glam::{IVec2, Vec3};
use physics::{ColliderHandle, PhysicsWorld};
use world::{LootSpot, PoiInstance, PropInstance, WorldGenerator};

use crate::props::{Placed, PropLibrary};
use crate::terrain;

#[path = "streaming_metro.rs"]
mod metro;

pub const LOAD_RADIUS: i32 = 3;
pub const UNLOAD_RADIUS: i32 = 4;
/// More than Godot's 2: our builds are cheap and run on all cores.
const MAX_IN_FLIGHT: usize = 6;
const MAX_APPLIED_PER_FRAME: usize = 1; // new: DECISIONS 2026-11-25

pub(super) struct Built {
    pub(super) coord: IVec2,
    pub(super) ground: Vec<ColorVertex>,
    pub(super) heights: Vec<f32>,
    pub(super) props: Vec<PropInstance>,
    pub(super) pois: Vec<PoiInstance>,
    pub(super) crates: Vec<LootSpot>,
}

pub struct LoadedChunk {
    pub ground: GpuMesh,
    pub curbs: Option<GpuMesh>,
    pub origin: Vec3,
    pub placed: Vec<Placed>,
    /// Loot crate spots of this chunk (opened with E; see crates.rs).
    pub crates: Vec<LootSpot>,
    /// Ladders on this chunk's buildings (watchtowers, the camp's towers), from the models' markers.
    pub ladders: Vec<LadderRail>,
    /// Standing trees that an axe can fell.
    pub trees: Vec<TreeSlot>,
    /// The POI buildings on this chunk (stray dogs, map notes).
    pub pois: Vec<PoiInstance>,
    pub(super) colliders: Vec<ColliderHandle>,
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
    pub(super) world_gen: Arc<WorldGenerator>,
    pub chunks: HashMap<IVec2, LoadedChunk>,
    in_flight: HashSet<IVec2>,
    tx: Sender<Built>,
    rx: Receiver<Built>,
    /// The camp is one model on the chunk that holds its centre.
    pub(super) camp_chunk: IVec2,
    /// Collision shapes of each model, read from the pack once.
    pub(super) shapes: HashMap<String, Vec<Vec<[f32; 3]>>>,
    /// Felled trees (chunk, prop index): left out when their chunk loads.
    pub felled: HashSet<(IVec2, usize)>,
    /// Cached dog POI sites across all loaded chunks, rebuilt only on chunk load/unload.
    pub dog_sites: Vec<DogPoiSite>,
    /// Cached ladders across all loaded chunks, rebuilt only on chunk load/unload.
    pub ladders: Vec<LadderRail>,
    /// Ladders not tied to a chunk (the safe house rope while it is down, R44). Kept by `rebuild_ladders`.
    pub extra_ladders: Vec<LadderRail>,
    /// Colliders of chunks dropped by `reload_chunks`. They stay solid until the new chunk is built
    /// (so the player never falls through), then they are removed.
    stale: Vec<(IVec2, Vec<ColliderHandle>)>,
    /// Monotonic counter incremented whenever chunks load or unload.
    pub generation: u64,
    far_scratch: Vec<IVec2>,
    missing_scratch: Vec<IVec2>,
    rx_scratch: Vec<Built>,
}

impl Streamer {
    pub fn new(world_gen: Arc<WorldGenerator>) -> Self {
        let (tx, rx) = channel();
        let camp_chunk = world_gen
            .settlements()
            .first()
            .map_or(IVec2::new(i32::MAX, i32::MAX), |s| terrain::chunk_of(s.center));
        Self {
            world_gen,
            chunks: HashMap::new(),
            in_flight: HashSet::new(),
            tx,
            rx,
            camp_chunk,
            shapes: HashMap::new(),
            felled: HashSet::new(),
            dog_sites: Vec::new(),
            ladders: Vec::new(),
            extra_ladders: Vec::new(),
            stale: Vec::new(),
            generation: 0,
            far_scratch: Vec::new(),
            missing_scratch: Vec::new(),
            rx_scratch: Vec::new(),
        }
    }

    /// Rebuilds cached dog POI sites across all currently loaded chunks.
    pub fn rebuild_dog_sites(&mut self) {
        self.dog_sites.clear();
        let cfg = self.world_gen.poi_config();
        for chunk in self.chunks.values() {
            for poi in &chunk.pois {
                self.dog_sites.push(crate::poi_links::build_dog_poi_site(poi, cfg));
            }
        }
    }

    /// Rebuilds cached ladder rails across all currently loaded chunks.
    pub fn rebuild_ladders(&mut self) {
        self.ladders.clear();
        for chunk in self.chunks.values() {
            self.ladders.extend(chunk.ladders.iter().cloned());
        }
        self.ladders.extend(self.extra_ladders.iter().cloned());
    }

    /// Drops the loaded chunks in `coords` so they are built again on the next update (R44 rope ladder).
    pub fn reload_chunks(&mut self, coords: &[IVec2]) {
        for c in coords {
            if let Some(chunk) = self.chunks.remove(c) {
                let handles: Vec<ColliderHandle> = chunk
                    .colliders
                    .into_iter()
                    .chain(chunk.trees.into_iter().filter_map(|t| t.collider))
                    .collect();
                self.stale.push((*c, handles));
            }
        }
        self.generation = self.generation.wrapping_add(1);
        self.rebuild_ladders();
    }

    /// True when every chunk within the load radius of `at` is on the GPU.
    pub fn is_ready_around(&self, at: Vec3) -> bool {
        let center = terrain::chunk_of(at);
        for dz in -LOAD_RADIUS..=LOAD_RADIUS {
            for dx in -LOAD_RADIUS..=LOAD_RADIUS {
                if !self.chunks.contains_key(&(center + IVec2::new(dx, dz))) {
                    return false;
                }
            }
        }
        true
    }

    /// Call once per frame with the player's position.
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        library: &mut PropLibrary,
        physics: &mut PhysicsWorld,
        player: Vec3,
    ) {
        let center = terrain::chunk_of(player);
        self.far_scratch.clear();
        self.far_scratch.extend(
            self.chunks
                .keys()
                .copied()
                .filter(|c| (c.x - center.x).abs().max((c.y - center.y).abs()) > UNLOAD_RADIUS),
        );
        let changed = !self.far_scratch.is_empty();
        for i in 0..self.far_scratch.len() {
            let c = self.far_scratch[i];
            if let Some(chunk) = self.chunks.remove(&c) {
                for h in chunk
                    .colliders
                    .into_iter()
                    .chain(chunk.trees.into_iter().filter_map(|t| t.collider))
                {
                    physics.remove_deferred(h);
                }
            }
        }
        // Stale colliders of chunks the player has left go too (see reload_chunks)
        self.stale.retain(|(c, handles)| {
            if (c.x - center.x).abs().max((c.y - center.y).abs()) <= UNLOAD_RADIUS {
                return true;
            }
            for h in handles {
                physics.remove_deferred(*h);
            }
            false
        });

        // Ask for the missing chunks, nearest first
        self.missing_scratch.clear();
        for dz in -LOAD_RADIUS..=LOAD_RADIUS {
            for dx in -LOAD_RADIUS..=LOAD_RADIUS {
                let c = center + IVec2::new(dx, dz);
                if !self.chunks.contains_key(&c) && !self.in_flight.contains(&c) {
                    self.missing_scratch.push(c);
                }
            }
        }
        self.missing_scratch.sort_by_key(|c| (*c - center).length_squared());
        let budget = MAX_IN_FLIGHT
            .saturating_sub(self.in_flight.len())
            .min(self.missing_scratch.len());
        for i in 0..budget {
            let coord = self.missing_scratch[i];
            self.in_flight.insert(coord);
            let (world_gen, tx) = (self.world_gen.clone(), self.tx.clone());
            rayon::spawn(move || {
                let (ground, heights) = terrain::chunk_vertices(&world_gen, coord);
                let (pois, props) = world_gen.chunk_contents(coord);
                let crates = world_gen.chunk_crates_from(coord, &pois, &props);
                let _ = tx.send(Built { coord, ground, heights, props, pois, crates });
            });
        }

        let mut applied = false;
        self.rx_scratch.clear();
        while self.rx_scratch.len() < MAX_APPLIED_PER_FRAME {
            if let Ok(built) = self.rx.try_recv() {
                self.rx_scratch.push(built);
            } else {
                break;
            }
        }
        let mut rx_scratch = std::mem::take(&mut self.rx_scratch);
        for built in rx_scratch.drain(..) {
            self.in_flight.remove(&built.coord);
            let c = built.coord;
            if (c.x - center.x).abs().max((c.y - center.y).abs()) > UNLOAD_RADIUS {
                continue; // the player moved on while it was building
            }
            let chunk = self.upload(device, pack, library, physics, built);
            self.chunks.insert(c, chunk);
            self.drop_stale(c, physics);
            applied = true;
        }
        self.rx_scratch = rx_scratch;
        if changed || applied {
            self.generation = self.generation.wrapping_add(1);
            self.rebuild_dog_sites();
            self.rebuild_ladders();
            // Queries (ground height, the player's mover) must see the new shapes this frame
            physics.update_broad_phase();
        }
    }

    /// Removes the old colliders of a chunk that has been built again.
    fn drop_stale(&mut self, coord: IVec2, physics: &mut PhysicsWorld) {
        self.stale.retain(|(c, handles)| {
            if *c != coord {
                return true;
            }
            for h in handles {
                physics.remove_deferred(*h);
            }
            false
        });
    }

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
