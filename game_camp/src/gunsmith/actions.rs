use game_core::items::{Inventory, ItemRegistry, ItemStack, WEIGHT_EPSILON};
use game_core::weapons::AttachmentRegistry;

use crate::gunsmith::config::GunsmithConfig;
use crate::gunsmith::stock::{GunsmithStock, ShopKind};

// new: DECISIONS 2026-11-26 (treat condition >= 0.9999 as undamaged)
const FULL_CONDITION_EPS: f32 = 1e-4;
// new: DECISIONS 2026-11-26 (minimum condition factor when scrapping a broken gun)
const BROKEN_SELL_FLOOR: f32 = 0.1;

/// Clear error enum for UI status messages and transaction refusals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GunsmithError {
    NotInShop,
    TierLocked { required: u8 },
    SoldOut,
    NotEnoughScrap { needed: u32, have: u32 },
    BagFull,
    TooHeavy,
    CannotSell,
    NotAWeapon,
    AlreadyRepaired,
    IncompatibleAttachment,
    AlreadyFitted,
    SlotOccupied { slot_mod: String },
    ModNotFitted,
}

impl std::fmt::Display for GunsmithError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInShop => write!(f, "Not stocked here"),
            Self::TierLocked { required } => write!(f, "Requires camp tier {required}"),
            Self::SoldOut => write!(f, "Sold out"),
            Self::NotEnoughScrap { needed, have } => write!(f, "Need {needed} scrap (have {have})"),
            Self::BagFull => write!(f, "Bag is full"),
            Self::TooHeavy => write!(f, "Too heavy"),
            Self::CannotSell => write!(f, "Cannot sell that"),
            Self::NotAWeapon => write!(f, "Not a weapon"),
            Self::AlreadyRepaired => write!(f, "Already at full condition"),
            Self::IncompatibleAttachment => write!(f, "Doesn't fit this weapon"),
            Self::AlreadyFitted => write!(f, "Already fitted"),
            Self::SlotOccupied { .. } => write!(f, "Slot already has an attachment"),
            Self::ModNotFitted => write!(f, "Attachment is not fitted"),
        }
    }
}

