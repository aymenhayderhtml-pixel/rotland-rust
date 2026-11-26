//! Camp upgrade visual models and colliders (walls, gates, towers, workshop, doctor, mechanic).
//!
//! Holds the active prop instances and physics colliders for the camp's upgrade levels.
//! When levels change (upgrade finishes or save loads), rebuilds props via `world::camp_upgrade_props`
//! and updates colliders in `PhysicsWorld`.

use std::sync::RwLock;

use engine::EngineArgs;
use pack::ModelPack;
use physics::{ColliderHandle, PhysicsWorld};
use world::{camp_upgrade_props, CampLevels, PropInstance, SettlementSite};

use crate::props::{Placed, PropLibrary};

static CLI_OVERRIDE: RwLock<Option<CampLevels>> = RwLock::new(None);

/// Parses `--camp-levels=W,G,K,T` (or `W,G,K,T,S`) into `CampLevels`.
pub fn parse_camp_levels(s: &str) -> Option<CampLevels> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 && parts.len() != 5 {
        return None;
    }
    let walls = parts[0].trim().parse::<u8>().ok()?;
    let guards = parts[1].trim().parse::<u8>().ok()?;
    let workshop = parts[2].trim().parse::<u8>().ok()?;
    let traders = parts[3].trim().parse::<u8>().ok()?;
    let gunsmith = match parts.get(4) {
        Some(p) => p.trim().parse::<u8>().ok()?,
        None => 0,
    };
    Some(CampLevels {
        walls,
        guards,
        workshop,
        traders,
        gunsmith,
    })
}

/// Sets the CLI override levels in-memory.
#[allow(dead_code)]
pub fn set_override_levels(levels: Option<CampLevels>) {
    if let Ok(mut lock) = CLI_OVERRIDE.write() {
        *lock = levels;
    }
}

/// Returns the active override levels from CLI args or in-memory override.
pub fn override_levels() -> Option<CampLevels> {
    if let Ok(lock) = CLI_OVERRIDE.read()
        && lock.is_some()
    {
        return *lock;
    }
    for arg in std::env::args() {
        if let Some(val) = arg.strip_prefix("--camp-levels=")
            && let Some(levels) = parse_camp_levels(val)
        {
            return Some(levels);
        }
    }
    None
}

/// Initializes CLI override from engine args if provided.
#[allow(dead_code)]
pub fn init(args: &EngineArgs) {
    if let Some(val) = args.value("--camp-levels") {
        set_override_levels(parse_camp_levels(val));
    }
}

/// Holds the current prop list and placed instances for the camp upgrade levels.
#[derive(Default)]
pub struct CampUpgradeView {
    current_levels: Option<CampLevels>,
    props: Vec<PropInstance>,
    placed: Vec<Placed>,
    colliders: Vec<ColliderHandle>,
}

impl CampUpgradeView {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn props(&self) -> &[PropInstance] {
        &self.props
    }

    #[allow(dead_code)]
    pub fn current_levels(&self) -> Option<CampLevels> {
        self.current_levels
    }

    pub fn placed(&self) -> &[Placed] {
        &self.placed
    }

    #[allow(dead_code)]
    pub fn colliders(&self) -> &[ColliderHandle] {
        &self.colliders
    }

    /// Pure prop synchronization: rebuilds prop list only if levels changed.
    /// Returns true if rebuilt, false if unchanged.
    pub fn sync_props(&mut self, site: &SettlementSite, levels: CampLevels) -> bool {
        if self.current_levels == Some(levels) {
            return false;
        }
        self.current_levels = Some(levels);
        self.props = camp_upgrade_props(site, levels);
        true
    }

    /// Synchronizes physics colliders using a shape lookup function.
    pub fn sync_colliders(
        &mut self,
        physics: &mut PhysicsWorld,
        shapes_for_model: impl Fn(&str) -> Vec<Vec<[f32; 3]>>,
    ) {
        for handle in self.colliders.drain(..) {
            physics.remove(handle);
        }
        for prop in &self.props {
            let shapes = shapes_for_model(&prop.model);
            if !shapes.is_empty() {
                let handle = physics.add_static(&shapes, prop.position, prop.yaw, prop.scale);
                self.colliders.push(handle);
            }
        }
        physics.update_broad_phase();
    }

    /// Clears any physics colliders currently registered.
    #[allow(dead_code)]
    pub fn clear_colliders(&mut self, physics: &mut PhysicsWorld) {
        for handle in self.colliders.drain(..) {
            physics.remove(handle);
        }
        physics.update_broad_phase();
    }

