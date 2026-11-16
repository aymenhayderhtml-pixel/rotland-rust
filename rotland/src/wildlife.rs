//! Wild animals on screen (game::animals): deer, rabbits and boars that graze, flee and (boars) charge.
//! Drawn like the zombies: one instanced batch per model. Also the butchering of a corpse with E
//! (Godot Animal.Interact / TickButcher / Finish) and `Prey`, which lets melee and guns hit zombies
//! and animals alike.

use std::collections::HashMap;

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::guns_fire::GunTarget;
use game::melee::MeleeTarget;
use game::{AnimalFrame, AnimalHunting, AnimalUpdateContext, Animals, DamageInfo, Inventory, SkillSystem};
use glam::{Mat4, Vec3};
use physics::PhysicsWorld;

use crate::combat::MeleeWorld;
use crate::shooting::TargetWorld;
use crate::zombies::Zombies;

/// Animal ids start here so they never meet zombie ids (zombies count up from 100).
pub const FIRST_ANIMAL_ID: u64 = 1 << 32;

struct Look {
    mesh: GpuMesh,
    rig: Rig,
    clips: Vec<pack::Clip>,
}

pub struct Wildlife {
    pub sim: Animals,
    looks: Vec<Look>,
    look_index: HashMap<String, usize>,
    /// The corpse being butchered, if any.
    butchering: Option<u64>,
}

impl Wildlife {
    pub fn new(device: &wgpu::Device, pack: &pack::ModelPack, seed: i32) -> Result<Self, String> {
        let config = game::AnimalConfigData::from_repo()?;
        let mut sim = Animals::new(config, seed as u64 ^ 0x9E37_79B9_7F4A_7C15);
        sim.set_first_id(FIRST_ANIMAL_ID);
        let mut looks = Vec::new();
        let mut look_index = HashMap::new();
        let names: Vec<String> = sim.config.species.values().flat_map(|s| std::iter::once(s.model.clone()).chain(s.buck_model.clone())).collect();
        for name in names {
            if look_index.contains_key(&name) {
                continue;
            }
            let Some(m) = pack.find(&name) else {
                log::warn!("animal model {name} is not in the pack");
                continue;
            };
            let Some(skel) = m.skeleton.as_ref() else { continue };
            look_index.insert(name.clone(), looks.len());
            looks.push(Look { mesh: GpuMesh::new(device, &name, &m.vertices), rig: Rig::new(skel), clips: m.clips.clone() });
        }
        Ok(Self { sim, looks, look_index, butchering: None })
    }

    pub fn update(&mut self, dt: f32, ctx: &AnimalUpdateContext, physics: &PhysicsWorld) -> AnimalFrame {
        self.sim.update(dt, ctx, physics)
    }

