use std::collections::HashMap;
use world::DotnetRandom;

use crate::farming::config::{FarmConfig, FarmConfigData, FarmLoadError};
use crate::farming::def::PlotView;
use crate::farming::record::PlotRecord;
use crate::farming::rules::FarmRules;
use crate::farming::save;
use game_core::skills::SkillSystem;

/// Owns every bed, the growth clock, charges, and farming rules.
/// Port of `scripts/Farming/FarmManager.cs`.
// Godot: FarmManager.cs
#[derive(Clone, Debug)]
pub struct FarmManager {
    pub(crate) config: FarmConfigData,
    pub(crate) plots: Vec<PlotRecord>,
    pub(crate) charges: u32,
    pub(crate) synced_hours: f64,
    pub(crate) next_id: u32,
    pub(crate) camp_checked: bool,
    pub(crate) rng: DotnetRandom,
    pub(crate) on_bed: HashMap<u64, String>,
}

impl FarmManager {
    pub const SAVE_ID: &'static str = save::SAVE_ID;

    /// Creates a new FarmManager with the given configuration.
    pub fn new(config: FarmConfigData) -> Self {
        let cap = config.can_capacity;
        Self {
            config,
            plots: Vec::new(),
            charges: cap,
            synced_hours: -1.0,
            next_id: 1,
            camp_checked: false,
            rng: DotnetRandom::new(12345),
            on_bed: HashMap::new(),
        }
    }

    /// Loads configuration from data/farming.json in the repo.
    pub fn from_repo() -> Result<Self, FarmLoadError> {
        let config = FarmConfig::from_repo()?;
        Ok(Self::new(config))
    }

    /// Creates a new FarmManager with fixed random seed.
    pub fn with_seed(config: FarmConfigData, seed: i32) -> Self {
        let mut m = Self::new(config);
        m.set_rng_seed(seed);
        m
    }

    pub fn set_rng_seed(&mut self, seed: i32) {
        self.rng = DotnetRandom::new(seed);
    }

    #[allow(dead_code)]
    fn set_rng(&mut self, rng: DotnetRandom) {
        self.rng = rng;
    }

    pub fn plots(&self) -> &[PlotRecord] {
        &self.plots
    }

    #[allow(dead_code)]
    fn plots_mut(&mut self) -> &mut [PlotRecord] {
        &mut self.plots
    }

    pub fn charges(&self) -> u32 {
        self.charges
    }

    #[allow(dead_code)]
    fn set_charges(&mut self, charges: u32) {
        self.charges = charges.min(self.config.can_capacity);
    }

    pub fn synced_hours(&self) -> f64 {
        self.synced_hours
    }

    pub fn config(&self) -> &FarmConfigData {
        &self.config
    }

    /// Finds a plot by id.
    // Godot: FarmManager.cs Find
    pub fn find(&self, id: &str) -> Option<&PlotRecord> {
        self.plots.iter().find(|p| p.id == id)
    }

    /// Finds a plot by id (mutable).
    pub fn find_mut(&mut self, id: &str) -> Option<&mut PlotRecord> {
        self.plots.iter_mut().find(|p| p.id == id)
    }

    /// Adds a plot to the manager.
    // Godot: FarmManager.cs AddPlot
    pub fn add_plot(&mut self, plot: PlotRecord) {
        self.plots.push(plot);
    }

    /// Generates a fresh unique plot id.
    // Godot: FarmManager.cs NewId
    pub fn new_id(&mut self, prefix: &str) -> String {
        let id = format!("{}_{}", prefix, self.next_id);
        self.next_id += 1;
        id
    }

    /// Creates the camp's fixed four beds once per camp site.
    // Godot: FarmManager.cs EnsureCampPlots
    pub fn ensure_camp_plots(&mut self, sites: &[world::SettlementSite]) {
        if self.camp_checked || sites.is_empty() {
            return;
        }
        self.camp_checked = true;
        let spots = self.config.camp_plots.clone();
        for site in sites {
            for (i, spot) in spots.iter().enumerate() {
                let id = format!("{}_plot_{}", site.id, i + 1);
                if self.find(&id).is_some() {
                    continue;
                }
                let at = site.center + site.local(spot[0], 0.0, spot[1]);
                let mut plot = PlotRecord::new(id, at);
                plot.camp = true;
                self.add_plot(plot);
            }
        }
    }

