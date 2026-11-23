//! Per-joint bone weight masks for layered humanoid animation blending.

use crate::blend::UPPER_BONES;

const LOWER_BONES: &[&str] = &[
    "root",
    "hips",
    "pelvis",
    "thigh_L",
    "shin_L",
    "foot_L",
    "toe_L",
    "upper_leg_L",
    "lower_leg_L",
    "thigh_R",
    "shin_R",
    "foot_R",
    "toe_R",
    "upper_leg_R",
    "lower_leg_R",
];

const SPINE_BONES: &[&str] = &["spine", "chest"];

const LEFT_ARM_BONES: &[&str] = &["shoulder_L", "upper_arm_L", "forearm_L", "hand_L"];

const RIGHT_ARM_BONES: &[&str] = &["shoulder_R", "upper_arm_R", "forearm_R", "hand_R"];

const HEAD_BONES: &[&str] = &["neck", "head"];

const WEIGHT_OFF: f32 = 0.0; // new: DECISIONS 2026-11-23
const WEIGHT_ON: f32 = 1.0; // new: DECISIONS 2026-11-23

/// Predefined anatomical regions for masked pose blending.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaskId {
    Full,
    Upper,
    Lower,
    Spine,
    Arms,
    LeftArm,
    RightArm,
    Head,
}

impl MaskId {
    const COUNT: usize = 8; // new: DECISIONS 2026-11-23

    const ALL: [MaskId; Self::COUNT] = [
        MaskId::Full,
        MaskId::Upper,
        MaskId::Lower,
        MaskId::Spine,
        MaskId::Arms,
        MaskId::LeftArm,
        MaskId::RightArm,
        MaskId::Head,
    ];

    const fn index(self) -> usize {
        match self {
            MaskId::Full => 0,
            MaskId::Upper => 1,
            MaskId::Lower => 2,
            MaskId::Spine => 3,
            MaskId::Arms => 4,
            MaskId::LeftArm => 5,
            MaskId::RightArm => 6,
            MaskId::Head => 7,
        }
    }
}

/// Per-skeleton table of 0.0..=1.0 bone weights for every `MaskId`.
#[derive(Clone, Debug)]
pub struct BoneMasks {
    weights: [Vec<f32>; MaskId::COUNT],
}

impl BoneMasks {
    /// Builds weight arrays matching the joint order of `joint_names`.
    /// Unknown bones receive 0.0 in every mask except `MaskId::Full`.
    pub fn new(joint_names: &[&str]) -> Self {
        let weights = MaskId::ALL.map(|id| {
            joint_names
                .iter()
                .map(|&name| default_weight(id, name))
                .collect()
        });
        Self { weights }
    }

    /// Returns the per-joint weight slice (0.0..=1.0) for `id`.
    pub fn weights(&self, id: MaskId) -> &[f32] {
        &self.weights[id.index()]
    }
}

fn contains(list: &[&str], name: &str) -> bool {
    list.contains(&name)
}

fn default_weight(id: MaskId, name: &str) -> f32 {
    let active = match id {
        MaskId::Full => true,
        MaskId::Upper => contains(UPPER_BONES, name),
        MaskId::Lower => contains(LOWER_BONES, name),
        MaskId::Spine => contains(SPINE_BONES, name),
        MaskId::Arms => contains(LEFT_ARM_BONES, name) || contains(RIGHT_ARM_BONES, name),
        MaskId::LeftArm => contains(LEFT_ARM_BONES, name),
        MaskId::RightArm => contains(RIGHT_ARM_BONES, name),
        MaskId::Head => contains(HEAD_BONES, name),
    };
    if active { WEIGHT_ON } else { WEIGHT_OFF }
}
