//! Per-frame weather and body warmth ticks (R32-03, R32-05).

use glam::Vec3;

use super::state::{ActorsState, PlaySide, WorldState};
use crate::temperature_play::TemperatureTickInput;

/// Advances weather, then body warmth; applies cold/heat damage through the normal damage path.
pub fn update_weather_and_warmth(dt: f32, play: &mut PlaySide, world: &WorldState, actors: &mut ActorsState) {
    let Some(generator) = world.world_gen.as_deref() else {
        return;
    };
    let player = actors.player;
    let dt_game = if play.clock_frozen { 0.0 } else { dt };
    if let Some(weather) = play.weather.as_mut() {
        weather.tick(dt_game, player, generator);
    }
    let (Some(temp), Some(weather), Some(pl)) = (
        play.temperature.as_mut(),
        play.weather.as_ref(),
        play.play.as_mut(),
    ) else {
        return;
    };
    if std::mem::take(&mut pl.hand_warmer_pending) {
        temp.activate_hand_warmer(&mut pl.inventory);
    }
    let sample = generator.biome_sample(player.x, player.z);
    let lit: Vec<Vec3> = pl
        .camp
        .campfires
        .campfires()
        .iter()
        .filter(|c| c.permanent || c.remaining_hours > 0.0)
        .map(|c| c.position)
        .collect();
    let in_camp = generator
        .settlements()
        .iter()
        .any(|s| s.contains(player, 0.0, generator.config()));
    let res = temp.tick(
        &TemperatureTickInput {
            dt: dt_game,
            player_pos: player,
            hour: play.hour,
            biome_registry: generator.biome_registry(),
            biome_sample: &sample,
            weather: &weather.state,
            lit_campfires: &lit,
            in_camp,
            indoors: generator.indoors(player.x, player.z),
            inventory: &pl.inventory,
        },
        Some(&mut pl.thirst),
    );
    if let Some(p) = actors.player_ctrl.as_mut() {
        p.stamina.warmth_regen_multiplier = res.stamina_regen_mult;
    }
    temp.hud_tier = sample.tier;
    use game::survival::temperature::TempState;
    play.hint_ctx.temp_cold = matches!(res.state, TempState::Freezing | TempState::Cold);
    play.hint_ctx.temp_hot = matches!(res.state, TempState::Hot | TempState::Overheated);
    if res.health_damage > 0.0
        && let Some(c) = actors.combat.as_mut()
    {
        let now = actors.downed.last_play_time();
        c.take_damage_with_downed(&[res.health_damage], &mut actors.downed.machine, now, false);
    }
}
