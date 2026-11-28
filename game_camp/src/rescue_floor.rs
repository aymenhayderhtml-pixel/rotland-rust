//! Rescue survivors waiting on climb-tower stop floors (R44). Pure: same input, same output.

use std::collections::HashSet;

use glam::Vec3;

use crate::guild::RecruitDef;
use crate::rescue::RescueConfig;
use crate::rescue_place::{parse_recruit_role, RescueSpot};

/// POI id prefix of climb-tower stop floors (`metro_floor_<tower>_<i>`).
const CLIMB_FLOOR_PREFIX: &str = "metro_floor_"; // new: DECISIONS 2026-11-28 (climb floor POI id prefix, R43-01)

/// A stop-floor POI where a survivor can wait.
#[derive(Clone, Debug)]
pub struct FloorRescueSeed {
    pub poi_id: String,
    pub building_id: String,
    pub position: Vec3,
    pub yaw: f32,
}

/// True if `poi_id` names a climb-tower stop floor.
pub fn is_climb_floor_poi(poi_id: &str) -> bool {
    poi_id.starts_with(CLIMB_FLOOR_PREFIX)
}

/// Builds one rescue spot per seed, up to `cfg.floor_survivors`.
///
/// Names come from `recruits` in order, skipping names already used. When every name is used,
/// the templates are reused from the start. With no recruits, the same fallback as
/// `place_survivors` is used.
pub fn floor_rescue_spots(
    seeds: &[FloorRescueSeed],
    cfg: &RescueConfig,
    recruits: &[RecruitDef],
) -> Vec<RescueSpot> {
    let fallback = [RecruitDef {
        id: "rescue_default".to_string(),
        name: "Alex Mercer".to_string(),
        role: "fighter".to_string(),
        level: 1,
        daily_wage: 15, // data: guild.json daily_wage
        health: 100,
        hire_cost: 0,
    }];
    let templates: &[RecruitDef] = if recruits.is_empty() { &fallback } else { recruits };

    let count = cfg.floor_survivors.min(seeds.len());
    let mut used: HashSet<&str> = HashSet::new();
    let mut spots = Vec::with_capacity(count);
    for (i, seed) in seeds.iter().take(count).enumerate() {
        let tmpl = templates
            .iter()
            .find(|r| !used.contains(r.name.as_str()))
            .unwrap_or(&templates[i % templates.len()]);
        used.insert(tmpl.name.as_str());
        spots.push(RescueSpot {
            id: format!("rescue_floor_{}", i + 1),
            poi_id: seed.poi_id.clone(),
            building_id: seed.building_id.clone(),
            name: tmpl.name.clone(),
            role: parse_recruit_role(&tmpl.role),
            level: tmpl.level,
            daily_wage: tmpl.daily_wage,
            max_health: tmpl.health,
            position: seed.position,
            yaw: seed.yaw,
        });
    }
    spots
}
