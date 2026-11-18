//! Shared item text, weapon stat, and action button helpers for inventory, crafting, shop, trunk, and bank screens.

use std::sync::OnceLock;

use engine::egui::{self, vec2, Response, Ui, WidgetText};
use game::{ItemDef, ItemEffects, RecipeDef, WeaponDef, WeaponRegistry, CAMPFIRE_STATION};

pub const ACTION_BTN_SIZE: egui::Vec2 = vec2(130.0, 32.0);

/// Reusable action button helper for bag and details panels (Issue #10).
pub fn action_button(ui: &mut Ui, text: impl Into<WidgetText>, enabled: bool) -> Response {
    ui.add_enabled(enabled, egui::Button::new(text).min_size(ACTION_BTN_SIZE))
}

/// Resolves an item's display name from its definition, falling back to its item ID.
pub fn format_item_name<'a>(id: &'a str, def: Option<&'a ItemDef>) -> &'a str {
    def.map(|d| d.name.as_str()).unwrap_or(id)
}

/// Formats an item category with capitalized first letter.
pub fn format_item_category(category: &str) -> String {
    let mut chars = category.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Formats single-item and total stack weight.
pub fn format_item_weight(weight_each: f32, count: u32) -> String {
    let total = weight_each * count as f32;
    format!("Weight: {weight_each:.2} kg each, {total:.2} kg total")
}

/// Formats an item row in the car trunk screen with optional blocker reason.
pub fn format_trunk_row(name: &str, count: u32, blocker: Option<&str>) -> String {
    match blocker {
        Some(b) => format!("{name}   x{count}   ({b})"),
        None => format!("{name}   x{count}"),
    }
}

/// Formats an item row in the bank screen with optional full notice.
pub fn format_bank_row(name: &str, count: u32, is_full: bool, full_msg: &str) -> String {
    if is_full {
        format!("{name}   x{count}   ({full_msg})")
    } else {
        format!("{name}   x{count}")
    }
}

/// Formats a buy or sell row in the trader shop screen.
pub fn format_shop_row(name: &str, price: u32, count: u32) -> String {
    format!("{name}   ${price}   x{count}")
}

/// Formats survival effects (hunger, thirst, health) for food, drink, and medicine.
pub fn format_item_effects(effects: &ItemEffects) -> String {
    let mut parts = Vec::new();
    if effects.hunger != 0.0 {
        parts.push(format!(
            "Hunger {}{:.0}",
            if effects.hunger > 0.0 { "+" } else { "" },
            effects.hunger
        ));
    }
    if effects.thirst != 0.0 {
        parts.push(format!(
            "Thirst {}{:.0}",
            if effects.thirst > 0.0 { "+" } else { "" },
            effects.thirst
        ));
    }
    if effects.health != 0.0 {
        parts.push(format!(
            "Health {}{:.0}",
            if effects.health > 0.0 { "+" } else { "" },
            effects.health
        ));
    }
    parts.join(", ")
}

/// Lazily loads and caches the weapon registry from data/weapons.json.
pub fn get_weapon_def(weapon_id: &str) -> Option<WeaponDef> {
    static REGISTRY: OnceLock<Option<WeaponRegistry>> = OnceLock::new();
    let reg = REGISTRY.get_or_init(|| {
        let path = paths::repo_path("data/weapons.json");
        WeaponRegistry::load(&path).ok()
    });
    reg.as_ref().and_then(|r| r.get(weapon_id).cloned())
}

/// Formats weapon stats for the item details panel (Issue #11).
pub fn format_weapon_stats(weapon: &WeaponDef, ammo_name: Option<&str>) -> Vec<String> {
    let mut lines = Vec::new();
    if weapon.is_gun() {
        if weapon.pellets > 1 {
            lines.push(format!(
                "Damage: {:.0} x {} (headshot x{:.1})",
                weapon.damage, weapon.pellets, weapon.headshot_multiplier
            ));
        } else {
            lines.push(format!(
                "Damage: {:.0} (headshot x{:.1})",
                weapon.damage, weapon.headshot_multiplier
            ));
        }
        lines.push(format!(
            "Fire rate: {:.2}s  ·  Range: {:.0}m",
            weapon.fire_interval, weapon.range
        ));
        let ammo_str = ammo_name.map_or(String::new(), |a| format!(", uses {a}"));
        lines.push(format!(
            "Magazine: {} rounds{}",
            weapon.magazine_size, ammo_str
        ));
        if weapon.scope.is_some() {
            lines.push("Scope: hold aim to look through it".to_string());
        }
        if weapon.auto {
            lines.push("Full-auto: hold fire to keep shooting".to_string());
        }
    } else {
        lines.push(format!("Damage: {:.0}", weapon.damage));
        lines.push(format!(
            "Cooldown: {:.2}s  ·  Reach: {:.1}m",
            weapon.cooldown, weapon.range
        ));
        lines.push(format!("Stamina cost: {:.0}", weapon.stamina_cost));
    }
    lines
}

/// Explains why crafting is blocked, returning a tooltip string for disabled Craft buttons.
pub fn craft_blocker_reason(
    recipe: &RecipeDef,
    inventory: Option<&game::Inventory>,
    near_lit_campfire: bool,
    is_crafting: bool,
    is_dead: bool,
    item_name_fn: impl Fn(&str) -> String,
) -> Option<String> {
    if is_dead {
        return Some("Player is dead".to_string());
    }
    if is_crafting {
        return Some("Craft in progress".to_string());
    }

    let mut reasons = Vec::new();
    if recipe.station == CAMPFIRE_STATION && !near_lit_campfire {
        reasons.push("Needs a lit campfire nearby".to_string());
    }

    if let Some(inv) = inventory {
        let mut missing = Vec::new();
        for input in &recipe.inputs {
            let have = inv.count(&input.id);
            if have < input.count {
                let diff = input.count - have;
                let name = item_name_fn(&input.id);
                missing.push(format!("{name} x{diff}"));
            }
        }
        if !missing.is_empty() {
            reasons.push(format!("Missing: {}", missing.join(", ")));
        }
    } else {
        reasons.push("No inventory".to_string());
    }

    if reasons.is_empty() {
        None
    } else {
        Some(reasons.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::{Inventory, ItemCategory, RecipeItem, WeaponKind};

    #[test]
    fn test_format_item_name() {
        let mut def = ItemDef::default();
        def.name = "Cooked Meat".to_string();
        assert_eq!(format_item_name("cooked_meat", Some(&def)), "Cooked Meat");
        assert_eq!(format_item_name("wood", None), "wood");
    }

    #[test]
    fn test_format_item_category() {
        assert_eq!(format_item_category("food"), "Food");
        assert_eq!(format_item_category("weapon"), "Weapon");
        assert_eq!(format_item_category(""), "");
    }

    #[test]
    fn test_format_item_weight() {
        assert_eq!(
            format_item_weight(0.4, 2),
            "Weight: 0.40 kg each, 0.80 kg total"
        );
    }

    #[test]
    fn test_format_trunk_row() {
        assert_eq!(format_trunk_row("Wood", 5, None), "Wood   x5");
        assert_eq!(
            format_trunk_row("Wood", 5, Some("Trunk is full")),
            "Wood   x5   (Trunk is full)"
        );
    }

    #[test]
    fn test_format_bank_row() {
        assert_eq!(format_bank_row("Bandage", 2, false, "Bank full"), "Bandage   x2");
        assert_eq!(
            format_bank_row("Bandage", 2, true, "Bank full"),
            "Bandage   x2   (Bank full)"
        );
    }

    #[test]
    fn test_format_shop_row() {
        assert_eq!(format_shop_row("Pistol", 120, 1), "Pistol   $120   x1");
    }

    #[test]
    fn test_format_item_effects() {
        // Canned beans: hunger 35
        let effects1 = ItemEffects {
            hunger: 35.0,
            thirst: 0.0,
            health: 0.0,
        };
        assert_eq!(format_item_effects(&effects1), "Hunger +35");

        // Dirty water: thirst 25, health -8
        let effects2 = ItemEffects {
            hunger: 0.0,
            thirst: 25.0,
            health: -8.0,
        };
        assert_eq!(format_item_effects(&effects2), "Thirst +25, Health -8");
    }

    #[test]
    fn test_format_weapon_stats_melee() {
        let bat = WeaponDef {
            name: "Baseball Bat".to_string(),
            kind: WeaponKind::Melee,
            damage: 25.0,
            range: 2.0,
            cooldown: 0.6,
            stamina_cost: 12.0,
            ..Default::default()
        };
        let lines = format_weapon_stats(&bat, None);
        assert_eq!(lines[0], "Damage: 25");
        assert_eq!(lines[1], "Cooldown: 0.60s  ·  Reach: 2.0m");
        assert_eq!(lines[2], "Stamina cost: 12");
    }

    #[test]
    fn test_format_weapon_stats_gun() {
        let pistol = WeaponDef {
            name: "Pistol".to_string(),
            kind: WeaponKind::Gun,
            damage: 30.0,
            headshot_multiplier: 2.0,
            range: 60.0,
            fire_interval: 0.25,
            magazine_size: 12,
            ammo_item: "pistol_ammo".to_string(),
            ..Default::default()
        };
        let lines = format_weapon_stats(&pistol, Some("Pistol Ammo"));
        assert_eq!(lines[0], "Damage: 30 (headshot x2.0)");
        assert_eq!(lines[1], "Fire rate: 0.25s  ·  Range: 60m");
        assert_eq!(lines[2], "Magazine: 12 rounds, uses Pistol Ammo");
    }

    #[test]
    fn test_craft_blocker_reason() {
        let mut recipe = RecipeDef {
            id: "cooked_meat".to_string(),
            station: CAMPFIRE_STATION.to_string(),
            inputs: vec![RecipeItem::new("raw_meat", 1)],
            ..Default::default()
        };

        let mut inv = Inventory::new(10, 50.0);
        // Not near campfire, missing raw meat
        let reason = craft_blocker_reason(
            &recipe,
            Some(&inv),
            false,
            false,
            false,
            |id| if id == "raw_meat" { "Raw Meat".to_string() } else { id.to_string() },
        );
        assert_eq!(
            reason,
            Some("Needs a lit campfire nearby\nMissing: Raw Meat x1".to_string())
        );

        // Near campfire, but still missing meat
        let reason_campfire = craft_blocker_reason(
            &recipe,
            Some(&inv),
            true,
            false,
            false,
            |_| "Raw Meat".to_string(),
        );
        assert_eq!(reason_campfire, Some("Missing: Raw Meat x1".to_string()));

        // Add meat, near campfire -> ready
        inv.add("raw_meat", 1);
        let ready = craft_blocker_reason(&recipe, Some(&inv), true, false, false, |_| {
            "Raw Meat".to_string()
        });
        assert_eq!(ready, None);
    }
}
