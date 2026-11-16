use serde::{Deserialize, Serialize};

use crate::items::Inventory;

/// What happens to carried items and equipment on player death.
/// Matches `scripts/Difficulty/DeathRule.cs`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathRule {
    /// Hardcore: everything carried is destroyed.
    #[default]
    LoseAll,
    /// Soft: a random half of the bag's stacks (rounded down kept) are lost.
    KeepHalf,
    /// Relaxed: nothing is lost.
    KeepAll,
}

impl DeathRule {
    /// Saved string id ("lose_all", "keep_half", "keep_all").
    // Godot: DeathRule.cs DeathRuleIds.IdOf
    pub fn id(&self) -> &'static str {
        match self {
            Self::LoseAll => "lose_all",
            Self::KeepHalf => "keep_half",
            Self::KeepAll => "keep_all",
        }
    }

    /// Parses a death rule id, returning None for unknown values.
    // Godot: DeathRule.cs DeathRuleIds.TryParse
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "lose_all" => Some(Self::LoseAll),
            "keep_half" => Some(Self::KeepHalf),
            "keep_all" => Some(Self::KeepAll),
            _ => None,
        }
    }

    /// Applies the death penalty to an inventory.
    /// Returns the total item count and dollar value destroyed.
    // Godot: DeathPenalty.cs OnPlayerDied / LoseHalf
    pub fn apply_to_inventory(
        &self,
        inventory: &mut Inventory,
        rng: &mut impl FnMut() -> f32,
    ) -> DeathPenaltyResult {
        match self {
            Self::KeepAll => DeathPenaltyResult {
                lost_count: 0,
                lost_value: 0,
            },
            Self::LoseAll => {
                let (count, value) = tally_inventory(inventory);
                inventory.clear();
                DeathPenaltyResult {
                    lost_count: count,
                    lost_value: value,
                }
            }
            Self::KeepHalf => {
                let mut non_empty_slots: Vec<usize> = inventory
                    .slots()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, slot)| slot.as_ref().map(|_| i))
                    .collect();

                let keep = non_empty_slots.len() / 2;

                // Sort by random float, matching C# OrderBy(_ => random.Next()).Skip(keep)
                let mut keyed: Vec<(f32, usize)> = non_empty_slots
                    .into_iter()
                    .map(|idx| (rng(), idx))
                    .collect();
                keyed.sort_by(|a, b| a.0.total_cmp(&b.0));

                let registry = inventory.registry().clone();
                let mut count = 0;
                let mut value = 0;

                for (_, slot) in keyed.into_iter().skip(keep) {
                    if let Some(stack) = inventory.slot(slot).cloned() {
                        let item_value = registry.get(&stack.item_id).map_or(0, |def| def.value);
                        let removed = inventory.remove_from_slot(slot, stack.count);
                        count += removed;
                        value += item_value * removed;
                    }
                }

                DeathPenaltyResult {
                    lost_count: count,
                    lost_value: value,
                }
            }
        }
    }

    /// Convenience helper using a pseudo-random seed (PCG-like LCG).
    pub fn apply_to_inventory_with_seed(
        &self,
        inventory: &mut Inventory,
        seed: u64,
    ) -> DeathPenaltyResult {
        let mut state = seed.wrapping_add(1);
        let mut rng = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 32) as f32) / (u32::MAX as f32)
        };
        self.apply_to_inventory(inventory, &mut rng)
    }
}

/// Tally of items and total dollar value lost upon player death.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeathPenaltyResult {
    pub lost_count: u32,
    pub lost_value: u32,
}

/// Counts carried items and their total dollar value without altering the inventory.
// Godot: DeathPenalty.cs Tally
pub fn tally_inventory(inventory: &Inventory) -> (u32, u32) {
    let mut count = 0;
    let mut value = 0;
    let registry = inventory.registry();
    for slot in inventory.slots() {
        if let Some(stack) = slot {
            count += stack.count;
            let item_value = registry.get(&stack.item_id).map_or(0, |def| def.value);
            value += item_value * stack.count;
        }
    }
    (count, value)
}
