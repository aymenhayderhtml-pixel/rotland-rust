//! Gunsmith shop session, view-model construction, action execution, camp bench, and town shopkeepers.

use std::sync::OnceLock;

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::{AttachSlot, AttachmentDef, AttachmentRegistry, ShopKind, WeaponDef};
use glam::{Mat4, Vec3};
use physics::{ColliderHandle, PhysicsWorld};
use rotland_ui::gunsmith_ui::{
    GunsmithAction, GunsmithBuyItem, GunsmithFittedMod, GunsmithModOption, GunsmithRepairItem,
    GunsmithScreen, GunsmithSellItem, GunsmithSlot, GunsmithSlotState, GunsmithTab,
    GunsmithUpgradeWeapon, GunsmithView, GunsmithWeaponStats,
};
use world::towns::{TownShopkeeper, TownsData};
use world::{CampLayoutConfigData, WorldGenerator};

use crate::hud::{Hud, NotificationKind};
use crate::play::PlayState;
use crate::props::{Placed, PropLibrary};

const DEFAULT_BASE_FOV: f32 = 75.0; // data: settings.json default fov
const SHOPKEEPER_VIEW_DIST: f32 = 60.0; // data: camp_life.json farRange
const SHOPKEEPER_PHASE_STEP: f32 = 0.37; // new: DECISIONS 2026-11-25

/// Active open gunsmith shop UI session.
pub struct GunsmithSession {
    pub screen: GunsmithScreen,
    pub kind: ShopKind,
    /// NPC world position when opened in-world (`None` for `--gunsmith*` dev flags).
    pub anchor: Option<Vec3>,
}

impl GunsmithSession {
    pub fn new(tab: GunsmithTab, kind: ShopKind, anchor: Option<Vec3>) -> Self {
        Self {
            screen: GunsmithScreen::new(tab),
            kind,
            anchor,
        }
    }
}

struct KeeperLook {
    mesh: GpuMesh,
    rig: Rig,
    clips: Vec<pack::Clip>,
}

/// World visuals and colliders for the camp `gunsmith_bench` and the 4 town `shopkeeper_<key>` NPCs.
#[derive(Default)]
pub struct GunsmithWorld {
    bench_pos: Option<(Vec3, f32)>,
    bench_placed: Option<Placed>,
    bench_collider: Option<ColliderHandle>,
    shopkeepers: Vec<TownShopkeeper>,
    keeper_look: Option<KeeperLook>,
    keeper_time: f32,
}

impl GunsmithWorld {
    pub fn new(
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        world_gen: &WorldGenerator,
        physics: &mut PhysicsWorld,
    ) -> Self {
        let mut bench_pos = None;
        let mut bench_collider = None;
        if let Some(site) = world_gen.settlements().first() {
            let layout_path = paths::repo_path("data/camp_layout.json");
            let cfg = CampLayoutConfigData::load_from_file(&layout_path).unwrap_or_default();
            let pos = site.center + site.local(cfg.gunsmith_offset[0], 0.0, cfg.gunsmith_offset[1]);
            let yaw = site.yaw + cfg.gunsmith_yaw;
            bench_pos = Some((pos, yaw));
            if let Some(model) = pack.find("gunsmith_bench") {
                let shapes: Vec<Vec<[f32; 3]>> =
                    model.colliders.iter().map(|c| c.points.clone()).collect();
                if !shapes.is_empty() {
                    bench_collider = Some(physics.add_static(&shapes, pos, yaw, 1.0));
                    physics.update_broad_phase();
                }
            }
        }

        let towns_data = TownsData::load_embedded();
        let shopkeepers = world_gen
            .towns()
            .iter()
            .filter_map(|town| {
                let lot = town.gun_shop_lot()?;
                let ground_y = world_gen.height(lot.pos.x, lot.pos.y);
                town.shopkeeper_spawn(&towns_data, ground_y)
            })
            .collect();

        let keeper_look = pack
            .find("npc_mechanic")
            .or_else(|| pack.find("npc_trader"))
            .and_then(|m| {
                let skel = m.skeleton.as_ref()?;
                Some(KeeperLook {
                    mesh: GpuMesh::new(device, &m.name, &m.vertices),
                    rig: Rig::new(skel),
                    clips: m.clips.clone(),
                })
            });

        Self {
            bench_pos,
            bench_placed: None,
            bench_collider,
            shopkeepers,
            keeper_look,
            keeper_time: 0.0,
        }
    }

