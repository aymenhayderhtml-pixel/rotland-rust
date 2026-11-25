# R31-01: guns kick and flash

Owns: game_player/src/gun_feel.rs, data/gun_feel.json, game/tests/all/gun_feel.rs, rotland/src/gun_feel_view.rs, rotland/src/shooting.rs, rotland/src/fp_weapon.rs, game/src/camera_rig.rs, rotland/src/light.rs
Rules: `tasks/R31-common.md`. Owner decision: `docs/DECISIONS.md` row 2026-11-25 (weapons).

**Goal:** shooting feels weak today. `recoil_climb` is parsed in `game_core/src/weapons.rs`, but nothing uses it, and there is no muzzle flash. Add:
1. **View kick:** each shot pitches the camera up by `kickDeg` plus a small random sideways `kickYawDeg`. It eases back down at `recoverDegPerSec`. Automatic guns (ak47, smg) climb while you hold the trigger, up to `maxClimbDeg`. When aiming, the kick is `aimMul` (0.6) of normal. Works in first and third person.
2. **First-person gun kick:** the FP gun model jumps back `fpBack` m and tilts up `fpTiltDeg`, then springs back in ~0.12 s (`fp_weapon.rs`).
3. **Muzzle flash:** for 0.05 s, a small bright star-shaped quad (2 crossed quads, additive or unlit yellow-white) at the barrel tip, plus a short warm point light (radius `flashLight` m) using the existing light code. Bow and crossbow: no flash, small kick only.
4. **Data:** `data/gun_feel.json`, per weapon id, with these starting values:
   pistol kick 2.5 / revolver 5 / rifle 6 / hunting_rifle 7 / shotgun 8 / ak47 1.4 (climb, max 9) / smg 0.9 (climb, max 7) / bow 0.6 / crossbow 1.5. recoverDegPerSec 12, fpBack 0.06, fpTiltDeg 4, flashLight 4, aimMul 0.6.
- Pure math (kick state, ease back, climb clamp, aim multiplier) goes in `game_player/src/gun_feel.rs` with tests. `rotland/src/gun_feel_view.rs` holds `GunFeelView` with `on_shot(&mut self, weapon_id: &str)` plus a per-frame update that applies the offsets. Call `on_shot` from the place in `shooting.rs` where a shot is fired (and from the bow/crossbow fire).
- The kick must not change where bullets go after the shot has left. Only the view moves.
