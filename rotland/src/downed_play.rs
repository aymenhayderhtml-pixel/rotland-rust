//! Gameplay wiring for the knocked-down state: bleed-out, zombie bites, self-bandage,
//! teammate revive, `downed_crawl` animation, save restoration, and death/respawn transitions.

use game::{BANDAGE_ITEM, Downed, DownedConfig, DownedEvent};
use humanoid::FullBodyOptions;

use crate::app::state::{ActorsState, PlaySide, RenderState, SessionState};
use crate::{combat, hud, play, player_visual};

const CRAWL_CLIP: &str = "downed_crawl";
const CRAWL_SPEED: f32 = 1.0; // new: DECISIONS 2026-11-25
const DEATH_SEED_MS_SCALE: f32 = 1000.0; // new: DECISIONS 2026-11-25

/// Runtime state for the player's knocked-down state machine and animation edge tracking.
pub struct DownedPlay {
    pub machine: Downed,
    self_revive_held: bool,
    was_down: bool,
    last_play_time: f64,
}

impl Default for DownedPlay {
    fn default() -> Self {
        let cfg = DownedConfig::from_repo().unwrap_or_else(|e| {
            log::warn!("data/downed.json not loaded, using defaults: {e}");
            DownedConfig::default()
        });
        Self {
            machine: Downed::new(cfg),
            self_revive_held: false,
            was_down: false,
            last_play_time: 0.0,
        }
    }
}

impl DownedPlay {
    /// True while the player is knocked down (crawling, waiting for revive or bleed-out).
    pub fn is_down(&self) -> bool {
        self.machine.is_down()
    }

    /// Movement speed multiplier while downed (`data/downed.json` `crawlSpeedMul`).
    pub fn crawl_speed_mul(&self) -> f32 {
        self.machine.config().crawl_speed_mul
    }

    /// Maximum distance in metres for a `Follow` teammate to come revive (`data/downed.json` `teammateRange`).
    pub fn teammate_range(&self) -> f32 {
        self.machine.config().teammate_range
    }

    /// Clears downed state and re-down history (new world, load, respawn, or drowning).
    pub fn reset(&mut self) {
        self.machine.reset();
        self.self_revive_held = false;
        self.was_down = false;
    }

    /// Records whether the interact key is held for self-revive and caches world play time.
    pub fn set_frame_input(&mut self, self_revive_held: bool, play_time: f64) {
        self.self_revive_held = self_revive_held;
        self.last_play_time = play_time;
    }

    /// Latest cached session play time in seconds.
    pub fn last_play_time(&self) -> f64 {
        self.last_play_time
    }
}

/// Applies instant weapon holstering and starts the `downed_crawl` loop on knockdown.
pub fn on_knocked_down(
    render: &mut RenderState,
    play: &mut PlaySide,
    downed: &mut DownedPlay,
    combat: &mut combat::Combat,
    player_visual: Option<&mut player_visual::PlayerVisual>,
    pose_frozen: bool,
    audio: &mut Option<audio::Audio>,
) {
    downed.was_down = true;
    holster_and_cancel(render, play, combat);
    if !pose_frozen && let Some(pv) = player_visual {
        start_downed_crawl(pv);
    }
    if let Some(a) = audio.as_mut() {
        a.event("playerHurt", None);
    }
}

/// Keeps `downed_crawl` active while downed and stops it when standing back up.
pub fn sync_downed_visual(
    downed: &mut DownedPlay,
    render: &mut RenderState,
    play: &mut PlaySide,
    combat: Option<&mut combat::Combat>,
    player_visual: Option<&mut player_visual::PlayerVisual>,
    pose_frozen: bool,
) {
    if downed.is_down() {
        if !downed.was_down {
            downed.was_down = true;
            if let Some(c) = combat {
                holster_and_cancel(render, play, c);
            } else {
                play.build_mode.set_active(false);
            }
        }
        if !pose_frozen
            && let Some(pv) = player_visual
            && !pv.animator.is_full_body_active()
        {
            start_downed_crawl(pv);
        }
    } else if downed.was_down {
        downed.was_down = false;
        if !pose_frozen
            && combat.is_some_and(|c| !c.is_dead())
            && let Some(pv) = player_visual
        {
            pv.animator.stop_full_body(None);
        }
    }
}

/// Advances the downed timer, self-bandage, teammate revive, and bleed-out death transition.
pub fn tick(
    dt: f32,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &SessionState,
    audio: &mut Option<audio::Audio>,
) {
    let Some(c) = actors.combat.as_mut() else {
        return;
    };
    if c.is_dead() || !actors.downed.is_down() {
        return;
    }

    let reviving_teammate = play.team.reviving_player;
    let self_revive_held = actors.downed.self_revive_held;
    let has_bandage = play
        .play
        .as_ref()
        .is_some_and(|ps| ps.inventory.has(BANDAGE_ITEM, 1));
    let event = actors.downed.machine.tick(
        dt,
        reviving_teammate,
        self_revive_held,
        has_bandage,
    );

    match event {
        DownedEvent::BandageUsed => {
            if let Some(ps) = play.play.as_mut() {
                ps.inventory.remove(BANDAGE_ITEM, 1);
                ps.hotbar.on_inventory_changed(&ps.inventory);
            }
            finish_revive(
                c,
                &mut actors.downed,
                play,
                actors.player_visual.as_mut(),
                session.anim_shot.is_pose_frozen(),
                "Revived with bandage",
            );
            if let Some(a) = audio.as_mut() {
                a.event("pickup", None);
            }
        }
        DownedEvent::Revived => {
            finish_revive(
                c,
                &mut actors.downed,
                play,
                actors.player_visual.as_mut(),
                session.anim_shot.is_pose_frozen(),
                "Revived by teammate",
            );
        }
        DownedEvent::BledOut => {
            actors.downed.was_down = false;
            actors.downed.self_revive_held = false;
            c.kill();
            if !session.anim_shot.is_pose_frozen()
                && let Some(pv) = actors.player_visual.as_mut()
            {
                pv.animator.play_die();
            }
            if let Some(ps) = play.play.as_mut() {
                let seed = (session.time * DEATH_SEED_MS_SCALE) as u64 ^ session.seed as u64;
                ps.camp.on_death(&mut ps.inventory, seed);
                ps.hotbar.on_inventory_changed(&ps.inventory);
                crate::app::frame_guild::clear_hooks(&mut play.guild_hooks);
            }
            log::info!("the player bled out (kills so far: {})", c.kills);
        }
        DownedEvent::None => {}
    }
}

