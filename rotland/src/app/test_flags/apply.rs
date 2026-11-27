//! Startup application of the dev test flags, run once after the world is initialized.

use engine::Gpu;

use crate::app::state::{ActorsState, PlaySide, RenderState, SessionState, UiState, WorldState};

use super::gear::{apply_gunsmith_town, apply_team, apply_wear_and_mods};
use super::metro_tp::apply_metro_tp;
use super::tp::{
    apply_biome_tp, apply_bridge_tp, apply_river_tp, apply_tp_rescue, apply_town_tp,
    apply_trailhead_tp,
};

/// Applies startup dev flags (`--biome`, `--weather`, `--tp-rescue`, `--team`, `--gunsmith*`, `--wear`, `--mods`) after the world is initialized.
pub fn apply_startup(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &SessionState,
) {
    if let Some(biome) = session.test_flags.biome {
        apply_biome_tp(gpu, render, world, actors, session, biome);
    }
    if let Some(key) = session.test_flags.town.as_deref() {
        apply_town_tp(gpu, render, world, actors, session, key);
    }
    if let Some(key) = session.test_flags.metro.as_deref() {
        apply_metro_tp(gpu, render, world, actors, play, session, key);
    }
    if let Some(index) = session.test_flags.trailhead {
        apply_trailhead_tp(gpu, render, world, actors, session, index);
    }
    if let Some(index) = session.test_flags.bridge {
        apply_bridge_tp(gpu, render, world, actors, session, index);
    }
    if let Some(index) = session.test_flags.river {
        apply_river_tp(gpu, render, world, actors, session, index);
    }
    if let (Some(kind), Some(weather)) = (session.test_flags.weather, play.weather.as_mut()) {
        weather.force(kind);
    }
    if let Some(kind) = session.test_flags.spawn_zombie.as_deref()
        && let (Some(zombies), Some(pc)) = (actors.zombies.as_mut(), actors.player_ctrl.as_ref())
    {
        zombies.spawn_in_front_of_player(kind, pc.position, pc.yaw, &world.physics);
    }
    if session.test_flags.tp_rescue {
        apply_tp_rescue(gpu, render, world, actors, play, session);
    }
    if session.test_flags.team > 0 {
        apply_team(actors, play, session.test_flags.team);
    }
    apply_wear_and_mods(render, world, actors, play, session);
    if let Some(key) = session.test_flags.gunsmith_town.as_deref() {
        apply_gunsmith_town(gpu, render, world, actors, play, ui, session, key);
    } else if let Some(tab) = session.test_flags.gunsmith {
        let tier = play
            .play
            .as_ref()
            .map_or(0, |p| p.camp.upgrades.level(game::UpgradeLine::Gunsmith));
        ui.gunsmith = Some(crate::gunsmith_play::GunsmithSession::new(
            tab,
            game::ShopKind::Camp { tier },
            None,
        ));
        ui.cursor_captured = false;
    }
    if session.test_flags.nvg
        && let Some(play_state) = play.play.as_mut()
    {
        play_state.inventory.add(game::BATTERY_ITEM, 2);
        play_state
            .night_vision
            .toggle(&mut play_state.inventory, &play_state.boosts_cfg.night_vision);
        play_state.hotbar.on_inventory_changed(&play_state.inventory);
    }
    if session.test_flags.map && ui.gunsmith.is_none() && ui.hub.is_none() {
        ui.hub = Some(crate::hub_ui::HubUiState::new(
            crate::hub_ui::HubScreen::Tab(crate::hub_ui::HubTab::Map),
        ));
        ui.cursor_captured = false;
    }
}
