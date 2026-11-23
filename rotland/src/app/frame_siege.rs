//! Base siege wiring: feeds the living zombies into the pure siege simulation each frame,
//! and picks the damaged part the player is looking at for the HUD health label.

use engine::render::Camera;
use game::building::siege::{SiegeAttacker, SiegeConfig, SiegeSystem};
use game::building::{BaseState, BuildConfig};
use game::zombie::ZombieState;
use glam::Vec3;
use physics::PhysicsWorld;

use super::frame_input::build_mode::sync_colliders;
use super::state::{PlaySide, UiState};
use crate::zombies::Zombies;

/// How close the player must be to a building part to see its health label (metres).
// new: DECISIONS 2026-11-23
const PART_HEALTH_LOOK_RANGE: f32 = 4.0;

/// Loads `data/siege.json`. A broken or missing file logs a warning and turns the feature off.
pub fn load_system() -> Option<SiegeSystem> {
    match SiegeConfig::from_repo() {
        Ok(cfg) => Some(SiegeSystem::new(cfg)),
        Err(e) => {
            log::warn!("base siege disabled: could not load data/siege.json: {e}");
            None
        }
    }
}

/// Forgets all siege timers and the alert. Call when the player dies or a save loads.
pub fn clear(play: &mut PlaySide) {
    if let Some(siege) = play.siege.as_mut() {
        siege.clear();
    }
}

/// Runs one siege step: zombies that are alive and near the base hit blocking parts.
/// Rebuilds the base colliders (once, at most) when a part broke or collapsed, so zombies
/// navigate around the new gaps. Visuals follow automatically in the render pass.
pub fn update_siege(
    dt: f32,
    pack: Option<&pack::ModelPack>,
    play: &mut PlaySide,
    physics: &mut PhysicsWorld,
    zombies: Option<&Zombies>,
    player: Vec3,
    alive: bool,
) {
    let Some(siege) = play.siege.as_mut() else {
        return;
    };
    if !alive {
        siege.clear();
        return;
    }
    if play.build_mode.base.is_empty() {
        // No base: nothing to attack, and no alert should linger
        siege.clear();
        return;
    }

    // Living zombies become attackers; the siege system decides who is close and chasing
    let attackers: Vec<SiegeAttacker<'_>> = zombies
        .map(|z| {
            z.list
                .iter()
                .filter(|q| !q.health.is_dead())
                .map(|q| SiegeAttacker {
                    // zombie ids are u64; the siege timer map keys on u32 (ids never get near 2^32 per run)
                    id: q.id as u32,
                    kind: q.kind.as_str(),
                    position: q.position,
                    chasing: matches!(q.brain.state(), ZombieState::Chase | ZombieState::Attack),
                })
                .collect()
        })
        .unwrap_or_default();

    let geometry_changed = {
        let frame = siege.tick(
            &mut play.build_mode.base,
            &play.build_mode.config,
            &attackers,
            player,
            dt,
        );
        frame.geometry_changed
    };

    if geometry_changed
        && let Some(pack) = pack
    {
        sync_colliders(&mut play.build_mode, physics, pack);
    }
}

/// The damaged part the player is looking at (within reach), as `(name, health fraction)`.
/// Returns `None` when nothing is aimed at, or the aimed part is at full health.
pub fn aimed_damaged_part(base: &BaseState, config: &BuildConfig, camera: &Camera) -> Option<(String, f32)> {
    let part_id = base.raycast_part(camera.position, camera.forward(), PART_HEALTH_LOOK_RANGE)?;
    let part = base.get(part_id)?;
    let def = config.get(&part.part)?;
    if def.hp > 0.0 && part.hp < def.hp {
        Some((def.name.clone(), part.hp / def.hp))
    } else {
        None
    }
}

/// True when a menu screen (other than none) is open, so the in-world labels stay hidden.
pub fn menu_open(ui: &UiState) -> bool {
    ui.menus
        .as_ref()
        .is_some_and(|m| !matches!(m.screen(), crate::menus::Screen::None))
}
