//! Hint trigger names, parsing, and validation vocabulary.
//!
//! Port of Godot C# scripts/UI/HintTriggers.cs.

pub const TRIGGER_NAMES: [&str; 12] = [
    "start",
    "after",
    "safe_zone_left",
    "zombie_near",
    "crate_near",
    "board_near",
    "near_npc",
    "need_below",
    "hour_from",
    "died",
    "bag_over",
    "has_item",
];

pub const NUMBER_TRIGGERS: [&str; 7] = [
    "after",
    "zombie_near",
    "crate_near",
    "board_near",
    "need_below",
    "hour_from",
    "bag_over",
];

pub const DONE_NAMES: [&str; 5] = [
    "npc_talked",
    "item_used",
    "item_picked",
    "melee_swung",
    "board_used",
];

/// Splits "name:arg" into ("name", "arg") or ("name", "") if no colon exists.
// Godot: HintTriggers.cs Split
pub fn split(text: &str) -> (&str, &str) {
    if let Some(colon) = text.find(':') {
        (&text[..colon], &text[colon + 1..])
    } else {
        (text, "")
    }
}

/// Parses a float argument, returning 0.0 if invalid.
// Godot: HintTriggers.cs Number
pub fn parse_number(arg: &str) -> f32 {
    arg.parse::<f32>().unwrap_or(0.0)
}

/// Splits "breeder:6" into ("breeder", 6.0).
// Godot: HintTriggers.cs SplitNpc
pub fn split_npc(arg: &str) -> (&str, f32) {
    if let Some(colon) = arg.rfind(':') {
        let role = &arg[..colon];
        let dist = parse_number(&arg[colon + 1..]);
        (role, dist)
    } else {
        (arg, 0.0)
    }
}

/// Validates a trigger string from data/hints.json.
/// Returns Ok(()) if valid, or Err(problem_description) if invalid.
// Godot: HintTriggers.cs CheckTrigger
pub fn check_trigger(trigger: &str) -> Result<(), String> {
    let (name, arg) = split(trigger);
    if !TRIGGER_NAMES.contains(&name) {
        return Err(format!("unknown trigger '{name}'"));
    }
    if NUMBER_TRIGGERS.contains(&name) && arg.parse::<f32>().is_err() {
        return Err(format!("trigger '{trigger}' needs a number after the colon"));
    }
    if name == "has_item" && arg.is_empty() {
        return Err("trigger 'has_item' needs an item id after the colon".to_string());
    }
    if name == "near_npc" && !is_valid_npc_arg(arg) {
        return Err(format!(
            "trigger '{trigger}' needs a role and a number, like near_npc:breeder:6"
        ));
    }
    Ok(())
}

fn is_valid_npc_arg(arg: &str) -> bool {
    if let Some(colon) = arg.rfind(':') {
        colon > 0 && arg[colon + 1..].parse::<f32>().is_ok()
    } else {
        false
    }
}