    /// Calculates absolute in-game hours from day and hour (day 1 at 00:00 = 0).
    // Godot: FarmManager.cs NowHours
    pub fn now_hours(day: i32, hour: f32) -> f64 {
        (day as f64 - 1.0) * 24.0 + (hour as f64)
    }

    /// Brings every bed up to current in-game hour, one whole hour at a time.
    // Godot: FarmManager.cs Sync
    pub fn sync(&mut self, now_hours: f64, growth_bonus: f32) {
        if now_hours < 0.0 {
            return;
        }
        if self.synced_hours < 0.0 || now_hours < self.synced_hours {
            self.synced_hours = now_hours.floor();
            return;
        }
        if now_hours < self.synced_hours + 1.0 {
            return;
        }
        let rate = 1.0 + (growth_bonus as f64);
        while self.synced_hours + 1.0 <= now_hours {
            self.synced_hours += 1.0;
            let current_hour = self.synced_hours;
            for plot in &mut self.plots {
                Self::step_plot_hour(plot, current_hour, rate, &self.config);
            }
        }
    }

    /// Syncs growth using the game clock and player's farming skills.
    pub fn sync_clock(&mut self, clock: &world::time::Clock, skills: Option<&SkillSystem>) {
        let now = Self::now_hours(clock.day, clock.hour);
        let bonus = skills.map_or(0.0, |s| s.get_bonus("cropGrowth"));
        self.sync(now, bonus);
    }

    /// Syncs growth using day, hour, and explicit growth bonus.
    #[allow(dead_code)]
    fn sync_day_hour(&mut self, day: i32, hour: f32, growth_bonus: f32) {
        let now = Self::now_hours(day, hour);
        self.sync(now, growth_bonus);
    }

    /// Steps a single hour for all plots directly.
    pub fn step_hour(&mut self, hour: f64, rate: f64) {
        for plot in &mut self.plots {
            Self::step_plot_hour(plot, hour, rate, &self.config);
        }
    }

    /// One in-game hour for one bed: watered grows, unwatered dries and eventually dies.
    // Godot: FarmManager.cs StepHour
    pub(crate) fn step_plot_hour(
        plot: &mut PlotRecord,
        hour: f64,
        rate: f64,
        cfg: &FarmConfigData,
    ) {
        let Some(crop_id) = &plot.crop else {
            return;
        };
        if plot.dead {
            return;
        }
        let Some(crop) = cfg.try_crop(crop_id) else {
            return;
        };
        if FarmRules::is_watered(hour, plot.watered_at, cfg.water_window_hours as f64) {
            plot.dry_hours = 0.0;
            plot.growth = (plot.growth + rate).min(crop.total_hours());
        } else {
            plot.dry_hours += 1.0;
            if plot.dry_hours >= cfg.dry_hours_to_die as f64 {
                plot.dead = true;
            }
        }
    }

    /// Output view of one plot for visual rendering.
    pub fn plot_view(&self, plot: &PlotRecord) -> PlotView {
        let (crop_id, stage_idx, stage_model) = match &plot.crop {
            Some(cid) => match self.config.try_crop(cid) {
                Some(c) => {
                    let st = FarmRules::stage_of(plot.growth, c.total_hours());
                    (Some(cid.clone()), Some(st.index()), Some(c.stage_model(st)))
                }
                None => (Some(cid.clone()), None, None),
            },
            None => (None, None, None),
        };
        let watered = self.synced_hours >= 0.0
            && plot.is_watered(self.synced_hours, self.config.water_window_hours as f64);

        PlotView {
            id: plot.id.clone(),
            position: plot.position,
            camp: plot.camp,
            crop_id,
            stage_index: stage_idx,
            stage_model,
            watered,
            dead: plot.dead,
        }
    }

    /// Output views of all plots.
    pub fn plot_views(&self) -> Vec<PlotView> {
        self.plots.iter().map(|p| self.plot_view(p)).collect()
    }

    /// Serializes farming state to Godot save dictionary format.
    // Godot: FarmManager.Save.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        save::to_save_manager(self)
    }

    /// Deserializes farming state from Godot save dictionary format.
    // Godot: FarmManager.Save.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) -> Result<(), String> {
        save::from_save_manager(self, data)
    }
}