    /// Lazily resolves the camp `gunsmith_bench` prop slot.
    pub fn ensure_bench_placed(
        &mut self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        library: &mut PropLibrary,
    ) {
        if self.bench_placed.is_none()
            && let Some((pos, yaw)) = self.bench_pos
            && let Some(slot) = library.get(device, pack, "gunsmith_bench", true)
        {
            self.bench_placed = Some(library.place(slot, pos, yaw, 1.0));
        }
        let _ = &self.bench_collider;
    }

    /// Returns the camp `gunsmith_bench` prop instance for rendering.
    pub fn bench_placed(&self) -> Option<&Placed> {
        self.bench_placed.as_ref()
    }

    pub fn update(&mut self, dt: f32) {
        self.keeper_time += dt;
    }

    /// Poses town gun shopkeepers within `SHOPKEEPER_VIEW_DIST` of `player`.
    pub fn pose_shopkeepers(&self, player: Vec3, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let Some(look) = &self.keeper_look else {
            return Vec::new();
        };
        let Some(clip) = look.clips.iter().find(|c| c.name == "idle") else {
            return Vec::new();
        };
        let max_sq = SHOPKEEPER_VIEW_DIST * SHOPKEEPER_VIEW_DIST;
        let mut instances = Vec::new();
        for (i, keeper) in self.shopkeepers.iter().enumerate() {
            let dx = player.x - keeper.position.x;
            let dz = player.z - keeper.position.z;
            if dx * dx + dz * dz > max_sq {
                continue;
            }
            let phase = (i as f32 * SHOPKEEPER_PHASE_STEP) % 1.0;
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, self.keeper_time + phase, &mut bones[base..]);
            let model = Mat4::from_translation(keeper.position) * Mat4::from_rotation_y(keeper.yaw);
            instances.push(SkinInstance::new(model, base as u32));
        }
        if instances.is_empty() {
            Vec::new()
        } else {
            vec![SkinnedBatch {
                mesh: &look.mesh,
                instances,
            }]
        }
    }

    /// Returns the nearest town shopkeeper in front of the player within `range` metres.
    pub fn facing_shopkeeper(
        &self,
        player: Vec3,
        facing: Vec3,
        range: f32,
    ) -> Option<(&TownShopkeeper, f32)> {
        let flat_fwd = Vec3::new(facing.x, 0.0, facing.z).normalize_or_zero();
        if flat_fwd == Vec3::ZERO {
            return None;
        }
        let mut best: Option<(&TownShopkeeper, f32)> = None;
        for keeper in &self.shopkeepers {
            let to = keeper.position - player;
            let dist = to.length();
            if dist > range {
                continue;
            }
            let flat_to = Vec3::new(to.x, 0.0, to.z).normalize_or_zero();
            if (flat_to == Vec3::ZERO || flat_to.dot(flat_fwd) >= 0.45)
                && best.is_none_or(|(_, d)| dist < d)
            {
                best = Some((keeper, dist));
            }
        }
        best
    }
}

fn gun_feel_cfg() -> &'static game::gun_feel::GunFeelConfig {
    static CFG: OnceLock<game::gun_feel::GunFeelConfig> = OnceLock::new();
    CFG.get_or_init(|| game::gun_feel::GunFeelConfig::load().unwrap_or_default())
}

fn slot_to_ui(slot: AttachSlot) -> GunsmithSlot {
    match slot {
        AttachSlot::Optic => GunsmithSlot::Optic,
        AttachSlot::Muzzle => GunsmithSlot::Muzzle,
        AttachSlot::Magazine => GunsmithSlot::Magazine,
        AttachSlot::Grip => GunsmithSlot::Grip,
        AttachSlot::Stock => GunsmithSlot::Stock,
    }
}

