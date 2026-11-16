//! Every file named in data/sounds.json exists and decodes (no sound device needed, nothing plays).

use kira::sound::static_sound::StaticSoundData;

#[test]
fn every_sound_file_decodes() {
    let data = audio::SoundData::from_repo().expect("data/sounds.json");
    let mut decoded = 0;
    for (id, def) in &data.sounds {
        assert!(!def.files.is_empty(), "{id} has no files");
        for f in &def.files {
            let path = audio::SoundData::file_path(f);
            StaticSoundData::from_file(&path).unwrap_or_else(|e| panic!("{id}: {} does not decode: {e}", path.display()));
            decoded += 1;
        }
    }
    println!("decoded {decoded} sound files");
    assert!(decoded >= 28, "only {decoded} files decoded");
    // Every weapon and event points at a known sound
    for (w, m) in &data.weapons {
        for s in m.values() {
            assert!(data.sounds.contains_key(s), "weapon {w} uses unknown sound {s}");
        }
    }
    for (e, d) in &data.events {
        assert!(data.sounds.contains_key(&d.sound), "event {e} uses unknown sound {}", d.sound);
    }
}
