//! Downed-player revive behaviour for hired teammates.

use game::guild::companion::CompanionAnim;
use game::guild::{GuildRecruit, RecruitState};
use game::TeamAnimConfig;
use glam::{Vec2, Vec3};
use physics::{CharacterMover, PhysicsWorld};

use super::frame_team::{
    move_teammate, planar_dist, ReviverState, TeamFrame, TeamScratch,
};
#[cfg(test)]
use super::frame_team::TeamFigure;

/// Distance beside the downed player where a reviving teammate stops and kneels/stands.
const REVIVE_STOP_RADIUS: f32 = 1.35; // new: DECISIONS 2026-11-25
/// Planar distance beyond which a reviving teammate runs to reach the downed player.
const REVIVE_RUN_DIST: f32 = 2.5; // new: DECISIONS 2026-11-25
/// Target stop inset fraction when stepping toward the downed player.
const REVIVE_STEP_INSET: f32 = 0.9; // new: DECISIONS 2026-11-25
/// Minimum distance needed to turn toward the player when already beside them.
const MIN_FACE_DIST: f32 = 0.001; // new: DECISIONS 2026-11-25

/// Sends one alive `Follow` teammate within `teammate_range` to walk to the downed player and stand/revive.
pub(super) fn step_player_revive(
    frame: &TeamFrame,
    hired: &[GuildRecruit],
    mover: &CharacterMover,
    physics: &PhysicsWorld,
    anim_cfg: &TeamAnimConfig,
    team: &mut TeamScratch,
) {
    if !frame.player_downed {
        team.reviver = None;
        team.reviving_player = false;
        return;
    }

    let is_eligible = |id: &str, pos: Vec3| {
        let is_follow = hired
            .iter()
            .any(|r| r.id == id && r.state == RecruitState::FollowingPlayer);
        let is_alive = team
            .figures
            .iter()
            .any(|f| f.draw.id == id && f.draw.anim != CompanionAnim::Downed);
        is_follow && is_alive && planar_dist(pos, frame.player) <= frame.teammate_range
    };

    if !team
        .reviver
        .as_ref()
        .is_some_and(|rev| is_eligible(&rev.id, rev.position))
    {
        team.reviver = None;
    }

    if team.reviver.is_none() {
        let best = team
            .figures
            .iter()
            .filter(|f| is_eligible(&f.draw.id, f.draw.position))
            .min_by(|a, b| {
                planar_dist(a.draw.position, frame.player)
                    .total_cmp(&planar_dist(b.draw.position, frame.player))
            });
        if let Some(fig) = best {
            team.reviver = Some(ReviverState {
                id: fig.draw.id.clone(),
                position: fig.draw.position,
                yaw: fig.draw.yaw,
            });
        }
    }

    let Some(rev) = team.reviver.as_mut() else {
        team.reviving_player = false;
        return;
    };
    let Some(fig) = team.figures.iter_mut().find(|f| f.draw.id == rev.id) else {
        team.reviver = None;
        team.reviving_player = false;
        return;
    };

    let to_player = Vec2::new(frame.player.x - rev.position.x, frame.player.z - rev.position.z);
    let dist = to_player.length();
    if dist > REVIVE_STOP_RADIUS {
        let dir = to_player / dist;
        let running = dist > REVIVE_RUN_DIST;
        if frame.dt > 0.0 {
            let speed = if running {
                anim_cfg.run_speed
            } else {
                anim_cfg.walk_speed
            };
            let step_len = (speed * frame.dt)
                .min((dist - REVIVE_STOP_RADIUS * REVIVE_STEP_INSET).max(0.0));
            let step = Vec3::new(dir.x * step_len, 0.0, dir.y * step_len);
            let moved = move_teammate(mover, physics, rev.position, step, frame.dt);
            if moved.is_finite() {
                rev.position = moved;
            }
        }
        rev.yaw = (-dir.x).atan2(-dir.y);
        fig.draw.anim = if running {
            CompanionAnim::Run
        } else {
            CompanionAnim::Walk
        };
        team.reviving_player = false;
    } else {
        if dist > MIN_FACE_DIST {
            rev.yaw = (-to_player.x).atan2(-to_player.y);
        }
        fig.draw.anim = CompanionAnim::Idle;
        fig.reviving = true;
        team.reviving_player = true;
    }
    fig.draw.position = rev.position;
    fig.draw.yaw = rev.yaw;
    fig.distance_to_player = rev.position.distance(frame.player);
    team.prev_positions.insert(rev.id.clone(), rev.position);
}