    /// Updates prop list, colliders, and visual placed instances when levels change.
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        pack: &ModelPack,
        library: &mut PropLibrary,
        physics: &mut PhysicsWorld,
        site: &SettlementSite,
        levels: CampLevels,
    ) -> bool {
        if !self.sync_props(site, levels) {
            return false;
        }

        self.sync_colliders(physics, |model| {
            pack.find(model)
                .map(|m| m.colliders.iter().map(|c| c.points.clone()).collect())
                .unwrap_or_default()
        });

        self.placed.clear();
        for prop in &self.props {
            // building = true gives LOD and shadow rules like other settlement buildings
            if let Some(slot) = library.get(device, pack, &prop.model, true) {
                self.placed.push(library.place(slot, prop.position, prop.yaw, prop.scale));
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn dummy_site() -> SettlementSite {
        SettlementSite::new("camp_0", Vec3::new(100.0, 5.0, -100.0), 0.0)
    }

    #[test]
    fn test_level_0_has_no_props() {
        let site = dummy_site();
        let mut view = CampUpgradeView::new();
        let changed = view.sync_props(&site, CampLevels::default());
        assert!(changed);
        assert!(view.props().is_empty(), "level 0 must have no props");
        assert_eq!(view.current_levels(), Some(CampLevels::default()));
    }

    #[test]
    fn test_level_change_produces_new_list() {
        let site = dummy_site();
        let mut view = CampUpgradeView::new();

        // Level 0 -> empty
        view.sync_props(&site, CampLevels::default());
        assert_eq!(view.props().len(), 0);

        // Level 1 walls -> generates walls + gate
        let lvl1 = CampLevels {
            walls: 1,
            ..Default::default()
        };
        let changed = view.sync_props(&site, lvl1);
        assert!(changed);
        assert!(!view.props().is_empty());
        let count_lvl1 = view.props().len();
        assert_eq!(view.current_levels(), Some(lvl1));

        // Level 2 with guards and workshop -> different props list
        let lvl2 = CampLevels {
            walls: 2,
            guards: 1,
            workshop: 1,
            traders: 2,
            gunsmith: 0,
        };
        let changed2 = view.sync_props(&site, lvl2);
        assert!(changed2);
        assert_ne!(view.props().len(), count_lvl1);
        assert_eq!(view.current_levels(), Some(lvl2));
    }

    #[test]
    fn test_same_levels_produce_no_rebuild() {
        let site = dummy_site();
        let mut view = CampUpgradeView::new();

        let lvl = CampLevels {
            walls: 1,
            guards: 1,
            workshop: 0,
            traders: 0,
            gunsmith: 0,
        };
        let changed1 = view.sync_props(&site, lvl);
        assert!(changed1);
        let ptr1 = view.props().as_ptr();

        let changed2 = view.sync_props(&site, lvl);
        assert!(!changed2);
        let ptr2 = view.props().as_ptr();
        assert_eq!(ptr1, ptr2, "props slice should not be reallocated");
    }

    #[test]
    fn test_sync_colliders_and_clear() {
        let site = dummy_site();
        let mut view = CampUpgradeView::new();
        let mut physics = PhysicsWorld::new();

        let cube = vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 2.0, -1.0],
            [-1.0, 2.0, -1.0],
            [-1.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 2.0, 1.0],
            [-1.0, 2.0, 1.0],
        ];

        let lvl = CampLevels {
            walls: 1,
            ..Default::default()
        };
        view.sync_props(&site, lvl);
        view.sync_colliders(&mut physics, |_| vec![cube.clone()]);
        assert_eq!(view.colliders().len(), view.props().len());

        view.clear_colliders(&mut physics);
        assert_eq!(view.colliders().len(), 0);
    }

    #[test]
    fn test_parse_camp_levels() {
        assert_eq!(
            parse_camp_levels("1,2,3,4"),
            Some(CampLevels {
                walls: 1,
                guards: 2,
                workshop: 3,
                traders: 4,
                gunsmith: 0,
            })
        );
        assert_eq!(
            parse_camp_levels("0, 1, 0, 2"),
            Some(CampLevels {
                walls: 0,
                guards: 1,
                workshop: 0,
                traders: 2,
                gunsmith: 0,
            })
        );
        assert_eq!(
            parse_camp_levels("1,2,3,4,2"),
            Some(CampLevels {
                walls: 1,
                guards: 2,
                workshop: 3,
                traders: 4,
                gunsmith: 2,
            })
        );
        assert_eq!(parse_camp_levels("invalid"), None);
        assert_eq!(parse_camp_levels("1,2,3"), None);
        assert_eq!(parse_camp_levels("1,2,3,4,5,6"), None);

        let args = EngineArgs::parse(vec!["--camp-levels=3,2,1,0".to_string()]);
        init(&args);
        assert_eq!(
            override_levels(),
            Some(CampLevels {
                walls: 3,
                guards: 2,
                workshop: 1,
                traders: 0,
                gunsmith: 0,
            })
        );
        set_override_levels(None);
    }
}
