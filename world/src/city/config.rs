use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// District types inside the city.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum District {
    Outskirts,
    Downtown,
    Core,
    Industrial,
    Park,
}

/// Building weighted selection definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingWeight {
    pub id: String,
    pub weight: u32,
}

/// District building tables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CityDistrictsData {
    pub outskirts: Vec<BuildingWeight>,
    pub downtown: Vec<BuildingWeight>,
    pub core: Vec<BuildingWeight>,
    pub industrial: Vec<BuildingWeight>,
}

impl CityDistrictsData {
    pub fn weights_for(&self, district: District) -> &[BuildingWeight] {
        match district {
            District::Outskirts => &self.outskirts,
            District::Downtown => &self.downtown,
            District::Core => &self.core,
            District::Industrial => &self.industrial,
            District::Park => &[],
        }
    }
}

/// Shape of data/city.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CityConfigData {
    pub distance_from_camp: f32,
    pub blocks: i32,
    pub block_size: f32,
    pub street_width: f32,
    pub avenue_width: f32,
    pub avenue_every: i32,
    pub blocked_chance: f32,
    pub corner_drop_chance: f32,
    pub max_slope: f32,
    pub direction_tries: i32,
    pub core_blocks: i32,
    pub outskirts_rings: i32,
    pub industrial_strip: [i32; 2],
    pub parks: [i32; 2],
    pub districts: CityDistrictsData,
    pub buildings: HashMap<String, [f32; 2]>,
    pub lot_gap: f32,
}

pub type CityConfig = CityConfigData;

impl CityConfigData {
    pub fn validate(&self) -> Result<(), String> {
        let ok = self.distance_from_camp > 0.0
            && self.blocks > 0
            && self.block_size > 0.0
            && self.street_width > 0.0
            && self.avenue_width > 0.0
            && self.avenue_every > 0
            && self.blocked_chance >= 0.0
            && self.corner_drop_chance >= 0.0
            && self.max_slope > 0.0
            && self.direction_tries > 0
            && self.core_blocks > 0
            && self.outskirts_rings >= 0
            && self.lot_gap >= 0.0;
        if !ok {
            return Err("CityConfigData has invalid values".to_string());
        }
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        data.validate()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: Self = serde_json::from_str(json)?;
        Ok(data)
    }
}