fn weapon_stats(base: &WeaponDef, mods: &[String], reg: &AttachmentRegistry) -> GunsmithWeaponStats {
    let eff = game::effective_stats(base, mods, reg);
    let base_kick = gun_feel_cfg().get(&base.id).map_or(1.0, |g| g.kick_deg);
    let km = game::kick_mult(mods, &base.id, reg);
    GunsmithWeaponStats {
        damage: eff.damage,
        spread_deg: eff.spread_aim_deg,
        kick: base_kick * km,
        mag_size: eff.magazine_size,
        noise_radius: eff.noise_radius,
        zoom_x: eff
            .scope
            .as_ref()
            .map_or(1.0, |s| DEFAULT_BASE_FOV / s.fov.max(1.0)),
    }
}

fn format_mod_summary(att: &AttachmentDef) -> String {
    let mut parts = Vec::new();
    let fx = &att.effects;
    if let Some(fov) = fx.scope_fov {
        parts.push(format!("{:.1}x zoom", DEFAULT_BASE_FOV / fov.max(1.0)));
    }
    if (fx.spread_mult - 1.0).abs() > 1e-3 {
        parts.push(format!("Spread x{:.2}", fx.spread_mult));
    }
    if (fx.kick_mult - 1.0).abs() > 1e-3 {
        parts.push(format!("Kick x{:.2}", fx.kick_mult));
    }
    if (fx.noise_mult - 1.0).abs() > 1e-3 {
        parts.push(format!("Noise x{:.2}", fx.noise_mult));
    }
    if fx.mag_bonus > 0.0 {
        parts.push(format!("+{}% mag", (fx.mag_bonus * 100.0).round() as i32));
    }
    if (fx.reload_mult - 1.0).abs() > 1e-3 {
        parts.push(format!("Reload x{:.2}", fx.reload_mult));
    }
    parts.join(" · ")
}

