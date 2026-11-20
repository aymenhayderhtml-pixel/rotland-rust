//! In-game F1 developer overlay state, read-only frame view, and per-frame coordinator.

pub mod actions;
pub mod draw;

pub use actions::{apply_dev_actions, maintain_god_mode, DevAction};
pub use draw::draw_dev_panel;

use std::sync::OnceLock;

use engine::egui;
use game::dev_panel::{
    dev_adjacent_poi_spot, FpsHistory, DEV_ALL_ITEM_IDS, DEV_DEFAULT_POI_OFFSET,
    DEV_POI_APPROACH_BUFFER, DEV_ZOMBIE_TYPES,
};
use glam::{IVec2, Vec3};
use world::WorldGenerator;

use crate::app::state::{ActorsState, PlaySide, RenderState, SessionState, WorldState};

/// Milliseconds per second used to convert frame delta dt to ms.
const MS_PER_SECOND: f32 = 1000.0; // new: DECISIONS 2026-11-20

/// Default quantity for the item spawner stepper.
const DEFAULT_GIVE_COUNT: u32 = 1; // new: DECISIONS 2026-11-20

/// Chunk radius scanned around the player when falling back to WorldGenerator::chunk_contents.
const POI_SEARCH_CHUNK_RADIUS: i32 = 4; // new: DECISIONS 2026-11-20

/// Persistent state for the F1 developer overlay.
#[derive(Clone, Debug)]
pub struct DevPanelState {
    pub open: bool,
    pub history: FpsHistory,
    pub custom_x: String,
    pub custom_z: String,
    pub selected_zombie_type: String,
    pub item_search: String,
    pub give_count: u32,
    pub god_mode: bool,
    pub status_msg: Option<String>,
    cached_poi_chunk: Option<IVec2>,
    cached_poi: Option<(String, Vec3)>,
}

impl Default for DevPanelState {
    fn default() -> Self {
        Self::new()
    }
}

impl DevPanelState {
    /// Creates a closed developer panel with default selections.
    pub fn new() -> Self {
        Self {
            open: false,
            history: FpsHistory::new(),
            custom_x: String::new(),
            custom_z: String::new(),
            selected_zombie_type: DEV_ZOMBIE_TYPES[0].to_string(),
            item_search: String::new(),
            give_count: DEFAULT_GIVE_COUNT,
            god_mode: false,
            status_msg: None,
            cached_poi_chunk: None,
            cached_poi: None,
        }
    }
}

/// Read-only snapshot gathered each frame for rendering the developer overlay.
pub struct DevPanelView<'a> {
    pub player_pos: Vec3,
    pub player_yaw: f32,
    pub fps: f32,
    pub draws: u32,
    pub triangles: u64,
    pub shadow_draws: u32,
    pub shadow_triangles: u64,
    pub current_hour: f32,
    pub clock_frozen: bool,
    pub all_item_ids: &'a [String],
    pub zombie_types: &'a [&'static str],
    pub nearest_poi: Option<(String, Vec3)>,
    pub camp_spawn: Option<Vec3>,
}

/// Ticks frame-time history, renders the dev window when open, applies actions,
/// and keeps god mode active.
///
/// Returns true if the user closed the window via its title-bar (X) button this frame
/// so the caller can leave the cursor uncaptured until the next click.
pub fn update_dev_panel(
    ctx: &egui::Context,
    state: &mut DevPanelState,
    render: &RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &mut SessionState,
) -> bool {
    if session.last_dt > 0.0 {
        state.history.push(session.last_dt * MS_PER_SECOND);
    }

    let was_open = state.open;
    if state.open {
        let (player_pos, player_yaw) = actors
            .player_ctrl
            .as_ref()
            .map_or((actors.player, 0.0), |p| (p.position, p.yaw));
        let (draws, triangles, shadow_draws, shadow_triangles) =
            render.renderer.as_ref().map_or((0, 0, 0, 0), |r| {
                (
                    r.last.draws,
                    r.last.triangles,
                    r.last.shadow_draws,
                    r.last.shadow_triangles,
                )
            });
        let nearest_poi = resolve_nearest_poi(state, world, player_pos);
        let camp_spawn = camp_spawn_pos(world);
        let view = DevPanelView {
            player_pos,
            player_yaw,
            fps: render.fps,
            draws,
            triangles,
            shadow_draws,
            shadow_triangles,
            current_hour: play.hour,
            clock_frozen: play.clock_frozen,
            all_item_ids: all_item_ids(),
            zombie_types: DEV_ZOMBIE_TYPES,
            nearest_poi,
            camp_spawn,
        };
        let actions = draw_dev_panel(ctx, state, &view);
        if !actions.is_empty() {
            apply_dev_actions(&actions, state, world, actors, play, session);
        }
    }

    maintain_god_mode(state.god_mode, actors, play);
    was_open && !state.open
}

