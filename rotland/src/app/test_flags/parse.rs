//! Value parsers for the dev test flags (`--wear=`, `--mods=`, non-negative numbers).

pub(super) fn parse_wear_flag(raw: &str) -> Option<(String, f32)> {
    let (weapon, val) = raw.split_once(':')?;
    let weapon = weapon.trim().to_ascii_lowercase();
    let cond: f32 = val.trim().parse().ok()?;
    if weapon.is_empty() || !cond.is_finite() {
        log::warn!("--wear: invalid value '{raw}'");
        return None;
    }
    Some((weapon, cond.clamp(0.0, 1.0)))
}

pub(super) fn parse_mods_flag(raw: &str) -> Option<(String, Vec<String>)> {
    let (weapon, list) = raw.split_once(':')?;
    let weapon = weapon.trim().to_ascii_lowercase();
    let mods: Vec<String> = list
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if weapon.is_empty() || mods.is_empty() {
        log::warn!("--mods: invalid value '{raw}'");
        return None;
    }
    Some((weapon, mods))
}

pub(super) fn parse_non_negative_f32(raw: &str) -> Option<f32> {
    let val: f32 = raw.trim().parse().ok()?;
    (val.is_finite() && val >= 0.0).then_some(val)
}
