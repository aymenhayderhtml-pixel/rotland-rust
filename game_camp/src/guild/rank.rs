//! Guild reputation rank calculations and progression.

use super::config::GuildRankDef;

pub const RANK_ROOKIE: &str = "rookie";
pub const RANK_VETERAN: &str = "veteran";
pub const RANK_ELITE: &str = "elite";
pub const RANK_MASTER: &str = "master";
pub const RANK_LEGEND: &str = "legend";

static FALLBACK_RANK: GuildRankDef = GuildRankDef {
    id: String::new(),
    name: String::new(),
    min_rep: 0,
    max_recruits: 1,
    shop_tier: 1,
};

/// Finds the player's current rank based on accumulated reputation.
pub fn current_rank(reputation: i32, ranks: &[GuildRankDef]) -> &GuildRankDef {
    let mut current = None;
    for rank in ranks {
        if reputation >= rank.min_rep {
            current = Some(rank);
        } else {
            break;
        }
    }
    current.unwrap_or(&FALLBACK_RANK)
}

/// Finds the next rank the player can advance to, if any.
pub fn next_rank(reputation: i32, ranks: &[GuildRankDef]) -> Option<&GuildRankDef> {
    ranks.iter().find(|r| r.min_rep > reputation)
}

/// Calculates progress towards the next rank as (current_rep, Option<next_rank_min_rep>).
pub fn rank_progress(reputation: i32, ranks: &[GuildRankDef]) -> (i32, Option<i32>) {
    let next = next_rank(reputation, ranks).map(|r| r.min_rep);
    (reputation, next)
}