/// Builds the UI snapshot for `GunsmithScreen`.
pub fn build_view(kind: &ShopKind, play: &PlayState) -> GunsmithView {
    let cfg = &play.camp.gunsmith_config;
    let stock = &play.camp.gunsmith;
    let bag = &play.inventory;
    let items = &play.items;
    let atts = &play.attachments;

    let shop_title = cfg.shop_name(kind);
    let subtitle = match kind {
        ShopKind::Camp { tier } => format!("Camp tier {tier} · Scrap metal trade"),
        ShopKind::Town { key } => cfg
            .towns
            .get(key)
            .map(|t| t.flavour.clone())
            .unwrap_or_default(),
    };

    let mut buy_items = Vec::new();
    if let Some(shop_stock) = stock.shop(kind) {
        for item_id in shop_stock.order() {
            let Some(def) = items.get(item_id) else { continue };
            let Some(entry) = cfg.price_entry(item_id) else { continue };
            let req_tier = cfg.required_tier(kind, item_id).unwrap_or(0);
            let locked_camp_tier = match kind {
                ShopKind::Camp { tier } if req_tier > *tier => Some(req_tier),
                _ => None,
            };
            let qty = shop_stock.qty(item_id);
            let blocker_reason = if locked_camp_tier.is_none() && qty > 0 {
                match game::buy_check(kind, stock, item_id, bag, cfg, items) {
                    Err(e @ (game::GunsmithError::BagFull | game::GunsmithError::TooHeavy)) => {
                        Some(e.to_string())
                    }
                    _ => None,
                }
            } else {
                None
            };
            let summary = if entry.pack_size > 1 {
                format!("Pack of {} · {}", entry.pack_size, def.description)
            } else if let Some(att) = atts.get(item_id) {
                format_mod_summary(att)
            } else {
                def.description.clone()
            };
            buy_items.push(GunsmithBuyItem {
                item_id: item_id.clone(),
                name: def.name.clone(),
                category: def.category.as_str().to_string(),
                price_scrap: entry.price,
                stock: qty,
                color_hex: def.color.clone(),
                locked_camp_tier,
                blocker_reason,
                summary,
            });
        }
    }

    let mut sell_items = Vec::new();
    let mut repair_items = Vec::new();
    let mut upgrade_weapons = Vec::new();

    for (slot, stack_opt) in bag.slots().iter().enumerate() {
        let Some(stack) = stack_opt else { continue };
        let Some(def) = items.get(&stack.item_id) else { continue };
        let cond = stack.condition();

        if let Some(scrap_each) = game::sell_value(stack, 1, cfg, items) {
            sell_items.push(GunsmithSellItem {
                slot,
                item_id: stack.item_id.clone(),
                name: def.name.clone(),
                count: stack.count,
                scrap_each,
                condition: def.is_weapon().then_some(cond),
                mod_count: stack.mods.len(),
                color_hex: def.color.clone(),
            });
        }

        if def.is_weapon() && def.max_stack == 1 {
            repair_items.push(GunsmithRepairItem {
                slot,
                item_id: stack.item_id.clone(),
                name: def.name.clone(),
                condition: cond,
                is_broken: game::is_broken(cond),
                repair_cost_scrap: game::repair_cost(stack, cfg, items),
                color_hex: def.color.clone(),
            });
        }

        if def.is_weapon()
            && !def.weapon_id.is_empty()
            && let Some(base_weapon) = play.weapons.get(&def.weapon_id)
            && atts.for_weapon(&base_weapon.id).next().is_some()
        {
            let base_stats = weapon_stats(base_weapon, &stack.mods, atts);
            let slots = std::array::from_fn(|idx| {
                let attach_slot = AttachSlot::ALL[idx];
                let ui_slot = slot_to_ui(attach_slot);
                let mut options = Vec::new();
                for att in atts.for_weapon(&base_weapon.id) {
                    if att.slot != attach_slot {
                        continue;
                    }
                    let owned_in_bag = bag.has(&att.id, 1);
                    let locked_reason = if !owned_in_bag {
                        match cfg.required_tier(kind, &att.id) {
                            None => Some("Not stocked here".to_string()),
                            Some(req) => match kind {
                                ShopKind::Camp { tier } if req > *tier => {
                                    Some(format!("Camp tier {req}"))
                                }
                                _ if stock.qty(kind, &att.id) == 0 => {
                                    Some("Sold out".to_string())
                                }
                                _ => None,
                            },
                        }
                    } else {
                        None
                    };
                    let mut sim_mods: Vec<String> = stack
                        .mods
                        .iter()
                        .filter(|m| atts.get(m).is_some_and(|d| d.slot != attach_slot))
                        .cloned()
                        .collect();
                    sim_mods.push(att.id.clone());
                    options.push(GunsmithModOption {
                        id: att.id.clone(),
                        name: att.name.clone(),
                        slot: ui_slot,
                        price_scrap: game::fit_cost(&att.id, cfg, atts),
                        owned_in_bag,
                        locked_reason,
                        after_stats: weapon_stats(base_weapon, &sim_mods, atts),
                        summary: format_mod_summary(att),
                    });
                }
                let fitted_mod = stack
                    .mods
                    .iter()
                    .find_map(|m| atts.get(m).filter(|d| d.slot == attach_slot))
                    .map(|d| GunsmithFittedMod {
                        id: d.id.clone(),
                        name: d.name.clone(),
                        summary: format_mod_summary(d),
                    });
                GunsmithSlotState {
                    slot: ui_slot,
                    supported: !options.is_empty(),
                    fitted_mod,
                    options,
                }
            });
            upgrade_weapons.push(GunsmithUpgradeWeapon {
                slot,
                item_id: stack.item_id.clone(),
                name: def.name.clone(),
                condition: cond,
                color_hex: def.color.clone(),
                base_stats,
                slots,
            });
        }
    }

    GunsmithView {
        shop_title,
        subtitle,
        scrap_count: bag.count(&cfg.scrap_item),
        buy_items,
        sell_items,
        repair_items,
        upgrade_weapons,
    }
}

