//! Execution of developer panel commands against live world, actor, and play state.

use game::dev_panel::{
    dev_zombie_spawn_spot, DEV_KILL_ZOMBIE_DAMAGE, DEV_KILL_ZOMBIE_RADIUS,
    DEV_TELEPORT_GROUND_OFFSET, DEV_ZOMBIE_SPAWN_DISTANCE,
};
use game::DamageInfo;
use glam::Vec3;

use crate::app::state::{ActorsState, PlaySide, SessionState, WorldState};

use super::DevPanelState;

/// Total hours in one in-game day for clamping time slider updates.
const HOURS_PER_DAY: f32 = 24.0; // new: DECISIONS 2026-11-20

/// Commands emitted by the developer panel UI for application this frame.
#[derive(Clone, Debug)]
pub enum DevAction {
    Teleport(Vec3),
    SetTime(f32),
    SetFreezeTime(bool),
    SetGodMode(bool),
    SpawnZombie(String),
    KillZombiesNear,
    GiveItem { id: String, count: u32 },
}

/// Applies all developer actions queued by the UI this frame.
pub fn apply_dev_actions(
    actions: &[DevAction],
    state: &mut DevPanelState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &mut SessionState,
) {
    for action in actions {
        match action {
            DevAction::Teleport(pos) => teleport_player(*pos, state, world, actors),
            DevAction::SetTime(hour) => set_time(*hour, state, play),
            DevAction::SetFreezeTime(frozen) => set_freeze_time(*frozen, state, play),
            DevAction::SetGodMode(enabled) => set_god_mode(*enabled, state, session),
            DevAction::SpawnZombie(kind) => spawn_zombie(kind, state, world, actors),
            DevAction::KillZombiesNear => kill_zombies_near(state, actors),
            DevAction::GiveItem { id, count } => give_item(id, *count, state, play),
        }
    }
}

/// Keeps player health, stamina, hunger, and thirst topped up every frame while god mode is on.
pub fn maintain_god_mode(god_mode: bool, actors: &mut ActorsState, play: &mut PlaySide) {
    if !god_mode {
        return;
    }
    if let Some(combat) = actors.combat.as_mut() {
        let max_hp = combat.health.max();
        if combat.health.is_dead() {
            combat.health.revive(Some(max_hp));
        } else if combat.health.current() < max_hp {
            combat.health.heal(max_hp - combat.health.current());
        }
    }
    if let Some(player) = actors.player_ctrl.as_mut() {
        player.stamina.current = player.stamina.max;
    }
    if let Some(play_state) = play.play.as_mut() {
        play_state.needs_respawn();
    }
}

fn teleport_player(
    pos: Vec3,
    state: &mut DevPanelState,
    world: &WorldState,
    actors: &mut ActorsState,
) {
    let ground_y = world
        .physics
        .ground_height(pos.x, pos.z)
        .unwrap_or_else(|| world.height(pos.x, pos.z))
        + DEV_TELEPORT_GROUND_OFFSET;
    let dest = Vec3::new(pos.x, ground_y, pos.z);
    actors.player = dest;
    if let Some(player) = actors.player_ctrl.as_mut() {
        player.position = dest;
        player.velocity = Vec3::ZERO;
        player.on_floor = false;
    }
    state.status_msg = Some(format!(
        "Teleported to ({:.1}, {:.1}, {:.1})",
        dest.x, dest.y, dest.z
    ));
}

fn set_time(hour: f32, state: &mut DevPanelState, play: &mut PlaySide) {
    let clamped = hour.clamp(0.0, HOURS_PER_DAY);
    play.clock.set_time(clamped);
    play.hour = play.clock.hour;
    state.status_msg = Some(format!(
        "Time set to {}",
        world::time::Clock::format_clock(play.hour)
    ));
}

fn set_freeze_time(frozen: bool, state: &mut DevPanelState, play: &mut PlaySide) {
    play.clock_frozen = frozen;
    state.status_msg = Some(if frozen {
        "Time frozen".to_string()
    } else {
        "Time running".to_string()
    });
}

fn set_god_mode(enabled: bool, state: &mut DevPanelState, session: &mut SessionState) {
    state.god_mode = enabled;
    if enabled {
        session.die_at = None;
    }
    state.status_msg = Some(if enabled {
        "God mode enabled".to_string()
    } else {
        "God mode disabled".to_string()
    });
}

fn spawn_zombie(
    kind: &str,
    state: &mut DevPanelState,
    world: &WorldState,
    actors: &mut ActorsState,
) {
    let (player_pos, player_yaw) = actors
        .player_ctrl
        .as_ref()
        .map_or((actors.player, 0.0), |p| (p.position, p.yaw));
    let target = dev_zombie_spawn_spot(player_pos, player_yaw, DEV_ZOMBIE_SPAWN_DISTANCE);
    let ground_y = world
        .physics
        .ground_height(target.x, target.z)
        .unwrap_or_else(|| world.height(target.x, target.z));
    let spawn_pos = Vec3::new(target.x, ground_y, target.z);

    let Some(zombies) = actors.zombies.as_mut() else {
        state.status_msg = Some("Zombie system not initialized".to_string());
        return;
    };
    if let Some(id) = zombies.spawn_kind(kind, spawn_pos) {
        state.status_msg = Some(format!(
            "Spawned {kind} #{id} at ({:.1}, {:.1}, {:.1})",
            spawn_pos.x, spawn_pos.y, spawn_pos.z
        ));
    } else {
        state.status_msg = Some(format!("Failed to spawn zombie type: {kind}"));
    }
}

fn kill_zombies_near(state: &mut DevPanelState, actors: &mut ActorsState) {
    let player_pos = actors
        .player_ctrl
        .as_ref()
        .map_or(actors.player, |p| p.position);
    let Some(zombies) = actors.zombies.as_mut() else {
        state.status_msg = Some("Zombie system not initialized".to_string());
        return;
    };
    let radius_sq = DEV_KILL_ZOMBIE_RADIUS * DEV_KILL_ZOMBIE_RADIUS;
    let ids: Vec<u64> = zombies
        .list
        .iter()
        .filter(|z| !z.health.is_dead() && z.position.distance_squared(player_pos) <= radius_sq)
        .map(|z| z.id)
        .collect();
    let mut killed = 0usize;
    for id in ids {
        let mut info = DamageInfo::new(DEV_KILL_ZOMBIE_DAMAGE);
        info.headshot = true;
        if zombies.hit(id, info, Vec3::ZERO) {
            killed += 1;
        }
    }
    state.status_msg = Some(format!(
        "Killed {killed} zombie(s) within {:.0} m",
        DEV_KILL_ZOMBIE_RADIUS
    ));
}

fn give_item(id: &str, count: u32, state: &mut DevPanelState, play: &mut PlaySide) {
    let Some(play_state) = play.play.as_mut() else {
        state.status_msg = Some("Play state not initialized".to_string());
        return;
    };
    let qty = count.max(1);
    let before = play_state.inventory.count(id);
    let _ = play_state.inventory.add(id, qty);
    let added = play_state.inventory.count(id).saturating_sub(before);
    play_state.hotbar.on_inventory_changed(&play_state.inventory);
    if added > 0 {
        let msg = format!("Gave {added}x {id}");
        play_state.show_toast(msg.clone());
        state.status_msg = Some(msg);
    } else {
        state.status_msg = Some(format!("Could not add {id} (bag full or unknown)"));
    }
}
