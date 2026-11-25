//! Icon texture cache for egui in Rotland.
//!
//! Loads item icons from `assets/icons/<item_id>.png` once and caches them.
//! Reusable across the HUD hotbar, the player Bag screen (hub_ui), and the Map screen.
//!
//! Port of Godot C# `scripts/UI/ItemIcons.cs`.

use std::collections::HashMap;
use engine::egui::{ColorImage, Context, TextureHandle, TextureId, TextureOptions};

/// Reusable cache for item icon textures in egui.
///
/// Textures are loaded lazily on first access and retained so they are not re-read from disk
/// each frame. If an icon file does not exist or fails to decode, `None` is cached so the disk
/// is not queried repeatedly.
#[derive(Default)]
pub struct IconCache {
    textures: HashMap<String, Option<TextureHandle>>,
}

impl IconCache {
    /// Creates an empty icon cache.
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// Returns the egui `TextureId` for the given item id, loading and caching it on first request.
    /// Returns `None` if the PNG file does not exist or fails to decode.
    pub fn icon(&mut self, ctx: &Context, item_id: &str) -> Option<TextureId> {
        if let Some(cached) = self.textures.get(item_id) {
            return cached.as_ref().map(|h| h.id());
        }

        let path = paths::repo_path(&format!("assets/icons/{item_id}.png"));
        let handle = if path.is_file() {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let rgba = img.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let color_image = ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());
                    Some(ctx.load_texture(
                        format!("icon_{item_id}"),
                        color_image,
                        TextureOptions::LINEAR,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let id = handle.as_ref().map(|h| h.id());
        self.textures.insert(item_id.to_string(), handle);
        id
    }

    /// Retrieves an icon for an explicit item id or falls back to resolving by item label/name.
    pub fn icon_or_label(
        &mut self,
        ctx: &Context,
        item_id: Option<&str>,
        label: &str,
    ) -> Option<TextureId> {
        if let Some(id) = item_id
            && let Some(tid) = self.icon(ctx, id)
        {
            return Some(tid);
        }

        let fallback = label_to_item_id(label);
        if !fallback.is_empty() {
            return self.icon(ctx, &fallback);
        }
        None
    }

    /// Returns true if the given item ID has already been queried (either loaded or confirmed missing).
    #[allow(dead_code)]
    fn is_cached(&self, item_id: &str) -> bool {
        self.textures.contains_key(item_id)
    }

    /// Clears all cached texture handles.
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

/// Maps short item names/labels (e.g. from `items.json` or `HotbarSlot::label`)
/// to their corresponding item IDs in `assets/icons/<id>.png`.
pub fn label_to_item_id(label: &str) -> String {
    let lower = label.trim().to_lowercase();
    match lower.as_str() {
        "bat" | "baseball bat" => "bat".to_string(),
        "beans" | "canned beans" => "canned_beans".to_string(),
        "cooked" | "cooked meat" => "cooked_meat".to_string(),
        "water" | "water bottle" => "water_bottle".to_string(),
        "dirty" | "dirty water" => "dirty_water".to_string(),
        "9mm" | "pistol ammo" => "pistol_ammo".to_string(),
        "shells" | "12 gauge shells" => "shells_12g".to_string(),
        "rounds" | "rifle rounds" => "rifle_rounds".to_string(),
        "7.62" | "7.62 ammo" => "ammo_762".to_string(),
        "light" | "flashlight" => "flashlight".to_string(),
        "scrap" | "scrap metal" => "scrap_metal".to_string(),
        "campfire" | "campfire kit" => "campfire_kit".to_string(),
        "fuel" | "fuel can" => "fuel_can".to_string(),
        "p.seeds" | "potato seeds" => "seeds_potato".to_string(),
        "c.seeds" | "carrot seeds" => "seeds_carrot".to_string(),
        "seeds" | "wheat seeds" => "seeds_wheat".to_string(),
        "can" | "watering can" => "watering_can".to_string(),
        "baked" | "baked potato" => "baked_potato".to_string(),
        "stew" | "vegetable stew" => "vegetable_stew".to_string(),
        other => other.replace(' ', "_"),
    }
}
