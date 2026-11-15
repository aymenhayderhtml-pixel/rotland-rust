use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// Melee weapons swing; guns fire. Unknown kinds fail the load so a typo in data is caught early.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeaponKind {
    #[default]
    Melee,
    Gun,
}

/// Clip names the player plays with a weapon (the "anim" block of weapons.json).
/// A missing clip means no animation for that action. `impact` is the clip time (s) of the hit moment.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WeaponAnim {
    pub attack: Option<String>,
    pub impact: f32,
    pub aim: Option<String>,
    pub fire: Option<String>,
    pub reload: Option<String>,
    pub bash: Option<String>,
}

/// Offset of the model from the default grip. Pos in metres, rot in degrees (x, y, z).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WeaponHold {
    pub pos: [f32; 3],
    pub rot: [f32; 3],
}

/// Zoom while aiming: `fov` in degrees, mouse look scaled by `sensitivity`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WeaponScope {
    pub fov: f32,
    pub sensitivity: f32,
}

impl Default for WeaponScope {
    fn default() -> Self {
        Self { fov: 25.0, sensitivity: 1.0 }
    }
}

/// Stats for one weapon, one entry of data/weapons.json (port of scripts/Combat/WeaponDef.cs).
/// Defaults match the C# class, so fields missing from the JSON get the same values.
/// Melee fields stay at zero for guns and the other way round. Unknown JSON keys are ignored, as in C#.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WeaponDef {
    /// The key in weapons.json. The registry fills it in.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    pub kind: WeaponKind,

    pub damage: f32,
    pub range: f32,
    pub arc_deg: f32,
    pub windup: f32,
    pub cooldown: f32,
    pub stamina_cost: f32,
    pub knockback: f32,
    pub stagger: f32,
    pub max_targets: u32,
    pub noise_radius: f32,

    // Gun fields
    pub headshot_multiplier: f32,
    pub fire_interval: f32,
    pub magazine_size: u32,
    /// Item id of the ammo this gun uses, e.g. "pistol_ammo".
    pub ammo_item: String,
    pub reload_time: f32,
    /// Half-angle of the random cone when hip firing, in degrees.
    pub spread_hip_deg: f32,
    /// Half-angle of the random cone when aiming, in degrees.
    pub spread_aim_deg: f32,
    /// Rays per shot. A shotgun fires several pellets.
    pub pellets: u32,
    pub scope: Option<WeaponScope>,
    /// Automatic: holding fire keeps shooting at the fire interval.
    pub auto: bool,
    /// Extra view kick per shot, in degrees, on top of the base recoil.
    pub recoil_climb: f32,
    /// Damage of the empty gun used as a club.
    pub bash_damage: f32,
    /// Seconds into the bash clip when the bash hits.
    pub bash_impact: f32,
    /// Reload seconds [from, to) during which the magazine is hidden. None if none.
    pub mag_swap: Option<[f32; 2]>,

    pub anim: WeaponAnim,
    /// Path of the weapon's .glb model.
    pub model: String,
    pub hold: WeaponHold,
    /// Third-person hold used while aiming (two-handed guns). None keeps `hold`.
    pub aim_hold: Option<WeaponHold>,
    /// Offset of the first-person view model. None leaves it at the view mount.
    pub fp_hold: Option<WeaponHold>,
    /// Free labels for later systems, e.g. "chop" for the axe.
    pub tags: Vec<String>,
    /// Placeholder colour as hex, e.g. "#8a5a2b".
    pub color: String,
    /// Placeholder box size in metres: width, height, length.
    pub size: [f32; 3],
}

impl Default for WeaponDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: WeaponKind::Melee,
            damage: 0.0,
            range: 0.0,
            arc_deg: 0.0,
            windup: 0.0,
            cooldown: 0.0,
            stamina_cost: 0.0,
            knockback: 0.0,
            stagger: 0.0,
            max_targets: 1,
            noise_radius: 0.0,
            headshot_multiplier: 1.0,
            fire_interval: 0.0,
            magazine_size: 0,
            ammo_item: String::new(),
            reload_time: 0.0,
            spread_hip_deg: 0.0,
            spread_aim_deg: 0.0,
            pellets: 1,
            scope: None,
            auto: false,
            recoil_climb: 0.0,
            bash_damage: 0.0,
            bash_impact: 0.3,
            mag_swap: None,
            anim: WeaponAnim::default(),
            model: String::new(),
            hold: WeaponHold::default(),
            aim_hold: None,
            fp_hold: None,
            tags: Vec::new(),
            color: "#ffffff".to_string(),
            size: [0.1, 0.1, 0.5],
        }
    }
}

impl WeaponDef {
    pub fn is_gun(&self) -> bool {
        self.kind == WeaponKind::Gun
    }
}

/// Every weapon from weapons.json, keyed by id (port of scripts/Combat/WeaponRegistry.cs).
/// A BTreeMap keeps `iter` in the same order every run.
#[derive(Clone, Debug, Default)]
pub struct WeaponRegistry {
    defs: BTreeMap<String, WeaponDef>,
}

#[derive(Debug)]
pub enum WeaponLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl std::fmt::Display for WeaponLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponLoadError::Io(e) => write!(f, "could not read weapons file: {e}"),
            WeaponLoadError::Parse(e) => write!(f, "could not parse weapons file: {e}"),
        }
    }
}

impl std::error::Error for WeaponLoadError {}

impl WeaponRegistry {
    /// Reads a weapons.json file. Callers pass the path (see `paths::repo_path`), so tests and the
    /// game can point at different data.
    pub fn load(path: &Path) -> Result<Self, WeaponLoadError> {
        let text = std::fs::read_to_string(path).map_err(WeaponLoadError::Io)?;
        Self::from_json(&text).map_err(WeaponLoadError::Parse)
    }

    /// Parses weapons.json text. Each weapon's `id` is set from its key.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let mut defs: BTreeMap<String, WeaponDef> = serde_json::from_str(text)?;
        for (id, def) in defs.iter_mut() {
            def.id = id.clone();
        }
        Ok(Self { defs })
    }

    /// The weapon with this id, or None if there is none.
    pub fn get(&self, id: &str) -> Option<&WeaponDef> {
        self.defs.get(id)
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Every weapon, in id order.
    pub fn iter(&self) -> impl Iterator<Item = &WeaponDef> {
        self.defs.values()
    }
}
