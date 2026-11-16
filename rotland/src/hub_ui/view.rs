//! Read-only view structures providing data to hub screens.

use std::collections::HashMap;

use game::{
    Bank, BountyManager, CraftingSystem, DifficultyConfigData, DifficultySettings, Hotbar,
    Inventory, ItemDef, ItemRegistry, RecipeRegistry, SkillRegistry, SkillSystem, Wallet,
};

/// Read-only snapshot of dog state provided to dog screens.
/// Plain fields only (R7-01 writes dog logic in parallel).
#[derive(Clone, Debug)]
pub struct DogView<'a> {
    pub name: String,
    pub mode: String,
    pub hunger: f32,
    pub health: f32,
    pub max_health: f32,
    pub age: u32,
    pub has_food_in_bag: bool,
    pub pack_inventory: Option<&'a Inventory>,
    pub pack_max_weight: f32,
    pub pack_slots: usize,
}

impl<'a> Default for DogView<'a> {
    fn default() -> Self {
        Self {
            name: "Rex".to_string(),
            mode: "Follow".to_string(),
            hunger: 0.0,
            health: 80.0,
            max_health: 80.0,
            age: 1,
            has_food_in_bag: false,
            pack_inventory: None,
            pack_max_weight: 10.0,
            pack_slots: 4,
        }
    }
}

/// Read-only snapshot of game state provided to player hub screens for rendering.
pub struct HubView<'a> {
    pub inventory: Option<&'a Inventory>,
    pub hotbar: Option<&'a Hotbar>,
    pub item_registry: Option<&'a ItemRegistry>,
    pub crafting: Option<&'a CraftingSystem>,
    pub recipe_registry: Option<&'a RecipeRegistry>,
    pub skills: Option<&'a SkillSystem>,
    pub skill_registry: Option<&'a SkillRegistry>,
    pub bounty_manager: Option<&'a BountyManager>,
    pub wallet: Option<&'a Wallet>,
    pub bank: Option<&'a Bank>,
    pub difficulty_settings: Option<&'a DifficultySettings>,
    pub difficulty_config: Option<&'a DifficultyConfigData>,
    pub dog: Option<DogView<'a>>,
    pub dogs_lost: Option<u32>,
    pub days_survived: i32,
    pub play_time_seconds: f64,
    pub kills: u32,
    pub kills_by_type: Option<&'a HashMap<String, u32>>,
    pub headshots: u32,
    pub deaths: u32,
    pub distance_metres: f32,
    pub trunk_inventory: Option<&'a Inventory>,
    pub trunk_max_weight: f32,
    pub near_lit_campfire: bool,
    pub is_dead: bool,
    pub health: Option<(f32, f32)>,
    /// What the Map tab shows.
    pub map: Option<crate::map_ui::MapView<'a>>,
}

impl<'a> Default for HubView<'a> {
    fn default() -> Self {
        Self {
            inventory: None,
            hotbar: None,
            item_registry: None,
            crafting: None,
            recipe_registry: None,
            skills: None,
            skill_registry: None,
            bounty_manager: None,
            wallet: None,
            bank: None,
            difficulty_settings: None,
            difficulty_config: None,
            dog: None,
            dogs_lost: None,
            days_survived: 0,
            play_time_seconds: 0.0,
            kills: 0,
            kills_by_type: None,
            headshots: 0,
            deaths: 0,
            distance_metres: 0.0,
            trunk_inventory: None,
            trunk_max_weight: 40.0,
            near_lit_campfire: false,
            is_dead: false,
            health: None,
            map: None,
        }
    }
}

impl<'a> HubView<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_dollars(&self) -> i32 {
        self.wallet.map(|w| w.dollars()).unwrap_or(0)
    }

    pub fn bank_dollars(&self) -> i32 {
        self.bank.map(|b| b.dollars()).unwrap_or(0)
    }

    pub fn item_def<'s>(&'s self, id: &str) -> Option<&'s ItemDef> {
        self.item_registry.and_then(|r| r.get(id))
    }

    pub fn item_name<'s>(&'s self, id: &'s str) -> &'s str {
        if let Some(def) = self.item_def(id) {
            &def.name
        } else {
            id
        }
    }

    /// Formats play time as "Xh YYm", matching Godot HubProfilePage.FormatPlayTime.
    pub fn format_play_time(seconds: f64) -> String {
        let total_secs = seconds.max(0.0) as u64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        format!("{hours}h {minutes:02}m")
    }

    /// Formats distance as "X m" or "X.XX km", matching Godot HubProfilePage.FormatDistance.
    pub fn format_distance(metres: f32) -> String {
        if metres >= 1000.0 {
            format!("{:.2} km", metres / 1000.0)
        } else {
            format!("{:.0} m", metres)
        }
    }

    /// Formats kills by type as "Walker 9, Runner 3", descending by count, then name.
    /// Returns "None yet" if empty. Matches Godot HubProfilePage.FormatByType.
    pub fn format_by_type(kills: &HashMap<String, u32>) -> String {
        if kills.is_empty() {
            return "None yet".to_string();
        }
        let mut list: Vec<(&String, &u32)> = kills.iter().collect();
        list.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        let parts: Vec<String> = list
            .into_iter()
            .map(|(k, v)| format!("{} {v}", Self::capitalise(k)))
            .collect();
        parts.join(", ")
    }

    /// Formats the player's highest skill as "Combat, level 5" or "None".
    pub fn format_best_skill(skills: &SkillSystem, registry: &SkillRegistry) -> String {
        let mut best_name = None;
        let mut best_level = 0;
        for def in registry.all() {
            let level = skills.get_level(&def.id);
            if level > best_level {
                best_name = Some(&def.name);
                best_level = level;
            }
        }
        match best_name {
            Some(name) => format!("{name}, level {best_level}"),
            None => "None".to_string(),
        }
    }

    /// Formats the player's dog status as "Rex (64/80 HP), together 3 days" or "No dog".
    pub fn format_dog(dog: Option<&DogView>) -> String {
        match dog {
            Some(d) => {
                let days = d.age.max(1);
                let suffix = if days == 1 { "" } else { "s" };
                format!(
                    "{} ({:.0}/{:.0} HP), together {} day{}",
                    d.name, d.health, d.max_health, days, suffix
                )
            }
            None => "No dog".to_string(),
        }
    }

    fn capitalise(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}
