//! Which props get which visibility range, and which ones cast a shadow (ScatterConfig.RangeFor,
//! ChunkProps.ShouldCastShadow).

/// Distances from data/scatter.json "visibility".
pub(super) const TREE: f32 = 170.0; // data: scatter.json visibility.tree
const ROCK: f32 = 80.0; // data: scatter.json visibility.rock
const BUSH: f32 = 65.0; // data: scatter.json visibility.bush
const GROUND: f32 = 35.0; // data: scatter.json visibility.ground
const PROP: f32 = 120.0; // data: scatter.json visibility.prop

pub(super) fn is_ground(id: &str) -> bool {
    [
        "grass_",
        "flowers_",
        "fern",
        "mushrooms",
        "reeds",
        "lily_mud_patch",
        "desert_shrub",
        "snow_shrub",
        "tumbleweed",
        "bleached_bones",
        "snow_bush_small",
        "ice_shards",
        "cattails",
        "swamp_moss_clump",
        "scree_patch",
        "alpine_flowers",
    ]
    .iter()
    .any(|p| id.starts_with(p))
}

fn is_big_rock(id: &str) -> bool {
    id == "boulder_big" || id == "rock_cliff_01"
}

pub(super) fn range_for(id: &str) -> f32 {
    let starts = |p: &str| id.starts_with(p);
    if starts("tree_") || starts("pine_") || is_big_rock(id) {
        TREE
    } else if starts("rock_") {
        ROCK
    } else if starts("bush_") || starts("cactus_") {
        BUSH
    } else if is_ground(id) {
        GROUND
    } else {
        PROP
    }
}

fn is_no_shadow_metro_prop(id: &str) -> bool {
    [
        "weeds_",
        "moss_",
        "leaves_",
        "graffiti_",
        "poster_",
        "vine_",
        "roof_bush",
        "roof_tree",
        "iv_stand",
        "traffic_cone",
        "hose_line",
        "ash_patch",
        "stair_digit_",
        "stair_sign_",
        "stair_arrow_",
        "roof_ladder_seg",
    ]
    .iter()
    .any(|p| id.starts_with(p))
}

pub(super) fn casts_shadow(id: &str) -> bool {
    !is_ground(id)
        && !is_no_shadow_metro_prop(id)
        && !is_big_rock(id)
        && !id.starts_with("rock_")
        && !id.starts_with("bush_")
        && !id.starts_with("cactus_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_and_shadow_rules() {
        assert!(is_ground("grass_tuft_a"));
        assert!(is_ground("flowers_white"));
        assert!(is_ground("fern"));
        assert!(is_ground("mushrooms"));
        assert!(is_ground("reeds"));
        assert!(is_ground("lily_mud_patch"));
        assert!(is_ground("desert_shrub"));
        assert!(is_ground("snow_shrub"));
        assert!(!is_ground("bush_01"));
        assert!(!is_ground("cactus_tall"));
        assert!(!is_ground("tree_pine_01"));

        assert!(!casts_shadow("grass_tuft_a"));
        assert!(!casts_shadow("flowers_yellow"));
        assert!(!casts_shadow("fern"));
        assert!(!casts_shadow("mushrooms"));
        assert!(!casts_shadow("reeds"));
        assert!(!casts_shadow("lily_mud_patch"));
        assert!(!casts_shadow("desert_shrub"));
        assert!(!casts_shadow("snow_shrub"));
        assert!(!casts_shadow("rock_01"));
        assert!(!casts_shadow("rock_sandstone_01"));
        assert!(!casts_shadow("rock_cliff_01"));
        assert!(!casts_shadow("boulder_big"));
        assert!(!casts_shadow("bush_01"));
        assert!(!casts_shadow("cactus_tall"));
        assert!(!casts_shadow("cactus_round"));
        assert!(casts_shadow("tree_pine_01"));
        assert!(casts_shadow("pine_snow"));
        assert!(casts_shadow("tree_swamp"));
        assert!(casts_shadow("tree_desert_dead"));

        // R40 tiny props & decals cast no shadow; car_wall_stack still casts.
        for id in [
            "weeds_crack",
            "moss_patch",
            "leaves_litter",
            "graffiti_dead_inside",
            "poster_wall",
            "vine_wall_s",
            "grass_tall_patch",
            "roof_bush",
            "roof_tree",
        ] {
            assert!(!casts_shadow(id), "{id} must not cast shadow");
        }
        assert!(casts_shadow("car_wall_stack"));
    }

    #[test]
    fn test_biome_model_ranges() {
        assert_eq!(range_for("pine_snow"), TREE);
        assert_eq!(range_for("tree_swamp"), TREE);
        assert_eq!(range_for("tree_desert_dead"), TREE);
        assert_eq!(range_for("boulder_big"), TREE);
        assert_eq!(range_for("rock_cliff_01"), TREE);
        assert_eq!(range_for("rock_sandstone_01"), ROCK);
        assert_eq!(range_for("rock_ice_01"), ROCK);
        assert_eq!(range_for("cactus_tall"), BUSH);
        assert_eq!(range_for("cactus_round"), BUSH);
        assert_eq!(range_for("reeds"), GROUND);
        assert_eq!(range_for("lily_mud_patch"), GROUND);
        assert_eq!(range_for("desert_shrub"), GROUND);
        assert_eq!(range_for("snow_shrub"), GROUND);
    }
}
