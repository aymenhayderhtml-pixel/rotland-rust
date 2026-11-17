# R5-02: Survival needs, skills and difficulty — Rust port

Read `tasks/R5-common.md` first.

Port to `rust/game/src/survival/`, `rust/game/src/skills/`, `rust/game/src/difficulty/`:
- `scripts/Survival/*.cs` (SurvivalConfig/Data from `data/survival.json`, NeedDef, NeedStat for hunger and thirst, WellFedDef/WellFedRegen, StaminaDef): needs drain over game time, damage when empty, eating/drinking items restore them (item effects come from `game::items::ItemDef` effects), well-fed health regen. Note `game::player` already has a `Stamina`: do not duplicate it; only port StaminaDef data if the existing one lacks fields, and say so.
- `scripts/Skills/*.cs` (SkillRegistry from `data/skills.json`, SkillDef, SkillSystem, XpRewards): XP, levels, perks/multipliers the skills give.
- `scripts/Difficulty/*.cs` (DifficultyConfig from `data/difficulty.json`, DifficultySettings, DeathRule, RunDifficulty, WorldDifficulty): presets, multipliers (zombies, loot, damage), death rule (lose all / keep half / keep all) applied to an Inventory. Note `rust/game/src/loot/difficulty.rs` already loads part of difficulty.json: reuse or move it, don't make a second copy.
- Godot save ids: "need_hunger", "need_thirst", "skills", "difficulty" (check each `.cs`).
Tests: drain after N game hours on real data, starving damage, eating restores, well-fed regen, XP to level thresholds on real data, death rule results on a sample inventory, save round trips.