#[cfg(test)]
mod tests {
    use super::super::frame_team::{BODY_HEIGHT, BODY_RADIUS};
    use super::*;
    use game::guild::companion::CompanionDraw;
    use game::guild::{RecruitOrder, RecruitRole};
    use game::NpcAnimRole;
    use glam::IVec2;
    use physics::CHUNK_SAMPLES;

    #[test]
    fn follow_teammate_walks_to_downed_player_and_revives() {
        let mut physics = PhysicsWorld::new();
        physics.add_ground_chunk(IVec2::ZERO, &vec![0.0; CHUNK_SAMPLES * CHUNK_SAMPLES]);
        let mover = CharacterMover::new(BODY_RADIUS, BODY_HEIGHT);
        let anim_cfg = TeamAnimConfig::default();

        let mut guard = GuildRecruit::new("g1", "Guard", RecruitRole::Guard, 1, 10, 100, 50);
        guard.set_order(RecruitOrder::Guard);
        let mut follower = GuildRecruit::new("f1", "Follower", RecruitRole::Fighter, 1, 10, 100, 50);
        follower.set_order(RecruitOrder::Follow);
        let hired = vec![guard, follower];

        let mut team = TeamScratch::default();
        let make_figures = |f_pos: Vec3| {
            vec![
                TeamFigure {
                    draw: CompanionDraw {
                        id: "g1".into(),
                        name: "Guard".into(),
                        position: Vec3::new(2.0, 0.0, 0.0),
                        yaw: 0.0,
                        anim: CompanionAnim::Idle,
                        health_fraction: 1.0,
                    },
                    model: "npc_guard",
                    order_label: "Guard",
                    anim_role: NpcAnimRole::Guard,
                    velocity: Vec3::ZERO,
                    just_hit: false,
                    reviving: false,
                    distance_to_player: 2.0,
                },
                TeamFigure {
                    draw: CompanionDraw {
                        id: "f1".into(),
                        name: "Follower".into(),
                        position: f_pos,
                        yaw: 0.0,
                        anim: CompanionAnim::Idle,
                        health_fraction: 1.0,
                    },
                    model: "npc_guard",
                    order_label: "Follow",
                    anim_role: NpcAnimRole::Fighter,
                    velocity: Vec3::ZERO,
                    just_hit: false,
                    reviving: false,
                    distance_to_player: f_pos.length(),
                },
            ]
        };

        // Player downed at (5, 0, 5), follower at (10, 0, 5) (5 m away <= 12 m).
        let player = Vec3::new(5.0, 0.0, 5.0);
        let frame = TeamFrame {
            dt: 0.5,
            player,
            player_yaw: 0.0,
            camp: Some(Vec3::ZERO),
            player_downed: true,
            teammate_range: 12.0,
        };

        team.figures = make_figures(Vec3::new(10.0, 0.0, 5.0));
        step_player_revive(&frame, &hired, &mover, &physics, &anim_cfg, &mut team);
        assert!(!team.reviving_player);
        assert_eq!(team.figures[1].draw.anim, CompanionAnim::Run);
        assert!(team.figures[1].draw.position.x < 10.0);

        // Advance until follower reaches revive stop radius.
        for _ in 0..10 {
            let pos = team.figures[1].draw.position;
            team.figures = make_figures(pos);
            step_player_revive(&frame, &hired, &mover, &physics, &anim_cfg, &mut team);
        }
        assert!(team.reviving_player);
        assert!(team.figures[1].reviving);
        assert_eq!(team.figures[1].draw.anim, CompanionAnim::Idle);

        // Once player stands up, reviver clears.
        let up_frame = TeamFrame { player_downed: false, ..frame };
        step_player_revive(&up_frame, &hired, &mover, &physics, &anim_cfg, &mut team);
        assert!(!team.reviving_player);
    }
}
