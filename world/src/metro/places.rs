//! R42: the hospital, police station and fire station — where each stands and what a chunk gets.
//!
//! One place per reserved 2x2 block group: the main lot is the lot that touches an avenue (the
//! other three lots are its yards). Everything is deterministic: no rng.

use std::cell::RefCell;
use std::rc::Rc;

use glam::{Vec2, Vec3};

use crate::biome::BiomeId;
use crate::loot::LootSpot;
use crate::metro::layout::{MetroBlock, MetroLayout, Reserved};
use crate::metro::loot_helpers::in_chunk;
use crate::metro::places_data::{PlaceData, PlaceEntry, place_data};
use crate::metro::state::MetroWorld;
use crate::poi::PoiInstance;
use crate::scatter::PropInstance;

/// Blocks per side of a place group (all three place plots are 2x2).
const GROUP_BLOCKS: i32 = 2; // data: metro.json reserved
/// How far back from its lot centre the main building sits, so the front keeps a forecourt.
const FORECOURT_PUSH_M: f32 = 2.0; // new: DECISIONS 2026-11-27
/// Lot sides of a block as site-local outward directions: west, east, south, north.
const SIDES: [Vec2; 4] = [Vec2::NEG_X, Vec2::X, Vec2::NEG_Y, Vec2::Y]; // new: DECISIONS 2026-11-27
/// Step of the probe that finds the nearest avenue when the group touches none itself.
const PROBE_STEP_M: f32 = 4.0; // new: DECISIONS 2026-11-27
/// How many probe steps (two blocks' worth) that search walks.
const PROBE_STEPS: usize = 32; // new: DECISIONS 2026-11-27

/// The three big places: reserved kind, `data/places/<key>.json` key, fallback loot table.
pub const PLACE_KINDS: [(Reserved, &str, &str); 3] = [ // new: DECISIONS 2026-11-27
    (Reserved::Hospital, "hospital", "metro_hospital"),
    (Reserved::Police, "police", "metro_police"),
    (Reserved::FireStation, "fire_station", "metro_fire"),
];

/// One place: its main lot, its three yards and where the building stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceSite {
    pub kind: Reserved,
    /// Grid cell of the lot the main building stands on.
    pub main_lot: (i32, i32),
    /// Grid cells of the yards: `a` beside the main lot along its avenue, `b` diagonal, `c` last.
    pub yard_lots: [(i32, i32); 3],
    /// World XZ of the main building's origin (main lot centre, pushed back from the avenue).
    pub pos: Vec2,
    /// Yaw that turns the building's front (local -Z) toward its avenue.
    pub yaw: f32,
}

impl PlaceSite {
    /// Unit world XZ direction from the main building toward the avenue it faces.
    pub fn front(&self) -> Vec2 {
        Vec2::new(-self.yaw.sin(), -self.yaw.cos())
    }

    /// World XZ centre of one of the site's lots, or None when that block is absent.
    pub fn lot_centre(&self, layout: &MetroLayout, lot: (i32, i32)) -> Option<Vec2> {
        Some(layout.site.to_world(block_centre(layout.block_at(lot.0, lot.1)?)))
    }
}

/// Which part of a place one block is (used by the ground paint). The vectors are site-local,
/// so the paint can compare them with the point it has already converted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LotRole {
    /// The main building's lot: `front` is the outward avenue direction, `edge` its edg midpoint.
    Main { front: Vec2, edge: Vec2 },
    /// One of the yards: `slot` is 0 for `a`, 1 for `b`, 2 for `c`.
    Yard { kind: Reserved, slot: usize },
}

/// Everything one chunk gets from the places (shared with the R42 army line).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlaceChunk {
    pub pois: Vec<PoiInstance>,
    pub props: Vec<PropInstance>,
    pub crates: Vec<LootSpot>,
}

