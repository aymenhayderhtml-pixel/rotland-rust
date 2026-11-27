//! Rescued-style recruit builder for `--team=<n>`, from `data/guild.json` recruit templates.

use game::guild::{GuildRecruit, RecruitDef, RecruitRole};

/// Fallback recruit definition if `data/guild.json` has no `recruits` entries.
const FALLBACK_NAME: &str = "Mark Vance";
const FALLBACK_LEVEL: u32 = 1;
const FALLBACK_WAGE: u32 = 15; // data: guild.json daily_wage
const FALLBACK_HEALTH: u32 = 100; // data: guild.json health

fn parse_recruit_role(raw: &str) -> RecruitRole {
    match raw.trim().to_ascii_lowercase().as_str() {
        "scavenger" => RecruitRole::Scavenger,
        "guard" => RecruitRole::Guard,
        _ => RecruitRole::Fighter,
    }
}

/// Builds `count` free rescued-style recruits from `data/guild.json` recruit templates.
pub fn build_rescued_recruits(count: usize, templates: &[RecruitDef]) -> Vec<GuildRecruit> {
    if count == 0 {
        return Vec::new();
    }
    let fallback = [RecruitDef {
        id: "rec_mark".to_string(),
        name: FALLBACK_NAME.to_string(),
        role: "fighter".to_string(),
        level: FALLBACK_LEVEL,
        daily_wage: FALLBACK_WAGE,
        health: FALLBACK_HEALTH,
        hire_cost: 0,
    }];
    let pool: &[RecruitDef] = if templates.is_empty() {
        &fallback
    } else {
        templates
    };

    (0..count)
        .map(|i| {
            let tmpl = &pool[i % pool.len()];
            GuildRecruit::new(
                format!("dev_team_{}", i + 1),
                &tmpl.name,
                parse_recruit_role(&tmpl.role),
                tmpl.level,
                tmpl.daily_wage,
                tmpl.health,
                0,
            )
        })
        .collect()
}
