use std::collections::HashSet;
use glam::Vec3;

use crate::farming::manager::FarmManager;
use crate::farming::record::PlotRecord;
use crate::farming::rules::FarmRules;

impl FarmManager {
    /// True when any field bed has a planted crop (camp beds are immune).
    // Godot: FarmManager.Trample.cs HasVulnerableCrops
    pub fn has_vulnerable_crops(&self) -> bool {
        self.plots.iter().any(|plot| !plot.camp && plot.crop.is_some())
    }

    /// The bed whose square (half-size metres each way in XZ) holds this point, or None.
    // Godot: FarmManager.Trample.cs BedUnder
    pub fn bed_under(&self, point: Vec3, half: f32) -> Option<&PlotRecord> {
        self.plots.iter().find(|plot| {
            (point.x - plot.position.x).abs() <= half && (point.z - plot.position.z).abs() <= half
        })
    }

    /// Checks moving entities (zombies/animals) over plots. Returns list of trampled plot IDs.
    /// Each entity that walks onto a field bed rolls once (tramplePerPass) for its crop.
    // Godot: FarmManager.Trample.cs TrampleCheck
    pub fn trample_check<F>(
        &mut self,
        walkers: impl IntoIterator<Item = (u64, Vec3)>,
        mut rng: F,
    ) -> Vec<String>
    where
        F: FnMut() -> f64,
    {
        if !self.has_vulnerable_crops() {
            if !self.on_bed.is_empty() {
                self.on_bed.clear();
            }
            return Vec::new();
        }

        let half = self.config.trample_half_size;
        let trample_chance = self.config.trample_per_pass as f64;
        let mut seen = HashSet::new();
        let mut trampled = Vec::new();

        for (id, pos) in walkers {
            seen.insert(id);
            let bed_info = self.bed_under(pos, half).map(|b| (b.id.clone(), b.camp, b.crop.is_some()));
            let Some((bed_id, is_camp, has_crop)) = bed_info else {
                self.on_bed.remove(&id);
                continue;
            };

            if let Some(current) = self.on_bed.get(&id)
                && current == &bed_id
            {
                continue;
            }

            self.on_bed.insert(id, bed_id.clone());

            if is_camp || !has_crop {
                continue;
            }

            if FarmRules::passes(rng(), trample_chance) {
                if let Some(plot) = self.find_mut(&bed_id) {
                    plot.clear();
                }
                trampled.push(bed_id);
            }
        }

        self.on_bed.retain(|id, _| seen.contains(id));
        trampled
    }

    /// Trample check using manager's internal RNG.
    pub fn trample_step(&mut self, walkers: impl IntoIterator<Item = (u64, Vec3)>) -> Vec<String> {
        // The RNG is lent out for the call, so `self` stays free to borrow
        let mut rng = std::mem::replace(&mut self.rng, world::DotnetRandom::new(0));
        let result = self.trample_check(walkers, || rng.next_double());
        self.rng = rng;
        result
    }
}
