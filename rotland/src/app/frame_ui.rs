//! Immediate-mode GUI rendering for HUD, hints, camp screens, player hub, and pause menus.

use engine::egui;
use glam::Vec3;

use super::frame_input::apply_hand;
use super::save_load::menu_action;
use super::state::{ActorsState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::{bubbles, camp_ui, hub_ui, hud, map_ui, trunk_play};

/// A tip's key names ({interact}, {slot_1}...) as the keys this game uses.
pub fn fill_keys(text: &str) -> String {
    let keys = [
        ("{interact}", "E"),
        ("{inventory}", "Tab"),
        ("{sprint}", "Shift"),
        ("{toggle_camera}", "V"),
        ("{dog_stay}", "G"),
    ];
    let mut out = text.to_string();
    for (k, v) in keys {
        out = out.replace(k, v);
    }
    for n in 1..=6 {
        out = out.replace(&format!("{{slot_{n}}}"), &n.to_string());
    }
    out
}

/// A button on the dog's menu or pack screen (Godot DogMenu / DogPackScreen). Returns a status line.
pub fn dog_command(
    dogs: &mut game::Dogs,
    cmd: hub_ui::DogCommand,
    state: &mut hub_ui::HubUiState,
    bag: &mut game::items::Inventory,
) -> Option<String> {
    use hub_ui::DogCommand as C;
    let id = dogs.dog()?.id();
    match cmd {
        C::OpenPack => {
            state.set_screen(hub_ui::HubScreen::DogPack);
            None
        }
        C::Feed => Some(dogs.feed(id, bag).unwrap_or_else(|e| e)),
        C::ToggleStay => {
            dogs.toggle_stay();
            dogs.dog().map(|d| format!("{:?}", d.mode()))
        }
        C::StorePack { bag_slot, count } => {
            let pack = dogs.dog_mut()?.pack_mut();
            Some(move_stack(bag, pack, bag_slot, count))
        }
        C::TakePack { pack_slot, count } => {
            let pack = dogs.dog_mut()?.pack_mut();
            Some(move_stack(pack, bag, pack_slot, count))
        }
        C::Close => {
            state.set_screen(hub_ui::HubScreen::DogMenu);
            None
        }
    }
}

/// Moves `count` (0 = all) of a slot's items into another inventory; what does not fit goes back.
pub fn move_stack(
    from: &mut game::items::Inventory,
    to: &mut game::items::Inventory,
    slot: usize,
    count: u32,
) -> String {
    let Some(stack) = from.slot(slot).cloned() else {
        return String::new();
    };
    let n = if count == 0 {
        stack.count
    } else {
        count.min(stack.count)
    };
    let taken = from.remove_from_slot(slot, n);
    let left = to.add(&stack.item_id, taken);
    if left > 0 {
        from.add(&stack.item_id, left);
    }
    format!("Moved {}", taken - left)
}

/// Draws all egui user interfaces: speech bubbles, HUD, camp screens, player hub, and pause menus.
pub fn draw_ui(
    ctx: &egui::Context,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &mut SessionState,
) {
    let on_main = ui.menus.as_ref().is_some_and(|m| {
        matches!(
            m.screen(),
            crate::menus::Screen::Main | crate::menus::Screen::SlotPicker { .. }
        )
    });

    // Speech bubbles over camp people chatting near the player (Godot CampChatBubble)
    if let (Some(npcs), false) = (actors.camp_npcs.as_ref(), on_main) {
        let range = npcs.director.data.bubble_range;
        let list: Vec<(Vec3, &str, f32)> = npcs
            .director
            .residents()
            .iter()
            .filter(|r| r.visible && r.bubble.is_saying() && r.position.distance(actors.player) < range)
            .map(|r| (r.position + Vec3::Y * 2.0, r.bubble.text.as_str(), r.bubble.alpha()))
            .collect();
        let r = ctx.content_rect();
        bubbles::draw_bubbles(
            ctx,
            render.camera.view_proj(r.width() / r.height().max(1.0)),
            (r.width(), r.height()),
            &list,
        );
    }

    if let (Some(play_state), Some(p), Some(c), false) = (
        play.play.as_ref(),
        actors.player_ctrl.as_ref(),
        actors.combat.as_ref(),
        on_main,
    ) {
        let show_fps = ui.menus.as_ref().is_none_or(|m| m.settings.show_fps);
        let fps = if show_fps { render.fps } else { 0.0 };
        let state = play_state.hud(
            c.health.current(),
            c.health.max(),
            p.stamina.current,
            p.stamina.max,
            play.hour,
            p.yaw,
            fps,
            c.is_dead(),
        );
        let mut state = state;
        // R10-04 inputs: what is in the hand, the scope, the camp / waypoint on the compass, the dog
        state.held = if play_state.gun.is_some() {
            hud::HeldKind::Gun
        } else if c.weapon_id().is_some() {
            hud::HeldKind::Melee
        } else {
            hud::HeldKind::None
        };
        state.scoped = ui.cursor_captured
            && ui.right_held
            && play_state.gun.as_ref().is_some_and(|g| g.scope.is_some());
        let at = p.position;
        if let Some(g) = world.world_gen.as_ref() {
            state.in_safe_zone = g.settlements().iter().any(|s| s.contains(at, 0.0, g.config()));
            if let Some(s) = g.settlements().first() {
                let dist = glam::Vec2::new(s.center.x - at.x, s.center.z - at.z).length();
                state.camp = Some(hud::CampHudInfo::new(
                    game::hud_math::relative_angle_deg(at.x, at.z, s.center.x, s.center.z, p.yaw),
                    dist,
                ));
            }
        }
        if let Some(wp) = play_state.map.waypoint {
            let dist = glam::Vec2::new(wp.x - at.x, wp.y - at.z).length();
            state.waypoint = Some(hud::WaypointHudInfo::new(
                game::hud_math::relative_angle_deg(at.x, at.z, wp.x, wp.y, p.yaw),
                dist,
            ));
        }
        if let Some(d) = actors.dogs.as_ref().and_then(|d| d.sim.dog()).filter(|d| !d.is_dead()) {
            state.dog = Some(hud::DogHudInfo::new(d.name().to_string(), d.health().current(), d.health().max()));
        }
        // Driving dashboard: shown bottom-right while a car is driven
        if let Some(car) = actors.cars.as_ref().and_then(|c| c.driven_car()) {
            state.vehicle = Some(hud::VehicleHudInfo {
                speed_kmh: car.speed_kmh(),
                forward_speed: car.forward_speed(),
                rpm: car.engine_rpm,
                fuel_litres: car.tank.litres(),
                fuel_fraction: car.tank.fraction(),
                health: car.condition.current(),
                max_health: car.condition.max(),
                lights_on: car.lights_on,
                handbrake: car.controls.handbrake,
                is_wrecked: car.condition.is_wrecked(),
                is_empty: car.tank.is_empty(),
                is_flipped: car.body.is_flipped(),
                camera_label: actors.car_camera.mode().label(),
            });
            state.key_labels.key_help = Some(
                "[W/S] Drive [A/D] Steer [Space] Brake [L] Lights [H] Horn [V] Cam [E] Exit".to_string(),
            );
        }
        hud::draw_hud(ctx, &state, &mut ui.icons);
        super::minimap::draw_minimap_hud(ctx, world, actors, play_state, ui);
        if let (Some(t), false) = (&play.hint_text, c.is_dead()) {
            hud::hint::draw_hint(ctx, t);
        }
    }

    // Camp screens over the game (not over the main menu)
    if let (Some(play_state), false) = (play.play.as_mut(), on_main) {
        let dead = actors.combat.as_ref().is_some_and(|c| c.is_dead());
        let action = {
            let view = camp_ui::CampView {
                wallet: Some(&play_state.camp.wallet),
                inventory: Some(&play_state.inventory),
                bank: Some(&play_state.camp.bank),
                trader_stock: Some(&play_state.camp.trader),
                crafting: Some(&play_state.camp.crafting),
                recipe_registry: Some(&play_state.camp.recipes),
                bounty_manager: Some(&play_state.camp.bounties),
                item_registry: Some(&play_state.items),
                economy_config: Some(&play_state.camp.economy),
                near_lit_campfire: play_state.camp.campfires.near_campfire(actors.player),
                player_pos: actors.player,
                is_dead: dead,
            };
            camp_ui::draw_bounty_tracker(ctx, &view);
            match ui.camp_ui.as_mut() {
                Some(state) => camp_ui::draw(ctx, state.screen(), state, &view),
                None => None,
            }
        };
        match action {
            Some(camp_ui::CampAction::Close) => ui.camp_ui = None,
            Some(a) => {
                let items = play_state.items.clone();
                let status = play_state.camp.apply(a, &mut play_state.inventory, &items, dead);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
                if let (Some(s), Some(state)) = (status, ui.camp_ui.as_mut()) {
                    state.set_status(s);
                }
            }
            None => {}
        }
    }

    if let (Some(o), false) = (&ui.overlays, on_main) {
        hub_ui::draw_overlays(ctx, o);
    }

    // The hub (bag, crafting, skills, bounties, profile)
    let mut hand = None;
    let mut close_hub = false;
    let empty_camps = [];
    let camps: &[glam::Vec2] = if ui.hub.is_some() {
        if world.camps.is_empty()
            && let Some(g) = world.world_gen.as_ref()
        {
            world.camps = g
                .settlements()
                .iter()
                .map(|s| glam::Vec2::new(s.center.x, s.center.z))
                .collect();
        }
        &world.camps
    } else {
        &empty_camps
    };
    let yaw = actors.player_ctrl.as_ref().map_or(0.0, |p| p.yaw);
    let day = play.clock.day;
    if let (Some(play_state), Some(state), false) = (play.play.as_mut(), ui.hub.as_mut(), on_main) {
        let dog = actors.dogs.as_ref().and_then(|d| {
            d.sim
                .dog()
                .filter(|g| !g.is_dead())
                .map(|g| (g, d.sim.config.feed_items.clone()))
        });
        // The map opens centred on the player (Godot MapView)
        if state.map_ui.centre == glam::Vec2::ZERO {
            state
                .map_ui
                .centre_on(glam::Vec2::new(actors.player.x, actors.player.z));
        }
        let dead = actors.combat.as_ref().is_some_and(|c| c.is_dead());
        let trunk_car = actors
            .open_trunk
            .as_deref()
            .and_then(|id| actors.cars.as_ref()?.get(id));
        let action = {
            let view = hub_ui::HubView {
                inventory: Some(&play_state.inventory),
                hotbar: Some(&play_state.hotbar),
                item_registry: Some(&play_state.items),
                crafting: Some(&play_state.camp.crafting),
                recipe_registry: Some(&play_state.camp.recipes),
                skills: Some(&play_state.camp.skills),
                skill_registry: Some(play_state.camp.skills.registry()),
                bounty_manager: Some(&play_state.camp.bounties),
                wallet: Some(&play_state.camp.wallet),
                bank: Some(&play_state.camp.bank),
                difficulty_settings: Some(&play_state.camp.difficulty),
                difficulty_config: Some(&play_state.camp.difficulty_config),
                days_survived: play.clock.day,
                play_time_seconds: session.play_time,
                kills: play_state.stats.kills(),
                kills_by_type: Some(play_state.stats.kills_by_type()),
                headshots: play_state.stats.headshots(),
                deaths: play_state.stats.deaths(),
                distance_metres: play_state.stats.distance_metres(),
                is_dead: dead,
                near_lit_campfire: play_state.camp.campfires.near_campfire(actors.player),
                dog: dog.as_ref().map(|(g, food)| hub_ui::DogView {
                    name: g.name().to_string(),
                    mode: format!("{:?}", g.mode()),
                    hunger: 0.0,
                    health: g.health().current(),
                    max_health: g.health().max(),
                    age: (day - g.bought_day).max(1) as u32,
                    has_food_in_bag: food.iter().any(|f| play_state.inventory.has(f, 1)),
                    pack_inventory: Some(g.pack()),
                    pack_max_weight: g.pack().max_weight(),
                    pack_slots: g.pack().slot_count(),
                }),
                map: Some(crate::map_ui::MapView {
                    player_pos: Some(glam::Vec2::new(actors.player.x, actors.player.z)),
                    player_yaw: yaw,
                    map_state: &play_state.map,
                    generator: world.world_gen.as_deref(),
                    camps,
                    bounties: play_state.camp.bounties.active(),
                }),
                health: actors
                    .combat
                    .as_ref()
                    .map(|c| (c.health.current(), c.health.max())),
                trunk_inventory: trunk_car.map(|c| &c.trunk),
                trunk_max_weight: trunk_car.map_or(40.0, |c| c.trunk.max_weight()),
                ..Default::default()
            };
            hub_ui::draw(ctx, state, &view)
        };
        match action {
            Some(hub_ui::HubAction::Close) => {
                ui.hub = None;
                actors.open_trunk = None;
            }
            Some(hub_ui::HubAction::TrunkMove { from_trunk, slot, count }) => {
                if let (Some(id), Some(cars)) = (actors.open_trunk.as_deref(), actors.cars.as_mut())
                    && let Some(car) = cars.get_mut(id)
                {
                    let action = hub_ui::HubAction::TrunkMove { from_trunk, slot, count };
                    if let Some(toast) = trunk_play::apply_trunk_action(
                        &action,
                        car,
                        &mut play_state.inventory,
                        &play_state.items,
                    ) {
                        play_state.show_toast(toast);
                    }
                    play_state.hotbar.on_inventory_changed(&play_state.inventory);
                }
            }
            Some(hub_ui::HubAction::Craft { recipe }) => {
                let items = play_state.items.clone();
                let status = play_state.camp.apply(
                    camp_ui::CampAction::Craft { recipe },
                    &mut play_state.inventory,
                    &items,
                    dead,
                );
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
                if let Some(s) = status {
                    state.set_status(s);
                }
            }
            Some(hub_ui::HubAction::DogCommand(hub_ui::DogCommand::Close))
                if state.screen() == hub_ui::HubScreen::DogMenu =>
            {
                close_hub = true
            }
            Some(hub_ui::HubAction::DogCommand(cmd)) => {
                if let Some(d) = actors.dogs.as_mut() {
                    let status = dog_command(&mut d.sim, cmd, state, &mut play_state.inventory);
                    play_state.hotbar.on_inventory_changed(&play_state.inventory);
                    if let Some(s) = status {
                        state.set_status(s);
                    }
                }
            }
            Some(hub_ui::HubAction::RenameDog { name }) => {
                if let Some(d) = actors.dogs.as_mut() {
                    d.sim.set_name(&name);
                }
                state.set_screen(hub_ui::HubScreen::DogMenu);
            }
            Some(hub_ui::HubAction::Map(map_ui::MapAction::AddMarker(at))) => {
                play_state.map.set_waypoint(at)
            }
            Some(hub_ui::HubAction::Map(_)) => play_state.map.clear_waypoint(),
            Some(hub_ui::HubAction::CancelCraft) => {
                let items = play_state.items.clone();
                if let Some(s) = play_state.camp.apply(
                    camp_ui::CampAction::CancelCraft,
                    &mut play_state.inventory,
                    &items,
                    dead,
                ) {
                    state.set_status(s);
                }
            }
            Some(a) => {
                let (change, status) = play_state.bag_action(&a, state.split_count(), actors.player);
                hand = change;
                if let Some(s) = status {
                    state.set_status(s);
                }
            }
            None => {}
        }
    }
    if close_hub {
        ui.hub = None;
        if actors.open_trunk.is_some() {
            actors.open_trunk = None;
        }
    }
    if let Some(change) = hand {
        apply_hand(change, render, play, actors, world);
    }
    let Some(m) = ui.menus.as_mut() else { return };
    if let Some(action) = m.ui(ctx, &session.slot_cards) {
        menu_action(action, session, world, actors, play, ui);
    }
    // F1 developer panel (R12-03)
    if !on_main
        && crate::dev_panel::update_dev_panel(
            ctx,
            &mut ui.dev_panel,
            render,
            world,
            actors,
            play,
            session,
        )
    {
        ui.cursor_captured = false;
    }
}
