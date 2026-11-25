//! Tree felling manager.
//! Port of `scripts/World/TreeFelling.cs`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use glam::{IVec2, Vec3};
use serde_json::Value;

use game_core::items::Inventory;
use game_core::skills::SkillSystem;
use game_core::weapons::WeaponRegistry;

use super::falling::{FallingTree, TREE_FALL_DURATION};
use super::save::{from_save_fallen_trees, to_save_fallen_trees};
use super::strike::{is_tree_kind, tree_id};

/// Number of axe hits required to fell a tree.
/// Port of `scripts/World/TreeFelling.cs:FellHits`.
pub const TREE_FELL_HITS: u32 = 4;

/// Minimum wood dropped when a tree is felled.
/// Port of `scripts/World/TreeFelling.cs:WoodMin`.
pub const TREE_WOOD_MIN: u32 = 3;

/// Maximum wood dropped when a tree is felled.
/// Port of `scripts/World/TreeFelling.cs:WoodMax`.
pub const TREE_WOOD_MAX: u32 = 5;

/// How far from the trunk wood lands on the +X side.
/// Port of `scripts/World/TreeFelling.cs:DropOffset`.
pub const TREE_DROP_OFFSET: f32 = 1.5;

/// Half the height of a world item box, so the wood sits on the ground.
/// Port of `scripts/World/TreeFelling.cs:ItemHalfHeight`.
pub const TREE_ITEM_HALF_HEIGHT: f32 = 0.125;

/// Item id for wood in `data/items.json`.
pub const WOOD_ITEM_ID: &str = "wood";

/// Sound event played when chopping wood.
pub const HIT_WOOD_SOUND: &str = "hit_wood";

/// Result of striking a tree.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeStrikeResult {
    /// Hits remaining until the tree is felled. 0 if felled or already felled.
    pub hits_left: u32,
    /// Wood units granted upon felling (0 for non-felling strikes).
    pub wood_given: u32,
    /// True if the tree was felled by this strike or was already felled.
    pub felled: bool,
    /// Sound event name to play (e.g. "hit_wood"). None if no wood was struck.
    pub sound_event: Option<&'static str>,
}

impl TreeStrikeResult {
    /// Adds any granted wood directly into the player's inventory.
    /// Returns leftover wood that could not fit (0 if all wood fit).
    pub fn add_to_inventory(&self, inventory: &mut Inventory) -> u32 {
        if self.wood_given > 0 {
            inventory.add(WOOD_ITEM_ID, self.wood_given)
        } else {
            0
        }
    }
}

/// Tracks tree hits, felled trees, and active falling animations.
/// Port of `scripts/World/TreeFelling.cs`.
#[derive(Clone, Debug)]
pub struct TreeFelling {
    /// Ids of trees that have been felled: `"tree_{chunkX}_{chunkY}_{propIndex}"`.
    pub fallen: HashSet<String>,
    /// Partial hit counts for standing trees. Not saved across sessions.
    pub hits: HashMap<String, u32>,
    /// Active falling tree animations.
    pub falling: Vec<FallingTree>,
    /// Weapon registry for checking weapon tags (e.g. tag "chop" on axes).
    pub weapons: Option<Arc<WeaponRegistry>>,
    rng_state: u64,
    /// Test override for deterministic wood yields.
    pub fixed_wood: Option<u32>,
    /// Duration of the fall animation in seconds.
    pub fall_duration: f32,
}

impl Default for TreeFelling {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeFelling {
    pub const FELL_HITS: u32 = TREE_FELL_HITS;
    #[allow(dead_code)]
    const WOOD_MIN: u32 = TREE_WOOD_MIN;
    #[allow(dead_code)]
    const WOOD_MAX: u32 = TREE_WOOD_MAX;
    #[allow(dead_code)]
    const DROP_OFFSET: f32 = TREE_DROP_OFFSET;

    /// Creates a TreeFelling manager, loading `data/weapons.json` from the repo if available.
    pub fn new() -> Self {
        let weapons = match paths::repo_path("data/weapons.json") {
            path if path.is_file() => WeaponRegistry::load(&path).ok().map(Arc::new),
            _ => None,
        };
        Self::with_weapons(weapons)
    }

    /// Creates a TreeFelling manager with an explicit weapon registry.
    pub fn with_weapons(weapons: Option<Arc<WeaponRegistry>>) -> Self {
        Self {
            fallen: HashSet::new(),
            hits: HashMap::new(),
            falling: Vec::new(),
            weapons,
            rng_state: 0x9E37_79B9_7F4A_7C15,
            fixed_wood: None,
            fall_duration: TREE_FALL_DURATION,
        }
    }

    /// Creates a TreeFelling manager with a specific random seed.
    pub fn with_seed(seed: u64) -> Self {
        let mut tf = Self::new();
        tf.set_seed(seed);
        tf
    }