/// The data key and loot table of a place kind, or None for the other reserved kinds.
pub fn place_info(reserved: Reserved) -> Option<(&'static str, &'static str)> {
    PLACE_KINDS.iter().find(|(kind, _, _)| *kind == reserved).map(|&(_, key, t)| (key, t))
}

/// The three place sites of a layout, in `PLACE_KINDS` order.
pub fn place_sites(layout: &MetroLayout) -> Vec<PlaceSite> {
    PLACE_KINDS.iter().filter_map(|&(kind, _, _)| place_site(layout, kind)).collect()
}

/// The site of one place kind, or None when its 2x2 group is not all there.
pub fn place_site(layout: &MetroLayout, kind: Reserved) -> Option<PlaceSite> {
    let (ix0, iz0) = group_origin(layout, kind)?;
    let cells: Vec<(i32, i32)> = (0..GROUP_BLOCKS)
        .flat_map(|dz| (0..GROUP_BLOCKS).map(move |dx| (ix0 + dx, iz0 + dz)))
        .collect();
    if cells.iter().any(|&(ix, iz)| layout.block_at(ix, iz).is_none()) {
        return None;
    }

    // Main lot: the lot touching an avenue (the one nearest the city centre), else grid order.
    let mut main = cells[0];
    let mut nearest = f32::MAX;
    for &cell in &cells {
        let block = layout.block_at(cell.0, cell.1)?;
        if SIDES.into_iter().any(|n| is_avenue_edge(layout, block, n)) {
            let d = block_centre(block).length();
            if d < nearest {
                nearest = d;
                main = cell;
            }
        }
    }

    let main_block = layout.block_at(main.0, main.1)?;
    let local_front = avenue_side(layout, main_block)?;
    let centre = block_centre(main_block);
    let front = world_dir(layout, centre, local_front);
    let pos = layout.site.to_world(centre) - front * FORECOURT_PUSH_M;
    let yaw = (-front.x).atan2(-front.y);

    // Yards: beside the main lot along its avenue, diagonal, then the last.
    let (ox, oz) = (main.0 - ix0, main.1 - iz0);
    let last = GROUP_BLOCKS - 1;
    let beside =
        if local_front.x != 0.0 { (main.0, iz0 + last - oz) } else { (ix0 + last - ox, main.1) };
    let diagonal = (ix0 + last - ox, iz0 + last - oz);
    let last_yard = cells.iter().copied().find(|&c| c != main && c != beside && c != diagonal)?;
    Some(PlaceSite { kind, main_lot: main, yard_lots: [beside, diagonal, last_yard], pos, yaw })
}

/// What part of a place `block` is, for the ground paint. None when it is not a place block.
///
/// This runs per terrain vertex, so the sites are memoised per layout.
pub fn lot_role(layout: &MetroLayout, block: &MetroBlock) -> Option<LotRole> {
    // Only the three R42 places get a painted plaza; Park and the other reserved kinds keep theirs.
    place_info(block.reserved?)?;
    let sites = place_sites_cached(layout);
    let cell = (block.ix, block.iz);
    for site in sites.iter() {
        if cell == site.main_lot {
            let main = layout.block_at(cell.0, cell.1)?;
            let centre = block_centre(main);
            let front = local_dir(layout, centre, site.front());
            let edge = centre + front * (main.rect.1 - main.rect.0) * 0.5;
            return Some(LotRole::Main { front, edge });
        }
        if let Some(slot) = site.yard_lots.iter().position(|&lot| lot == cell) {
            return Some(LotRole::Yard { kind: site.kind, slot });
        }
    }
    None
}

