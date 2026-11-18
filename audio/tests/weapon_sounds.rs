//! Verifies weapon sound mappings and fallbacks for data/weapons.json and data/sounds.json.

use std::collections::BTreeMap;

#[test]
fn every_weapon_id_has_sound_entry() {
    let weapons_path = paths::repo_path("data/weapons.json");
    let weapons_text = std::fs::read_to_string(&weapons_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", weapons_path.display()));
    let weapons_map: BTreeMap<String, serde_json::Value> = serde_json::from_str(&weapons_text)
        .expect("valid data/weapons.json");

    let sound_data = audio::SoundData::from_repo().expect("valid data/sounds.json");

    let mut missing = Vec::new();
    let mut checked = 0;

    for (id, val) in &weapons_map {
        let kind = val.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        let required_event = match kind {
            "gun" => "shot",
            "melee" => "swing",
            other => panic!("unknown weapon kind '{other}' for weapon '{id}'"),
        };

        let weapon_slots = sound_data.weapons.get(id);
        let sound_name = weapon_slots.and_then(|slots| slots.get(required_event).map(String::as_str));

        match sound_name {
            Some(sound_id) => {
                if !sound_data.sounds.contains_key(sound_id) {
                    missing.push(format!(
                        "weapon '{id}' ({kind}) references missing sound '{sound_id}' for event '{required_event}'"
                    ));
                }
            }
            None => {
                missing.push(format!(
                    "weapon '{id}' ({kind}) has no sound entry for event '{required_event}' in data/sounds.json"
                ));
            }
        }
        checked += 1;
    }

    assert!(
        missing.is_empty(),
        "weapons missing sound entries in data/sounds.json:\n{missing:#?}"
    );
    assert_eq!(checked, 11, "expected 11 weapons in data/weapons.json, checked {checked}");
}

#[test]
fn weapon_sound_fallback_when_missing() {
    let sound_data = audio::SoundData::from_repo().expect("valid data/sounds.json");

    // Existing weapons map to their configured sound
    assert_eq!(sound_data.weapon_sound("ak47", "shot"), Some("rifle_shot"));
    assert_eq!(sound_data.weapon_sound("smg", "shot"), Some("pistol_shot"));
    assert_eq!(sound_data.weapon_sound("crowbar", "swing"), Some("swing_heavy"));
    assert_eq!(sound_data.weapon_sound("katana", "swing"), Some("swing_light"));

    // Missing weapons fall back by weapon class (action)
    assert_eq!(
        sound_data.weapon_sound("future_gun_unlisted", "shot"),
        Some("pistol_shot")
    );
    assert_eq!(
        sound_data.weapon_sound("future_melee_unlisted", "swing"),
        Some("swing_light")
    );
    assert_eq!(
        sound_data.weapon_sound("future_weapon_unlisted", "unknown_action"),
        None
    );
}
