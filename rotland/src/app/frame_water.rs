//! Per-frame fishing and swimming wiring: hand stowing in deep water, the fishing rod, catches and the water HUD state.

use game::swimming::WaterWarning;
use game::{FishingConfig, FishingEvent, FishingInput, FishingSession, SwimHands, SwimHandsChange, SwimmingConfig};

use super::frame_input::apply_hand;
use super::state::{ActorsState, PlaySide, RenderState, WorldState};
use crate::play::HandChange;

/// Fishing and swimming state kept between frames.
pub struct WaterPlay {
    pub fishing_cfg: FishingConfig,
    pub swim_cfg: SwimmingConfig,
    pub fishing: FishingSession,
    pub hands: SwimHands,
    pub in_deep: bool,
    pub can_fish: bool,
    pub has_rod: bool,
    pub warning: WaterWarning,
    pub stamina_fraction: f32,
    last_health: f32,
}

impl Default for WaterPlay {
    fn default() -> Self {
        // data: fishing.json and swimming.json (defaults when a file is missing)
        let fishing_cfg = FishingConfig::from_repo().unwrap_or_default();
        let swim_cfg = SwimmingConfig::from_repo().unwrap_or_default();
        Self {
            fishing: FishingSession::new(fishing_cfg),
            fishing_cfg,
            swim_cfg,
            hands: SwimHands::new(),
            in_deep: false,
            can_fish: false,
            has_rod: false,
            warning: WaterWarning::None,
            stamina_fraction: 0.0,
            last_health: 0.0,
        }
    }
}

/// Runs the hand stowing, fishing and water HUD state for one frame. Call after the actors update.
pub fn update_water_play(
    dt: f32,
    attack: bool,
    time: f64,
    render: &mut RenderState,
    play: &mut PlaySide,
    actors: &mut ActorsState,
    world: &WorldState,
) {
    let health = actors.combat.as_ref().map_or(0.0, |c| c.health.current());
    let dead = actors.combat.as_ref().is_some_and(|c| c.is_dead());
    let hurt = health < play.water.last_health;
    play.water.last_health = health;

    if dead {
        play.water.hands.clear();
        play.water.fishing.tick(
            &FishingInput { has_rod: false, can_fish_here: false, click: false, hurt: true, random_01: 0.0 },
            dt,
        );
        play.water.can_fish = false;
        play.water.has_rod = false;
        play.water.warning = WaterWarning::None;
        return;
    }

    // Hands: stow the held item in deep water, give it back when leaving
    let in_deep = play.water.in_deep;
    let held = play.play.as_ref().and_then(|ps| ps.hotbar.equipped_item_id().map(str::to_owned));
    match play.water.hands.update(in_deep, held.as_deref()) {
        Some(SwimHandsChange::Stow) => {
            if let Some(ps) = play.play.as_mut() {
                ps.hotbar.holster();
            }
            apply_hand(HandChange::Weapon(None), render, play, actors, world);
        }
        Some(SwimHandsChange::Restore(id)) => {
            let change = play.play.as_mut().and_then(|ps| {
                if ps.inventory.has(&id, 1) && ps.hotbar.equipped_item_id().is_none() {
                    ps.hotbar.toggle_equip_item(&id, &ps.inventory, &ps.items);
                    Some(ps.equipped_hand_change())
                } else {
                    None
                }
            });
            if let Some(change) = change {
                apply_hand(change, render, play, actors, world);
            }
        }
        None => {
            // The player pressed a hotbar key while in deep water: put it away again
            let pressed = in_deep && play.play.as_ref().is_some_and(|ps| ps.hotbar.equipped_item_id().is_some());
            if pressed {
                if let Some(ps) = play.play.as_mut() {
                    ps.hotbar.holster();
                }
                apply_hand(HandChange::Weapon(None), render, play, actors, world);
            }
        }
    }

    let Some(p) = actors.player_ctrl.as_ref() else {
        play.water.can_fish = false;
        play.water.has_rod = false;
        play.water.warning = WaterWarning::None;
        return;
    };

    // Fishing: needs the rod in hand, shore water in front, and not driving
    let held_now = play.play.as_ref().and_then(|ps| ps.hotbar.equipped_item_id());
    let has_rod = held_now == Some("fishing_rod");
    let driving = actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
    let lake = world.world_gen.as_ref().and_then(|w| w.lake());
    let can_fish = has_rod
        && !in_deep
        && !driving
        && lake.is_some_and(|l| game::can_fish_at(p.position, p.yaw, l, play.water.fishing_cfg.max_water_distance));
    // Cheap pseudo-random roll for the bite wait, from the session clock
    let random_01 = ((time * 12.9898).sin() * 43758.5453).fract().abs() as f32;
    let input = FishingInput { has_rod, can_fish_here: can_fish, click: attack, hurt, random_01 };
    let event = play.water.fishing.tick(&input, dt);
    let fish_cue = match event.as_ref() {
        Some(FishingEvent::Cast) => Some(humanoid::ActionCue::FishCast),
        Some(FishingEvent::Bite) => Some(humanoid::ActionCue::FishReel),
        Some(FishingEvent::Caught | FishingEvent::Escaped | FishingEvent::Cancelled) => Some(humanoid::ActionCue::FishStop),
        None => None,
    };
    if let (Some(cue), Some(ps)) = (fish_cue, play.play.as_mut()) {
        ps.anim_cue = Some(cue);
    }
    play.water.has_rod = has_rod;
    play.water.can_fish = can_fish;

    let (pos, stamina) = (p.position, (p.stamina.current, p.stamina.max));
    if let Some(ps) = play.play.as_mut() {
        match event {
            Some(FishingEvent::Caught) => {
                let leftover = ps.inventory.add_with_registry("fish_raw", 1, &ps.items);
                ps.hotbar.on_inventory_changed(&ps.inventory);
                if leftover > 0 {
                    ps.world_items.add("fish_raw".to_string(), 1, pos);
                    ps.show_toast("Bag full: fish dropped".to_string());
                } else {
                    ps.show_toast("Caught a fish!".to_string());
                }
            }
            Some(FishingEvent::Escaped) => ps.show_toast("The fish got away".to_string()),
            _ => {}
        }
    }

    // Stamina warning and bar for the HUD
    let (current, max) = stamina;
    play.water.warning = game::swimming::water_warning(current, max, in_deep, &play.water.swim_cfg);
    play.water.stamina_fraction = if max > 0.0 { current / max } else { 0.0 };
}