impl std::error::Error for GunsmithError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuyResult {
    pub item_id: String,
    pub count_added: u32,
    pub scrap_spent: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SellResult {
    pub item_id: String,
    pub count_sold: u32,
    pub scrap_gained: u32,
    pub mods_returned: Vec<String>,
}

fn capacity_error(bag: &Inventory, id: &str, count: u32, items: &ItemRegistry) -> GunsmithError {
    let Some(def) = items.get(id) else {
        return GunsmithError::BagFull;
    };
    let room = bag.max_weight() - bag.total_weight_with(items) + WEIGHT_EPSILON;
    if def.weight * (count as f32) > room {
        GunsmithError::TooHeavy
    } else {
        GunsmithError::BagFull
    }
}

fn check_shop_offer(
    kind: &ShopKind,
    stock: &GunsmithStock,
    item_id: &str,
    cfg: &GunsmithConfig,
) -> Result<(), GunsmithError> {
    let Some(req_tier) = cfg.required_tier(kind, item_id) else {
        return Err(GunsmithError::NotInShop);
    };
    if let ShopKind::Camp { tier } = kind
        && req_tier > *tier
    {
        return Err(GunsmithError::TierLocked { required: req_tier });
    }
    if stock.qty(kind, item_id) == 0 {
        return Err(GunsmithError::SoldOut);
    }
    Ok(())
}

/// Checks whether 1 pack of `item_id` can be bought at `kind`, returning `(scrap_price, pack_size)`.
pub fn buy_check(
    kind: &ShopKind,
    stock: &GunsmithStock,
    item_id: &str,
    bag: &Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<(u32, u32), GunsmithError> {
    check_shop_offer(kind, stock, item_id, cfg)?;
    if !items.contains(item_id) {
        return Err(GunsmithError::NotInShop);
    }
    let entry = cfg.price_entry(item_id).ok_or(GunsmithError::NotInShop)?;
    let have = bag.count(&cfg.scrap_item);
    if have < entry.price {
        return Err(GunsmithError::NotEnoughScrap { needed: entry.price, have });
    }
    let mut test_bag = bag.clone();
    test_bag.remove(&cfg.scrap_item, entry.price);
    let pack = entry.pack_size.max(1);
    if !test_bag.can_add_with(item_id, pack, items) {
        return Err(capacity_error(&test_bag, item_id, pack, items));
    }
    Ok((entry.price, pack))
}

/// Buys 1 pack of `item_id` from `kind`, paying in `scrap_metal` from `bag`.
pub fn buy(
    kind: &ShopKind,
    stock: &mut GunsmithStock,
    item_id: &str,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<BuyResult, GunsmithError> {
    let (price, pack) = buy_check(kind, stock, item_id, bag, cfg, items)?;
    bag.remove(&cfg.scrap_item, price);
    bag.add_with_registry(item_id, pack, items);
    stock.take(kind, item_id, 1);
    Ok(BuyResult { item_id: item_id.to_string(), count_added: pack, scrap_spent: price })
}

/// Scrap gained for selling `count` units of `stack` (`None` if not sellable or 0 scrap).
pub fn sell_value(
    stack: &ItemStack,
    count: u32,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Option<u32> {
    if count == 0 || stack.item_id == cfg.scrap_item {
        return None;
    }
    let def = items.get(&stack.item_id)?;
    let unit_price = if let Some(entry) = cfg.price_entry(&stack.item_id) {
        entry.price as f32 / entry.pack_size.max(1) as f32
    } else if def.value > 0 {
        def.value as f32 / cfg.scrap_dollar_value.max(1) as f32
    } else {
        return None;
    };
    let cond = if def.is_weapon() {
        stack.condition().clamp(0.0, 1.0).max(BROKEN_SELL_FLOOR)
    } else {
        1.0
    };
    let scrap = (unit_price * (count as f32) * cfg.sell_rate * cond).floor() as u32;
    (scrap > 0).then_some(scrap)
}

fn finish_sell(
    test_bag: &mut Inventory,
    mods: &[String],
    scrap: u32,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<(), GunsmithError> {
    for mod_id in mods {
        if !test_bag.can_add_with(mod_id, 1, items) {
            return Err(capacity_error(test_bag, mod_id, 1, items));
        }
        test_bag.add_with_registry(mod_id, 1, items);
    }
    if !test_bag.can_add_with(&cfg.scrap_item, scrap, items) {
        return Err(capacity_error(test_bag, &cfg.scrap_item, scrap, items));
    }
    test_bag.add_with_registry(&cfg.scrap_item, scrap, items);
    Ok(())
}

/// Sells the whole `stack`, returning any fitted mods to `bag` first, then adding scrap.
pub fn sell(
    stack: &ItemStack,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<u32, GunsmithError> {
    let scrap = sell_value(stack, stack.count, cfg, items).ok_or(GunsmithError::CannotSell)?;
    let mut test_bag = bag.clone();
    finish_sell(&mut test_bag, &stack.mods, scrap, cfg, items)?;
    *bag = test_bag;
    bag.set_changed();
    Ok(scrap)
}

/// Sells `count` (or whole stack if 0) from `bag` at `slot_index`, returning fitted mods first.
pub fn sell_slot(
    slot_index: usize,
    count: u32,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<SellResult, GunsmithError> {
    let stack = bag.slot(slot_index).cloned().ok_or(GunsmithError::CannotSell)?;
    let sell_count = if count == 0 { stack.count } else { count.min(stack.count) };
    let scrap = sell_value(&stack, sell_count, cfg, items).ok_or(GunsmithError::CannotSell)?;

    let mut test_bag = bag.clone();
    test_bag.remove_from_slot(slot_index, sell_count);
    let mods_returned = if sell_count == stack.count { stack.mods.clone() } else { Vec::new() };
    finish_sell(&mut test_bag, &mods_returned, scrap, cfg, items)?;
    *bag = test_bag;
    bag.set_changed();

    Ok(SellResult {
        item_id: stack.item_id,
        count_sold: sell_count,
        scrap_gained: scrap,
        mods_returned,
    })
}

/// Scrap cost to repair `stack` to 1.0 condition (`0` if undamaged or not repairable).
pub fn repair_cost(stack: &ItemStack, cfg: &GunsmithConfig, items: &ItemRegistry) -> u32 {
    let Some(def) = items.get(&stack.item_id) else {
        return 0;
    };
    if !def.is_weapon() || def.max_stack > 1 {
        return 0;
    }
    let cond = stack.condition().clamp(0.0, 1.0);
    if cond >= 1.0 - FULL_CONDITION_EPS {
        return 0;
    }
    let price = cfg
        .price_entry(&stack.item_id)
        .map(|e| e.price)
        .unwrap_or_else(|| (def.value / cfg.scrap_dollar_value.max(1)).max(1));
    (cfg.repair_rate * (price as f32) * (1.0 - cond)).ceil().max(1.0) as u32
}

/// Repairs `stack` to full condition (`wear = None`), charging `repair_cost` scrap.
pub fn repair(
    stack: &mut ItemStack,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<u32, GunsmithError> {
    let Some(def) = items.get(&stack.item_id) else {
        return Err(GunsmithError::NotAWeapon);
    };
    if !def.is_weapon() || def.max_stack > 1 {
        return Err(GunsmithError::NotAWeapon);
    }
    let cost = repair_cost(stack, cfg, items);
    if cost == 0 {
        return Err(GunsmithError::AlreadyRepaired);
    }
    let have = bag.count(&cfg.scrap_item);
    if have < cost {
        return Err(GunsmithError::NotEnoughScrap { needed: cost, have });
    }
    bag.remove(&cfg.scrap_item, cost);
    stack.wear = None;
    Ok(cost)
}

pub fn repair_all_cost(bag: &Inventory, cfg: &GunsmithConfig, items: &ItemRegistry) -> u32 {
    bag.slots().iter().flatten().map(|s| repair_cost(s, cfg, items)).sum()
}

/// Repairs every worn weapon in `bag` atomically. Returns `(weapons_repaired, total_scrap_spent)`.
pub fn repair_all(
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<(u32, u32), GunsmithError> {
    let total = repair_all_cost(bag, cfg, items);
    if total == 0 {
        return Err(GunsmithError::AlreadyRepaired);
    }
    let have = bag.count(&cfg.scrap_item);
    if have < total {
        return Err(GunsmithError::NotEnoughScrap { needed: total, have });
    }
    bag.remove(&cfg.scrap_item, total);
    let mut count = 0;
    for slot in bag.slots_mut().iter_mut().flatten() {
        if repair_cost(slot, cfg, items) > 0 {
            slot.wear = None;
            count += 1;
        }
    }
    bag.set_changed();
    Ok((count, total))
}

/// Scrap cost to buy and fit `attachment_id` (`price + fit_labour`).
pub fn fit_cost(
    attachment_id: &str,
    cfg: &GunsmithConfig,
    attachments: &AttachmentRegistry,
) -> u32 {
    cfg.price_entry(attachment_id)
        .map(|e| e.price)
        .or_else(|| attachments.get(attachment_id).map(|a| a.price_scrap))
        .map_or(0, |price| price + cfg.fit_labour)
}

fn fit_inner(
    shop: Option<(&ShopKind, &mut GunsmithStock)>,
    stack: &mut ItemStack,
    attachment_id: &str,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
    attachments: &AttachmentRegistry,
) -> Result<u32, GunsmithError> {
    let Some(item_def) = items.get(&stack.item_id) else {
        return Err(GunsmithError::NotAWeapon);
    };
    if !item_def.is_weapon() || item_def.weapon_id.is_empty() {
        return Err(GunsmithError::NotAWeapon);
    }
    let Some(att_def) = attachments.get(attachment_id) else {
        return Err(GunsmithError::IncompatibleAttachment);
    };
    if !att_def.fits_weapon(&item_def.weapon_id) {
        return Err(GunsmithError::IncompatibleAttachment);
    }
    if stack.has_mod(attachment_id) {
        return Err(GunsmithError::AlreadyFitted);
    }
    for existing in &stack.mods {
        if let Some(existing_def) = attachments.get(existing)
            && existing_def.slot == att_def.slot
        {
            return Err(GunsmithError::SlotOccupied { slot_mod: existing.clone() });
        }
    }

    let from_bag = bag.has(attachment_id, 1);
    if !from_bag
        && let Some((kind, ref stock)) = shop
    {
        check_shop_offer(kind, stock, attachment_id, cfg)?;
    }
    let cost = if from_bag { cfg.fit_labour } else { fit_cost(attachment_id, cfg, attachments) };
    let have = bag.count(&cfg.scrap_item);
    if have < cost {
        return Err(GunsmithError::NotEnoughScrap { needed: cost, have });
    }
    if from_bag {
        bag.remove(attachment_id, 1);
    } else if let Some((kind, stock)) = shop {
        stock.take(kind, attachment_id, 1);
    }
    if cost > 0 {
        bag.remove(&cfg.scrap_item, cost);
    }
    stack.mods.push(attachment_id.to_string());
    Ok(cost)
}

/// Fits `attachment_id` onto `stack` (charges `fit_labour` if carried in `bag`, else `fit_cost`).
pub fn fit(
    stack: &mut ItemStack,
    attachment_id: &str,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
    attachments: &AttachmentRegistry,
) -> Result<u32, GunsmithError> {
    fit_inner(None, stack, attachment_id, bag, cfg, items, attachments)
}

/// Fits `attachment_id` onto `stack` at a shop (enforcing stock & tier when buying the mod).
#[allow(clippy::too_many_arguments)]
pub fn fit_at_shop(
    kind: &ShopKind,
    stock: &mut GunsmithStock,
    stack: &mut ItemStack,
    attachment_id: &str,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
    attachments: &AttachmentRegistry,
) -> Result<u32, GunsmithError> {
    fit_inner(Some((kind, stock)), stack, attachment_id, bag, cfg, items, attachments)
}

/// Removes `attachment_id` from `stack` and places the attachment item into `bag`.
pub fn remove_mod(
    stack: &mut ItemStack,
    attachment_id: &str,
    bag: &mut Inventory,
    items: &ItemRegistry,
) -> Result<(), GunsmithError> {
    let Some(pos) = stack.mods.iter().position(|m| m == attachment_id) else {
        return Err(GunsmithError::ModNotFitted);
    };
    if !bag.can_add_with(attachment_id, 1, items) {
        return Err(capacity_error(bag, attachment_id, 1, items));
    }
    stack.mods.remove(pos);
    bag.add_with_registry(attachment_id, 1, items);
    Ok(())
}

fn with_bag_slot<T>(
    slot_index: usize,
    bag: &mut Inventory,
    f: impl FnOnce(&mut ItemStack, &mut Inventory) -> Result<T, GunsmithError>,
) -> Result<T, GunsmithError> {
    let mut stack = bag.slot(slot_index).cloned().ok_or(GunsmithError::NotAWeapon)?;
    let out = f(&mut stack, bag)?;
    bag.slots_mut()[slot_index] = Some(stack);
    bag.set_changed();
    Ok(out)
}

pub fn repair_slot(
    slot_index: usize,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
) -> Result<u32, GunsmithError> {
    with_bag_slot(slot_index, bag, |s, b| repair(s, b, cfg, items))
}

pub fn fit_slot(
    slot_index: usize,
    attachment_id: &str,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
    attachments: &AttachmentRegistry,
) -> Result<u32, GunsmithError> {
    with_bag_slot(slot_index, bag, |s, b| fit(s, attachment_id, b, cfg, items, attachments))
}

#[allow(clippy::too_many_arguments)]
pub fn fit_slot_at_shop(
    kind: &ShopKind,
    stock: &mut GunsmithStock,
    slot_index: usize,
    attachment_id: &str,
    bag: &mut Inventory,
    cfg: &GunsmithConfig,
    items: &ItemRegistry,
    attachments: &AttachmentRegistry,
) -> Result<u32, GunsmithError> {
    with_bag_slot(slot_index, bag, |s, b| {
        fit_at_shop(kind, stock, s, attachment_id, b, cfg, items, attachments)
    })
}

pub fn remove_mod_slot(
    slot_index: usize,
    attachment_id: &str,
    bag: &mut Inventory,
    items: &ItemRegistry,
) -> Result<(), GunsmithError> {
    with_bag_slot(slot_index, bag, |s, b| remove_mod(s, attachment_id, b, items))
}
