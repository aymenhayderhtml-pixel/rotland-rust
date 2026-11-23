//! Hired teammates in the world (R23-N5): they walk with the player, guard the camp and fight. Each frame this
//! keeps the squad in step with the guild, moves and fights them, lets zombies hit back, and draws the party
//! panel and name tags. The pure rules live in game::guild::companion.

use engine::egui;
use game::guild::companion::{
    CompanionAnim, CompanionConfig, CompanionDraw, CompanionEvent, CompanionSquad, CompanionTarget,
    CompanionWorld,
};
use game::guild::{GuildRecruit, RecruitState};
use game::DamageInfo;
use glam::{Mat4, Vec2, Vec3};
use physics::{CharacterMover, PhysicsWorld};

use super::state::PlaySide;
use crate::companion_model::{model_for_role, CompanionModels};
use crate::companion_view::PartyMember;
use crate::hud::{Hud, NotificationKind};
use crate::zombies::Zombies;

/// Same capsule the dogs walk with (new: DECISIONS 2026-11-23, copied from game/src/dogs/manager.rs).
const BODY_RADIUS: f32 = 0.3; // new: DECISIONS 2026-11-23
const BODY_HEIGHT: f32 = 0.8; // new: DECISIONS 2026-11-23
/// Teammates within this many metres get a name tag when they are in front of the camera.
const NAME_TAG_RANGE: f32 = 25.0; // new: DECISIONS 2026-11-23 (task spec R23-N5)
/// Height above the feet where the name tag sits.
const NAME_TAG_HEAD_HEIGHT: f32 = 2.0; // new: DECISIONS 2026-11-23
/// Name tags lower than this share of the screen height are skipped, so they never sit on the hotbar.
const NAME_TAG_MAX_Y_FRACTION: f32 = 0.85; // new: DECISIONS 2026-11-23
/// Shortest zombie attack cooldown used to turn a hit into damage per second.
const MIN_ATTACK_COOLDOWN: f32 = 0.1; // new: DECISIONS 2026-11-23

/// One teammate as drawn this frame: the runtime snapshot plus its model and order label.
pub struct TeamFigure {
    pub draw: CompanionDraw,
    pub model: &'static str,
    pub order_label: &'static str,
}

/// Reused buffers and the guild key the squad was last synced with (kept in PlaySide, so frames do not allocate).
#[derive(Default)]
pub struct TeamScratch {
    /// Hired recruits and camp point the squad was last synced with.
    key: Option<(Vec<GuildRecruit>, Vec3)>,
    targets: Vec<CompanionTarget>,
    names: Vec<(String, Vec3)>,
    events: Vec<CompanionEvent>,
    /// Teammates drawn this frame, filled by `update`.
    pub figures: Vec<TeamFigure>,
}

impl TeamScratch {
    /// Forgets the synced state so the next frame rebuilds the squad from the guild.
    pub fn reset(&mut self) {
        self.key = None;
        self.figures.clear();
    }

    /// Re-syncs the squad only when the hired recruits (or their orders) or the camp point changed.
    fn sync_if_changed(&mut self, squad: &mut CompanionSquad, hired: &[GuildRecruit], camp: Vec3) {
        let same = self
            .key
            .as_ref()
            .is_some_and(|(h, c)| h.as_slice() == hired && *c == camp);
        if !same {
            squad.sync(hired, camp);
            self.key = Some((hired.to_vec(), camp));
        }
    }
}

/// What the teammates need to know about the world this frame.
pub struct TeamFrame {
    pub dt: f32,
    pub player: Vec3,
    pub player_yaw: f32,
    pub camp: Option<Vec3>,
}

/// Loads the teammate tuning. A failure logs a warning and turns the feature off (never a crash).
pub fn load_squad() -> Option<CompanionSquad> {
    match CompanionConfig::from_repo() {
        Ok(cfg) => Some(CompanionSquad::new(cfg)),
        Err(e) => {
            log::warn!("data/companions.json not loaded, teammates off: {e}");
            None
        }
    }
}

/// The first settlement's centre: where teammates guard and wait.
pub fn camp_point(world_gen: Option<&world::WorldGenerator>) -> Option<Vec3> {
    world_gen.and_then(|g| g.settlements().first().map(|s| s.center))
}

/// A new world or a loaded save: a fresh squad, synced from the guild (with saved health) on the next frame.
pub fn reset(side: &mut PlaySide) {
    side.companions = load_squad();
    side.team.reset();
}

/// Before a save: copies each teammate's health onto its recruit. `saved_health` never returns 0.
pub fn write_health_back(side: &mut PlaySide) {
    let (Some(squad), Some(pl)) = (side.companions.as_ref(), side.play.as_mut()) else {
        return;
    };
    for recruit in pl.camp.guild.manager_mut().recruits_mut().hired.iter_mut() {
        if let Some(hp) = squad.saved_health(&recruit.id) {
            recruit.health = hp;
        }
    }
}

/// Morning rollover: downed teammates get back up at camp with part of their health.
pub fn on_morning(
    companions: &mut Option<CompanionSquad>,
    team: &mut TeamScratch,
    camp: Option<Vec3>,
    hud: &mut Hud,
) {
    let (Some(squad), Some(camp)) = (companions.as_mut(), camp) else {
        return;
    };
    squad.on_morning(camp, &mut team.events);
    for ev in team.events.drain(..) {
        announce(ev, hud);
    }
}

