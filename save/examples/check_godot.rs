//! Reads (never writes) the owner's Godot save slots and prints what the Rust save crate understood.
//! `cargo run -p save --example check_godot`
fn main() {
    let dir = save::godot_save_dir();
    for slot in 1..=3 {
        let path = dir.join(format!("slot{slot}.json"));
        let Ok(text) = std::fs::read_to_string(&path) else { println!("slot {slot}: none"); continue };
        match save::SaveFile::from_json_str(&text) {
            Ok(f) => {
                let s = &f.systems;
                println!(
                    "slot {slot}: OK v{} saved {} seed {:?} day {:?} hour {:?} player {:?} stacks {} looted {} dollars {:?} other systems {:?}",
                    f.version,
                    f.saved_at_utc,
                    s.world.as_ref().map(|w| w.seed),
                    s.time.as_ref().map(|t| t.day),
                    s.time.as_ref().map(|t| t.hour),
                    s.player.as_ref().map(|p| (p.position.x, p.position.z, p.health)),
                    s.inventory.as_ref().map_or(0, |i| i.stacks.len()),
                    s.worldstate.as_ref().map_or(0, |w| w.looted.len()),
                    s.wallet.as_ref().map(|w| w.dollars),
                    { let mut k: Vec<_> = s.extra.keys().collect(); k.sort(); k },
                );
            }
            Err(e) => println!("slot {slot}: FAILED {e:?}"),
        }
    }
}
