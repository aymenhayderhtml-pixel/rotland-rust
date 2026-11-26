//! Dev-only gameplay/screenshot test flags (`--down-at`, `--team`, `--tp-rescue`, `--hold-e`, `--biome`, `--weather`).
//!
//! All flags default to off so normal gameplay is untouched.

use std::f32::consts::TAU;

use engine::{EngineArgs, Gpu};
use game::guild::{GuildConfig, GuildRecruit, RecruitDef, RecruitRole};
use game::rescue_place::RescueSpot;
use glam::{Vec2, Vec3};
use rotland_ui::gunsmith_ui::GunsmithTab;
use world::biome::layout::BiomeSample;
use world::weather::WeatherKind;
use world::BiomeId;

use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};

/// Distance in metres to place the player from the nearest unrescued survivor for `--tp-rescue`.
const RESCUE_TP_DISTANCE_M: f32 = 6.0; // new: DECISIONS 2026-11-25

/// Extra metres past `core_radius + blend_band` when teleporting into an outer biome.
const BIOME_TP_PAST_BLEND_M: f32 = 300.0; // new: DECISIONS 2026-11-25

/// Search radii (m) to detect where the outer sectors (`outer >= 1.0`) begin.
const OUTER_PROBE_RADII: [f32; 7] = [500.0, 1000.0, 2000.0, 3500.0, 6000.0, 10_000.0, 16_000.0]; // new: DECISIONS 2026-11-25
const SECTOR_SWEEP_STEPS: usize = 72; // new: DECISIONS 2026-11-25
const RAY_STEP_M: f32 = 25.0; // new: DECISIONS 2026-11-25
const RAY_MAX_M: f32 = 20_000.0; // new: DECISIONS 2026-11-25
const CORE_SEARCH_STEP_M: f32 = 32.0; // new: DECISIONS 2026-11-25
const CORE_SEARCH_MAX_M: f32 = 4000.0; // new: DECISIONS 2026-11-25

/// Distance in metres to place the player from town/trailhead/river targets.
const WORLD_TP_DISTANCE_M: f32 = 25.0; // new: DECISIONS 2026-11-25

/// Fallback recruit definition if `data/guild.json` has no `recruits` entries.
const FALLBACK_NAME: &str = "Mark Vance";
const FALLBACK_LEVEL: u32 = 1;
const FALLBACK_WAGE: u32 = 15; // data: guild.json daily_wage
const FALLBACK_HEALTH: u32 = 100; // data: guild.json health

/// Parsed dev-only test flags.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TestFlags {
    pub down_at: Option<f32>,
    pub team: usize,
    pub tp_rescue: bool,
    pub hold_e: bool,
    pub biome: Option<BiomeId>,
    pub weather: Option<WeatherKind>,
    /// `--spawn-zombie=<kind>`: one zombie of that kind in front of the player (R32-07).
    pub spawn_zombie: Option<String>,
    /// `--town=<key>`: teleport 25 m from that town's centre, facing it (R33-05).
    pub town: Option<String>,
    /// `--trailhead=<n>`: teleport 25 m before trailhead `n`, facing outward along the road (R33-05).
    pub trailhead: Option<usize>,
    /// `--bridge=<n>`: teleport onto the `n`-th road-river bridge deck (R33-05).
    pub bridge: Option<usize>,
    /// `--river=<n>`: teleport 25 m from river `n` on a bank, facing the water (R33-05).
    pub river: Option<usize>,
    /// `--gunsmith=<tab>`: open the camp gunsmith UI on `buy|sell|repair|upgrade` (R37-08).
    pub gunsmith: Option<GunsmithTab>,
    /// `--gunsmith-town=<key>`: teleport to that town and open its gun shop UI (R37-08).
    pub gunsmith_town: Option<String>,
    /// `--wear=<weapon>:<0..1>`: set condition on `<weapon>` in the bag (adding if absent) (R37-08).
    pub wear: Option<(String, f32)>,
    /// `--mods=<weapon>:<id,id>`: fit attachments on `<weapon>` in the bag (adding if absent) (R37-08).
    pub mods: Option<(String, Vec<String>)>,
}

