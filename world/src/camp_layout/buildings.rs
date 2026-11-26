//! Placement logic for internal camp buildings (towers, workshop, doctor, mechanic).

use crate::scatter::PropInstance;
use crate::settlement::SettlementSite;
use super::types::{CampLayoutConfigData, CampLevels};

pub(crate) fn place_buildings(
    site: &SettlementSite,
    levels: CampLevels,
    cfg: &CampLayoutConfigData,
    out: &mut Vec<PropInstance>,
) {
    place_towers(site, levels.guards, cfg, out);
    place_workshop(site, levels.workshop, cfg, out);
    place_traders(site, levels.traders, cfg, out);
}

fn place_towers(
    site: &SettlementSite,
    guards_level: u8,
    cfg: &CampLayoutConfigData,
    out: &mut Vec<PropInstance>,
) {
    if guards_level == 0 {
        return;
    }

    let (model, corner_indices): (&str, Vec<usize>) = match guards_level {
        1 => {
            let idxs = cfg
                .guard_tower_corners
                .get("1")
                .cloned()
                .unwrap_or_else(|| vec![1]);
            ("camp_tower_wood", idxs)
        }
        2 => {
            let idxs = cfg
                .guard_tower_corners
                .get("2")
                .cloned()
                .unwrap_or_else(|| vec![1]);
            ("camp_tower_wood", idxs)
        }
        _ => {
            let idxs = cfg
                .guard_tower_corners
                .get("3")
                .cloned()
                .unwrap_or_else(|| vec![1, 3]);
            ("camp_tower_metal", idxs)
        }
    };

    for idx in corner_indices {
        if let Some(corner) = cfg.tower_corners.get(idx) {
            let pos = site.center + site.local(corner.x, 0.0, corner.z);
            let yaw = site.yaw + corner.yaw;
            out.push(PropInstance::new(model, pos, yaw, 1.0));
        }
    }
}

fn place_workshop(
    site: &SettlementSite,
    workshop_level: u8,
    cfg: &CampLayoutConfigData,
    out: &mut Vec<PropInstance>,
) {
    let model = match workshop_level {
        0 => return,
        1 => "camp_workshop_1",
        2 => "camp_workshop_2",
        _ => "camp_workshop_3",
    };

    let pos = site.center + site.local(cfg.workshop_offset[0], 0.0, cfg.workshop_offset[1]);
    let yaw = site.yaw + cfg.workshop_yaw;
    out.push(PropInstance::new(model, pos, yaw, 1.0));
}

fn place_traders(
    site: &SettlementSite,
    traders_level: u8,
    cfg: &CampLayoutConfigData,
    out: &mut Vec<PropInstance>,
) {
    if traders_level >= 2 {
        let doc_pos = site.center + site.local(cfg.doctor_offset[0], 0.0, cfg.doctor_offset[1]);
        let doc_yaw = site.yaw + cfg.doctor_yaw;
        out.push(PropInstance::new("camp_doctor", doc_pos, doc_yaw, 1.0));
    }

    if traders_level >= 3 {
        let mech_pos = site.center + site.local(cfg.mechanic_offset[0], 0.0, cfg.mechanic_offset[1]);
        let mech_yaw = site.yaw + cfg.mechanic_yaw;
        out.push(PropInstance::new("camp_mechanic", mech_pos, mech_yaw, 1.0));
    }
}

/// Returns the camp gunsmith workbench prop inside the settlement (present at all gunsmith tiers, 0..=3).
#[allow(dead_code)] // re-exported in camp_layout/mod.rs and lib.rs by Lead
pub fn camp_gunsmith_prop(site: &SettlementSite) -> PropInstance {
    camp_gunsmith_prop_with_config(site, &CampLayoutConfigData::load_default())
}

/// Returns the camp gunsmith workbench prop using a specific layout configuration.
#[allow(dead_code)] // re-exported in camp_layout/mod.rs and lib.rs by Lead
pub fn camp_gunsmith_prop_with_config(
    site: &SettlementSite,
    cfg: &CampLayoutConfigData,
) -> PropInstance {
    let pos = site.center + site.local(cfg.gunsmith_offset[0], 0.0, cfg.gunsmith_offset[1]);
    let yaw = site.yaw + cfg.gunsmith_yaw;
    PropInstance::new("gunsmith_bench", pos, yaw, 1.0)
}
