use std::collections::HashMap;

use world::DotnetRandom;

use crate::items::ItemStack;
use crate::loot::table::{LootEntryData, LootTableData};

/// Bit-exact port of C# LootRoller (scripts/World/LootRoller.cs).
/// Rolls are deterministic given (world_seed, crate_id).
pub struct LootRoller;

impl LootRoller {
    /// A stable seed from the world seed and the crate id.
    pub fn seed(world_seed: i32, crate_id: &str) -> i32 {
        crate_seed(world_seed, crate_id)
    }

    /// Scales the number of rolls by the loot multiplier from difficulty (1.0 = table as written).
    pub fn scale_rolls(rolls: i32, amount: f32) -> i32 {
        scale_rolls(rolls, amount)
    }

    /// Rolls a loot table deterministically from (world_seed, crate_id).
    pub fn roll(table: &LootTableData, world_seed: i32, crate_id: &str, amount: f32) -> Vec<ItemStack> {
        roll(table, world_seed, crate_id, amount)
    }

    /// Dollars the crate holds, or 0. Seeded apart from items using crate_id + "#cash".
    pub fn roll_cash(table: &LootTableData, world_seed: i32, crate_id: &str) -> i32 {
        roll_cash(table, world_seed, crate_id)
    }
}

/// A stable seed from the world seed and the crate id (port of LootRoller.Seed).
pub fn crate_seed(world_seed: i32, crate_id: &str) -> i32 {
    let mut h = 2166136261u32 ^ (world_seed as u32);
    for c in crate_id.chars() {
        h ^= c as u32;
        h = h.wrapping_mul(16777619u32);
    }
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352du32);
    h ^= h >> 15;
    h as i32
}

/// Rolls after the loot multiplier (port of LootRoller.ScaleRolls).
/// A crate that gives something at 1x still gives at least one roll.
/// Rounding is away from zero (e.g. 0.5x of 3 rolls gives 2).
pub fn scale_rolls(rolls: i32, amount: f32) -> i32 {
    if rolls <= 0 || (amount - 1.0).abs() < 1e-5 {
        return rolls;
    }
    1.max((rolls as f32 * amount).round() as i32)
}

/// Rolls a loot table deterministically from (world_seed, crate_id).
pub fn roll(table: &LootTableData, world_seed: i32, crate_id: &str, amount: f32) -> Vec<ItemStack> {
    if table.entries.is_empty() {
        return Vec::new();
    }
    let mut rng = DotnetRandom::new(crate_seed(world_seed, crate_id));
    let raw_rolls = rng.next_range(table.rolls[0], table.rolls[1] + 1);
    let rolls = scale_rolls(raw_rolls, amount);
    let total_weight: i32 = table.entries.iter().map(|e| e.weight).sum();
    if total_weight <= 0 {
        return Vec::new();
    }

    let mut counts: HashMap<String, u32> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for _ in 0..rolls {
        let pick_val = rng.next(total_weight);
        let Some(entry) = pick_entry(table, pick_val) else {
            continue;
        };
        let count = rng.next_range(entry.count[0], entry.count[1] + 1) as u32;
        match counts.entry(entry.id.clone()) {
            std::collections::hash_map::Entry::Vacant(v) => {
                order.push(entry.id.clone());
                v.insert(count);
            }
            std::collections::hash_map::Entry::Occupied(mut o) => {
                *o.get_mut() += count;
            }
        }
    }

    order
        .into_iter()
        .map(|id| {
            let count = counts.remove(&id).unwrap_or(0);
            ItemStack::new(id, count)
        })
        .collect()
}

fn pick_entry(table: &LootTableData, pick: i32) -> Option<&LootEntryData> {
    let mut acc = 0;
    for entry in &table.entries {
        acc += entry.weight;
        if pick < acc {
            return Some(entry);
        }
    }
    table.entries.last()
}

/// Dollars the crate holds, or 0 (port of LootRoller.RollCash).
pub fn roll_cash(table: &LootTableData, world_seed: i32, crate_id: &str) -> i32 {
    let Some(cash) = &table.cash else {
        return 0;
    };
    let cash_id = format!("{crate_id}#cash");
    let seed = crate_seed(world_seed, &cash_id);
    let mut rng = DotnetRandom::new(seed);
    if rng.next_double() >= cash.chance {
        return 0;
    }
    rng.next_range(cash.min, cash.max + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_loot_table_rolls_nothing_without_panic() {
        let table = LootTableData {
            rolls: [1, 3],
            entries: Vec::new(),
            cash: None,
        };
        let items = roll(&table, 1234, "empty_crate", 1.0);
        assert!(items.is_empty());
        let cash = roll_cash(&table, 1234, "empty_crate");
        assert_eq!(cash, 0);
    }
}

