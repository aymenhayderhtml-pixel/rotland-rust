//! Per-frame dev timers: `--down-at` knock-down and `--hold-e` interaction.

use crate::app::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState};

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
