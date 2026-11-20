//! Engine stats, display requests, and window cursor management.

use engine::GameStats;

use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::display;

/// Render scale and V-Sync from Settings, applied while playing (--scale / --vsync still win).
pub fn display(ui: &UiState) -> Option<engine::DisplayRequest> {
    ui.menus.as_ref().map(|m| display::display_request(&m.settings))
}

/// Returns whether the mouse cursor should be captured by the window.
pub fn cursor_grab(ui: &UiState, session: &SessionState) -> Option<bool> {
    if session.bench.is_some() {
        return Some(false);
    }
    if session.scene == "play" && !session.free_cam {
        Some(
            ui.cursor_captured
                && !ui.screen_open()
                && !ui.menus.as_ref().is_some_and(|m| m.wants_cursor()),
        )
    } else {
        None
    }
}

/// Returns whether the game loop should terminate.
pub fn should_quit(session: &SessionState) -> bool {
    session.quit
}

/// Collects runtime statistics for headless tests, benchmarks, and debug dumps.
pub fn get_stats(
    render: &RenderState,
    world: &WorldState,
    actors: &ActorsState,
    play: &PlaySide,
    _ui: &UiState,
    session: &SessionState,
    audio: &AudioState,
) -> GameStats {
    let mut s = GameStats::new();
    if let Some(r) = &render.renderer {
        s.insert("drawCalls".into(), r.last.draws.into());
        s.insert("triangles".into(), r.last.triangles.into());
        s.insert("shadowDrawCalls".into(), r.last.shadow_draws.into());
        s.insert("shadowTriangles".into(), r.last.shadow_triangles.into());
    }
    s.insert("colliders".into(), world.physics.collider_count().into());
    let (x, z) = (actors.player.x, actors.player.z);
    s.insert(
        "playerPosition".into(),
        format!("{:.2} {:.2} {:.2}", actors.player.x, actors.player.y, actors.player.z).into(),
    );
    if let Some(h) = world.physics.ground_height(x, z) {
        // Must equal the generator's height (same triangles): a quick health check of the collision world
        s.insert(
            "groundCheck".into(),
            format!("physics {h:.3} vs terrain {:.3}", world.height(x, z)).into(),
        );
    }
    s.insert("scene".into(), session.scene.clone().into());
    if let Some(a) = &audio.audio {
        // Counted even when muted, so scripted runs can check sounds without playing them
        s.insert("soundsPlayed".into(), a.played.into());
        s.insert("zombieVoices".into(), play.voices_played.into());
        s.insert("poisCleared".into(), play.pois_cleared.into());
        if let Some(pz) = &world.poi_zombies {
            s.insert("poiZombiesAwake".into(), pz.awake_count().into());
            s.insert("poisVisited".into(), pz.visited_ids().len().into());
        }
        if let Some(z) = &actors.zombies {
            let near = z
                .list
                .iter()
                .filter(|q| !q.health.is_dead())
                .map(|q| q.position.distance(actors.player))
                .fold(f32::MAX, f32::min);
            s.insert("nearestZombie".into(), format!("{near:.1}").into());
        }
        s.insert("lastSound".into(), a.last_played.clone().into());
        s.insert("soundLive".into(), a.is_live().into());
    }
    s.insert("seed".into(), session.seed.into());
    if let Some(st) = &world.streamer {
        s.insert("groundChunks".into(), st.chunks.len().into());
        s.insert(
            "placedModels".into(),
            st.chunks.values().map(|c| c.placed.len()).sum::<usize>().into(),
        );
    }
    if let Some(z) = &actors.zombies {
        s.insert("zombies".into(), z.list.len().into());
        s.insert("zombiesAlive".into(), z.alive().into());
    }
    if let Some(st) = &world.streamer {
        s.insert(
            "ladders".into(),
            st.chunks.values().map(|c| c.ladders.len()).sum::<usize>().into(),
        );
    }
    s.insert("climbing".into(), actors.climb.is_active().into());
    if let Some(play_state) = &play.play {
        s.insert(
            "campfires".into(),
            play_state.camp.campfires.campfires().len().into(),
        );
        if let Some(g) = actors.dogs.as_ref().and_then(|d| d.sim.dog()) {
            s.insert(
                "dogPosition".into(),
                format!("{:.2} {:.2} {:.2}", g.position().x, g.position().y, g.position().z).into(),
            );
            s.insert("dogClip".into(), g.current_clip().to_string().into());
        }
        if let Some(car) = actors.cars.as_ref().and_then(|c| c.get(game::CAMP_CAR_ID)) {
            let at = car.body.position;
            s.insert(
                "campCar".into(),
                format!(
                    "{:.2} {:.2} {:.2} {:.0}kmh fuel {:.1}",
                    at.x,
                    at.y,
                    at.z,
                    car.speed_kmh(),
                    car.tank.litres()
                )
                .into(),
            );
        }
        s.insert("mapChunks".into(), play_state.map.explore.chunk_count().into());
        s.insert("hint".into(), play.hint_text.clone().unwrap_or_default().into());
        s.insert("felledTrees".into(), world.trees.fallen.len().into());
        s.insert(
            "guardShots".into(),
            actors.camp_guards.as_ref().map_or(0, |g| g.total_shots_fired()).into(),
        );
        s.insert("parkedSolids".into(), actors.car_solids.count().into());
        s.insert("mapBuildings".into(), play_state.map.found.len().into());
        s.insert(
            "strayDogs".into(),
            actors.dogs.as_ref().map_or(0, |d| d.sim.strays().len()).into(),
        );
        s.insert(
            "profile".into(),
            format!(
                "kills {} headshots {} deaths {} distance {:.0}",
                play_state.stats.kills(),
                play_state.stats.headshots(),
                play_state.stats.deaths(),
                play_state.stats.distance_metres()
            )
            .into(),
        );
        s.insert(
            "bubbles".into(),
            actors.camp_npcs.as_ref().map_or(0, |c| {
                c.director
                    .residents()
                    .iter()
                    .filter(|r| r.visible && r.bubble.is_saying())
                    .count()
            })
            .into(),
        );
        s.insert(
            "campChats".into(),
            actors.camp_npcs.as_ref().map_or(0, |c| c.director.chats().len()).into(),
        );
        s.insert(
            "campfiresLit".into(),
            play_state
                .camp
                .campfires
                .campfires()
                .iter()
                .filter(|f| f.is_lit())
                .count()
                .into(),
        );
    }
    if let Some(w) = &actors.wildlife {
        s.insert("animals".into(), w.sim.live_count().into());
        let species: Vec<String> = w
            .sim
            .list
            .iter()
            .filter(|a| !a.is_dead())
            .map(|a| a.species_id.clone())
            .collect();
        s.insert("animalSpecies".into(), species.join(",").into());
    }
    if let Some(c) = &actors.combat {
        s.insert("playerHealth".into(), format!("{:.0}", c.health.current()).into());
        s.insert("kills".into(), c.kills.into());
        s.insert("shots".into(), c.shots.into());
        if let Some(play_state) = &play.play {
            s.insert("ammoBag".into(), play_state.inventory.count("pistol_ammo").into());
            s.insert("ammoMag".into(), play_state.last_shot.ammo.0.into());
        }
        s.insert("playerDead".into(), c.is_dead().into());
    }
    if let Some(p) = &actors.player_ctrl {
        s.insert(
            "playerSpeed".into(),
            format!("{:.2}", p.horizontal_speed()).into(),
        );
        s.insert("playerOnFloor".into(), p.on_floor.into());
        s.insert("playerCrouching".into(), p.crouching.into());
        s.insert("playerSprinting".into(), p.is_sprinting.into());
        s.insert(
            "playerStamina".into(),
            format!("{:.1}", p.stamina.current).into(),
        );
    }
    s
}
