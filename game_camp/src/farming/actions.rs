use glam::Vec3;

use crate::farming::config::FarmConfigData;
use crate::farming::def::{CropDef, CropStage, PlotAction, TillError};
use crate::farming::manager::FarmManager;
use crate::farming::record::PlotRecord;
use crate::farming::rules::FarmRules;
use game_core::items::Inventory;
use game_core::skills::SkillSystem;

impl FarmManager {
    /// Tills a new bed at position on flat ground.
    // Godot: FarmManager.Actions.cs GroundTill
    pub fn till(&mut self, position: Vec3) -> Result<String, TillError> {
        self.till_with_surface(position, Vec3::Y, None, None)
    }

    /// Tills a new bed with explicit ground normal, optional hoe check in inventory, and XP reward.
    // Godot: FarmManager.Actions.cs GroundTill
    pub fn till_with_surface(
        &mut self,
        position: Vec3,
        normal: Vec3,
        inventory: Option<&Inventory>,
        mut skills: Option<&mut SkillSystem>,
    ) -> Result<String, TillError> {
        if let Some(bag) = inventory
            && !bag.has("hoe", 1)
        {
            return Err(TillError::NoHoe);
        }
        if !FarmRules::is_flat(normal) {
            return Err(TillError::GroundTooSteep);
        }
        let others = self.plots.iter().map(|p| &p.position);
        if !FarmRules::spot_is_free(position, others, self.config.till_spacing) {
            return Err(TillError::TooCloseToAnotherBed);
        }

        let id = self.new_id("bed");
        self.add_plot(PlotRecord::new(id.clone(), position));

        if let Some(ref mut sk) = skills {
            sk.add_xp("farming", self.config.xp.till);
        }

        Ok(id)
    }