impl TestFlags {
    /// Parses dev-only test flags from CLI args.
    pub fn from_args(args: &EngineArgs) -> Self {
        let biome = args.value("--biome").and_then(|raw| {
            let key = raw.trim().to_ascii_lowercase();
            let parsed = BiomeId::from_key(&key);
            if parsed.is_none() {
                log::warn!("--biome: unknown biome key '{raw}'");
            }
            parsed
        });
        let weather = args.value("--weather").and_then(|raw| {
            let key = raw.trim().to_ascii_lowercase();
            let parsed = WeatherKind::from_key(&key);
            if parsed.is_none() {
                log::warn!("--weather: unknown weather kind '{raw}'");
            }
            parsed
        });
        let gunsmith = args.value("--gunsmith").and_then(|raw| {
            let parsed = GunsmithTab::parse_name(raw);
            if parsed.is_none() {
                log::warn!("--gunsmith: unknown tab '{raw}' (expected buy|sell|repair|upgrade)");
            }
            parsed
        });
        Self {
            down_at: args.value("--down-at").and_then(parse_non_negative_f32),
            team: args
                .value("--team")
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(0),
            tp_rescue: args.flag("--tp-rescue"),
            hold_e: args.flag("--hold-e"),
            biome,
            weather,
            spawn_zombie: args
                .value("--spawn-zombie")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            town: args
                .value("--town")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            trailhead: args
                .value("--trailhead")
                .and_then(|s| s.trim().parse::<usize>().ok()),
            bridge: args
                .value("--bridge")
                .and_then(|s| s.trim().parse::<usize>().ok()),
            river: args
                .value("--river")
                .and_then(|s| s.trim().parse::<usize>().ok()),
            gunsmith,
            gunsmith_town: args
                .value("--gunsmith-town")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            wear: args.value("--wear").and_then(parse_wear_flag),
            mods: args.value("--mods").and_then(parse_mods_flag),
        }
    }
}

fn parse_wear_flag(raw: &str) -> Option<(String, f32)> {
    let (weapon, val) = raw.split_once(':')?;
    let weapon = weapon.trim().to_ascii_lowercase();
    let cond: f32 = val.trim().parse().ok()?;
    if weapon.is_empty() || !cond.is_finite() {
        log::warn!("--wear: invalid value '{raw}'");
        return None;
    }
    Some((weapon, cond.clamp(0.0, 1.0)))
}

fn parse_mods_flag(raw: &str) -> Option<(String, Vec<String>)> {
    let (weapon, list) = raw.split_once(':')?;
    let weapon = weapon.trim().to_ascii_lowercase();
    let mods: Vec<String> = list
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if weapon.is_empty() || mods.is_empty() {
        log::warn!("--mods: invalid value '{raw}'");
        return None;
    }
    Some((weapon, mods))
}

fn parse_non_negative_f32(raw: &str) -> Option<f32> {
    let val: f32 = raw.trim().parse().ok()?;
    (val.is_finite() && val >= 0.0).then_some(val)
}

fn parse_recruit_role(raw: &str) -> RecruitRole {
    match raw.trim().to_ascii_lowercase().as_str() {
        "scavenger" => RecruitRole::Scavenger,
        "guard" => RecruitRole::Guard,
        _ => RecruitRole::Fighter,
    }
}

/// Computes the player position (`6 m` away) and Godot yaw facing the nearest unrescued survivor.
pub fn tp_rescue_pose(player_pos: Vec3, spots: &[RescueSpot]) -> Option<(Vec3, f32)> {
    let nearest = spots.iter().min_by(|a, b| {
        let da = (a.position.x - player_pos.x).powi(2) + (a.position.z - player_pos.z).powi(2);
        let db = (b.position.x - player_pos.x).powi(2) + (b.position.z - player_pos.z).powi(2);
        da.total_cmp(&db)
    })?;

    let mut from_spot = Vec3::new(
        player_pos.x - nearest.position.x,
        0.0,
        player_pos.z - nearest.position.z,
    );
    if from_spot.length_squared() <= 1e-4 {
        from_spot = Vec3::Z;
    } else {
        from_spot = from_spot.normalize();
    }

    let pos = nearest.position + from_spot * RESCUE_TP_DISTANCE_M;
    let to_spot = nearest.position - pos;
    let yaw = (-to_spot.x).atan2(-to_spot.z);
    Some((pos, yaw))
}

