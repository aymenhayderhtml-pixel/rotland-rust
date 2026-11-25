use glam::Vec2;

use crate::random::DotnetRandom;
use crate::towns::{BOARDWALK_MODEL, BuildingEntry, TownDef, TownLot, TownsData};

/// Minimum positive weight required for a building entry to be eligible in weighted draws.
// new: DECISIONS 2026-11-25
const MIN_WEIGHT: f32 = 1e-6;

/// One street-side building slot positioned along the main street.
#[derive(Debug, Clone, Copy)]
struct StreetSlot {
    along: f32,
    side: f32,
}

/// Converts a unit XZ facing direction to Godot yaw where `forward = (-sin yaw, -cos yaw)`.
/// Facing east (`+X`) produces `-PI / 2`.
pub fn yaw_from_forward(forward: Vec2) -> f32 {
    (-forward.x).atan2(-forward.y)
}

/// Generates non-overlapping building lots (and optional boardwalk sections) along both sides
/// of a straight main street through `center`.
pub fn generate_lots(
    def: &TownDef,
    data: &TownsData,
    center: Vec2,
    street_dir: Vec2,
    seed: i32,
) -> Vec<TownLot> {
    let street_dir = street_dir.normalize_or_zero();
    let street_right = Vec2::new(-street_dir.y, street_dir.x);
    let mut rng = DotnetRandom::new(seed);

    let slots = build_center_ordered_slots(def.street_length, def.lot_spacing);
    let mut unique_models: Vec<&str> = def
        .buildings
        .iter()
        .filter(|b| b.unique)
        .map(|b| b.model.as_str())
        .collect();
    shuffle_deterministic(&mut unique_models, &mut rng);

    let repeatable: Vec<&BuildingEntry> = def.buildings.iter().filter(|b| !b.unique).collect();
    let mut repeatable_counts = vec![0usize; repeatable.len()];

    let mut lots: Vec<TownLot> = Vec::with_capacity(slots.len());
    let mut unique_idx = 0usize;

    for slot in slots {
        let chosen_model = if unique_idx < unique_models.len() {
            let m = unique_models[unique_idx];
            unique_idx += 1;
            m
        } else if let Some(idx) = pick_weighted_repeatable(&repeatable, &repeatable_counts, &mut rng)
        {
            repeatable_counts[idx] += 1;
            repeatable[idx].model.as_str()
        } else {
            break;
        };

        let footprint = data.footprint(chosen_model);
        let lateral = data.placement.street_setback_m + footprint.y; // data: towns.json placement.street_setback_m
        let pos = center + street_dir * slot.along + street_right * (slot.side * lateral);
        let facing = -street_right * slot.side;
        let yaw = yaw_from_forward(facing);
        let loot_table = def
            .loot_tables
            .get(chosen_model)
            .cloned()
            .unwrap_or_default();

        let candidate = TownLot {
            building: chosen_model.to_string(),
            pos,
            yaw,
            footprint,
            loot_table,
        };

        if !lots.iter().any(|existing| existing.overlaps(&candidate)) {
            lots.push(candidate);
        }
    }

    if def.boardwalk {
        append_boardwalk_lots(def, data, center, street_dir, street_right, &mut lots);
    }

    lots
}

/// Builds symmetric street-side slots every `lot_spacing` along `street_length`, sorted from
/// closest to the town center outward so unique buildings land nearest the centre.
fn build_center_ordered_slots(street_length: f32, lot_spacing: f32) -> Vec<StreetSlot> {
    if lot_spacing <= 0.0 || street_length <= 0.0 {
        return Vec::new();
    }

    let per_side = ((street_length / lot_spacing).floor() as usize).max(1);
    // new: DECISIONS 2026-11-25 (center slots symmetrically around 0)
    let half_index = (per_side as f32 - 1.0) * 0.5;

    let mut slots = Vec::with_capacity(per_side * 2);
    for i in 0..per_side {
        let along = ((i as f32) - half_index) * lot_spacing;
        slots.push(StreetSlot { along, side: 1.0 });
        slots.push(StreetSlot { along, side: -1.0 });
    }

    slots.sort_by(|a, b| {
        a.along
            .abs()
            .total_cmp(&b.along.abs())
            .then_with(|| a.along.total_cmp(&b.along))
            .then_with(|| b.side.total_cmp(&a.side))
    });
    slots
}

/// Fisher-Yates shuffle using `DotnetRandom::next_double`.
fn shuffle_deterministic<T>(items: &mut [T], rng: &mut DotnetRandom) {
    for i in (1..items.len()).rev() {
        let j = ((rng.next_double() * ((i + 1) as f64)) as usize).min(i);
        items.swap(i, j);
    }
}

