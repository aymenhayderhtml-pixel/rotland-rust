use serde::{Deserialize, Serialize};

use crate::weapons::{WeaponDef, WeaponRegistry};

/// Categories of items in Rotland (matches scripts/Items/ItemRegistry.cs Categories).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemCategory {
    Food,
    Drink,
    Medical,
    Weapon,
    Ammo,
    #[default]
    Material,
    Tool,
    Placeable,
    Fuel,
    Attachment,
}

impl ItemCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Food => "food",
            Self::Drink => "drink",
            Self::Medical => "medical",
            Self::Weapon => "weapon",
            Self::Ammo => "ammo",
            Self::Material => "material",
            Self::Tool => "tool",
            Self::Placeable => "placeable",
            Self::Fuel => "fuel",
            Self::Attachment => "attachment",
        }
    }
}

impl std::fmt::Display for ItemCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq<str> for ItemCategory {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for ItemCategory {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<ItemCategory> for str {
    fn eq(&self, other: &ItemCategory) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<ItemCategory> for &str {
    fn eq(&self, other: &ItemCategory) -> bool {
        *self == other.as_str()
    }
}

impl AsRef<str> for ItemCategory {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Change to each survival stat when an item is used (port of scripts/Items/ItemEffects.cs).
/// Negative values hurt (e.g. raw meat, dirty water).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemEffects {
    pub hunger: f32,
    pub thirst: f32,
    pub health: f32,
}

/// One item type definition loaded from data/items.json (port of scripts/Items/ItemDef.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemDef {
    /// The key in items.json. Filled in by ItemRegistry on load.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    /// Optional short label for the hotbar (8 letters or fewer). Empty means use name.
    pub short_name: String,
    /// One of food, drink, medical, weapon, ammo, material, tool, placeable, fuel.
    pub category: ItemCategory,
    /// Weight of one item, in kg.
    pub weight: f32,
    /// How many of this item fit in one inventory slot.
    pub max_stack: u32,
    /// Base price in dollars for economy later.
    pub value: u32,
    /// Placeholder icon colour as a hex string such as "#8a5a2b".
    pub color: String,
    pub description: String,
    /// What eating or drinking does.
    pub effects: ItemEffects,
    /// For weapon items: the id in data/weapons.json.
    pub weapon_id: String,
    /// For ammo items: the ammo type, e.g. "pistol".
    pub ammo_type: String,
    /// A tool held in hand like a weapon
    pub holdable: bool,
}

impl Default for ItemDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            short_name: String::new(),
            category: ItemCategory::Material,
            weight: 0.0,
            max_stack: 1,
            value: 0,
            color: "#ffffff".to_string(),
            description: String::new(),
            effects: ItemEffects::default(),
            weapon_id: String::new(),
            ammo_type: String::new(),
            holdable: false,
        }
    }
}

impl ItemDef {
    /// The label the hotbar shows: short_name when set, otherwise name.
    pub fn hotbar_label(&self) -> &str {
        if !self.short_name.is_empty() {
            &self.short_name
        } else {
            &self.name
        }
    }

    pub fn is_weapon(&self) -> bool {
        self.category == ItemCategory::Weapon
    }

    /// Links an item to its weapon definition in WeaponRegistry by weapon_id.
    pub fn weapon<'a>(&self, weapons: &'a WeaponRegistry) -> Option<&'a WeaponDef> {
        if self.is_weapon() && !self.weapon_id.is_empty() {
            weapons.get(&self.weapon_id)
        } else {
            None
        }
    }
}
