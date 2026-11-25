//! Zombie simulation, horde/director HUD events, POI zombie sites, and zombie voices.

use glam::Vec3;

use super::state::{AudioState, PlaySide, WorldState};
use crate::zombies::types::ToxicBurst;

/// Body warmth lost when bitten by a frozen zombie. // new: DECISIONS 2026-11-25
const FROZEN_BITE_CHILL: f32 = 8.0;

/// Side-effects from one frame of zombie simulation for the caller to route to player/world systems.
pub(super) struct ZombieStepOut {
    pub hit_from: Option<Vec3>,
    pub big_knock: Vec3,
    pub chill: f32,
    pub toxic_bursts: Vec<ToxicBurst>,
}

/// Runs the zombie simulation, processes screams, guard blocks, knockback, and horde/director HUD events.
pub(super) fn step_zombies_and_events(
    dt: f32,
    z: &mut crate::zombies::Zombies,
    alive: bool,
    player: Vec3,
    combat: Option<&crate::combat::Combat>,
    mut player_ctrl: Option<&mut game::Player>,
    player_visual: Option<&mut crate::player_visual::PlayerVisual>,
    guard: &mut game::GuardState,
    heard_scratch: &mut Vec<game::zombie::Noise>,
    scream_noises: &mut Vec<game::zombie::Noise>,
    player_unreachable: bool,
    player_in_safe_zone: bool,
    world_gen: Option<&world::WorldGenerator>,
    physics: &physics::PhysicsWorld,
    play: &mut PlaySide,
    hud: &mut crate::hud::Hud,
    audio: &mut AudioState,
    bites: &mut Vec<f32>,
) -> ZombieStepOut {
    z.camp_center = world_gen.and_then(|g| g.settlements().first().map(|s| s.center));
    z.camp_wall_radius = hud.config.horde_events.camp_wall_radius_m;
    let mut heard = std::mem::take(heard_scratch);
    heard.clear();
    if let Some(c) = combat {
        heard.extend(c.noises.iter().cloned());
    }
    heard.append(scream_noises);
    let frame = z.update_full(
        dt,
        alive.then_some(player),
        &heard,
        physics,
        player_unreachable,
        player_in_safe_zone,
    );
    *heard_scratch = heard;
    scream_noises.extend(frame.noises.iter().cloned());
    if let Some(a) = audio.audio.as_mut() {
        for at in &frame.screams {
            a.play_at("zombie_groan", *at);
        }
    }
    let mut chill = frame.chill;
    {
        static GUARD_CFG: std::sync::OnceLock<game::GuardConfig> = std::sync::OnceLock::new();
        let guard_cfg = GUARD_CFG.get_or_init(|| game::GuardConfig::from_repo().unwrap_or_default());
        // frame.bites are pushed in z.list order, so the biters line up with them
        let mut biters = z
            .list
            .iter()
            .filter(|zz| zz.last_out.as_ref().is_some_and(|o| o.attack_hit.is_some()))
            .map(|zz| (zz.position, zz.kind.as_str()));
        let mut blocked_any = false;
        for b in &frame.bites {
            let (attacker, attacker_kind) = match biters.next() {
                Some((pos, kind)) => (Some(pos), Some(kind)),
                None => (None, None),
            };
            let dmg = if let Some(pc) = player_ctrl.as_deref_mut() {
                let (d, blocked) = guard.filter_bite(
                    b.damage,
                    pc.position,
                    pc.yaw,
                    attacker,
                    &mut pc.stamina,
                    guard_cfg,
                );
                blocked_any |= blocked;
                d
            } else {
                b.damage
            };
            if dmg > 0.0 && attacker_kind == Some("frozen") {
                chill += FROZEN_BITE_CHILL;
            }
            bites.push(dmg);
        }
        if blocked_any && let Some(pv) = player_visual {
            pv.animator.play_action(&guard_cfg.block.hit_clip, 1.0);
        }
    }
    bites.extend(frame.special_hits.iter().map(|h| h.amount));
    if let (true, Some(pc)) = (alive, player_ctrl) {
        for h in &frame.special_hits {
            pc.push(h.knockback);
        }
    }
    let mut hit_from: Option<Vec3> = None;
    let big_knock = frame
        .special_hits
        .iter()
        .map(|h| h.knockback)
        .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
        .unwrap_or(Vec3::ZERO);
    if alive {
        for z_actor in &z.list {
            if z_actor.last_out.as_ref().is_some_and(|o| o.attack_hit.is_some()) {
                hud.on_damage(z_actor.position);
                hit_from.get_or_insert(z_actor.position);
            }
        }
    }

    // Process zombie simulation events for HUD notifications
    for event in &frame.events {
        match event {
            crate::zombies::ZombieEvent::HordeNear { .. } => {
                let cd = hud.config.horde_events.cooldown_horde_near_s;
                if hud.cooldowns.try_trigger(crate::hud::EventKind::HordeNear, hud.last_time, cd) {
                    let horde_pos = z
                        .tracker
                        .last_near_horde_pos
                        .unwrap_or(player + Vec3::new(0.0, 0.0, -60.0));
                    let dir = crate::hud::direction_word_to_target(player, horde_pos);
                    hud.notify(
                        format!("A horde is coming from the {}", dir),
                        crate::hud::NotificationKind::Horde,
                    );
                    hud.on_horde_near(horde_pos);
                }
            }
            crate::zombies::ZombieEvent::HordeFormed { .. } => {}
            crate::zombies::ZombieEvent::SpecialSeen { kind } => {
                let cd = hud.config.horde_events.cooldown_special_seen_s;
                if hud.cooldowns.try_trigger(crate::hud::EventKind::SpecialSeen, hud.last_time, cd) {
                    let name = match kind.as_str() {
                        "brute" => "Brute",
                        "screamer" => "Screamer",
                        "crawler" => "Crawler",
                        "frozen" => "Frozen",
                        "husk" => "Husk",
                        "bloater" => "Bloater",
                        other => other,
                    };
                    hud.notify(
                        format!("{} nearby!", name),
                        crate::hud::NotificationKind::Warning,
                    );
                }
            }
            crate::zombies::ZombieEvent::CampAttacked => {
                let cd = hud.config.horde_events.cooldown_camp_attack_s;
                if hud.cooldowns.try_trigger(crate::hud::EventKind::CampAttack, hud.last_time, cd) {
                    hud.notify(
                        "The camp is under attack!",
                        crate::hud::NotificationKind::Danger,
                    );
                }
            }
        }
    }

    // Process director horde and wave events from PlayState
    if let Some(pl) = play.play.as_mut() {
        let horde_events = std::mem::take(&mut pl.pending_horde_events);
        for event in horde_events {
            match event {
                game::director::HordeEvent::CampAttack { .. } => {
                    let cd = hud.config.horde_events.cooldown_camp_attack_s;
                    if hud.cooldowns.try_trigger(crate::hud::EventKind::CampAttack, hud.last_time, cd) {
                        hud.notify("The camp is under attack!", crate::hud::NotificationKind::Danger);
                    }
                }
                game::director::HordeEvent::HordeIncoming { .. } => {
                    let cd = hud.config.horde_events.cooldown_wave_s;
                    if hud.cooldowns.try_trigger(crate::hud::EventKind::WaveIncoming, hud.last_time, cd) {
                        hud.notify("Wave incoming", crate::hud::NotificationKind::Horde);
                    }
                }
                game::director::HordeEvent::PhaseChange { from, to } => {
                    let cd = hud.config.horde_events.cooldown_phase_s;
                    if hud.cooldowns.try_trigger(crate::hud::EventKind::PhaseChange, hud.last_time, cd) {
                        match to {
                            game::director::DirectorState::Wave => {
                                hud.notify("Wave incoming", crate::hud::NotificationKind::Horde);
                            }
                            game::director::DirectorState::Building => {
                                hud.notify("Tension is rising...", crate::hud::NotificationKind::Warning);
                            }
                            game::director::DirectorState::Calm if from == game::director::DirectorState::Relax => {
                                hud.notify("The area has calmed down", crate::hud::NotificationKind::Info);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    ZombieStepOut {
        hit_from,
        big_knock,
        chill,
        toxic_bursts: frame.toxic_bursts,
    }
}

/// Wakes POI zombies near the player, despawns distant ones, and credits cleared POIs.
pub(super) fn step_poi_zombies(
    z: &mut crate::zombies::Zombies,
    world: &mut WorldState,
    pack: Option<&pack::ModelPack>,
    play: &mut PlaySide,
    player: Vec3,
    zombie_pos_scratch: &mut Vec<(u64, Vec3)>,
) {
    // Buildings near the player wake the zombies waiting inside; a building is cleared when they are all dead
    if let (Some(pz), Some(st), Some(pack)) = (
        world.poi_zombies.as_mut(),
        world.streamer.as_ref(),
        pack,
    ) {
        if world.poi_sites_generation != st.generation {
            world.poi_sites_generation = st.generation;
            let markers = &mut world.poi_markers;
            world.poi_zombie_sites.clear();
            for chunk in st.chunks.values() {
                for poi in &chunk.pois {
                    let spawns = markers
                        .entry(poi.building_id.clone())
                        .or_insert_with(|| {
                            pack.find(&poi.building_id)
                                .map(|m| {
                                    m.markers
                                        .iter()
                                        .filter(|k| k.name.starts_with("spawn_"))
                                        .map(|k| Vec3::from(k.translation))
                                        .collect()
                                })
                                .unwrap_or_default()
                        })
                        .clone();
                    world.poi_zombie_sites.push(game::PoiZombieSite::new(
                        &poi.id,
                        &poi.building_id,
                        poi.position,
                        poi.yaw,
                        spawns,
                    ));
                }
            }
        }
        let alive_count = z.list.iter().filter(|q| !q.health.is_dead()).count();
        let mut spawns = pz.update(player, &world.poi_zombie_sites, alive_count);
        crate::rescue_play::top_up_poi_zombies(
            &mut spawns,
            pz,
            &world.poi_zombie_sites,
            play.rescue.as_ref(),
            &world.physics,
        );
        for spawn in spawns {
            if let Some(id) = z.spawn_kind(&spawn.kind, spawn.position) {
                pz.on_zombie_spawned(&spawn.poi_id, id);
            }
        }
        // Far behind: gone for good (the building does not wait for them)
        let mut at = std::mem::take(zombie_pos_scratch);
        at.clear();
        at.extend(z.list.iter().map(|q| (q.id, q.position)));
        let far = pz.check_despawns(player, &at);
        *zombie_pos_scratch = at;
        z.despawn(&far);

        let mut cleared = std::mem::take(&mut world.cleared_poi_scratch);
        cleared.clear();
        cleared.extend(far.iter().filter_map(|id| pz.on_zombie_despawned(*id)));
        // Dead, or removed by the spawner
        let mut tracked = std::mem::take(&mut world.tracked_zombie_scratch);
        tracked.clear();
        tracked.extend(pz.zombie_to_poi.keys().copied());
        for &id in &tracked {
            if z.list.iter().find(|q| q.id == id).is_none_or(|q| q.health.is_dead())
                && let Some(poi) = pz.on_zombie_died(id)
            {
                cleared.push(poi);
            }
        }
        world.tracked_zombie_scratch = tracked;
        if let Some(play_state) = play.play.as_mut() {
            for poi in &cleared {
                play_state.camp.bounties.on_poi_cleared(poi);
                // The guild names buildings by type (data/guild.json poi_id), not by the spot's own id
                let building = world
                    .poi_zombie_sites
                    .iter()
                    .find(|s| s.poi_id == *poi)
                    .map(|s| s.building_id.as_str());
                if let Some(building) = building {
                    play_state.camp.guild.on_poi_cleared(building);
                }
                play.pois_cleared += 1;
            }
        }
        world.cleared_poi_scratch = cleared;
    }
}

/// Plays ambient zombie groans and attack snarls (Godot ZombieVoices).
pub(super) fn step_zombie_voices(
    dt: f32,
    z: &crate::zombies::Zombies,
    voices: Option<&mut game::ZombieVoices>,
    audio: &mut AudioState,
    player: Vec3,
    voice_zombies_scratch: &mut Vec<game::VoiceZombie>,
    voices_played: &mut u32,
) {
    if let (Some(voices), Some(a)) = (voices, audio.audio.as_mut()) {
        let mut list = std::mem::take(voice_zombies_scratch);
        list.clear();
        list.extend(z.list.iter().map(|q| {
            let state = if q.health.is_dead() {
                game::VoiceState::Dead
            } else {
                q.brain.state().into()
            };
            game::VoiceZombie::new(q.id, q.position, state)
        }));
        for ev in voices.update(dt, player, &list) {
            *voices_played += 1;
            a.play_at(&ev.sound_id, ev.position);
        }
        *voice_zombies_scratch = list;
    }
}