    /// Adds every animal's skin matrices to `bones` and returns one batch per model.
    pub fn pose(&self, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut per_look: Vec<Vec<SkinInstance>> = vec![Vec::new(); self.looks.len()];
        for a in &self.sim.list {
            if a.retired || !a.visible {
                continue;
            }
            let Some(&li) = self.look_index.get(&a.model_id) else { continue };
            let look = &self.looks[li];
            let Some(clip) = look.clips.iter().find(|c| c.name == a.anim_clip).or_else(|| look.clips.iter().find(|c| c.name == "idle")) else {
                continue;
            };
            // Death plays once and holds its last pose; everything else loops
            let t = if a.is_dead() { a.anim_time.min(clip.duration - 0.01).max(0.0) } else { a.anim_time };
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, t, &mut bones[base..]);
            let model = Mat4::from_translation(a.position) * Mat4::from_rotation_y(a.yaw);
            per_look[li].push(SkinInstance::new(model, base as u32));
        }
        per_look
            .into_iter()
            .zip(&self.looks)
            .filter(|(v, _)| !v.is_empty())
            .map(|(instances, look)| SkinnedBatch { mesh: &look.mesh, instances })
            .collect()
    }

    /// The corpse in front of the player that can still be butchered: (id, species name).
    pub fn corpse_in_front(&self, player: Vec3, facing: Vec3) -> Option<(u64, String)> {
        let reach = self.sim.config.butcher.range;
        let flat = Vec3::new(facing.x, 0.0, facing.z).normalize_or_zero();
        self.sim
            .list
            .iter()
            .filter(|a| a.is_dead() && !a.retired && !a.butcher.done)
            .filter(|a| {
                let d = Vec3::new(a.position.x - player.x, 0.0, a.position.z - player.z);
                d.length() < reach && (d.length() < 0.8 || d.normalize_or_zero().dot(flat) > 0.4)
            })
            .min_by(|a, b| a.position.distance_squared(player).total_cmp(&b.position.distance_squared(player)))
            .map(|a| (a.id, a.species_data.name.clone()))
    }

    /// E on a corpse and the timer that follows. Returns the prompt to show and a toast when done.
    pub fn butcher_tick(
        &mut self,
        dt: f32,
        pressed: bool,
        player: Vec3,
        facing: Vec3,
        bag: &mut Inventory,
        skills: &mut SkillSystem,
    ) -> (Option<String>, Option<String>) {
        let cfg = self.sim.config.butcher.clone();
        if let Some(id) = self.butchering {
            let Some(a) = self.sim.list.iter_mut().find(|a| a.id == id) else {
                self.butchering = None;
                return (None, None);
            };
            let far = Vec3::new(a.position.x - player.x, 0.0, a.position.z - player.z).length() > cfg.give_up_range;
            if far {
                a.butcher.cancel();
                self.butchering = None;
                return (None, Some("Butchering stopped".into()));
            }
            if !a.butcher.tick(dt) {
                return (Some(format!("Butchering... {:.0}s", a.butcher.left.max(0.0).ceil())), None);
            }
            let name = a.species_data.name.to_lowercase();
            self.butchering = None;
            let got = self.sim.butcher_with_hunting_level(id, bag, skills.get_level("hunting"));
            if cfg.xp > 0 {
                skills.add_xp("hunting", cfg.xp as f32);
            }
            let list: Vec<String> = got.iter().map(|s| format!("{} {}", s.count, s.item_id.replace('_', " "))).collect();
            let toast = if list.is_empty() { format!("Butchered the {name}") } else { format!("Butchered the {name}: {}", list.join(", ")) };
            return (None, Some(toast));
        }
        let Some((id, name)) = self.corpse_in_front(player, facing) else { return (None, None) };
        if pressed && let Some(a) = self.sim.list.iter_mut().find(|a| a.id == id) {
            a.butcher.start(AnimalHunting::butcher_seconds(bag, &cfg));
            self.butchering = Some(id);
        }
        (Some(format!("[E] Butcher {}", name.to_lowercase())), None)
    }
}

/// Everything a swing or a bullet can hit this frame: the zombies, and the animals when there are any.
/// Hits on animals do more with the hunting skill (Godot AnimalHunting.DamageMultiplier).
pub struct Prey<'a> {
    pub zombies: &'a mut Zombies,
    pub animals: Option<&'a mut Animals>,
    /// The skills' "animalDamage" bonus.
    pub animal_bonus: f32,
}

impl Prey<'_> {
    fn hit_any(&mut self, id: u64, mut damage: DamageInfo, knockback: Vec3) -> bool {
        match self.animals.as_deref_mut() {
            Some(animals) if id >= FIRST_ANIMAL_ID => {
                damage.amount *= AnimalHunting::damage_multiplier(true, self.animal_bonus);
                animals.hit(id, damage, knockback)
            }
            _ => self.zombies.hit(id, damage, knockback),
        }
    }
}

impl MeleeWorld for Prey<'_> {
    fn melee_targets(&self) -> Vec<MeleeTarget> {
        let mut list = self.zombies.targets();
        if let Some(a) = &self.animals {
            list.extend(a.targets());
        }
        list
    }

    fn melee_hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        self.hit_any(id, damage, knockback)
    }
}

impl TargetWorld for Prey<'_> {
    fn gun_targets(&self) -> Vec<GunTarget> {
        let mut list = self.zombies.gun_targets();
        if let Some(animals) = &self.animals {
            list.extend(animals.list.iter().filter(|a| !a.is_dead() && !a.retired).map(|a| {
                let sp = &a.species_data;
                // No head shots on animals: a zero-size head on top only sets the body's height
                GunTarget { id: a.id, position: a.position, body_radius: sp.body_radius, head_center: a.position + Vec3::Y * sp.body_height, head_radius: 0.0 }
            }));
        }
        list
    }

    fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        self.hit_any(id, damage, knockback)
    }
}