/// Selects one eligible non-unique building entry proportional to its weight.
fn pick_weighted_repeatable(
    entries: &[&BuildingEntry],
    counts: &[usize],
    rng: &mut DotnetRandom,
) -> Option<usize> {
    let mut total_weight = 0.0_f32;
    for (i, entry) in entries.iter().enumerate() {
        if is_eligible(entry, counts[i]) {
            total_weight += entry.weight;
        }
    }
    if total_weight <= MIN_WEIGHT {
        return None;
    }

    let mut roll = (rng.next_double() as f32) * total_weight;
    let mut last_eligible = None;
    for (i, entry) in entries.iter().enumerate() {
        if !is_eligible(entry, counts[i]) {
            continue;
        }
        last_eligible = Some(i);
        roll -= entry.weight;
        if roll <= 0.0 {
            return Some(i);
        }
    }
    last_eligible
}

fn is_eligible(entry: &BuildingEntry, count: usize) -> bool {
    entry.weight > MIN_WEIGHT && entry.max_count.is_none_or(|cap| count < cap)
}

/// Maximum number of boardwalk sections placed in front of one building.
// new: DECISIONS 2026-11-25
const MAX_BOARDWALK_SECTIONS_PER_BUILDING: usize = 3;
/// Multiplier converting a half-extent to a full width.
// new: DECISIONS 2026-11-25
const HALF_TO_FULL: f32 = 2.0;

/// Appends non-overlapping `boardwalk_section` walkways parallel to the street in front of
/// each building's door side, keeping all sections outside the street surface.
fn append_boardwalk_lots(
    def: &TownDef,
    data: &TownsData,
    center: Vec2,
    street_dir: Vec2,
    street_right: Vec2,
    lots: &mut Vec<TownLot>,
) {
    let footprint = data.footprint(BOARDWALK_MODEL);
    let step = data.placement.boardwalk_step_m; // data: towns.json placement.boardwalk_step_m
    let offset = data.placement.boardwalk_offset_m; // data: towns.json placement.boardwalk_offset_m
    if step <= 0.0 {
        return;
    }

    let loot_table = def
        .loot_tables
        .get(BOARDWALK_MODEL)
        .cloned()
        .unwrap_or_default();
    let yaw = yaw_from_forward(street_dir);
    let building_spans: Vec<(Vec2, f32)> = lots
        .iter()
        .map(|lot| (lot.pos, lot.footprint.x))
        .collect();

    for (b_pos, half_width_along_street) in building_spans {
        let rel = b_pos - center;
        let along = rel.dot(street_dir);
        let side = rel.dot(street_right).signum();
        if side == 0.0 {
            continue;
        }

        let section_count = ((half_width_along_street * HALF_TO_FULL) / step)
            .round()
            .clamp(1.0, MAX_BOARDWALK_SECTIONS_PER_BUILDING as f32) as usize;
        // new: DECISIONS 2026-11-25 (center walkway sections symmetrically in front of building)
        let half_index = (section_count as f32 - 1.0) * 0.5;

        for i in 0..section_count {
            let sec_along = along + ((i as f32) - half_index) * step;
            let pos = center + street_dir * sec_along + street_right * (side * offset);
            let candidate = TownLot {
                building: BOARDWALK_MODEL.to_string(),
                pos,
                yaw,
                footprint,
                loot_table: loot_table.clone(),
            };
            if !lots.iter().any(|existing| existing.overlaps(&candidate)) {
                lots.push(candidate);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blackwater_boardwalks_are_parallel_walkways_outside_street() {
        let data = TownsData::load_embedded();
        let blackwater = data
            .towns
            .iter()
            .find(|t| t.key == "blackwater")
            .expect("blackwater in data/towns.json");
        let center = Vec2::new(100.0, -50.0);
        let street_dir = Vec2::new(0.6, -0.8).normalize();
        let street_right = Vec2::new(-street_dir.y, street_dir.x);
        let lots = generate_lots(blackwater, &data, center, street_dir, 42);

        let buildings: Vec<&TownLot> = lots
            .iter()
            .filter(|l| l.building != BOARDWALK_MODEL)
            .collect();
        let boardwalks: Vec<&TownLot> = lots
            .iter()
            .filter(|l| l.building == BOARDWALK_MODEL)
            .collect();
        assert!(!buildings.is_empty());
        assert!(!boardwalks.is_empty());
        assert!(
            boardwalks.len() <= buildings.len() * MAX_BOARDWALK_SECTIONS_PER_BUILDING,
            "boardwalk count ({}) should be at most {} per building ({})",
            boardwalks.len(),
            MAX_BOARDWALK_SECTIONS_PER_BUILDING,
            buildings.len()
        );

        let cos_1_deg = 1.0_f32.to_radians().cos();
        let street_half_width = 4.0_f32;
        for bw in &boardwalks {
            assert!(
                bw.forward().dot(street_dir) >= cos_1_deg,
                "boardwalk forward {:?} not parallel to street_dir {:?}",
                bw.forward(),
                street_dir
            );
            for corner in bw.corners() {
                let lateral = (corner - center).dot(street_right).abs();
                assert!(
                    lateral > street_half_width,
                    "boardwalk corner lateral {lateral} enters street half-width {street_half_width}"
                );
            }
        }

        for i in 0..lots.len() {
            for j in (i + 1)..lots.len() {
                assert!(!lots[i].overlaps(&lots[j]), "overlapping lots {i} and {j}");
            }
        }
    }
}