fn all_item_ids() -> &'static [String] {
    static IDS: OnceLock<Vec<String>> = OnceLock::new();
    IDS.get_or_init(|| {
        DEV_ALL_ITEM_IDS
            .iter()
            .map(|id| (*id).to_string())
            .collect()
    })
}

fn camp_spawn_pos(world: &WorldState) -> Option<Vec3> {
    let camp = world.camps.first()?;
    let y = world
        .physics
        .ground_height(camp.x, camp.y)
        .unwrap_or_else(|| world.height(camp.x, camp.y));
    Some(Vec3::new(camp.x, y, camp.y))
}

fn resolve_nearest_poi(
    state: &mut DevPanelState,
    world: &WorldState,
    player_pos: Vec3,
) -> Option<(String, Vec3)> {
    let mut best_marker: Option<(&str, Vec3, f32)> = None;
    for (name, positions) in &world.poi_markers {
        for &pos in positions {
            let d2 = (pos.x - player_pos.x).powi(2) + (pos.z - player_pos.z).powi(2);
            if best_marker.is_none_or(|(_, _, best_d2)| d2 < best_d2) {
                best_marker = Some((name.as_str(), pos, d2));
            }
        }
    }
    if let Some((name, pos, _)) = best_marker {
        let offset = poi_adjacent_offset(world, name);
        let spot = dev_adjacent_poi_spot(player_pos, pos, offset);
        return Some((name.to_string(), spot));
    }

    let wgen = world.world_gen.as_ref()?;
    let chunk = IVec2::new(
        (player_pos.x / WorldGenerator::CHUNK_SIZE).floor() as i32,
        (player_pos.z / WorldGenerator::CHUNK_SIZE).floor() as i32,
    );
    if state.cached_poi_chunk != Some(chunk) {
        let mut best: Option<(String, Vec3, f32)> = None;
        for dz in -POI_SEARCH_CHUNK_RADIUS..=POI_SEARCH_CHUNK_RADIUS {
            for dx in -POI_SEARCH_CHUNK_RADIUS..=POI_SEARCH_CHUNK_RADIUS {
                let (pois, _) = wgen.chunk_contents(IVec2::new(chunk.x + dx, chunk.y + dz));
                for poi in pois {
                    let d2 = (poi.position.x - player_pos.x).powi(2)
                        + (poi.position.z - player_pos.z).powi(2);
                    if best.as_ref().is_none_or(|(_, _, best_d2)| d2 < *best_d2) {
                        best = Some((poi.building_id, poi.position, d2));
                    }
                }
            }
        }
        state.cached_poi = best.map(|(name, pos, _)| {
            let offset = poi_adjacent_offset(world, &name);
            (name, dev_adjacent_poi_spot(player_pos, pos, offset))
        });
        state.cached_poi_chunk = Some(chunk);
    }
    state.cached_poi.clone()
}

fn poi_adjacent_offset(world: &WorldState, building_id: &str) -> f32 {
    if let Some(wgen) = world.world_gen.as_ref()
        && let Some(def) = wgen.poi_config().get(building_id)
    {
        return def.reach() + wgen.poi_config().lot_margin + DEV_POI_APPROACH_BUFFER;
    }
    DEV_DEFAULT_POI_OFFSET
}