    /// The first crop in config whose seed item the bag carries, or None.
    // Godot: FarmManager.Actions.cs SeedCropIn
    pub fn seed_crop_in<'a>(cfg: &'a FarmConfigData, inventory: &Inventory) -> Option<&'a CropDef> {
        cfg.crops.iter().find(|crop| inventory.has(&crop.seed, 1))
    }

    /// The action E does at this bed for this inventory state.
    // Godot: FarmManager.Actions.cs ActionFor
    pub fn action_for(&self, plot: &PlotRecord, inventory: &Inventory) -> PlotAction {
        let Some(crop_id) = &plot.crop else {
            return if Self::seed_crop_in(&self.config, inventory).is_some() {
                PlotAction::Plant
            } else {
                PlotAction::None
            };
        };
        if plot.dead {
            return PlotAction::Clear;
        }
        if let Some(crop) = self.config.try_crop(crop_id)
            && FarmRules::stage_of(plot.growth, crop.total_hours()) == CropStage::Ripe
        {
            return PlotAction::Harvest;
        }
        if inventory.has("watering_can", 1) && self.charges > 0 {
            PlotAction::Water
        } else {
            PlotAction::None
        }
    }

    /// Prompt text displayed for the current action at this plot.
    // Godot: FarmManager.Actions.cs PromptFor
    pub fn prompt_for(&self, plot: &PlotRecord, inventory: &Inventory) -> String {
        match self.action_for(plot, inventory) {
            PlotAction::Plant => {
                let name = Self::seed_crop_in(&self.config, inventory)
                    .map_or("crop", |c| c.name.as_str());
                format!("Plant {name} seeds")
            }
            PlotAction::Water => "Water crop".to_string(),
            PlotAction::Harvest => {
                let name = plot
                    .crop
                    .as_deref()
                    .and_then(|id| self.config.try_crop(id))
                    .map_or("crop", |c| c.name.as_str());
                format!("Harvest {name}")
            }
            PlotAction::Clear => "Clear dead crop".to_string(),
            PlotAction::None => String::new(),
        }
    }

    /// Plants seeds into a tilled bed.
    pub fn plant(
        &mut self,
        plot_id: &str,
        inventory: &mut Inventory,
        mut skills: Option<&mut SkillSystem>,
    ) -> Result<String, String> {
        let crop = Self::seed_crop_in(&self.config, inventory)
            .cloned()
            .ok_or_else(|| "No seeds in inventory".to_string())?;

        let plot = self
            .find_mut(plot_id)
            .ok_or_else(|| "Plot not found".to_string())?;

        if plot.crop.is_some() {
            return Err("Plot already has a crop".to_string());
        }

        if !inventory.remove(&crop.seed, 1) {
            return Err("Failed to consume seeds".to_string());
        }

        plot.crop = Some(crop.id.clone());
        plot.growth = 0.0;
        plot.dry_hours = 0.0;
        plot.watered_at = -1.0;
        plot.dead = false;

        if let Some(ref mut sk) = skills {
            sk.add_xp("farming", self.config.xp.plant);
        }

        Ok(crop.id)
    }

    /// Waters a bed, consuming one charge from the can.
    pub fn water(
        &mut self,
        plot_id: &str,
        mut skills: Option<&mut SkillSystem>,
    ) -> Result<(), String> {
        if self.charges == 0 {
            return Err("Watering can is empty".to_string());
        }
        let synced = self.synced_hours;
        if self.find(plot_id).is_none() {
            return Err("Plot not found".to_string());
        }
        self.charges -= 1;
        let plot = self
            .find_mut(plot_id)
            .ok_or_else(|| "Plot not found".to_string())?;
        plot.watered_at = if synced >= 0.0 { synced } else { 0.0 };
        plot.dry_hours = 0.0;

        if let Some(ref mut sk) = skills {
            sk.add_xp("farming", self.config.xp.water);
        }

        Ok(())
    }

    /// Harvests a ripe crop, rolling seed back and granting XP.
    // Godot: FarmManager.Actions.cs Harvest
    pub fn harvest<F>(
        &mut self,
        plot_id: &str,
        inventory: &mut Inventory,
        mut skills: Option<&mut SkillSystem>,
        mut rng: F,
    ) -> Result<u32, String>
    where
        F: FnMut() -> f64,
    {
        let (crop_id, growth) = {
            let plot = self
                .find(plot_id)
                .ok_or_else(|| "Plot not found".to_string())?;
            let cid = plot.crop.clone().ok_or_else(|| "No crop on plot".to_string())?;
            (cid, plot.growth)
        };

        let crop = self
            .config
            .try_crop(&crop_id)
            .cloned()
            .ok_or_else(|| "Unknown crop".to_string())?;

        if FarmRules::stage_of(growth, crop.total_hours()) != CropStage::Ripe {
            return Err("Crop is not ripe yet".to_string());
        }

        let level = skills.as_ref().map_or(1, |s| s.get_level("farming"));
        let count = FarmRules::yield_for(crop.yield_count, level, &self.config.yield_bonus_levels);

        if !inventory.can_add(&crop.product, count) {
            return Err("No room in the bag".to_string());
        }

        inventory.add(&crop.product, count);

        let roll = rng();
        if FarmRules::passes(roll, self.config.seed_back_chance as f64)
            && inventory.can_add(&crop.seed, 1)
        {
            inventory.add(&crop.seed, 1);
        }

        if let Some(plot) = self.find_mut(plot_id) {
            plot.clear();
        }

        if let Some(ref mut sk) = skills {
            sk.add_xp("farming", self.config.xp.harvest);
        }

        Ok(count)
    }

    /// Clears a dead or empty bed.
    pub fn clear(&mut self, plot_id: &str) -> Result<(), String> {
        let plot = self
            .find_mut(plot_id)
            .ok_or_else(|| "Plot not found".to_string())?;
        plot.clear();
        Ok(())
    }

    /// Performs the appropriate action on a plot using the player's inventory and internal RNG.
    // Godot: FarmManager.Actions.cs Act
    pub fn act(
        &mut self,
        plot_id: &str,
        inventory: &mut Inventory,
        skills: Option<&mut SkillSystem>,
    ) -> Result<(), String> {
        // The RNG is lent out for the call, so `self` stays free to borrow
        let mut rng = std::mem::replace(&mut self.rng, world::DotnetRandom::new(0));
        let result = self.act_with_rng(plot_id, inventory, skills, || rng.next_double());
        self.rng = rng;
        result
    }

    /// Performs the appropriate action on a plot with an explicit RNG closure.
    // Godot: FarmManager.Actions.cs Act
    pub fn act_with_rng<F>(
        &mut self,
        plot_id: &str,
        inventory: &mut Inventory,
        skills: Option<&mut SkillSystem>,
        rng: F,
    ) -> Result<(), String>
    where
        F: FnMut() -> f64,
    {
        let plot = self
            .find(plot_id)
            .ok_or_else(|| "Plot not found".to_string())?;
        let action = self.action_for(plot, inventory);

        match action {
            PlotAction::Plant => {
                self.plant(plot_id, inventory, skills)?;
                Ok(())
            }
            PlotAction::Water => self.water(plot_id, skills),
            PlotAction::Harvest => {
                self.harvest(plot_id, inventory, skills, rng)?;
                Ok(())
            }
            PlotAction::Clear => self.clear(plot_id),
            PlotAction::None => Err("Nothing to do here".to_string()),
        }
    }

    /// Refills the watering can to full using one water bottle from the inventory.
    // Godot: FarmManager.Actions.cs RefillCan
    pub fn refill_can(&mut self, inventory: &mut Inventory) -> bool {
        if !inventory.remove("water_bottle", 1) {
            return false;
        }
        self.charges = self.config.can_capacity;
        true
    }
}
