//! New Halden district loot placement (pure, no GPU).
//!
//! Places lobby lockers/med cabinets, rooftop camp stashes, district army caches,
//! and highway jam car trunk spots.

use std::sync::OnceLock;

use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::biome::BiomeId;
use crate::loot::LootSpot;
use crate::metro::layout::{MetroBlock, MetroDistrict};
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::traces::camp_sites;
use crate::random::DotnetRandom;

const CHUNK_SIZE_M: f32 = 64.0; // data: metro.json blockSize
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27
const LOBBY_OFFSET_X: f32 = 0.28; // new: DECISIONS 2026-11-27 (clear of lobbyProps and lobbyDressing)
const LOBBY_OFFSET_Z: f32 = 0.40; // new: DECISIONS 2026-11-27 (against back wall of lobby)
const CAMP_STASH_OFFSET_M: f32 = 1.8; // new: DECISIONS 2026-11-27 (beside tent inside 6m camp)
const SIDEWALK_CORNER_INSET_M: f32 = 1.2; // new: DECISIONS 2026-11-27 (on sidewalk corner, clear of towers)
const CAR_TRUNK_OFFSET_M: f32 = 2.0; // new: DECISIONS 2026-11-27 (at rear trunk of jam car)

/// Per-district chances (`lobbyChance`, `medicalChance` in `data/metro.json` `loot`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DistrictChances {
    pub edge: f32,
    pub midtown: f32,
    pub core: f32,
}

impl Default for DistrictChances {
    fn default() -> Self {
        Self {
            edge: 0.35,    // data: metro.json lobbyChance
            midtown: 0.45, // data: metro.json lobbyChance
            core: 0.55,    // data: metro.json lobbyChance
        }
    }
}

impl DistrictChances {
    fn for_district(&self, d: MetroDistrict) -> f32 {
        match d {
            MetroDistrict::Edge => self.edge,
            MetroDistrict::Midtown => self.midtown,
            MetroDistrict::Core => self.core,
        }
    }
}

/// Fixed army cache counts per district (`armyCaches` in `data/metro.json` `loot`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ArmyCachesConfig {
    pub midtown: usize,
    pub core: usize,
}

impl Default for ArmyCachesConfig {
    fn default() -> Self {
        Self {
            midtown: 2, // data: metro.json armyCaches
            core: 5,    // data: metro.json armyCaches
        }
    }
}

/// District loot placement settings (`loot` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MetroLootConfig {
    pub lobby_chance: DistrictChances,
    pub medical_chance: DistrictChances,
    pub army_caches: ArmyCachesConfig,
    pub camp_stash: f32,
    pub car_trunk_chance: f32,
}

impl Default for MetroLootConfig {
    fn default() -> Self {
        Self {
            lobby_chance: DistrictChances::default(),
            medical_chance: DistrictChances {
                edge: 0.05,    // data: metro.json medicalChance
                midtown: 0.08, // data: metro.json medicalChance
                core: 0.12,    // data: metro.json medicalChance
            },
            army_caches: ArmyCachesConfig::default(),
            camp_stash: 1.0,       // data: metro.json campStash
            car_trunk_chance: 0.08, // data: metro.json carTrunkChance
        }
    }
}

#[derive(Deserialize)]
struct LootWrapper {
    #[serde(default)]
    loot: MetroLootConfig,
}

/// Cached `MetroLootConfig` loaded from `data/metro.json` (falls back to `Default`).
pub fn default_loot_config() -> &'static MetroLootConfig {
    static CACHE: OnceLock<MetroLootConfig> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let Ok(json) = std::fs::read_to_string(&path) else {
            return MetroLootConfig::default();
        };
        serde_json::from_str::<LootWrapper>(&json)
            .map(|w| w.loot)
            .unwrap_or_default()
    })
}

/// Places New Halden loot spots inside `chunk` using `data/metro.json` `loot` settings.
pub fn metro_loot_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    seed: u64,
) -> Vec<LootSpot> {
    metro_loot_in_chunk_with_config(metro, default_loot_config(), chunk, height, seed)
}

/// Places New Halden loot spots inside `chunk` with an explicit `MetroLootConfig`.
pub fn metro_loot_in_chunk_with_config(
    metro: &MetroWorld,
    cfg: &MetroLootConfig,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    seed: u64,
) -> Vec<LootSpot> {
    let mut out = Vec::new();
    append_lobby_loot(metro, cfg, chunk, height, seed, &mut out);
    append_camp_loot(metro, cfg, chunk, seed, &mut out);
    append_army_loot(metro, cfg, chunk, height, seed, &mut out);
    append_car_loot(metro, cfg, chunk, height, seed, &mut out);
    out
}

fn append_lobby_loot(
    metro: &MetroWorld,
    cfg: &MetroLootConfig,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    seed: u64,
    out: &mut Vec<LootSpot>,
) {
    for (idx, tower) in metro.towers.iter().enumerate() {
        if metro.damage.tower_is_replaced(idx) || metro.damage.tower_is_burnt(tower.block) {
            continue;
        }
        let Some(block) = metro.layout.block_at(tower.block.0, tower.block.1) else {
            continue;
        };
        if block.reserved.is_some() {
            continue;
        }
        let (sin_y, cos_y) = tower.yaw.sin_cos();
        let lx = LOBBY_OFFSET_X * tower.size as f32;
        let lz = LOBBY_OFFSET_Z * tower.size as f32;
        let wx = tower.pos.x + lx * cos_y + lz * sin_y;
        let wz = tower.pos.y - lx * sin_y + lz * cos_y;
        if !in_chunk(wx, wz, chunk) || !metro.layout.site.contains_square(Vec2::new(wx, wz)) {
            continue;
        }

        let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, seed, idx as u64, b"metro_lobby"));
        if rng.next_double() >= f64::from(cfg.lobby_chance.for_district(block.district)) {
            continue;
        }
        let is_med = rng.next_double() < f64::from(cfg.medical_chance.for_district(block.district));
        let (kind, table, model) = if is_med {
            ("med", "metro_medical", "med_cabinet")
        } else {
            ("lobby", district_table(block.district), "metro_locker")
        };
        let pos = Vec3::new(wx, height(wx, wz), wz);
        out.push(make_spot(kind, pos, tower.yaw, table, Some(model)));
    }
}

