//! Helpers for New Halden loot placement: chunk test, seeding, spot building
//! and reserved-block checks. Pure logic: no GPU.

use glam::{Vec2, Vec3};

use crate::biome::BiomeId;
use crate::loot::LootSpot;
use crate::metro::layout::MetroDistrict;
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;

const CHUNK_SIZE_M: f32 = 64.0; // data: metro.json blockSize
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27

pub(super) fn is_reserved_at(metro: &MetroWorld, world_xz: Vec2) -> bool {
    metro
        .layout
        .block_at_point(world_xz)
        .is_some_and(|b| b.reserved.is_some())
}

pub(super) fn district_table(d: MetroDistrict) -> &'static str {
    match d {
        MetroDistrict::Edge => "metro_edge",
        MetroDistrict::Midtown => "metro_midtown",
        MetroDistrict::Core => "metro_core",
    }
}

pub(super) fn make_spot(kind: &str, pos: Vec3, yaw: f32, table: &str, model: Option<&str>) -> LootSpot {
    let crate_id = format!("metro_{kind}_{}_{}", pos.x.round() as i32, pos.z.round() as i32);
    let mut spot = LootSpot::new(crate_id, pos, yaw, BiomeId::Plains);
    spot.table = Some(table.to_string());
    spot.model = model.map(str::to_string);
    spot
}

pub(super) fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}

pub(super) fn mix_seed(site: &MetroSite, seed: u64, index: u64, salt: &[u8]) -> i32 {
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
