use world::DotnetRandom;

use crate::bounties::config::{BountyConfigData, BountyTemplateData, CLEAR_POI};
use crate::bounties::def::BountyDef;
use game_core::loot::crate_seed;

/// Pure rules for the board: which bounty day it is, and what the daily and weekly lists hold.
/// Every roll is seeded by the world seed and the day (or week), so the same world offers the
/// same bounties every run.
// Godot: BountyRoller.cs
pub struct BountyRoller;

impl BountyRoller {
    /// The bounty day in force: a new one starts at the reset hour, so 02:00 still belongs to yesterday.
    // Godot: BountyRoller.cs BountyDay
    pub fn bounty_day(day: i32, hour: f32, cfg: &BountyConfigData) -> i32 {
        if hour >= cfg.reset_hour as f32 {
            day
        } else {
            day - 1
        }
    }

    // Godot: BountyRoller.cs WeekOf
    pub fn week_of(bounty_day: i32, cfg: &BountyConfigData) -> i32 {
        bounty_day.max(0) / cfg.weekly_days
    }

    /// The last bounty day of a week. Weekly bounties are lost after it.
    // Godot: BountyRoller.cs WeekEnd
    pub fn week_end(week: i32, cfg: &BountyConfigData) -> i32 {
        (week + 1) * cfg.weekly_days - 1
    }

    /// Deterministically rolls the daily bounties offered for `bounty_day`.
    // Godot: BountyRoller.cs RollDaily
    pub fn roll_daily(seed: i32, bounty_day: i32, cfg: &BountyConfigData) -> Vec<BountyDef> {
        let mut offered = Vec::with_capacity(cfg.daily_count as usize);
        for i in 0..cfg.daily_count {
            let roll_seed = crate_seed(seed, &format!("bounty_daily_{bounty_day}_{i}"));
            let mut rng = DotnetRandom::new(roll_seed);
            let template_idx = rng.next(cfg.daily.len() as i32) as usize;
            let template = &cfg.daily[template_idx];
            offered.push(Self::make(
                template,
                &mut rng,
                format!("d{bounty_day}_{i}"),
                bounty_day,
                false,
            ));
        }
        offered
    }

    /// Deterministically rolls the weekly bounty offered for `week`.
    // Godot: BountyRoller.cs RollWeekly
    pub fn roll_weekly(seed: i32, week: i32, cfg: &BountyConfigData) -> BountyDef {
        let roll_seed = crate_seed(seed, &format!("bounty_weekly_{week}"));
        let mut rng = DotnetRandom::new(roll_seed);
        let template_idx = rng.next(cfg.weekly.len() as i32) as usize;
        let template = &cfg.weekly[template_idx];
        Self::make(
            template,
            &mut rng,
            format!("w{week}"),
            Self::week_end(week, cfg),
            true,
        )
    }

    // Godot: BountyRoller.cs Make
    fn make(
        t: &BountyTemplateData,
        rng: &mut DotnetRandom,
        id: String,
        expires_on: i32,
        weekly: bool,
    ) -> BountyDef {
        let count = rng.next_range(t.count[0], t.count[1] + 1);
        let building = if !t.buildings.is_empty() {
            let b_idx = rng.next(t.buildings.len() as i32) as usize;
            Some(t.buildings[b_idx].clone())
        } else {
            None
        };
        let dollars = rng.next_range(t.dollars[0], t.dollars[1] + 1);
        let xp = rng.next_range(t.xp[0], t.xp[1] + 1);
        let what = if let Some(b) = &building {
            b.replace('_', " ")
        } else if let Some(item) = &t.item {
            item.replace('_', " ")
        } else if let Some(z) = &t.zombie {
            z.clone()
        } else {
            "zombie".to_string()
        };
        let text = t
            .text
            .replace("{n}", &count.to_string())
            .replace("{what}", &what);
        let target = if t.bounty_type == CLEAR_POI { 1 } else { count };

        BountyDef {
            id,
            bounty_type: t.bounty_type.clone(),
            text,
            target,
            progress: 0,
            dollars,
            xp,
            skill: t.skill.clone(),
            zombie: t.zombie.clone(),
            animal: t.animal.clone(),
            item: t.item.clone(),
            building,
            poi_id: None,
            target_x: 0.0,
            target_z: 0.0,
            expires_on,
            weekly,
        }
    }
}
