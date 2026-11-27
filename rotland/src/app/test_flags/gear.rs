//! Weapon, team and gun-shop dev flags (`--team`, `--wear`, `--mods`, `--gunsmith-town`).

use engine::Gpu;
use game::guild::GuildConfig;
use rotland_ui::gunsmith_ui::GunsmithTab;

use crate::app::state::{ActorsState, PlaySide, RenderState, SessionState, UiState, WorldState};

use super::recruits::build_rescued_recruits;
use super::tp::apply_town_tp;

pub(super) fn apply_team(actors: &ActorsState, play: &mut PlaySide, count: usize) {
    let templates = GuildConfig::from_repo()
        .map(|cfg| cfg.recruits)
        .unwrap_or_default();
    let to_add = build_rescued_recruits(count, &templates);
    let Some(ps) = play.play.as_mut() else {
        return;
    };
    let recruits = ps.camp.guild.manager_mut().recruits_mut();
    let max_slots = recruits.hired().len() + to_add.len();
    for recruit in to_add {
        if let Err(e) = recruits.add_rescued(recruit, max_slots) {
            log::warn!("--team: failed to add recruit: {e:?}");
        }
    }
    if let Some(squad) = play.companions.as_mut() {
        let hired = ps.camp.guild.manager().recruits().hired();
        squad.sync(hired, actors.player);
    }
}

fn ensure_weapon_slot(play_state: &mut crate::play::PlayState, weapon_id: &str) -> Option<usize> {
    let find_slot = |ps: &crate::play::PlayState| {
        ps.inventory.slots().iter().position(|slot| {
            slot.as_ref().is_some_and(|st| {
                st.item_id == weapon_id
                    || ps
                        .items
                        .get(&st.item_id)
                        .is_some_and(|d| d.weapon_id == weapon_id)
            })
        })
    };
    if let Some(idx) = find_slot(play_state) {
        return Some(idx);
    }
    if play_state.items.get(weapon_id).is_some() {
        play_state.inventory.add(weapon_id, 1);
        play_state.hotbar.on_inventory_changed(&play_state.inventory);
        return find_slot(play_state);
    }
    None
}

pub(super) fn apply_wear_and_mods(
    render: &mut RenderState,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &SessionState,
) {
    let Some(play_state) = play.play.as_mut() else {
        return;
    };
    let mut touched_weapon: Option<String> = None;
    if let Some((weapon_id, cond)) = session.test_flags.wear.as_ref() {
        if let Some(slot_idx) = ensure_weapon_slot(play_state, weapon_id)
            && let Some(Some(stack)) = play_state.inventory.slots_mut().get_mut(slot_idx)
        {
            stack.wear = Some(*cond);
            play_state.inventory.set_changed();
            touched_weapon = Some(weapon_id.clone());
        } else {
            log::warn!("--wear: unknown weapon '{weapon_id}'");
        }
    }
    if let Some((weapon_id, mod_ids)) = session.test_flags.mods.as_ref() {
        if let Some(slot_idx) = ensure_weapon_slot(play_state, weapon_id) {
            let attachments = play_state.attachments.clone();
            if let Some(Some(stack)) = play_state.inventory.slots_mut().get_mut(slot_idx) {
                for mod_id in mod_ids {
                    let Some(def) = attachments.get(mod_id) else {
                        log::warn!("--mods: unknown attachment '{mod_id}'");
                        continue;
                    };
                    if !def.fits_weapon(weapon_id) {
                        log::warn!("--mods: '{mod_id}' does not fit '{weapon_id}'");
                        continue;
                    }
                    stack.mods.retain(|m| attachments.get(m).is_none_or(|d| d.slot != def.slot));
                    stack.mods.push(mod_id.clone());
                }
                play_state.inventory.set_changed();
                touched_weapon = Some(weapon_id.clone());
            }
        } else {
            log::warn!("--mods: unknown weapon '{weapon_id}'");
        }
    }
    if let Some(weapon_id) = touched_weapon
        && let Some(mut slot_idx) = ensure_weapon_slot(play_state, &weapon_id)
    {
        let hb_cap = play_state.hotbar.slot_count();
        if slot_idx >= hb_cap && hb_cap > 0 {
            let dst = hb_cap - 1;
            play_state.inventory.swap_slots(slot_idx, dst);
            play_state.hotbar.on_inventory_changed(&play_state.inventory);
            slot_idx = dst;
        }
        let target_item = play_state
            .inventory
            .slot(slot_idx)
            .map(|s| s.item_id.clone());
        let should_equip = session.anim_shot.weapon.as_deref() == Some(weapon_id.as_str())
            || session.anim_shot.weapon.is_none();
        if should_equip
            && slot_idx < hb_cap
            && play_state.hotbar.equipped_item_id() != target_item.as_deref()
            && let Some(change) = play_state.press(slot_idx + 1)
        {
            crate::app::frame_input::apply_hand(change, render, play, actors, world);
        }
        if let Some(ps) = play.play.as_mut() {
            ps.refresh_equipped_stats(actors.combat.as_mut());
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_gunsmith_town(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &PlaySide,
    ui: &mut UiState,
    session: &SessionState,
    key: &str,
) {
    let valid = play
        .play
        .as_ref()
        .is_some_and(|p| p.camp.gunsmith_config.towns.contains_key(key));
    if !valid {
        log::warn!("--gunsmith-town={key}: unknown town gun shop");
        return;
    }
    apply_town_tp(gpu, render, world, actors, session, key);
    let tab = session.test_flags.gunsmith.unwrap_or(GunsmithTab::Buy);
    ui.gunsmith = Some(crate::gunsmith_play::GunsmithSession::new(
        tab,
        game::ShopKind::Town {
            key: key.to_string(),
        },
        None,
    ));
    ui.cursor_captured = false;
}