/// Finds a teleport target `(xz, godot_yaw)` for `--biome=<key>`, facing outward from `camp`.
///
/// - For `Plains` / `Forest`: nearest point inside the core (`outer == 0.0`) of that biome.
/// - For outer biomes (`Desert`, `Snow`, `Swamp`, `Mountains`): searches outward from `camp`
///   along the biome's sector centre angle to the first point at `core_radius + blend_band + 300 m`.
pub fn find_biome_point(
    camp: Vec2,
    target: BiomeId,
    mut sample_fn: impl FnMut(f32, f32) -> BiomeSample,
) -> Option<(Vec2, f32)> {
    if matches!(target, BiomeId::Plains | BiomeId::Forest) {
        return find_core_biome_point(camp, target, &mut sample_fn);
    }

    let probe_r = OUTER_PROBE_RADII.into_iter().find(|&r| {
        (0..8).any(|k| {
            let a = k as f32 * (TAU / 8.0);
            let p = camp + Vec2::new(a.cos(), a.sin()) * r;
            sample_fn(p.x, p.y).outer >= 1.0 - 1e-4
        })
    })?;

    let sweep_r = probe_r + 500.0; // new: DECISIONS 2026-11-25
    let mut sum_cos = 0.0f32;
    let mut sum_sin = 0.0f32;
    let mut hits = 0usize;
    for i in 0..SECTOR_SWEEP_STEPS {
        let angle = i as f32 * (TAU / SECTOR_SWEEP_STEPS as f32);
        let p = camp + Vec2::new(angle.cos(), angle.sin()) * sweep_r;
        let s = sample_fn(p.x, p.y);
        if s.outer >= 1.0 - 1e-4 && s.primary == target {
            sum_cos += angle.cos();
            sum_sin += angle.sin();
            hits += 1;
        }
    }
    if hits == 0 {
        return None;
    }

    let center_angle = sum_sin.atan2(sum_cos);
    let dir = Vec2::new(center_angle.cos(), center_angle.sin());

    let mut r = 0.0f32;
    let mut outer_edge = None;
    while r <= RAY_MAX_M {
        let p = camp + dir * r;
        if sample_fn(p.x, p.y).outer >= 1.0 - 1e-4 {
            outer_edge = Some(r);
            break;
        }
        r += RAY_STEP_M;
    }
    let mut hi = outer_edge?;
    let mut lo = (hi - RAY_STEP_M).max(0.0);
    for _ in 0..10 {
        let mid = 0.5 * (lo + hi);
        let p = camp + dir * mid;
        if sample_fn(p.x, p.y).outer >= 1.0 - 1e-4 {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    let pos = camp + dir * (hi + BIOME_TP_PAST_BLEND_M);
    let yaw = (-dir.x).atan2(-dir.y);
    Some((pos, yaw))
}

fn find_core_biome_point(
    camp: Vec2,
    target: BiomeId,
    sample_fn: &mut impl FnMut(f32, f32) -> BiomeSample,
) -> Option<(Vec2, f32)> {
    let at_camp = sample_fn(camp.x, camp.y);
    if at_camp.outer <= 0.0 && at_camp.primary == target {
        return Some((camp, 0.0));
    }

    let mut r = CORE_SEARCH_STEP_M;
    while r <= CORE_SEARCH_MAX_M {
        let steps = ((TAU * r / CORE_SEARCH_STEP_M).round() as usize).clamp(12, 180);
        for i in 0..steps {
            let angle = i as f32 * (TAU / steps as f32);
            let dir = Vec2::new(angle.cos(), angle.sin());
            let p = camp + dir * r;
            let s = sample_fn(p.x, p.y);
            if s.outer <= 0.0 && s.primary == target {
                let yaw = (-dir.x).atan2(-dir.y);
                return Some((p, yaw));
            }
        }
        r += CORE_SEARCH_STEP_M;
    }
    None
}

/// Builds `count` free rescued-style recruits from `data/guild.json` recruit templates.
pub fn build_rescued_recruits(count: usize, templates: &[RecruitDef]) -> Vec<GuildRecruit> {
    if count == 0 {
        return Vec::new();
    }
    let fallback = [RecruitDef {
        id: "rec_mark".to_string(),
        name: FALLBACK_NAME.to_string(),
        role: "fighter".to_string(),
        level: FALLBACK_LEVEL,
        daily_wage: FALLBACK_WAGE,
        health: FALLBACK_HEALTH,
        hire_cost: 0,
    }];
    let pool: &[RecruitDef] = if templates.is_empty() {
        &fallback
    } else {
        templates
    };

    (0..count)
        .map(|i| {
            let tmpl = &pool[i % pool.len()];
            GuildRecruit::new(
                format!("dev_team_{}", i + 1),
                &tmpl.name,
                parse_recruit_role(&tmpl.role),
                tmpl.level,
                tmpl.daily_wage,
                tmpl.health,
                0,
            )
        })
        .collect()
}

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
}

fn teleport_player_and_stream(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    xz: Vec2,
    yaw: f32,
) {
    let mut pos = Vec3::new(xz.x, world.height(xz.x, xz.y), xz.y);
    if let (Some(pack), Some(streamer)) = (&render.pack, world.streamer.as_mut()) {
        while !streamer.is_ready_around(pos) {
            streamer.update(
                &gpu.device,
                pack,
                &mut render.library,
                &mut world.physics,
                pos,
            );
            std::thread::yield_now();
        }
    }
    pos.y = world
        .physics
        .ground_height(pos.x, pos.z)
        .unwrap_or_else(|| world.height(pos.x, pos.z));

    actors.player = pos;
    if let Some(p) = actors.player_ctrl.as_mut() {
        p.position = pos;
        p.yaw = yaw;
        p.velocity = Vec3::ZERO;
    }
    if let Some(rig) = actors.camera_rig.as_mut() {
        rig.yaw = yaw;
        if session.scene == "play" && !session.free_cam {
            render.camera = rig.camera(&pos, &world.physics);
        }
    }
}

fn apply_biome_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    target: BiomeId,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let camp = world_gen
        .settlements()
        .first()
        .map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
    let Some((xz, yaw)) = find_biome_point(camp, target, |x, z| world_gen.biome_sample(x, z))
    else {
        log::warn!("--biome={}: no matching biome point found", target.key());
        return;
    };
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

/// Computes `(pos_xz, godot_yaw)` 25 m from `town.center` along `-street_dir`, facing the town centre.
pub fn town_tp_pose(town: &world::towns::TownSite) -> (Vec2, f32) {
    let dir = town.street_dir.normalize_or(Vec2::X);
    let pos = town.center - dir * WORLD_TP_DISTANCE_M;
    let yaw = (-dir.x).atan2(-dir.y);
    (pos, yaw)
}

/// Computes `(pos_xz, godot_yaw)` 25 m before `trailhead`, facing outward along the nearest road.
pub fn trailhead_tp_pose(trailhead: Vec2, roads: &[world::roads::Road], camp: Vec2) -> (Vec2, f32) {
    let outward = (trailhead - camp).normalize_or(Vec2::X);
    let mut best_dist = f32::INFINITY;
    let mut road_dir = outward;
    for road in roads {
        if road.points.len() < 2 {
            continue;
        }
        let d_start = road.points[0].distance_squared(trailhead);
        if d_start < best_dist {
            best_dist = d_start;
            road_dir = (road.points[1] - road.points[0]).normalize_or(outward);
        }
        let last = road.points.len() - 1;
        let d_end = road.points[last].distance_squared(trailhead);
        if d_end < best_dist {
            best_dist = d_end;
            road_dir = (road.points[last - 1] - road.points[last]).normalize_or(outward);
        }
    }
    if road_dir.dot(outward) < 0.0 {
        road_dir = -road_dir;
    }
    let pos = trailhead - road_dir * WORLD_TP_DISTANCE_M;
    let yaw = (-road_dir.x).atan2(-road_dir.y);
    (pos, yaw)
}

/// Computes `(pos_xz, godot_yaw)` on the bank 25 m from river `river`'s midpoint, facing the water.
pub fn river_tp_pose(river: &world::rivers::River) -> Option<(Vec2, f32)> {
    if river.points.len() < 2 {
        return None;
    }
    let mid = river.points.len() / 2;
    let p_prev = river.points[mid.saturating_sub(1)];
    let p_next = river.points[(mid + 1).min(river.points.len() - 1)];
    let tangent = (p_next - p_prev).normalize_or(Vec2::X);
    let bank_normal = Vec2::new(-tangent.y, tangent.x);
    let center = river.points[mid];
    let pos = center + bank_normal * WORLD_TP_DISTANCE_M;
    let to_water = -bank_normal;
    let yaw = (-to_water.x).atan2(-to_water.y);
    Some((pos, yaw))
}

fn apply_town_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    key: &str,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let Some(town) = world_gen
        .towns()
        .iter()
        .find(|t| t.key.eq_ignore_ascii_case(key))
    else {
        log::warn!("--town={key}: no matching town found");
        return;
    };
    let (xz, yaw) = town_tp_pose(town);
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

fn apply_trailhead_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    index: usize,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let Some(&th) = world_gen.roads().trailheads().get(index) else {
        log::warn!("--trailhead={index}: trailhead index out of range");
        return;
    };
    let camp = world_gen
        .settlements()
        .first()
        .map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
    let (xz, yaw) = trailhead_tp_pose(th, world_gen.roads().roads(), camp);
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

fn apply_bridge_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    index: usize,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let crossings = world_gen.rivers().crossings(world_gen.roads());
    let Some(&(xz, road_yaw, _span)) = crossings.get(index) else {
        log::warn!("--bridge={index}: crossing index out of range");
        return;
    };
    teleport_player_and_stream(gpu, render, world, actors, session, xz, road_yaw);
}