/// Applies one UI action against inventory and gunsmith stock. Returns status text (`None` on close).
pub fn apply_action(
    action: GunsmithAction,
    kind: &ShopKind,
    play: &mut PlayState,
    combat: Option<&mut crate::combat::Combat>,
    hud: &mut Hud,
) -> Option<String> {
    let items = play.items.clone();
    let cfg = &play.camp.gunsmith_config;
    let stock = &mut play.camp.gunsmith;
    let msg = match action {
        GunsmithAction::Close => return None,
        GunsmithAction::Buy { item_id, count } => {
            let mut bought_units = 0u32;
            let mut spent = 0u32;
            let mut err_msg = None;
            for _ in 0..count.max(1) {
                match game::gunsmith::buy(kind, stock, &item_id, &mut play.inventory, cfg, &items) {
                    Ok(res) => {
                        bought_units += res.count_added;
                        spent += res.scrap_spent;
                    }
                    Err(e) => {
                        err_msg = Some(e.to_string());
                        break;
                    }
                }
            }
            if bought_units > 0 {
                hud.on_pickup(&item_id, bought_units);
                let label = items.get(&item_id).map_or(item_id.as_str(), |d| d.name.as_str());
                format!("Bought {bought_units} {label} (-{spent} scrap)")
            } else {
                err_msg.unwrap_or_else(|| "Can't buy that".to_string())
            }
        }
        GunsmithAction::Sell { slot, count, .. } => {
            match game::sell_slot(slot, count, &mut play.inventory, cfg, &items) {
                Ok(res) => {
                    let label = items
                        .get(&res.item_id)
                        .map_or(res.item_id.as_str(), |d| d.name.as_str());
                    format!("Sold {} {label} (+{} scrap)", res.count_sold, res.scrap_gained)
                }
                Err(e) => e.to_string(),
            }
        }
        GunsmithAction::Repair { slot } => {
            match game::repair_slot(slot, &mut play.inventory, cfg, &items) {
                Ok(cost) => {
                    hud.notify("Weapon repaired", NotificationKind::Info);
                    format!("Repaired (-{cost} scrap)")
                }
                Err(e) => e.to_string(),
            }
        }
        GunsmithAction::RepairAll => {
            match game::repair_all(&mut play.inventory, cfg, &items) {
                Ok((n, total)) => {
                    hud.notify("Weapons repaired", NotificationKind::Info);
                    format!("Repaired {n} weapon(s) (-{total} scrap)")
                }
                Err(e) => e.to_string(),
            }
        }
        GunsmithAction::FitMod { weapon_slot, attachment_id } => {
            match game::fit_slot_at_shop(
                kind,
                stock,
                weapon_slot,
                &attachment_id,
                &mut play.inventory,
                cfg,
                &items,
                &play.attachments,
            ) {
                Ok(cost) => {
                    let name = play
                        .attachments
                        .get(&attachment_id)
                        .map_or(attachment_id.as_str(), |d| d.name.as_str());
                    if cost > 0 {
                        format!("Fitted {name} (-{cost} scrap)")
                    } else {
                        format!("Fitted {name}")
                    }
                }
                Err(e) => e.to_string(),
            }
        }
        GunsmithAction::RemoveMod { weapon_slot, attachment_id } => {
            match game::remove_mod_slot(weapon_slot, &attachment_id, &mut play.inventory, &items) {
                Ok(()) => {
                    let name = play
                        .attachments
                        .get(&attachment_id)
                        .map_or(attachment_id.as_str(), |d| d.name.as_str());
                    format!("Removed {name}")
                }
                Err(e) => e.to_string(),
            }
        }
    };
    play.hotbar.on_inventory_changed(&play.inventory);
    play.refresh_equipped_stats(combat);
    Some(msg)
}
