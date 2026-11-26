use crate::crafting::recipe::{RecipeDef, RecipeXp, CAMPFIRE_STATION, HAND_STATION};
use crate::crafting::registry::RecipeRegistry;
use game_core::items::{Inventory, ItemStack};

/// Information on craft progress during an active craft.
#[derive(Clone, Debug, PartialEq)]
pub struct CraftProgress {
    pub recipe_id: String,
    pub progress: f32,
}

/// The result when a craft finishes successfully.
#[derive(Clone, Debug, PartialEq)]
pub struct CraftFinishResult {
    pub recipe_id: String,
    pub outputs_added: Vec<ItemStack>,
    /// Items that could not fit in the bag due to space/weight, to be dropped in front of the player.
    pub leftovers: Vec<ItemStack>,
    /// Skill XP earned for finishing the craft.
    pub xp: Option<RecipeXp>,
}

/// Why a craft was cancelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelReason {
    Manual,
    Dead,
    TooFarFromCampfire,
}

/// The result when a craft is cancelled or interrupted.
#[derive(Clone, Debug, PartialEq)]
pub struct CraftCancelResult {
    pub recipe_id: String,
    pub returned_inputs: Vec<ItemStack>,
    pub reason: CancelReason,
}

/// State update produced by CraftingSystem::update.
#[derive(Clone, Debug, PartialEq)]
pub enum CraftUpdate {
    None,
    Progress(CraftProgress),
    Finished(CraftFinishResult),
    Cancelled(CraftCancelResult),
}

/// Hand and campfire crafting state machine.
/// Port of `scripts/Crafting/CraftingSystem.cs`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CraftingSystem {
    active: Option<RecipeDef>,
    active_station: String,
    elapsed: f32,
}

impl CraftingSystem {
    pub fn new() -> Self {
        Self {
            active: None,
            active_station: HAND_STATION.to_string(),
            elapsed: 0.0,
        }
    }

    /// The craft in progress, or None.
    pub fn active(&self) -> Option<&RecipeDef> {
        self.active.as_ref()
    }

    /// Station being used for the active craft.
    pub fn active_station(&self) -> &str {
        &self.active_station
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Share of the active craft that is done, from 0.0 to 1.0.
    pub fn progress(&self) -> f32 {
        if let Some(recipe) = &self.active {
            if recipe.time <= 0.0 {
                1.0
            } else {
                (self.elapsed / recipe.time).clamp(0.0, 1.0)
            }
        } else {
            0.0
        }
    }

    pub fn is_crafting(&self) -> bool {
        self.active.is_some()
    }

    /// True when a craft at this station could start now: no craft running, player alive,
    /// station matches, campfire nearby if needed, and bag contains all inputs.
    /// // Godot: CraftingSystem.cs CanCraft
    pub fn can_craft(
        &self,
        recipe_id: &str,
        station: &str,
        inventory: &Inventory,
        near_lit_campfire: bool,
        is_dead: bool,
        registry: &RecipeRegistry,
    ) -> bool {
        if self.active.is_some() || is_dead {
            return false;
        }
        let Some(recipe) = registry.get(recipe_id) else {
            return false;
        };
        if recipe.station != station {
            return false;
        }
        if station == CAMPFIRE_STATION && !near_lit_campfire {
            return false;
        }
        recipe
            .inputs
            .iter()
            .all(|input| inventory.has(&input.id, input.count))
    }

    /// Takes the inputs and starts the craft. Returns false and makes no change if CanCraft fails.
    /// // Godot: CraftingSystem.cs StartCraft
    pub fn start_craft(
        &mut self,
        recipe_id: &str,
        station: &str,
        inventory: &mut Inventory,
        near_lit_campfire: bool,
        is_dead: bool,
        registry: &RecipeRegistry,
    ) -> bool {
        if !self.can_craft(recipe_id, station, inventory, near_lit_campfire, is_dead, registry) {
            return false;
        }

        let Some(recipe) = registry.get(recipe_id).cloned() else {
            return false;
        };
        for input in &recipe.inputs {
            inventory.remove(&input.id, input.count);
        }

        self.active = Some(recipe);
        self.active_station = station.to_string();
        self.elapsed = 0.0;
        true
    }

    /// Stops the active craft and returns its inputs to the inventory.
    /// // Godot: CraftingSystem.cs CancelCraft
    pub fn cancel_craft(
        &mut self,
        inventory: &mut Inventory,
        reason: CancelReason,
    ) -> Option<CraftCancelResult> {
        let recipe = self.active.take()?;
        let mut returned_inputs = Vec::with_capacity(recipe.inputs.len());

        for input in &recipe.inputs {
            let leftover = inventory.add(&input.id, input.count);
            let returned = input.count.saturating_sub(leftover);
            if returned > 0 {
                returned_inputs.push(ItemStack::new(&input.id, returned));
            }
        }

        Some(CraftCancelResult {
            recipe_id: recipe.id,
            returned_inputs,
            reason,
        })
    }

    /// Advances crafting time by dt seconds and handles cancellation or completion.
    /// // Godot: CraftingSystem.cs _PhysicsProcess
    pub fn update(
        &mut self,
        dt: f32,
        inventory: &mut Inventory,
        near_lit_campfire: bool,
        is_dead: bool,
    ) -> CraftUpdate {
        let Some(recipe) = self.active.clone() else {
            return CraftUpdate::None;
        };

        if is_dead {
            let res = self
                .cancel_craft(inventory, CancelReason::Dead)
                .expect("active craft exists");
            return CraftUpdate::Cancelled(res);
        }

        if self.active_station == CAMPFIRE_STATION && !near_lit_campfire {
            let res = self
                .cancel_craft(inventory, CancelReason::TooFarFromCampfire)
                .expect("active craft exists");
            return CraftUpdate::Cancelled(res);
        }

        self.elapsed += dt;
        if self.elapsed >= recipe.time {
            let res = self.finish(inventory);
            CraftUpdate::Finished(res)
        } else {
            CraftUpdate::Progress(CraftProgress {
                recipe_id: recipe.id,
                progress: self.progress(),
            })
        }
    }

    /// Finishes the craft, adding outputs to inventory and collecting any overflow or XP.
    /// // Godot: CraftingSystem.cs Finish
    fn finish(&mut self, inventory: &mut Inventory) -> CraftFinishResult {
        let recipe = self.active.take().expect("finish called with active craft");
        let mut outputs_added = Vec::new();
        let mut leftovers = Vec::new();

        for output in &recipe.outputs {
            let leftover = inventory.add(&output.id, output.count);
            let added = output.count.saturating_sub(leftover);
            if added > 0 {
                outputs_added.push(ItemStack::new(&output.id, added));
            }
            if leftover > 0 {
                leftovers.push(ItemStack::new(&output.id, leftover));
            }
        }

        CraftFinishResult {
            recipe_id: recipe.id,
            outputs_added,
            leftovers,
            xp: recipe.xp,
        }
    }
}
