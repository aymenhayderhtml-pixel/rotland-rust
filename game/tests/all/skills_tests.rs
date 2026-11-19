use std::sync::Arc;

use game::skills::{
    combat_xp_for_kill, AgilityTracker, SkillDef, SkillRegistry, SkillSystem,
};

#[test]
fn skills_data_loads_and_matches_registry() {
    let registry = SkillRegistry::from_repo().expect("skills.json loads");
    let all = registry.all();
    assert_eq!(all.len(), 5);

    let mut count = 0;
    for def in all {
        assert_eq!(def.max_level, 20);
        assert_eq!(def.xp_base, 100.0);
        assert_eq!(def.xp_growth, 1.35);
        count += 1;
    }
    assert_eq!(count, 5);

    let combat = registry.get("combat").expect("combat exists");
    assert_eq!(combat.name, "Combat");
    assert_eq!(combat.bonus_per_level.get("meleeDamage").copied(), Some(0.03));
    assert_eq!(combat.bonus_per_level.get("gunDamage").copied(), Some(0.02));
}

#[test]
fn xp_to_level_thresholds_traced_from_data() {
    let def = SkillDef {
        id: "combat".to_string(),
        name: "Combat".to_string(),
        max_level: 20,
        xp_base: 100.0,
        xp_growth: 1.35,
        bonus_per_level: std::collections::HashMap::new(),
    };

    // Level 1: 100 * 1.35^0 = 100
    assert_eq!(def.xp_to_next(1), 100);
    // Level 2: 100 * 1.35^1 = 135
    assert_eq!(def.xp_to_next(2), 135);
    // Level 3: 100 * 1.35^2 = 182.25 -> 182
    assert_eq!(def.xp_to_next(3), 182);
    // Level 4: 100 * 1.35^3 = 246.0375 -> 246
    assert_eq!(def.xp_to_next(4), 246);
    // Level 5: 100 * 1.35^4 = 332.150625 -> 332
    assert_eq!(def.xp_to_next(5), 332);

    let registry = Arc::new(SkillRegistry::from_repo().expect("registry"));
    let mut system = SkillSystem::new(registry);

    // 250 XP crosses two levels: 100 for level 2, then 135 for level 3, leaving 15
    // Matches Godot test SkillLevelUpMultiple.cs
    let (granted, new_levels) = system.add_xp("combat", 250.0);
    assert_eq!(granted, 250.0);
    assert_eq!(new_levels, vec![2, 3]);
    assert_eq!(system.get_level("combat"), 3);
    assert_eq!(system.get_xp("combat"), 15.0);
    assert_eq!(system.get_xp_to_next("combat"), 182);
}

#[test]
fn skill_bonuses_scale_with_level() {
    let registry = Arc::new(SkillRegistry::from_repo().expect("registry"));
    let mut system = SkillSystem::new(registry);

    // Initial level 1 gives 0 bonus
    assert_eq!(system.get_bonus("meleeDamage"), 0.0);
    assert_eq!(system.get_bonus("foodValue"), 0.0);
    assert_eq!(system.get_bonus("staminaMax"), 0.0);

    // Combat level 3 (2 levels above base): 0.03 * 2 = 0.06 meleeDamage, 0.02 * 2 = 0.04 gunDamage
    system.add_xp("combat", 250.0);
    assert!((system.get_bonus("meleeDamage") - 0.06).abs() < 1e-4);
    assert!((system.get_bonus("gunDamage") - 0.04).abs() < 1e-4);

    // Cooking level 2 (100 XP): 0.03 * 1 = 0.03 foodValue
    system.add_xp("cooking", 100.0);
    assert_eq!(system.get_level("cooking"), 2);
    assert!((system.get_bonus("foodValue") - 0.03).abs() < 1e-4);
}

#[test]
fn max_level_cap_and_excess_xp() {
    let registry = Arc::new(SkillRegistry::from_repo().expect("registry"));
    let mut system = SkillSystem::new(registry);

    // Add enough XP to max out combat (level 20)
    let (granted, levels) = system.add_xp("combat", 100_000.0);
    assert!(granted > 0.0);
    assert_eq!(levels.last().copied(), Some(20));
    assert_eq!(system.get_level("combat"), 20);
    assert_eq!(system.get_xp("combat"), 0.0);
    assert_eq!(system.get_xp_to_next("combat"), 0);

    // Adding more XP past max is rejected
    let (extra_granted, extra_levels) = system.add_xp("combat", 500.0);
    assert_eq!(extra_granted, 0.0);
    assert!(extra_levels.is_empty());
}

#[test]
fn xp_rewards_for_combat_and_agility() {
    // Walker base XP = 25.0
    // Normal kill gives 25 XP
    assert_eq!(combat_xp_for_kill(25.0, false), 25.0);
    // Headshot: 25.0 * 1.5 = 37.5 -> rounded away from zero = 38.0
    assert_eq!(combat_xp_for_kill(25.0, true), 38.0);

    // Agility sprint distance: 20m per 1 XP
    let mut tracker = AgilityTracker::new();
    assert_eq!(tracker.add_sprint_distance(15.0), 0);
    assert_eq!(tracker.sprint_metres, 15.0);

    // Add 30m -> total 45m -> 2 XP, 5m leftover
    assert_eq!(tracker.add_sprint_distance(30.0), 2);
    assert!((tracker.sprint_metres - 5.0).abs() < 1e-4);
}

#[test]
fn skills_save_load_roundtrip() {
    let registry = Arc::new(SkillRegistry::from_repo().expect("registry"));
    let mut system = SkillSystem::new(Arc::clone(&registry));

    system.add_xp("combat", 250.0); // Level 3, 15 XP
    system.add_xp("agility", 100.0); // Level 2, 0 XP

    let mut saved = system.to_save();
    // Inject bogus unknown skill id
    saved.as_object_mut().unwrap().insert(
        "bogus".to_string(),
        serde_json::json!({ "level": 9, "xp": 1.0 }),
    );

    let mut loaded = SkillSystem::new(registry);
    loaded.from_save(&saved);

    assert_eq!(loaded.get_level("combat"), 3);
    assert_eq!(loaded.get_xp("combat"), 15.0);
    assert_eq!(loaded.get_level("agility"), 2);
    assert_eq!(loaded.get_xp("agility"), 0.0);
    assert_eq!(loaded.get_level("bogus"), 1); // ignored

    // Clamping test
    let clamped_data = serde_json::json!({
        "combat": { "level": 99, "xp": 999.0 },
        "farming": { "level": 0, "xp": -50.0 }
    });
    loaded.from_save(&clamped_data);
    assert_eq!(loaded.get_level("combat"), 20);
    assert_eq!(loaded.get_xp("combat"), 0.0);
    assert_eq!(loaded.get_level("farming"), 1);
    assert_eq!(loaded.get_xp("farming"), 0.0);
}