/// Once a frame from the simulation: sync with the guild, move and fight, zombies hit back, and animate.
pub fn update(
    frame: &TeamFrame,
    z: &mut Zombies,
    physics: &PhysicsWorld,
    side: &mut PlaySide,
    hud: &mut Hud,
    models: Option<&mut CompanionModels>,
) {
    let PlaySide { play: play_state, companions, team, .. } = side;
    let (Some(squad), Some(pl), Some(camp)) = (companions.as_mut(), play_state.as_ref(), frame.camp) else {
        team.figures.clear();
        return;
    };
    let hired = pl.camp.guild.manager().recruits().hired();
    team.sync_if_changed(squad, hired, camp);

    // Living zombies as teammates' wall-free targets (ids that fit u32 only)
    team.targets.clear();
    team.targets.extend(z.list.iter().filter(|q| !q.health.is_dead()).filter_map(|q| {
        u32::try_from(q.id).ok().map(|id| CompanionTarget { id, position: q.position })
    }));

    let world = CompanionWorld {
        player: frame.player,
        player_forward: game::yaw_forward(frame.player_yaw),
        camp,
        zombies: &team.targets,
    };
    let mover = CharacterMover::new(BODY_RADIUS, BODY_HEIGHT);
    let dt = frame.dt;
    let mut step_fn = |from: Vec3, step: Vec3| move_teammate(&mover, physics, from, step, dt);
    squad.tick(&world, &mut step_fn, dt, &mut team.events);

    // Zombies in attack reach of a living teammate hit it at their own damage rate (cheap, no brain change)
    team.names.clear();
    squad.targets(&mut team.names);
    if !team.names.is_empty() {
        for q in z.list.iter().filter(|q| {
            !q.health.is_dead() && matches!(q.brain.state(), game::zombie::ZombieState::Attack)
        }) {
            let reach = q.def.attack_range;
            if let Some((id, _)) = team.names.iter().find(|(_, p)| planar_dist(*p, q.position) <= reach) {
                let per_second = q.def.attack_damage / q.def.attack_cooldown.max(MIN_ATTACK_COOLDOWN);
                squad.damage(id, per_second * dt, &mut team.events);
            }
        }
    }

    for ev in team.events.drain(..) {
        match ev {
            CompanionEvent::Hit { zombie_id, damage, .. } => {
                // Like the camp guards: no kill XP for the player (game::zombie path via hit_unclaimed)
                z.hit_unclaimed(u64::from(zombie_id), DamageInfo::new(damage), Vec3::ZERO);
            }
            other => announce(other, hud),
        }
    }

    team.figures.clear();
    for draw in squad.draws() {
        let recruit = hired.iter().find(|r| r.id == draw.id);
        let (model, order_label) = recruit.map_or(("npc_guard", "Stay"), |r| {
            (model_for_role(r.role), order_label(r.state))
        });
        team.figures.push(TeamFigure { draw, model, order_label });
    }
    if let Some(m) = models {
        m.advance(dt, &team.figures);
    }
}

/// Party panel on the left, and name tags over living teammates in range and in front of the camera.
pub fn draw_hud(ctx: &egui::Context, figures: &[TeamFigure], view_proj: Mat4, player: Vec3) {
    if figures.is_empty() {
        return;
    }
    let members: Vec<PartyMember<'_>> = figures
        .iter()
        .map(|f| PartyMember {
            name: &f.draw.name,
            health_fraction: f.draw.health_fraction,
            downed: f.draw.anim == CompanionAnim::Downed,
            order_label: f.order_label,
        })
        .collect();
    crate::companion_view::draw_party_panel(ctx, &members);

    let r = ctx.content_rect();
    let screen = (r.width(), r.height());
    for f in figures.iter().filter(|f| {
        f.draw.anim != CompanionAnim::Downed && f.draw.position.distance(player) <= NAME_TAG_RANGE
    }) {
        let head = f.draw.position + Vec3::Y * NAME_TAG_HEAD_HEIGHT;
        if let Some(p) = crate::bubbles::world_to_screen(head, view_proj, screen)
            && p.y <= screen.1 * NAME_TAG_MAX_Y_FRACTION
        {
            crate::companion_view::draw_name_tag(ctx, p, &f.draw.name, f.draw.health_fraction);
        }
    }
}

/// Ground and wall check for one teammate step: slides along walls, then sits on the ground below it.
fn move_teammate(mover: &CharacterMover, physics: &PhysicsWorld, from: Vec3, step: Vec3, dt: f32) -> Vec3 {
    let moved = if dt > 0.0 {
        mover.move_and_slide(physics, from, step / dt, dt).position
    } else {
        from
    };
    match physics.ground_height(moved.x, moved.z) {
        Some(y) => Vec3::new(moved.x, y, moved.z),
        None => moved,
    }
}

/// Tells the player about teammates going down or getting back up.
fn announce(ev: CompanionEvent, hud: &mut Hud) {
    match ev {
        CompanionEvent::Downed { companion } => {
            hud.notify(format!("{companion} is down until morning"), NotificationKind::Warning);
        }
        CompanionEvent::Recovered { companion } => {
            hud.notify(format!("{companion} is back on their feet"), NotificationKind::Info);
        }
        CompanionEvent::Hit { .. } => {}
    }
}

fn order_label(state: RecruitState) -> &'static str {
    match state {
        RecruitState::FollowingPlayer => "Follow",
        RecruitState::GuardingCamp => "Guard",
        RecruitState::IdleAtCamp | RecruitState::Dead => "Stay",
    }
}

fn planar_dist(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}
