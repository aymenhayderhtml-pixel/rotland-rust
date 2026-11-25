//! Deterministic guild task generation from seed and in-game day.

use world::DotnetRandom;

use super::config::TaskTemplateDef;
use super::tasks::{GuildReward, GuildTask, GuildTaskKind};
use game_core::loot::crate_seed;

/// Helper for rolling deterministic guild task boards.
pub struct GuildTaskRoller;

impl GuildTaskRoller {
    /// Generates `count` offered tasks for the board deterministically from world seed and day.
    pub fn roll_board(
        seed: i32,
        day: i32,
        count: usize,
        templates: &[TaskTemplateDef],
    ) -> Vec<GuildTask> {
        if templates.is_empty() || count == 0 {
            return Vec::new();
        }

        let mut offered = Vec::with_capacity(count);
        for slot in 0..count {
            let roll_seed = crate_seed(seed, &format!("guild_board_{day}_{slot}"));
            let mut rng = DotnetRandom::new(roll_seed);
            let tmpl_idx = rng.next(templates.len() as i32) as usize;
            let tmpl = &templates[tmpl_idx];

            let min_target = tmpl.target_range[0].min(tmpl.target_range[1]);
            let max_target = tmpl.target_range[0].max(tmpl.target_range[1]);
            let target = if max_target > min_target {
                rng.next_range(min_target as i32, max_target as i32 + 1) as u32
            } else {
                min_target
            };

            let min_dollars = tmpl.dollars_range[0].min(tmpl.dollars_range[1]);
            let max_dollars = tmpl.dollars_range[0].max(tmpl.dollars_range[1]);
            let dollars = if max_dollars > min_dollars {
                rng.next_range(min_dollars, max_dollars + 1)
            } else {
                min_dollars
            };

            let min_rep = tmpl.rep_range[0].min(tmpl.rep_range[1]);
            let max_rep = tmpl.rep_range[0].max(tmpl.rep_range[1]);
            let rep = if max_rep > min_rep {
                rng.next_range(min_rep, max_rep + 1)
            } else {
                min_rep
            };

            let kind = match tmpl.kind.as_str() {
                "kill" => GuildTaskKind::Kill {
                    zombie_type: tmpl.zombie_type.clone(),
                    in_city: tmpl.in_city.unwrap_or(false),
                },
                "fetch" => GuildTaskKind::Fetch {
                    item_id: tmpl.item_id.clone().unwrap_or_else(|| "wood".to_string()),
                },
                "explore" => GuildTaskKind::Explore {
                    place_name: tmpl
                        .place_name
                        .clone()
                        .unwrap_or_else(|| "Ashford core".to_string()),
                },
                "clear" => GuildTaskKind::Clear {
                    poi_id: tmpl
                        .poi_id
                        .clone()
                        .unwrap_or_else(|| "house_small".to_string()),
                },
                _ => GuildTaskKind::Kill {
                    zombie_type: None,
                    in_city: false,
                },
            };

            let item = tmpl
                .item_reward
                .as_ref()
                .map(|r| (r.item_id.clone(), r.count));

            let reward = GuildReward {
                dollars,
                reputation: rep,
                item,
            };

            let task_id = format!("gt_{day}_{slot}_{}", tmpl.id);
            offered.push(GuildTask::new(task_id, &tmpl.title, kind, target, reward));
        }
        offered
    }
}
