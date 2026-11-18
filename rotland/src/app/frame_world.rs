//! Survival needs, gun shooting, world entity interactions, and camp life updates.

use engine::Input;
use glam::Vec3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use super::state::{ActorsState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::{camp_ui, hub_ui, shooting, trunk_play, wildlife};

/// Ticks survival needs (hunger, thirst) and active crafting timers.
pub fn update_needs(
    dt: f32,
    play: &mut PlaySide,
    actors: &mut ActorsState,
    audio: &mut Option<audio::Audio>,
) {
    if let (Some(play_state), Some(p), Some(c)) = (
        play.play.as_mut(),
        actors.player_ctrl.as_mut(),
        actors.combat.as_mut(),
    ) {
        let dead = c.is_dead();
        let (damage, heal) = play_state.needs_tick(
            dt,
            p.is_sprinting,
            dead,
            (c.health.current(), c.health.max()),
            &mut p.stamina,
        );
        if heal > 0.0 {
            c.health.heal(heal);
        }
        if damage > 0.0 && c.take_bites(&[damage]).died {
            if let Some(pv) = actors.player_visual.as_mut() {
                pv.animator.play_die();
            }
            log::info!("the player died of hunger or thirst");
        }
        play_state.health_full = c.health.current() >= c.health.max();
        if let Some(toast) = play_state.camp.craft_tick(dt, &mut play_state.inventory, dead) {
            if let Some(a) = audio.as_mut() {
                a.event("craftDone", None);
            }
            play_state.hotbar.on_inventory_changed(&play_state.inventory);
            play_state.show_toast(toast);
        }
    }
}

/// Guns shooting, reload, bash, and world interaction prompts (crates, NPCs, cars, butchering).
pub fn update_world_interactions(
    input: &Input,
    dt: f32,
    render: &RenderState,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &SessionState,
    audio: &mut Option<audio::Audio>,
) {
    let alive_now = !actors.combat.as_ref().is_some_and(|c| c.is_dead());
    if let (Some(play_state), Some(p)) = (play.play.as_mut(), actors.player_ctrl.as_mut())
        && alive_now
        && session.scene == "play"
    {
        let captured = ui.cursor_captured;
        let shoot = shooting::ShootInput {
            // --autoattack clicks every other frame, so semi-automatic guns see a fresh press each time
            fire_pressed: (session.autoattack && ((session.time * 10.0) as u32).is_multiple_of(2))
                || (captured && input.mouse_pressed(MouseButton::Left)),
            fire_held: captured && input.mouse_held(MouseButton::Left),
            reload_pressed: input.pressed(ui.keys.reload),
            aiming: captured && input.mouse_held(MouseButton::Right),
        };
        let animal_bonus = play_state.camp.skills.get_bonus("animalDamage");
        if let Some(z) = actors.zombies.as_mut()
            && let Some(ev) = play_state.shoot_tick(
                dt,
                shoot,
                render.camera.position,
                render.camera.forward(),
                &mut wildlife::Prey {
                    zombies: z,
                    animals: actors.wildlife.as_mut().map(|w| &mut w.sim),
                    animal_bonus,
                },
                &world.physics,
            )
            && let Some(c) = actors.combat.as_mut()
        {
            c.kills += ev.kills;
            c.shots += ev.shots_fired;
            if ev.shots_fired > 0 {
                actors.fp_sway.trigger_shot();
            }
            // No bullets in the gun or the bag: the click swings it as a club (Godot GunBash)
            if ev.bash
                && let Some(gun) = play_state.gun.as_ref()
            {
                c.bash(gun, &mut p.stamina);
            }
            if let Some(a) = audio.as_mut() {
                let at = render.camera.position;
                if ev.shots_fired > 0
                    && let Some(sound) = play_state
                        .gun
                        .as_ref()
                        .and_then(|g| a.data.weapon_sound(&g.id, "shot"))
                        .map(str::to_string)
                {
                    a.play_at(&sound, at);
                }
                if ev.reload_started {
                    a.event("reload", Some(at));
                }
                if ev.empty_click {
                    a.event("dryFire", Some(at));
                }
            }
            c.noises.extend(ev.noise);
        }

        // Crates within reach: only the nearby chunks' spots are looked at
        let facing = game::yaw_forward(p.yaw);
        let near: Vec<world::LootSpot> = world
            .streamer
            .as_ref()
            .map(|st| {
                st.chunks
                    .values()
                    .flat_map(|c| c.crates.iter())
                    .filter(|c| c.position.distance_squared(p.position) < 25.0)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        // E talks to the camp person in front (trader, banker, bounty board) before crates and items
        let npc = actors
            .camp_npcs
            .as_ref()
            .and_then(|c| c.facing(p.position, facing));
        let pressed =
            input.pressed(ui.keys.interact) && ui.camp_ui.is_none() && ui.hub.is_none() && !ui.e_used;
        play_state.camp.player_at = p.position;

        // A car within reach: E drives it; with a fuel can, F pours it in (Godot CarDriver / CarFuel)
        let mut car_prompt = None;
        // The back of a parked car: E opens its trunk (Godot PlayerInteractor / TrunkScreen)
        if let (None, Some(cars)) = (&npc, actors.cars.as_ref())
            && cars.driven_car_id.is_none()
            && let Some(id) = trunk_play::trunk_in_front(cars, p.position, facing, render.car_markers.0)
        {
            car_prompt = Some(trunk_play::OPEN_TRUNK_PROMPT.to_string());
            if pressed {
                actors.open_trunk = Some(id);
                ui.hub = Some(hub_ui::HubUiState::new(hub_ui::HubScreen::Trunk));
            }
        }
        if let (None, None, Some(cars)) = (&npc, &car_prompt, actors.cars.as_mut())
            && cars.driven_car_id.is_none()
            && let Some(id) = cars
                .live_cars()
                .filter(|c| !c.condition.is_wrecked() && c.body.position.distance(p.position) < 3.5)
                .min_by(|a, b| {
                    a.body
                        .position
                        .distance(p.position)
                        .total_cmp(&b.body.position.distance(p.position))
                })
                .map(|c| c.id.clone())
        {
            let can = play_state.inventory.has("fuel_can", 1)
                && cars.get(&id).is_some_and(|c| c.tank.fraction() < 1.0);
            car_prompt = Some(if can {
                "[E] Drive   [F] Pour in fuel".to_string()
            } else {
                "[E] Drive".to_string()
            });
            if input.pressed(KeyCode::KeyF) && can && cars.refuel(&id, &mut play_state.inventory) {
                play_state.show_toast("Poured in 10 litres".into());
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
            }
            if pressed && cars.try_enter(p.position, facing).is_some() {
                play_state.show_toast(
                    "W/S drive, A/D steer, Space handbrake, H horn, L lights, E get out".into(),
                );
                car_prompt = None;
            }
        }
        let npc = if car_prompt.is_some() { None } else { npc };

        // A corpse in front (or one being butchered) comes next: E starts the butchering timer
        let mut corpse_prompt = None;
        if let (None, Some(w)) = (&npc, actors.wildlife.as_mut()) {
            let (prompt, toast) = w.butcher_tick(
                dt,
                pressed,
                p.position,
                facing,
                &mut play_state.inventory,
                &mut play_state.camp.skills,
            );
            if let Some(t) = toast {
                play_state.show_toast(t);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
            }
            corpse_prompt = prompt;
        }

        // A campfire in front: E cooks (opens the bag's crafting) or relights it with wood
        let mut fire_prompt = None;
        let mut open_crafting = false;
        if npc.is_none() && corpse_prompt.is_none() {
            let fires = play_state.camp.campfires.campfires_mut();
            if let Some(fire) = fires
                .iter_mut()
                .filter(|f| f.distance_flat(p.position) < 2.0)
                .min_by(|a, b| {
                    a.distance_flat(p.position)
                        .total_cmp(&b.distance_flat(p.position))
                })
            {
                fire_prompt = Some(fire.prompt(Some(&play_state.inventory)).to_string());
                if pressed {
                    match fire.interact(&mut play_state.inventory) {
                        game::CampfireInteractResult::OpenCrafting => open_crafting = true,
                        game::CampfireInteractResult::Relit => {
                            play_state.show_toast("The fire is burning again".into());
                            play_state.hotbar.on_inventory_changed(&play_state.inventory);
                        }
                        game::CampfireInteractResult::NeedsWood => {
                            play_state.show_toast("Needs wood".into())
                        }
                    }
                }
            }
        }
        if open_crafting {
            let mut hub = hub_ui::HubUiState::new(hub_ui::HubScreen::Tab(hub_ui::HubTab::Bag));
            hub.set_status("Cooking at the campfire");
            ui.hub = Some(hub);
        }
        let corpse_prompt = corpse_prompt.or(fire_prompt).or(car_prompt);
        let free = npc.is_none() && corpse_prompt.is_none();

        // A bed in front comes before crates; tilling with a hoe only when nothing else is offered
        let bed = free && play_state.camp.bed_in_front(p.position, facing).is_some();
        play_state.interact_tick(dt, pressed && free && !bed, p.position, facing, &near);
        if free && (bed || !play_state.has_prompt()) {
            let physics = &world.physics;
            let ground = |spot: Vec3| {
                physics
                    .raycast(spot + Vec3::Y * 3.0, Vec3::NEG_Y, 6.0)
                    .map(|h| (h.point, h.normal))
            };
            let farm_press = pressed && !play_state.picked_up;
            if let Some((prompt, toast)) = play_state.camp.farm_interact(
                p.position,
                facing,
                &mut play_state.inventory,
                farm_press,
                ground,
            ) {
                if !prompt.is_empty() {
                    play_state.set_prompt(prompt);
                }
                if let Some(t) = toast {
                    play_state.show_toast(t);
                    play_state.hotbar.on_inventory_changed(&play_state.inventory);
                }
            }
        }
        if let Some(text) = corpse_prompt {
            play_state.set_prompt(text);
        }
        if let (true, Some(a)) = (play_state.picked_up, audio.as_mut()) {
            a.event("pickup", None);
        }

        // E on the player's own dog opens its menu (Godot DogMenu)
        if let (None, Some(d)) = (&npc, actors.dogs.as_ref())
            && let Some((_, name)) = d.own_dog_in_front(p.position, facing)
        {
            play_state.set_prompt(format!("[E] {name}"));
            if pressed {
                ui.hub = Some(hub_ui::HubUiState::new(hub_ui::HubScreen::DogMenu));
            }
        }
        if let (Some((_, role, _)), Some(d)) = (&npc, actors.dogs.as_mut())
            && role == "breeder"
        {
            // The breeder sells puppies (Godot DogBreeder)
            let price = d.sim.config.price;
            play_state.set_prompt(if d.sim.has_dog() {
                "[E] Dog breeder (you already have a dog)".into()
            } else {
                format!("[E] Buy a puppy (${price})")
            });
            if pressed {
                let now = (play.clock.day as f64 - 1.0) * 24.0 + play.clock.hour as f64;
                let text = match d.sim.buy_from_breeder(
                    &mut play_state.camp.wallet,
                    p.position + facing * 1.5,
                    play.clock.day,
                    now,
                ) {
                    Ok(_) => "You bought a puppy. Press E on it for its menu, G to make it stay"
                        .to_string(),
                    Err(e) => e,
                };
                play_state.show_toast(text);
            }
        } else if let Some((_, role, prompt)) = npc {
            play_state.set_prompt(prompt);
            let screen = match role.as_str() {
                "trader" => Some(camp_ui::CampScreen::Shop),
                "banker" => Some(camp_ui::CampScreen::Bank),
                r if r.starts_with("bounty") => Some(camp_ui::CampScreen::Bounties),
                _ => None,
            };
            if let (true, Some(s)) = (pressed, screen) {
                ui.camp_ui = Some(camp_ui::CampUiState::new(s));
            }
        }
    }

    // Zombies chase the living player; while dead they lose the target
    if let (Some(play_state), Some(z), Some(w), Some(p)) = (
        play.play.as_mut(),
        actors.zombies.as_mut(),
        world.world_gen.as_ref(),
        actors.player_ctrl.as_ref(),
    ) {
        let forward = game::yaw_forward(p.yaw);
        let night = !(6.0..20.0).contains(&play.hour);
        play_state.spawn_tick(dt, actors.player, forward, night, z, &world.physics, w);
    }
    if !play.clock_frozen {
        let before = play.clock.day as f32 * 24.0 + play.clock.hour;
        play.clock.advance(dt);
        play.hour = play.clock.hour;
        // Fires burn in game hours (Godot Campfire: 5 h per load of wood)
        let hours = play.clock.day as f32 * 24.0 + play.clock.hour - before;
        if let Some(play_state) = play.play.as_mut() {
            play_state.camp.campfires.advance_hours(hours.max(0.0));
        }
    }
    if let Some(play_state) = play.play.as_mut() {
        for f in play_state.camp.campfires.campfires_mut() {
            f.visuals.update(dt);
        }
    }
    if let Some(c) = actors.camp_npcs.as_mut() {
        let alive = !actors.combat.as_ref().is_some_and(|cb| cb.is_dead());
        let player = alive.then_some(actors.player);
        c.update(dt, play.hour, player);
    }
}
