//! Combat hits, damage processing, corpse recording, and kill tracking.

use game::DamageInfo;
use glam::Vec3;

use super::types::BODY_RADIUS;
use super::Zombies;

impl Zombies {
    /// Applies a hit from the player. Returns true if it killed the zombie.
    pub fn hit(&mut self, id: u64, mut damage: DamageInfo, knockback: Vec3) -> bool {
        let vulnerable_multiplier = self.special_cfg.as_deref().map_or(1.0, |c| c.vulnerable_multiplier);
        let Some(z) = self.list.iter_mut().find(|z| z.id == id) else {
            return false;
        };
        let headshot = damage.headshot;
        let feet_y = z.position.y;
        if damage.hit_point != Vec3::ZERO
            && (damage.hit_point.y - feet_y) < self.defs.behaviour.leg_hit_height
        {
            damage.leg_hit = true;
        }
        if z.special_vulnerable {
            damage.amount *= vulnerable_multiplier;
        }
        let result = z.health.take_damage(damage.clone());
        if let Some(special) = z.special.as_mut() {
            special.on_hit(result.died_now);
        }
        if result.dealt > 0.0 {
            let flat_knock = Vec3::new(knockback.x, 0.0, knockback.z);
            if flat_knock.length_squared() > 0.0 {
                z.last_knock = flat_knock;
            }
            z.knock += flat_knock;
            z.pending_hit = Some(damage);
        }
        if result.died_now {
            self.total_kills += 1;
            self.kills.push((z.kind.clone(), headshot));
            self.corpses.record(z.position, self.game_time);
        }
        result.died_now
    }

    /// A shove from the player: flat push and stagger, no damage. Returns false for dead or immune zombies.
    pub fn shove(&mut self, id: u64, push: Vec3, stagger_seconds: f32, immune_kinds: &[String]) -> bool {
        let Some(z) = self.list.iter_mut().find(|z| z.id == id) else {
            return false;
        };
        if z.health.is_dead() || immune_kinds.contains(&z.kind) {
            return false;
        }
        let flat = Vec3::new(push.x, 0.0, push.z);
        z.knock += flat;
        z.last_knock = flat;
        z.brain.stagger_for(stagger_seconds);
        true
    }

    /// Applies a hit with explicit hit point height. Hits below 0.6 m above feet set leg_hit.
    pub fn hit_with_height(
        &mut self,
        id: u64,
        mut damage: DamageInfo,
        knockback: Vec3,
        hit_point_y: f32,
    ) -> bool {
        let feet_y = self.list.iter().find(|z| z.id == id).map_or(0.0, |z| z.position.y);
        damage.hit_point.y = hit_point_y;
        if (hit_point_y - feet_y) < self.defs.behaviour.leg_hit_height {
            damage.leg_hit = true;
        }
        self.hit(id, damage, knockback)
    }

    /// Convenience alias for hit_with_height matching task brief.
    pub fn hit_at_height(
        &mut self,
        id: u64,
        damage: DamageInfo,
        knockback: Vec3,
        hit_point_y: f32,
    ) -> bool {
        self.hit_with_height(id, damage, knockback, hit_point_y)
    }

    /// Records an external corpse position (e.g. from wildlife or camp guards) for zombie eating.
    pub fn record_corpse(&mut self, position: Vec3) {
        self.corpses.record(position, self.game_time);
    }

    /// A hit that is not the player's (camp guards): a kill does not count for the player.
    pub fn hit_unclaimed(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        let killed = self.hit(id, damage, knockback);
        if killed {
            self.total_kills = self.total_kills.saturating_sub(1);
            self.kills.pop();
        }
        killed
    }

    /// (type, was a headshot) of each zombie the player killed since the last call.
    pub fn take_kills(&mut self) -> Vec<(String, bool)> {
        std::mem::take(&mut self.kills)
    }

    /// Living zombies as melee targets.
    pub fn targets(&self) -> Vec<game::melee::MeleeTarget> {
        self.list
            .iter()
            .filter(|z| !z.health.is_dead())
            .map(|z| game::melee::MeleeTarget {
                id: z.id,
                position: z.position,
                radius: BODY_RADIUS,
            })
            .collect()
    }

    pub fn alive(&self) -> usize {
        self.list.iter().filter(|z| !z.health.is_dead()).count()
    }
}