/// Applies drowning or deep-water death immediately (never enters downed).
pub fn take_drowning(
    combat: &mut combat::Combat,
    downed: &mut DownedPlay,
    drowning_damage: f32,
    in_deep_water: bool,
) -> combat::CombatEvents {
    if downed.is_down() && (in_deep_water || drowning_damage > 0.0) {
        let mut ev = combat::CombatEvents::default();
        if combat.kill() {
            downed.reset();
            ev.died = true;
        }
        return ev;
    }
    let ev = combat.take_bites(&[drowning_damage]);
    if ev.died {
        downed.reset();
    }
    ev
}

/// Restores saved health: a downed player who quit at 0 HP loads as `Up` with `revive_health` (25).
pub fn restore_health(combat: &mut combat::Combat, downed: &mut DownedPlay, saved_health: f32) {
    downed.reset();
    let hp = if saved_health <= 0.0 {
        downed.machine.config().revive_health
    } else {
        saved_health
    };
    combat.health.set_current(hp);
}

/// Builds the HUD snapshot while the player is downed (`None` while standing or dead).
pub fn hud_info(downed: &DownedPlay, play: &play::PlayState) -> Option<hud::DownedHudInfo> {
    if !downed.is_down() {
        return None;
    }
    Some(hud::DownedHudInfo {
        bleed_remaining: downed.machine.bleed_remaining(),
        bleed_total: downed.machine.config().bleed_out_seconds,
        revive_fraction: downed.machine.revive_fraction(),
        revive_by: downed.machine.revive_by(),
        has_bandage: play.inventory.has(BANDAGE_ITEM, 1),
    })
}

fn finish_revive(
    combat: &mut combat::Combat,
    downed: &mut DownedPlay,
    play: &mut PlaySide,
    player_visual: Option<&mut player_visual::PlayerVisual>,
    pose_frozen: bool,
    toast: &str,
) {
    let hp = downed.machine.config().revive_health;
    combat.health.revive(Some(hp));
    downed.was_down = false;
    downed.self_revive_held = false;
    if let Some(ps) = play.play.as_mut() {
        ps.show_toast(toast.to_string());
    }
    if !pose_frozen && let Some(pv) = player_visual {
        pv.animator.stop_full_body(None);
    }
}

fn holster_and_cancel(render: &mut RenderState, play: &mut PlaySide, combat: &mut combat::Combat) {
    play.build_mode.set_active(false);
    if let Some(ps) = play.play.as_mut() {
        ps.hotbar.holster();
        ps.gun = None;
        ps.anim_cue = None;
    }
    combat.set_weapon(None);
    render.pending_weapon_model = Some(None);
}

fn start_downed_crawl(pv: &mut player_visual::PlayerVisual) {
    let opts = FullBodyOptions {
        speed: CRAWL_SPEED,
        looping: true,
        hold_last: false,
        ..FullBodyOptions::default()
    };
    pv.animator.play_full_body(CRAWL_CLIP, opts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_zero_health_restores_as_up_with_revive_health() {
        let mut combat = combat::Combat::new("bat");
        let mut downed = DownedPlay::default();
        downed.machine.on_lethal_hit(10.0);
        assert!(downed.is_down());

        restore_health(&mut combat, &mut downed, 0.0);
        assert!(!downed.is_down());
        assert!(!combat.is_dead());
        assert_eq!(combat.health.current(), 25.0);

        restore_health(&mut combat, &mut downed, 64.0);
        assert_eq!(combat.health.current(), 64.0);
    }

    #[test]
    fn deep_water_kills_downed_player_immediately() {
        let mut combat = combat::Combat::new("bat");
        let mut downed = DownedPlay::default();
        let ev = combat.take_damage_with_downed(&[100.0], &mut downed.machine, 5.0, true);
        assert!(ev.downed_now);
        assert!(downed.is_down());

        let drown = take_drowning(&mut combat, &mut downed, 0.0, true);
        assert!(drown.died);
        assert!(combat.is_dead());
        assert!(!downed.is_down());
    }

    #[test]
    fn drowning_while_up_kills_without_entering_downed() {
        let mut combat = combat::Combat::new("bat");
        let mut downed = DownedPlay::default();
        let drown = take_drowning(&mut combat, &mut downed, 100.0, true);
        assert!(drown.died);
        assert!(combat.is_dead());
        assert!(!downed.is_down());
    }
}