fn append_camp_loot(
    metro: &MetroWorld,
    cfg: &MetroLootConfig,
    chunk: (i32, i32),
    seed: u64,
    out: &mut Vec<LootSpot>,
) {
    for camp in camp_sites(metro) {
        let forward = Vec2::new(-camp.yaw.sin(), -camp.yaw.cos());
        let right = Vec2::new(-forward.y, forward.x);
        let xz = Vec2::new(camp.pos.x, camp.pos.z) + right * CAMP_STASH_OFFSET_M;
        if !in_chunk(xz.x, xz.y, chunk) || !metro.layout.site.contains_square(xz) {
            continue;
        }
        let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, seed, camp.tower as u64, b"metro_camp"));
        if rng.next_double() >= f64::from(cfg.camp_stash) {
            continue;
        }
        let pos = Vec3::new(xz.x, camp.pos.y, xz.y);
        out.push(make_spot("camp", pos, camp.yaw, "metro_camp", Some("camp_stash")));
    }
}

fn append_army_loot(
    metro: &MetroWorld,
    cfg: &MetroLootConfig,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    seed: u64,
    out: &mut Vec<LootSpot>,
) {
    let targets = [
        (MetroDistrict::Midtown, cfg.army_caches.midtown, b"metro_army_mid".as_slice()),
        (MetroDistrict::Core, cfg.army_caches.core, b"metro_army_core".as_slice()),
    ];
    for (district, count, salt) in targets {
        if count == 0 {
            continue;
        }
        let mut candidates: Vec<&MetroBlock> = metro
            .layout
            .blocks
            .iter()
            .filter(|b| {
                b.district == district
                    && b.reserved.is_none()
                    && !metro.damage.tower_is_burnt((b.ix, b.iz))
            })
            .collect();
        let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, seed, 0, salt));
        let take = count.min(candidates.len());
        for _ in 0..take {
            let pick = rng.next(candidates.len() as i32) as usize;
            let block = candidates.swap_remove(pick);
            let inset = metro.config.street_width * 0.5 - SIDEWALK_CORNER_INSET_M; // new: DECISIONS 2026-11-27
            let local = block.rect.0 + Vec2::splat(inset);
            let world_xz = metro.layout.site.to_world(local);
            if in_chunk(world_xz.x, world_xz.y, chunk)
                && metro.layout.site.contains_square(world_xz)
                && !metro.in_tower_footprint(world_xz)
            {
                let pos = Vec3::new(world_xz.x, height(world_xz.x, world_xz.y), world_xz.y);
                out.push(make_spot("army", pos, metro.layout.site.yaw, "metro_army", Some("army_crate")));
            }
        }
    }
}

fn append_car_loot(
    metro: &MetroWorld,
    cfg: &MetroLootConfig,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    seed: u64,
    out: &mut Vec<LootSpot>,
) {
    for (idx, prop) in metro.jam.iter().enumerate() {
        if !prop.model.starts_with("car_jam_") {
            continue;
        }
        let back = Vec2::new(prop.yaw.sin(), prop.yaw.cos());
        let trunk = Vec2::new(prop.pos.x, prop.pos.z) + back * CAR_TRUNK_OFFSET_M;
        if !in_chunk(trunk.x, trunk.y, chunk) || !metro.layout.site.contains_square(trunk) {
            continue;
        }
        if is_reserved_at(metro, trunk) || metro.in_tower_footprint(trunk) {
            continue;
        }
        let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, seed, idx as u64, b"metro_car"));
        if rng.next_double() >= f64::from(cfg.car_trunk_chance) {
            continue;
        }
        let pos = Vec3::new(trunk.x, height(trunk.x, trunk.y), trunk.y);
        // Empty string model means no extra crate mesh is drawn (car trunk is the prop).
        out.push(make_spot("car", pos, prop.yaw, "metro_car", Some("")));
    }
}

fn is_reserved_at(metro: &MetroWorld, world_xz: Vec2) -> bool {
    metro
        .layout
        .block_at_point(world_xz)
        .is_some_and(|b| b.reserved.is_some())
}

fn district_table(d: MetroDistrict) -> &'static str {
    match d {
        MetroDistrict::Edge => "metro_edge",
        MetroDistrict::Midtown => "metro_midtown",
        MetroDistrict::Core => "metro_core",
    }
}

fn make_spot(kind: &str, pos: Vec3, yaw: f32, table: &str, model: Option<&str>) -> LootSpot {
    let crate_id = format!("metro_{kind}_{}_{}", pos.x.round() as i32, pos.z.round() as i32);
    let mut spot = LootSpot::new(crate_id, pos, yaw, BiomeId::Plains);
    spot.table = Some(table.to_string());
    spot.model = model.map(str::to_string);
    spot
}

fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}

fn mix_seed(site: &MetroSite, seed: u64, index: u64, salt: &[u8]) -> i32 {
    let mut hash = FNV_OFFSET;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= u32::from(b);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    };
    feed(site.key.as_bytes());
    feed(&site.center.x.to_bits().to_le_bytes());
    feed(&site.center.y.to_bits().to_le_bytes());
    feed(&seed.to_le_bytes());
    feed(&index.to_le_bytes());
    feed(salt);
    hash as i32
}
