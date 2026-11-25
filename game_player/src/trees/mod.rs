//! Tree felling system: axe strikes, felling countdown, fall animation, and wood rewards.
//! Port of `scripts/World/TreeFelling.cs`, `TreeHit.cs`, and `TreeStrike.cs`.

mod falling;
mod felling;
mod save;
mod strike;

pub use falling::{FallingTree, TREE_FALL_DURATION};
pub use felling::{
    TreeFelling, TreeStrikeResult, HIT_WOOD_SOUND, TREE_DROP_OFFSET, TREE_FELL_HITS,
    TREE_ITEM_HALF_HEIGHT, TREE_WOOD_MAX, TREE_WOOD_MIN, WOOD_ITEM_ID,
};
pub use save::{
    from_save_fallen_trees, to_save_fallen_trees, TREE_FALLEN_KEY, TREE_WORLDSTATE_SAVE_ID,
};
pub use strike::{
    find_tree_strike, is_tree_kind, parse_tree_id, tree_id, TreeCandidate, TreeHit,
};