/// The layout's place sites, computed once per thread and layout (see [`lot_role`]).
fn place_sites_cached(layout: &MetroLayout) -> Rc<Vec<PlaceSite>> {
    struct Memo {
        center: Vec2,
        yaw: f32,
        half_size: f32,
        sites: Rc<Vec<PlaceSite>>,
    }
    thread_local! {
        static MEMO: RefCell<Option<Memo>> = const { RefCell::new(None) };
    }
    let site = &layout.site;
    let hit = MEMO.with(|memo| {
        let memo = memo.borrow();
        memo.as_ref().filter(|m| {
            m.center == site.center && m.yaw == site.yaw && m.half_size == site.half_size
        })
        .map(|m| Rc::clone(&m.sites))
    });
    if let Some(sites) = hit {
        return sites;
    }
    let sites = Rc::new(place_sites(layout));
    MEMO.with(|memo| {
        *memo.borrow_mut() = Some(Memo {
            center: site.center,
            yaw: site.yaw,
            half_size: site.half_size,
            sites: Rc::clone(&sites),
        });
    });
    sites
}

/// The POIs, props and crates of every place inside `chunk`, using the real data files.
pub fn places_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> PlaceChunk {
    places_in_chunk_with(metro, chunk, height, &place_data)
}

/// [`places_in_chunk`] with the place data supplied instead of read from `data/places/*.json`.
pub fn places_in_chunk_with<'a>(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    data: &dyn Fn(Reserved) -> Option<&'a PlaceData>,
) -> PlaceChunk {
    let mut out = PlaceChunk::default();
    for (kind, key, table) in PLACE_KINDS {
        let (Some(site), Some(file)) = (place_site(&metro.layout, kind), data(kind)) else {
            continue;
        };
        push_poi(&mut out, &site, key, file, table, chunk, height);
        // `props`/`crates` are in the main building's own frame; each yard is in its lot's frame.
        let mut index = 0;
        let main = LotFrame { origin: site.pos, yaw: site.yaw };
        for entry in &file.props {
            push_entry(&mut out, &main, entry, false, key, &mut index, table, chunk, height);
        }
        for entry in &file.crates {
            push_entry(&mut out, &main, entry, true, key, &mut index, table, chunk, height);
        }
        let yards = [
            (&file.yards.a, &file.yard_crates.a),
            (&file.yards.b, &file.yard_crates.b),
            (&file.yards.c, &file.yard_crates.c),
        ];
        for (slot, (props, crates)) in yards.into_iter().enumerate() {
            let Some(origin) = site.lot_centre(&metro.layout, site.yard_lots[slot]) else {
                continue;
            };
            let frame = LotFrame { origin, yaw: site.yaw };
            for entry in props {
                push_entry(&mut out, &frame, entry, false, key, &mut index, table, chunk, height);
            }
            for entry in crates {
                push_entry(&mut out, &frame, entry, true, key, &mut index, table, chunk, height);
            }
        }
    }
    out
}

/// Top-left cell of a reserved kind's block group (smallest ix, smallest iz).
fn group_origin(layout: &MetroLayout, kind: Reserved) -> Option<(i32, i32)> {
    layout
        .blocks
        .iter()
        .filter(|b| b.reserved == Some(kind))
        .map(|b| (b.ix, b.iz))
        .reduce(|(ax, az), (bx, bz)| (ax.min(bx), az.min(bz)))
}

fn block_centre(block: &MetroBlock) -> Vec2 {
    (block.rect.0 + block.rect.1) * 0.5
}

/// Site-local midpoint of one block edge, `normal` being the outward side.
fn edge_mid(block: &MetroBlock, normal: Vec2) -> Vec2 {
    block_centre(block) + normal * (block.rect.1 - block.rect.0) * 0.5
}

/// True when the street along this block edge is an avenue.
fn is_avenue_edge(layout: &MetroLayout, block: &MetroBlock, normal: Vec2) -> bool {
    let world = layout.site.to_world(edge_mid(block, normal));
    layout.street_at(world).is_some_and(|street| street.avenue)
}

