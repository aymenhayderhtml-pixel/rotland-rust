//! Camp progression, player profile stats, hints, map reveal, and farm synchronization.

use glam::{Mat4, Vec3};

use super::frame_ui::fill_keys;
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::hub_ui;

/// Updates player hints, profile stats, camp progression, overlays, map reveals, and farm growth.
pub fn update_camp_and_stats(
    dt: f32,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    _session: &SessionState,
    audio: &mut AudioState,
) {
    let _phase = engine::timing::PhaseScope::new("camp");
    let alive = !actors.combat.as_ref().is_some_and(|c| c.is_dead());
    let night = !(6.0..20.0).contains(&play.hour);
    let menu_open = ui.camp_ui.is_some() || ui.hub.is_some() || ui.menus.as_ref().is_some_and(|m| m.is_open());

    if let (Some(play_state), Some(z)) = (play.play.as_mut(), actors.zombies.as_mut()) {
        // Tips for new players: what is near and what the player has (Godot HintTriggers)
        if let Some(hints) = play_state.hints.as_mut() {
            let at = actors.player;
            play.hint_ctx.nearest_zombie_dist = z
                .list
                .iter()
                .filter(|q| !q.health.is_dead())
                .map(|q| q.position.distance(at))
                .reduce(f32::min);
            play.hint_ctx.nearest_crate_dist = world.streamer.as_ref().and_then(|s| {
                s.chunks
                    .values()
                    .flat_map(|c| c.crates.iter())
                    .map(|c| c.position.distance(at))
                    .reduce(f32::min)
            });
            play.hint_ctx.nearest_board_dist = None;
            for val in play.hint_ctx.npc_distances.values_mut() {
                *val = f32::MAX;
            }
            if let Some(c) = &actors.camp_npcs {
                for n in c.director.fixed_npcs() {
                    let d = n.position.distance(at);
                    if n.role.starts_with("bounty") {
                        play.hint_ctx.nearest_board_dist =
                            Some(play.hint_ctx.nearest_board_dist.map_or(d, |b: f32| b.min(d)));
                    }
                    if let Some(e) = play.hint_ctx.npc_distances.get_mut(&n.role) {
                        *e = (*e).min(d);
                    } else {
                        play.hint_ctx.npc_distances.insert(n.role.clone(), d);
                    }
                }
            }
            if let Some(v) = play.hint_ctx.needs.get_mut("food") {
                *v = play_state.hunger.current();
            } else {
                play.hint_ctx.needs.insert("food".into(), play_state.hunger.current());
            }
            if let Some(v) = play.hint_ctx.needs.get_mut("water") {
                *v = play_state.thirst.current();
            } else {
                play.hint_ctx.needs.insert("water".into(), play_state.thirst.current());
            }
            play.hint_ctx.hour = play.hour;
            play.hint_ctx.bag_weight_ratio = Some(
                play_state.inventory.total_weight() / play_state.inventory.max_weight().max(1.0),
            );

            // Rebuild inventory items only when inventory changes (fingerprint / hash check)
            let mut inv_hasher = std::hash::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            for (i, slot) in play_state.inventory.slots().iter().enumerate() {
                if let Some(stack) = slot {
                    i.hash(&mut inv_hasher);
                    stack.item_id.hash(&mut inv_hasher);
                    stack.count.hash(&mut inv_hasher);
                }
            }
            let inv_hash = inv_hasher.finish();
            if inv_hash != play.hint_inventory_hash {
                play.hint_inventory_hash = inv_hash;
                play.hint_ctx.inventory_items.clear();
                for slot in play_state.inventory.slots().iter().flatten() {
                    play.hint_ctx.inventory_items.insert(slot.item_id.clone());
                }
            }

            play.hint_ctx.is_menu_open = menu_open;
            play.hint_ctx.is_dead = !alive;
            if let Some(g) = world.world_gen.as_ref() {
                let safe_bonus = play_state.camp.effects().safe_zone_half_bonus;
                hints.on_safe_zone_changed(
                    g.settlements()
                        .iter()
                        .any(|s| s.contains(at, safe_bonus, g.config())),
                );
            }
            if play_state.picked_up {
                hints.on_event("item_picked");
            }
            if let Some(h) = hints.tick(dt, &play.hint_ctx) {
                if play.active_hint_id.as_deref() != Some(&h.id) {
                    play.active_hint_id = Some(h.id.clone());
                    play.hint_text = Some(fill_keys(&h.text));
                }
            } else {
                play.active_hint_id = None;
                play.hint_text = None;
            }
        }

        // Profile numbers: deaths, distance travelled (teleports skipped), kills and headshots
        let dead_now = actors.combat.as_ref().is_some_and(|c| c.is_dead());
        if dead_now && !actors.was_dead {
            play_state.stats.record_death();
        }
        actors.was_dead = dead_now;
        if !dead_now {
            play_state.stats.update_position(glam::Vec2::new(actors.player.x, actors.player.z));
        }
        play_state.stats.record_day(play.clock.day);
        for (kind, headshot) in z.take_kills() {
            play_state.stats.record_kill(&kind, headshot);
            play_state.camp.on_zombie_killed(&kind, night);
            if let Some(a) = audio.audio.as_mut() {
                a.event("zombieDeath", Some(actors.player));
            }
        }
        play_state.camp.tick(play.clock.day, play.hour);
        if let Some((line, lvl)) = play_state.camp.tick_upgrades(play.elapsed_hours) {
            let name = play_state.camp.upgrade_name(line, lvl);
            ui.hud.notify(format!("Upgrade finished: {name}"), crate::hud::NotificationKind::Info);
        }
        if let Some(g) = actors.camp_guards.as_mut() {
            let eff = play_state.camp.effects();
            g.config.guard_damage = eff.guard_damage;
            g.config.guard_range = eff.guard_range;
            g.config.guard_fire_interval = eff.guard_fire_interval;
        }

        // XP gains show as toasts, level-ups as a banner (Godot XpToastStack / LevelUpBanner)
        let overlays = ui
            .overlays
            .get_or_insert_with(|| hub_ui::HubUiState::new(hub_ui::HubScreen::Tab(hub_ui::HubTab::Bag)));
        let first = play.skill_seen.is_empty();
        for def in play_state.camp.skills.registry().all() {
            let now = (
                play_state.camp.skills.get_level(&def.id),
                play_state.camp.skills.get_xp(&def.id),
            );
            let before = if let Some(entry) = play.skill_seen.get_mut(&def.id) {
                let prev = *entry;
                *entry = now;
                Some(prev)
            } else {
                play.skill_seen.insert(def.id.clone(), now);
                None
            };
            if let (false, Some((level, xp))) = (first, before) {
                if now.0 > level {
                    overlays.trigger_level_up(&def.name, now.0);
                    overlays.add_xp_toast(&def.id, &def.name, now.1.max(1.0));
                } else if now.1 > xp + 0.01 {
                    overlays.add_xp_toast(&def.id, &def.name, now.1 - xp);
                }
            }
        }
        overlays.update_overlays(dt);

        // The map fills in around the player
        play_state.map.reveal(glam::Vec2::new(actors.player.x, actors.player.z));

        // Crops grow with the clock; zombies walking over field beds can trample them
        let camp = &mut play_state.camp;
        camp.farm.sync_clock(&play.clock, Some(&camp.skills));
        if camp.farm.has_vulnerable_crops()
            && !camp
                .farm
                .trample_step(z.active().into_iter().map(|a| (a.id, a.position)))
                .is_empty()
        {
            play_state.show_toast("A zombie trampled a crop".into());
        }
    }

    if dt > 0.0 {
        render.fps += (1.0 / dt - render.fps) * (dt * 2.0).min(1.0);
    }

    // The trunk screen shuts when the player walks off or the car drives away (Godot TrunkScreen)
    if let Some(id) = &actors.open_trunk {
        let gone = actors
            .cars
            .as_ref()
            .and_then(|c| c.get(id))
            .is_none_or(|c| c.is_driven() || c.body.position.distance(actors.player) > 3.5);
        let shown = ui.hub.as_ref().is_some_and(|h| h.screen() == hub_ui::HubScreen::Trunk);
        if gone || !shown {
            actors.open_trunk = None;
            if shown {
                ui.hub = None;
            }
        }
    }

    // Felled trees tip over away from the player, then vanish
    for f in world.trees.update(dt) {
        if let (Some((coord, slot)), Some(st)) = (world.falling.get(&f.id), world.streamer.as_mut())
            && let Some(pl) = st
                .chunks
                .get_mut(coord)
                .and_then(|ch| slot.placed.and_then(|i| ch.placed.get_mut(i)))
        {
            let scale = if f.finished { 0.0001 } else { slot.scale };
            let axis = Vec3::Y.cross(f.fall_dir).normalize_or(Vec3::X);
            pl.matrix = Mat4::from_translation(slot.position)
                * Mat4::from_axis_angle(axis, f.tilt_angle)
                * Mat4::from_rotation_y(slot.yaw)
                * Mat4::from_scale(Vec3::splat(scale));
        }
        if f.finished {
            world.falling.remove(&f.id);
        }
    }
}
