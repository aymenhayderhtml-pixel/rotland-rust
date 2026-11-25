//! Settlement NPC representations and interaction prompts (port of scripts/Settlement/SettlementNpc.cs).

use std::f32::consts::FRAC_PI_2;
use glam::Vec3;
use world::{SettlementConfigData, SettlementSite};

use crate::camp::math::CampMath;

/// Returns the default interaction prompt string for an NPC or settlement fixture.
// Godot: SettlementNpc.cs GetPrompt
pub fn prompt_for_role(role: &str) -> String {
    match role {
        "bounty_board" => "[E] Read bounty board".to_string(),
        "breeder" => "[E] Talk to dog breeder".to_string(),
        "bounty_keeper" | "bounty_board_keeper" => "[E] Talk to bounty board keeper".to_string(),
        other => format!("[E] Talk to {other}"),
    }
}

/// A settlement survivor, merchant or guard (Godot: SettlementNpc.cs).
#[derive(Debug, Clone, PartialEq)]
pub struct SettlementNpc {
    pub id: String,
    pub role: String,
    pub model_id: String,
    pub seed: String,
    pub position: Vec3,
    pub yaw: f32,
    pub is_resident: bool,
    pub talkable: bool,
    pub prompt: String,
    pub idle_phase: f32,
    pub tempo_scale: f32,
    pub animation: String,
}

impl SettlementNpc {
    pub const ROOT_LIFT: f32 = 1.1;
    pub const MIN_SURVIVOR_TEMPO: f32 = 0.85;
    pub const MAX_SURVIVOR_TEMPO: f32 = 1.15;

    pub fn new(
        role: impl Into<String>,
        model_id: impl Into<String>,
        seed: impl Into<String>,
        position: Vec3,
        yaw: f32,
        resident: bool,
    ) -> Self {
        let role = role.into();
        let model_id = model_id.into();
        let seed = seed.into();
        let talkable = matches!(role.as_str(), "trader" | "banker" | "bounty_board" | "bounty_keeper" | "bounty_board_keeper" | "breeder");
        let prompt = prompt_for_role(&role);

        let mut npc = Self {
            id: seed.clone(),
            role,
            model_id,
            seed,
            position,
            yaw,
            is_resident: resident,
            talkable,
            prompt,
            idle_phase: 0.0,
            tempo_scale: 1.0,
            animation: "idle".to_string(),
        };
        npc.apply_idle_variety();
        npc
    }

    pub fn can_interact(&self) -> bool {
        self.talkable
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Chest interaction point (Godot: RootLift).
    #[allow(dead_code)]
    fn chest_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, Self::ROOT_LIFT, 0.0)
    }

    // Godot: SettlementNpc.cs ApplyIdleVariety
    fn apply_idle_variety(&mut self) {
        let h1 = CampMath::hash(&self.seed, 1);
        self.idle_phase = CampMath::unit(h1) * 2.0;

        if self.role == "survivor" {
            let h2 = CampMath::hash(&self.seed, 2);
            self.tempo_scale = Self::MIN_SURVIVOR_TEMPO
                + (Self::MAX_SURVIVOR_TEMPO - Self::MIN_SURVIVOR_TEMPO) * CampMath::unit(h2);
        }
    }

    /// Default settlement marker layout from settlement.glb.
    pub fn default_settlement_npcs(site: &SettlementSite, cfg: &SettlementConfigData) -> Vec<Self> {
        let mut npcs = Vec::new();

        let resolve_model = |kind: &str| -> String {
            cfg.npc_models.get(kind).cloned().unwrap_or_else(|| kind.to_string())
        };

        // npc_trader: faces the lane
        let trader_local = Vec3::new(-8.2, 0.0, -7.7);
        let trader_pos = site.center + site.local(trader_local.x, trader_local.y, trader_local.z);
        npcs.push(Self::new(
            "trader",
            resolve_model("npc_trader"),
            "npc_trader",
            trader_pos,
            site.yaw - FRAC_PI_2,
            false,
        ));

        // npc_banker: faces the counter
        let banker_local = Vec3::new(7.2, 0.0, -7.5);
        let banker_pos = site.center + site.local(banker_local.x, banker_local.y, banker_local.z);
        npcs.push(Self::new(
            "banker",
            resolve_model("npc_banker"),
            "npc_banker",
            banker_pos,
            site.yaw + FRAC_PI_2,
            false,
        ));

        // npc_guard_1 & npc_guard_2 on platforms
        let g1_local = Vec3::new(-7.3, 2.7, -17.9);
        let g1_pos = site.center + site.local(g1_local.x, g1_local.y, g1_local.z);
        npcs.push(Self::new(
            "guard",
            resolve_model("npc_guard"),
            "npc_guard_1",
            g1_pos,
            site.yaw,
            false,
        ));

        let g2_local = Vec3::new(7.3, 2.7, -17.9);
        let g2_pos = site.center + site.local(g2_local.x, g2_local.y, g2_local.z);
        npcs.push(Self::new(
            "guard",
            resolve_model("npc_guard"),
            "npc_guard_2",
            g2_pos,
            site.yaw,
            false,
        ));

        // npc_survivors round the camp
        let s1_local = Vec3::new(-9.8, 0.0, 7.6);
        let s1_pos = site.center + site.local(s1_local.x, s1_local.y, s1_local.z);
        let s1_yaw = site.yaw + CampMath::yaw_toward(s1_local, Vec3::ZERO);
        npcs.push(Self::new(
            "survivor",
            resolve_model("npc_survivor"),
            "npc_survivor_1",
            s1_pos,
            s1_yaw,
            false,
        ));

        let s2_local = Vec3::new(-2.9, 0.0, 2.2);
        let s2_pos = site.center + site.local(s2_local.x, s2_local.y, s2_local.z);
        let s2_yaw = site.yaw + CampMath::yaw_toward(s2_local, Vec3::ZERO);
        npcs.push(Self::new(
            "survivor",
            resolve_model("npc_survivor"),
            "npc_survivor_2",
            s2_pos,
            s2_yaw,
            false,
        ));

        let s3_local = Vec3::new(9.4, 0.0, 7.2);
        let s3_pos = site.center + site.local(s3_local.x, s3_local.y, s3_local.z);
        let s3_yaw = site.yaw + CampMath::yaw_toward(s3_local, Vec3::ZERO);
        npcs.push(Self::new(
            "survivor",
            resolve_model("npc_survivor"),
            "npc_survivor_3",
            s3_pos,
            s3_yaw,
            false,
        ));

        // Bounty board
        let bb_local = Vec3::new(-3.6, 0.0, -9.0);
        let bb_pos = site.center + site.local(bb_local.x, bb_local.y, bb_local.z);
        npcs.push(Self::new(
            "bounty_board",
            "bounty_board",
            "bounty_board",
            bb_pos,
            site.yaw,
            false,
        ));

        npcs
    }
}