/// The avenue side a block faces. Sides with no other lot of the same place beyond them come
/// first, so a building never fronts the cross street inside its own group; a group that touches
/// no avenue edge borrows the first side whose outward line meets one.
fn avenue_side(layout: &MetroLayout, block: &MetroBlock) -> Option<Vec2> {
    let outward = |n: Vec2| {
        let next = layout.block_at(block.ix + n.x as i32, block.iz + n.y as i32);
        !next.is_some_and(|b| b.reserved == block.reserved)
    };
    let edge_avenue = |n: Vec2| is_avenue_edge(layout, block, n);
    if let Some(side) = SIDES.into_iter().find(|&n| outward(n) && edge_avenue(n)) {
        return Some(side);
    }
    if let Some(side) = SIDES.into_iter().find(|&n| edge_avenue(n)) {
        return Some(side);
    }
    let centre = block_centre(block);
    let reaches = |n: Vec2| {
        (1..=PROBE_STEPS).any(|step| {
            let at = centre + n * (step as f32 * PROBE_STEP_M);
            layout.street_at(layout.site.to_world(at)).is_some_and(|s| s.avenue)
        })
    };
    SIDES.into_iter().find(|&n| outward(n) && reaches(n)).or_else(|| SIDES.into_iter().find(|&n| reaches(n)))
}

/// A site-local direction at `at` as a world unit vector.
fn world_dir(layout: &MetroLayout, at: Vec2, dir: Vec2) -> Vec2 {
    (layout.site.to_world(at + dir) - layout.site.to_world(at)).normalize_or_zero()
}

/// A world direction at world point `at` (site-local input) as a site-local unit vector.
fn local_dir(layout: &MetroLayout, at: Vec2, dir: Vec2) -> Vec2 {
    (layout.site.to_local(layout.site.to_world(at) + dir) - at).normalize_or_zero()
}

/// One lot's local frame: building-local metres to world XZ (front = local -Z).
#[derive(Debug, Clone, Copy)]
struct LotFrame {
    origin: Vec2,
    yaw: f32,
}

impl LotFrame {
    fn to_world(self, lx: f32, lz: f32) -> Vec2 {
        let (sin_y, cos_y) = self.yaw.sin_cos();
        Vec2::new(
            self.origin.x + lx * cos_y + lz * sin_y,
            self.origin.y - lx * sin_y + lz * cos_y,
        )
    }
}

fn push_poi(
    out: &mut PlaceChunk,
    site: &PlaceSite,
    key: &str,
    file: &PlaceData,
    table: &str,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) {
    if !in_chunk(site.pos.x, site.pos.y, chunk) {
        return;
    }
    let y = height(site.pos.x, site.pos.y);
    let poi = PoiInstance::new(
        format!("metro_{key}"),
        &file.model,
        Vec3::new(site.pos.x, y, site.pos.y),
        site.yaw,
    );
    out.pois.push(poi.with_loot_table(table));
}

/// One data entry: a crate when `is_crate` is true, otherwise a pure prop.
#[allow(clippy::too_many_arguments)]
fn push_entry(
    out: &mut PlaceChunk,
    frame: &LotFrame,
    entry: &PlaceEntry,
    is_crate: bool,
    key: &str,
    index: &mut usize,
    table: &str,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) {
    let at = frame.to_world(entry.x, entry.z);
    if !in_chunk(at.x, at.y, chunk) {
        if is_crate {
            *index += 1;
        }
        return;
    }
    let y = height(at.x, at.y) + entry.y;
    let pos = Vec3::new(at.x, y, at.y);
    let yaw = frame.yaw + entry.yaw;
    if is_crate {
        let mut spot = LootSpot::new(format!("metro_{key}_c{index}"), pos, yaw, BiomeId::Plains);
        spot.table = Some(entry.table.clone().unwrap_or_else(|| table.to_string()));
        spot.model = Some(entry.model.clone());
        out.crates.push(spot);
        *index += 1;
    } else {
        out.props.push(PropInstance::new(&entry.model, pos, yaw, entry.scale));
    }
}
