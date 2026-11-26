//! Pure tower part assembly per chunk: turns `TowerSpot`s into `TowerPart`s (base, stacked
//! floors, setbacks, crown, lobby interior) ready for instanced placement.

use glam::IVec2;
use world::WorldGenerator;
use world::metro::{
    DamageMode, DamageSites, MetroConfig, StyleDef, TowerPart, TowerSpot, style_for,
    tower_parts_with_damage, tower_parts_with_extras,
};

use super::boxes::spot_seed;
use super::near::towers_in_chunk;

/// Visibility range for loaded-chunk tower parts: covers every loaded chunk up to `UNLOAD_RADIUS`
/// diagonally so no tower disappears before the far skyline takes over.
pub fn tower_load_ring_range() -> f32 {
    (crate::streaming::UNLOAD_RADIUS as f32 + 1.0)
        * WorldGenerator::CHUNK_SIZE
        * std::f32::consts::SQRT_2
}

/// Index of the tallest landmark tower in `towers`, if any.
pub fn tallest_landmark_index(towers: &[TowerSpot]) -> Option<usize> {
    towers
        .iter()
        .enumerate()
        .filter(|(_, t)| t.landmark)
        .max_by(|(_, a), (_, b)| a.height.total_cmp(&b.height))
        .map(|(i, _)| i)
}

/// Base model id (`tw_<style>_<size>_base`) for `spot`'s walk-in lobby shell.
pub fn base_model_id(spot: &TowerSpot) -> String {
    format!("tw_{}_{}_base", spot.style.key(), spot.size)
}

/// Collects every `TowerPart` for all towers whose centre stands in chunk `coord`.
/// Replaced towers (fallen / leaning set pieces) are skipped.
/// Pure and deterministic for the same inputs.
pub fn chunk_tower_parts(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    styles: &[StyleDef],
    cfg: &MetroConfig,
    damage: &DamageSites,
) -> Vec<TowerPart> {
    let tallest_idx = tallest_landmark_index(towers);
    let mut out = Vec::new();
    for (idx, t) in towers_in_chunk(towers, coord)
        .filter_map(|t| {
            let pos_idx = towers.iter().position(|x| std::ptr::eq(x, t))?;
            Some((pos_idx, t))
        })
    {
        if damage.tower_is_replaced(idx) {
            continue; // R40-08: replaced by fallen/leaning set piece
        }
        let Some(style) = style_for(styles, t.style).or_else(|| styles.first()) else {
            continue;
        };
        let parts = if damage.tower_is_burnt(t.block) {
            tower_parts_with_damage(t, style, ground_y, u64::from(spot_seed(t)), DamageMode::All)
        } else {
            tower_parts_with_extras(
                t,
                style,
                ground_y,
                u64::from(spot_seed(t)),
                &cfg.towers.setbacks,
                &cfg.towers.lobby_props,
                tallest_idx == Some(idx),
            )
        };
        out.extend(parts);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use world::metro::{TowerKind, TowerStyle, default_styles};

    fn test_spot(pos: Vec2, floors: u32, landmark: bool) -> TowerSpot {
        TowerSpot {
            pos,
            half: Vec2::splat(12.0),
            yaw: 0.5,
            height: 7.6 + floors as f32 * 3.8 + 5.0,
            floors,
            kind: if landmark { TowerKind::Landmark } else { TowerKind::Highrise },
            landmark,
            block: (0, 0),
            style: if landmark { TowerStyle::Deco } else { TowerStyle::Stone },
            size: 24,
        }
    }

    #[test]
    fn tallest_landmark_selects_highest_landmark_only() {
        let towers = vec![
            test_spot(Vec2::new(10.0, 10.0), 30, false),
            test_spot(Vec2::new(40.0, 10.0), 38, true),
            test_spot(Vec2::new(80.0, 10.0), 42, true),
        ];
        assert_eq!(tallest_landmark_index(&towers), Some(2));
        assert_eq!(tallest_landmark_index(&towers[..1]), None);
    }

    #[test]
    fn base_model_id_matches_style_and_size() {
        let t = test_spot(Vec2::ZERO, 12, false);
        assert_eq!(base_model_id(&t), "tw_stone_24_base");
    }

    #[test]
    fn chunk_tower_parts_filters_by_chunk_and_gives_mast_to_tallest() {
        let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
            .expect("metro.json loads");
        let styles = default_styles();
        let towers = vec![
            test_spot(Vec2::new(10.0, 10.0), 38, true),
            test_spot(Vec2::new(20.0, 20.0), 42, true),
            test_spot(Vec2::new(200.0, 10.0), 20, false),
        ];
        let no_damage = DamageSites {
            burnt_block: (999, 999),
            ..DamageSites::default()
        };
        let parts = chunk_tower_parts(&towers, IVec2::ZERO, 20.0, styles, &cfg, &no_damage);
        assert!(parts.iter().any(|p| p.model == "tw_landmark_mast"));
        assert!(parts.iter().all(|p| !p.model.starts_with("tw_stone_")));
        assert!(chunk_tower_parts(&towers, IVec2::new(99, 99), 20.0, styles, &cfg, &no_damage).is_empty());

        let mut damage = no_damage;
        damage.replaced_towers.push(0);
        damage.replaced_towers.push(1);
        assert!(chunk_tower_parts(&towers, IVec2::ZERO, 20.0, styles, &cfg, &damage).is_empty());
    }
}
