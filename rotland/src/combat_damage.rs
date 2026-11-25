//! Player damage, bites, downed knock-down, and death handling.

use game::DamageInfo;

use crate::combat::{Combat, CombatEvents};

impl Combat {
    /// Kills the player now (test option --die-at, bleed-out, or drowning). Returns true if they were alive.
    pub fn kill(&mut self) -> bool {
        if self.is_dead() {
            return false;
        }
        self.health.kill();
        self.dead_for = Some(0.0);
        self.cancel_actions();
        true
    }

    /// Bites from this frame's zombies. Returns events (bitten / died) with instant death.
    pub fn take_bites(&mut self, damage: &[f32]) -> CombatEvents {
        let mut ev = CombatEvents::default();
        if self.is_dead() {
            return ev;
        }
        for &amount in damage {
            let r = self.health.take_damage(DamageInfo::new(amount));
            ev.bitten |= r.dealt > 0.0;
            if r.died_now {
                ev.died = true;
                self.dead_for = Some(0.0);
                self.cancel_actions();
                break;
            }
        }
        ev
    }

    /// Applies damage that knocks the player down at 0 HP unless within the re-down window.
    /// While already downed, positive hits shorten the bleed timer when `bite_when_down` is true.
    pub fn take_damage_with_downed(
        &mut self,
        damage: &[f32],
        downed: &mut game::Downed,
        now: f64,
        bite_when_down: bool,
    ) -> CombatEvents {
        let mut ev = CombatEvents::default();
        if self.is_dead() {
            return ev;
        }
        if downed.is_down() {
            if bite_when_down {
                for &amount in damage {
                    if amount > 0.0 {
                        downed.bite();
                        ev.bitten = true;
                    }
                }
            }
            return ev;
        }
        for &amount in damage {
            let r = self.health.take_damage(DamageInfo::new(amount));
            ev.bitten |= r.dealt > 0.0;
            if r.died_now {
                self.cancel_actions();
                match downed.on_lethal_hit(now) {
                    game::DownedPhase::Down { .. } => {
                        ev.downed_now = true;
                    }
                    game::DownedPhase::Dead | game::DownedPhase::Up => {
                        ev.died = true;
                        self.dead_for = Some(0.0);
                    }
                }
                break;
            }
        }
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lethal_hit_knocks_down_and_bites_shorten_bleed_and_redown_kills() {
        let mut combat = Combat::new("bat");
        let mut downed = game::Downed::new(game::DownedConfig::default());

        let ev = combat.take_damage_with_downed(&[100.0], &mut downed, 10.0, true);
        assert!(ev.downed_now);
        assert!(!ev.died);
        assert!(!combat.is_dead());
        assert!(downed.is_down());
        assert_eq!(downed.bleed_remaining(), 30.0);

        let bite = combat.take_damage_with_downed(&[15.0], &mut downed, 11.0, true);
        assert!(bite.bitten);
        assert!(!bite.died);
        assert_eq!(downed.bleed_remaining(), 26.0);

        let rev = downed.tick(4.0, true, false, false);
        assert_eq!(rev, game::DownedEvent::Revived);
        combat.health.revive(Some(downed.config().revive_health));
        assert_eq!(combat.health.current(), 25.0);

        // Second lethal hit within 60 s causes real death immediately.
        let redown = combat.take_damage_with_downed(&[30.0], &mut downed, 20.0, true);
        assert!(redown.died);
        assert!(!redown.downed_now);
        assert!(combat.is_dead());
        assert!(downed.is_dead());
    }
}
