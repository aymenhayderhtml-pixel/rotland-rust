//! Dev-only gameplay/screenshot test flags (`--down-at`, `--team`, `--tp-rescue`, `--hold-e`).
//!
//! All flags default to off so normal gameplay is untouched.

use engine::{EngineArgs, Gpu};
use game::guild::{GuildConfig, GuildRecruit, RecruitDef, RecruitRole};
use game::rescue_place::RescueSpot;
use glam::Vec3;

use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, WorldState};

/// Distance in metres to place the player from the nearest unrescued survivor for `--tp-rescue`.
const RESCUE_TP_DISTANCE_M: f32 = 6.0; // new: DECISIONS 2026-11-25

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
}

impl TestFlags {
    /// Parses `--down-at`, `--team`, `--tp-rescue`, and `--hold-e` from CLI args.
    pub fn from_args(args: &EngineArgs) -> Self {
        Self {
            down_at: args.value("--down-at").and_then(parse_non_negative_f32),
            team: args
                .value("--team")
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(0),
            tp_rescue: args.flag("--tp-rescue"),
            hold_e: args.flag("--hold-e"),
        }
    }
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

/// Applies startup dev flags (`--tp-rescue` then `--team`) after the world is initialized.
pub fn apply_startup(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &SessionState,
) {
    if session.test_flags.tp_rescue {
        apply_tp_rescue(gpu, render, world, actors, play, session);
    }
    if session.test_flags.team > 0 {
        apply_team(actors, play, session.test_flags.team);
    }
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
    let Some((mut pos, yaw)) = tp_rescue_pose(actors.player, spots) else {
        log::warn!("--tp-rescue: no unrescued survivor spots found");
        return;
    };

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
        ]));
        assert_eq!(parsed.down_at, Some(1.5));
        assert_eq!(parsed.team, 3);
        assert!(parsed.tp_rescue);
        assert!(parsed.hold_e);

        let invalid = TestFlags::from_args(&make_args(&[
            "rotland",
            "--down-at=-2",
            "--team=abc",
        ]));
        assert_eq!(invalid.down_at, None);
        assert_eq!(invalid.team, 0);
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