    /// Sets the PRNG seed for wood drops.
    pub fn set_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed };
    }

    /// Overrides random wood roll with a fixed count (useful for tests).
    pub fn set_fixed_wood(&mut self, count: Option<u32>) {
        self.fixed_wood = count;
    }

    /// Sets the weapon registry for checking weapon tags.
    #[allow(dead_code)]
    fn set_weapons(&mut self, weapons: Arc<WeaponRegistry>) {
        self.weapons = Some(weapons);
    }

    /// True if the weapon has tag "chop" (or fallback to id "axe").
    pub fn can_chop(&self, weapon_id: &str) -> bool {
        if let Some(registry) = &self.weapons
            && let Some(def) = registry.get(weapon_id) {
                return def.tags.iter().any(|t| t == "chop");
            }
        weapon_id == "axe"
    }

    /// True if the tree with the given stable id has been felled.
    pub fn is_felled(&self, tree_id: &str) -> bool {
        self.fallen.contains(tree_id)
    }

    /// True if the tree at the given chunk coordinate and prop index has been felled.
    pub fn is_prop_felled(&self, chunk: IVec2, prop_index: usize) -> bool {
        self.is_felled(&tree_id(chunk, prop_index))
    }

    /// Counts one strike against a tree.
    ///
    /// - `tree_id`: Stable identifier (e.g. `"tree_0_0_3"`).
    /// - `tree_pos`: World position of the tree trunk.
    /// - `tree_kind`: Prop model name (e.g. `"tree_pine_01"`).
    /// - `weapon_id`: Weapon id, or None for bare hands. Only weapons with tag "chop" chop trees.
    /// - `_skills`: Player skills (Godot does not award skill XP for woodcutting).
    /// - `player_pos`: Player position (fall direction points away from the player).
    // Godot: TreeFelling.cs Strike & MeleeHandler.cs
    #[allow(clippy::too_many_arguments)]
    pub fn strike(
        &mut self,
        tree_id: &str,
        tree_pos: Vec3,
        tree_kind: &str,
        weapon_id: Option<&str>,
        _skills: Option<&SkillSystem>,
        player_pos: Vec3,
    ) -> TreeStrikeResult {
        if self.is_felled(tree_id) {
            return TreeStrikeResult {
                hits_left: 0,
                wood_given: 0,
                felled: true,
                sound_event: None,
            };
        }

        if !is_tree_kind(tree_kind) {
            let current_hits = self.hits.get(tree_id).copied().unwrap_or(0);
            return TreeStrikeResult {
                hits_left: TREE_FELL_HITS.saturating_sub(current_hits),
                wood_given: 0,
                felled: false,
                sound_event: None,
            };
        }

        let is_chop = match weapon_id {
            Some(id) if !id.is_empty() => self.can_chop(id),
            _ => false,
        };

        if !is_chop {
            let current_hits = self.hits.get(tree_id).copied().unwrap_or(0);
            return TreeStrikeResult {
                hits_left: TREE_FELL_HITS.saturating_sub(current_hits),
                wood_given: 0,
                felled: false,
                sound_event: None,
            };
        }

        let hits = self.hits.entry(tree_id.to_string()).or_insert(0);
        *hits += 1;
        let count_hits = *hits;

        if count_hits < TREE_FELL_HITS {
            TreeStrikeResult {
                hits_left: TREE_FELL_HITS - count_hits,
                wood_given: 0,
                felled: false,
                sound_event: Some(HIT_WOOD_SOUND),
            }
        } else {
            self.hits.remove(tree_id);
            self.fallen.insert(tree_id.to_string());

            let mut fall_dir = tree_pos - player_pos;
            fall_dir.y = 0.0;
            if fall_dir.length_squared() > 0.0001 {
                fall_dir = fall_dir.normalize();
            } else {
                fall_dir = Vec3::Z;
            }

            self.falling.push(FallingTree::new(
                tree_id,
                tree_pos,
                fall_dir,
                self.fall_duration,
            ));

            let wood = self.roll_wood();

            TreeStrikeResult {
                hits_left: 0,
                wood_given: wood,
                felled: true,
                sound_event: Some(HIT_WOOD_SOUND),
            }
        }
    }

    /// Convenience wrapper taking a string weapon id directly.
    #[allow(clippy::too_many_arguments)]
    pub fn strike_weapon(
        &mut self,
        tree_id: &str,
        tree_pos: Vec3,
        tree_kind: &str,
        weapon_id: &str,
        skills: Option<&SkillSystem>,
        player_pos: Vec3,
    ) -> TreeStrikeResult {
        let opt = if weapon_id.is_empty() || weapon_id == "bare_hands" || weapon_id == "unarmed" {
            None
        } else {
            Some(weapon_id)
        };
        self.strike(tree_id, tree_pos, tree_kind, opt, skills, player_pos)
    }

    /// Advances active falling animations and returns the current state of all falling trees.
    /// Finished trees are reported one final time with `finished = true` and removed on the next call.
    pub fn update(&mut self, dt: f32) -> Vec<FallingTree> {
        self.falling.retain(|t| !t.finished);
        for tree in &mut self.falling {
            tree.update(dt);
        }
        self.falling.clone()
    }

    /// Hides props of newly loaded chunk that are in the fallen set.
    /// Port of `scripts/World/TreeFelling.cs:ApplyFallen`.
    pub fn apply_fallen<'a>(
        &self,
        chunk: IVec2,
        props: impl IntoIterator<Item = (usize, &'a str)>,
    ) -> Vec<usize> {
        let mut felled_indices = Vec::new();
        for (idx, model) in props {
            if is_tree_kind(model) && self.is_prop_felled(chunk, idx) {
                felled_indices.push(idx);
            }
        }
        felled_indices
    }

    /// Serializes fallen trees to Godot-compatible WorldState JSON value.
    // Godot: WorldState.cs Save
    pub fn to_save(&self) -> Value {
        to_save_fallen_trees(&self.fallen)
    }

    /// Loads fallen trees from Godot WorldState save value.
    // Godot: WorldState.cs Load
    pub fn from_save(&mut self, data: &Value) {
        self.fallen = from_save_fallen_trees(data);
        self.hits.clear();
        self.falling.clear();
    }

    fn roll_wood(&mut self) -> u32 {
        if let Some(fixed) = self.fixed_wood {
            return fixed;
        }
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;

        let span = (TREE_WOOD_MAX - TREE_WOOD_MIN + 1) as u64;
        TREE_WOOD_MIN + (self.rng_state % span) as u32
    }
}
