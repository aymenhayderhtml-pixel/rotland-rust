//! Teammate animation application and HUD overlays (party panel and name tags).

use engine::egui;
use game::guild::companion::CompanionAnim;
#[cfg(test)]
use game::NpcAnimPlan;
use glam::{Mat4, Vec3};
#[cfg(test)]
use humanoid::{HumanoidAnimator, Locomotion};

use super::frame_team::TeamFigure;
use crate::companion_view::PartyMember;

/// Teammates within this many metres get a name tag when they are in front of the camera.
const NAME_TAG_RANGE: f32 = 25.0; // new: DECISIONS 2026-11-23 (task spec R23-N5)
/// Height above the feet where the name tag sits.
const NAME_TAG_HEAD_HEIGHT: f32 = 2.0; // new: DECISIONS 2026-11-23
/// Name tags lower than this share of the screen height are skipped, so they never sit on the hotbar.
const NAME_TAG_MAX_Y_FRACTION: f32 = 0.85; // new: DECISIONS 2026-11-23

/// Applies a resolved [`NpcAnimPlan`] to a [`HumanoidAnimator`] using the player's API.
#[cfg(test)]
pub fn apply_npc_anim(animator: &mut HumanoidAnimator, plan: &NpcAnimPlan<'_>, dt: f32) {
    animator.set_update_interval(plan.lod_interval);
    animator.set_weapon(plan.weapon);
    animator.set_aiming(plan.aiming);
    if plan.reset_alive {
        animator.reset_alive();
    }
    if plan.play_die {
        animator.play_die();
    }
    if plan.gun_fired {
        animator.on_gun_fired();
    }
    if let Some((clip, speed)) = plan.action_clip {
        animator.play_action(clip, speed);
    }
    if let Some(pose) = plan.held_pose {
        animator.hold_pose(pose);
    } else {
        animator.release_pose();
    }
    animator.set_move_velocity(plan.move_velocity, plan.yaw);
    animator.update(
        dt,
        Locomotion {
            speed: plan.move_speed,
            on_floor: true,
            crouching: false,
        },
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use game::{plan_npc_anim, NpcAnimInput, NpcAnimRole, NpcAnimTrack, TeamAnimConfig};
    use pack::{Joint, Skeleton};

    #[test]
    fn apply_npc_anim_drives_humanoid_animator() {
        let skel = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent: None,
                inverse_bind: Mat4::IDENTITY.to_cols_array(),
                rest_translation: [0.0; 3],
                rest_rotation: [0.0, 0.0, 0.0, 1.0],
                rest_scale: [1.0; 3],
            }],
        };
        let mut animator = HumanoidAnimator::new(&skel, &[]);
        let cfg = TeamAnimConfig::default();
        let mut track = NpcAnimTrack::default();
        let plan = plan_npc_anim(
            &cfg,
            &NpcAnimInput {
                role: NpcAnimRole::CampGuard,
                companion_anim: None,
                camp_clip: Some("idle"),
                aiming_at_target: true,
                just_fired: true,
                just_hit: false,
                velocity: Vec3::ZERO,
                yaw: 0.0,
                distance_to_camera: 15.0,
                on_screen: true,
            },
            &mut track,
        );
        apply_npc_anim(&mut animator, &plan, 0.05);
        assert_eq!(animator.update_interval(), 0.04);
    }
}