fn apply_river_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    index: usize,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let Some(river) = world_gen.rivers().rivers().get(index) else {
        log::warn!("--river={index}: river index out of range");
        return;
    };
    let Some((xz, yaw)) = river_tp_pose(river) else {
        log::warn!("--river={index}: river has too few points");
        return;
    };
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

fn apply_tp_rescue(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &PlaySide,
    session: &SessionState,
) {
    let Some(spots) = play.rescue.as_ref().map(|r| r.active_spots()) else {
        return;
    };
    let Some((pos, yaw)) = tp_rescue_pose(actors.player, spots) else {
        log::warn!("--tp-rescue: no unrescued survivor spots found");
        return;
    };
    teleport_player_and_stream(
        gpu,
        render,
        world,
        actors,
        session,
        Vec2::new(pos.x, pos.z),
        yaw,
    );
}

fn apply_team(actors: &ActorsState, play: &mut PlaySide, count: usize) {
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

fn apply_wear_and_mods(
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
            super::frame_input::apply_hand(change, render, play, actors, world);
        }
        if let Some(ps) = play.play.as_mut() {
            ps.refresh_equipped_stats(actors.combat.as_mut());
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_gunsmith_town(
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

/// Runs per-frame dev test flags (`--down-at` and `--hold-e`).
pub fn tick_timers(
    session: &mut SessionState,
    render: &mut RenderState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    audio: &mut AudioState,
) {
    if let (Some(t), Some(c)) = (session.test_flags.down_at, actors.combat.as_mut())
        && session.time >= t
    {
        session.test_flags.down_at = None;
        let lethal = c.health.current().max(c.health.max()) + 1.0; // new: DECISIONS 2026-11-25
        let ev = c.take_damage_with_downed(
            &[lethal],
            &mut actors.downed.machine,
            session.play_time,
            true,
        );
        if ev.downed_now {
            crate::downed_play::on_knocked_down(
                render,
                play,
                &mut actors.downed,
                c,
                actors.player_visual.as_mut(),
                session.anim_shot.is_pose_frozen(),
                &mut audio.audio,
            );
        } else if ev.died
            && let Some(pv) = actors.player_visual.as_mut()
        {
            pv.animator.play_die();
        }
    }

    if session.test_flags.hold_e && actors.downed.is_down() {
        actors.downed.set_frame_input(true, session.play_time);
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    use super::*;
    use game::guild::RecruitState;

    fn make_args(raw: &[&str]) -> EngineArgs {
        EngineArgs::parse(raw.iter().map(|s| (*s).to_string()).collect())
    }

    fn make_spot(id: &str, pos: Vec3) -> RescueSpot {
        RescueSpot {
            id: id.to_string(),
            poi_id: format!("poi_{id}"),
            building_id: "house_small".to_string(),
            name: "Sarah Jenkins".to_string(),
            role: RecruitRole::Scavenger,
            level: 1,
            daily_wage: 12,
            max_health: 80,
            position: pos,
            yaw: 0.0,
        }
    }

    fn fake_world_sample(camp: Vec2, x: f32, z: f32) -> BiomeSample {
        let d = Vec2::new(x - camp.x, z - camp.y);
        let r = d.length();
        let core_radius = 1000.0;
        let blend_band = 200.0;
        if r <= core_radius {
            let primary = if d.x > 120.0 {
                BiomeId::Forest
            } else {
                BiomeId::Plains
            };
            return BiomeSample {
                primary,
                secondary: primary,
                blend: 0.0,
                outer: 0.0,
                tier: 0,
            };
        }
        let outer = ((r - core_radius) / blend_band).clamp(0.0, 1.0);
        let mut ang = d.y.atan2(d.x);
        if ang < 0.0 {
            ang += TAU;
        }
        let primary = if ang < FRAC_PI_2 {
            BiomeId::Desert
        } else if ang < PI {
            BiomeId::Snow
        } else if ang < PI + FRAC_PI_2 {
            BiomeId::Swamp
        } else {
            BiomeId::Mountains
        };
        BiomeSample {
            primary,
            secondary: primary,
            blend: 0.0,
            outer,
            tier: 1,
        }
    }

    #[test]
    fn parses_all_test_flags_and_defaults() {
        let empty = TestFlags::from_args(&make_args(&["rotland"]));
        assert_eq!(empty, TestFlags::default());

        let parsed = TestFlags::from_args(&make_args(&[
            "rotland",
            "--down-at=1.5",
            "--team=3",
            "--tp-rescue",
            "--hold-e",
            "--biome=desert",
            "--weather=sandstorm",
            "--town=Dusthollow",
            "--trailhead=2",
            "--bridge=1",
            "--river=0",
            "--gunsmith=upgrade",
            "--gunsmith-town=Frostmere",
            "--wear=pistol:0.35",
            "--mods=rifle:scope_4x,suppressor",
        ]));
        assert_eq!(parsed.down_at, Some(1.5));
        assert_eq!(parsed.team, 3);
        assert!(parsed.tp_rescue);
        assert!(parsed.hold_e);
        assert_eq!(parsed.biome, Some(BiomeId::Desert));
        assert_eq!(parsed.weather, Some(WeatherKind::Sandstorm));
        assert_eq!(parsed.town.as_deref(), Some("dusthollow"));
        assert_eq!(parsed.trailhead, Some(2));
        assert_eq!(parsed.bridge, Some(1));
        assert_eq!(parsed.river, Some(0));
        assert_eq!(parsed.gunsmith, Some(GunsmithTab::Upgrade));
        assert_eq!(parsed.gunsmith_town.as_deref(), Some("frostmere"));
        assert_eq!(parsed.wear, Some(("pistol".to_string(), 0.35)));
        assert_eq!(
            parsed.mods,
            Some((
                "rifle".to_string(),
                vec!["scope_4x".to_string(), "suppressor".to_string()]
            ))
        );

        let invalid = TestFlags::from_args(&make_args(&[
            "rotland",
            "--down-at=-2",
            "--team=abc",
            "--biome=mars",
            "--weather=acid",
            "--gunsmith=smithing",
            "--wear=pistol:nope",
            "--mods=rifle:",
        ]));
        assert_eq!(invalid.down_at, None);
        assert_eq!(invalid.team, 0);
        assert_eq!(invalid.biome, None);
        assert_eq!(invalid.weather, None);
        assert_eq!(invalid.gunsmith, None);
        assert_eq!(invalid.wear, None);
        assert_eq!(invalid.mods, None);
    }

    #[test]
    fn find_biome_point_places_outer_biome_at_core_plus_blend_plus_300m_facing_outward() {
        let camp = Vec2::new(50.0, -30.0);
        let (pos, yaw) =
            find_biome_point(camp, BiomeId::Desert, |x, z| fake_world_sample(camp, x, z))
                .expect("desert point");
        let delta = pos - camp;
        let expected_r = 1000.0 + 200.0 + BIOME_TP_PAST_BLEND_M;
        assert!(
            (delta.length() - expected_r).abs() < 2.0,
            "expected radius {expected_r}, got {}",
            delta.length()
        );
        let angle = delta.y.atan2(delta.x);
        assert!(
            (angle - FRAC_PI_4).abs() < 0.08,
            "expected sector centre ~PI/4, got {angle}"
        );

        let forward = game::yaw_forward(yaw);
        let dir = delta.normalize();
        let dot = forward.x * dir.x + forward.z * dir.y;
        assert!(dot > 0.99, "expected player facing outward, dot={dot}");
    }

    #[test]
    fn find_biome_point_finds_nearest_core_point_for_forest_and_plains() {
        let camp = Vec2::ZERO;
        let (plains_pos, _) =
            find_biome_point(camp, BiomeId::Plains, |x, z| fake_world_sample(camp, x, z))
                .expect("plains");
        assert!(plains_pos.length() < 1.0);

        let (forest_pos, yaw) =
            find_biome_point(camp, BiomeId::Forest, |x, z| fake_world_sample(camp, x, z))
                .expect("forest");
        assert!(forest_pos.x > 120.0 && forest_pos.length() < 200.0);
        let sample = fake_world_sample(camp, forest_pos.x, forest_pos.y);
        assert_eq!(sample.primary, BiomeId::Forest);
        assert_eq!(sample.outer, 0.0);

        let forward = game::yaw_forward(yaw);
        let dir = forest_pos.normalize();
        assert!(forward.x * dir.x + forward.z * dir.y > 0.99);
    }

    #[test]
    fn tp_rescue_pose_places_player_6m_away_facing_nearest_spot() {
        let spots = vec![
            make_spot("far", Vec3::new(100.0, 0.0, 0.0)),
            make_spot("near", Vec3::new(20.0, 2.0, 0.0)),
        ];
        let (pos, yaw) = tp_rescue_pose(Vec3::new(0.0, 0.0, 0.0), &spots).expect("spot");
        let dist = Vec3::new(pos.x - 20.0, 0.0, pos.z).length();
        assert!((dist - 6.0).abs() < 1e-4, "expected 6m distance, got {dist}");
        assert!((pos.x - 14.0).abs() < 1e-4);

        // Player is west of spot (14, 0) -> faces east (+X) -> Godot yaw = -PI/2.
        let forward = game::yaw_forward(yaw);
        assert!(forward.x > 0.99 && forward.z.abs() < 1e-4);
    }

    #[test]
    fn build_rescued_recruits_cycles_templates_and_sets_follow_when_added() {
        let templates = vec![
            RecruitDef {
                id: "rec_mark".to_string(),
                name: "Mark Vance".to_string(),
                role: "fighter".to_string(),
                level: 1,
                daily_wage: 15,
                health: 100,
                hire_cost: 30,
            },
            RecruitDef {
                id: "rec_sarah".to_string(),
                name: "Sarah Jenkins".to_string(),
                role: "scavenger".to_string(),
                level: 1,
                daily_wage: 12,
                health: 80,
                hire_cost: 25,
            },
        ];
        let built = build_rescued_recruits(3, &templates);
        assert_eq!(built.len(), 3);
        assert_eq!(built[0].name, "Mark Vance");
        assert_eq!(built[1].name, "Sarah Jenkins");
        assert_eq!(built[2].name, "Mark Vance");

        let mut roster = game::guild::GuildRecruits::default();
        for r in built {
            roster.add_rescued(r, 3).expect("add_rescued");
        }
        assert_eq!(roster.hired().len(), 3);
        assert!(
            roster
                .hired()
                .iter()
                .all(|r| r.hire_cost == 0 && r.state == RecruitState::FollowingPlayer)
        );
    }
}
