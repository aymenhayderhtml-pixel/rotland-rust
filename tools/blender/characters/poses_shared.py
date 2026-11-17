"""Shared pose data: arms-down offset and the aim poses used by several clips.

Arm Z in the aim poses is relative to the arms-down offset (ARM_DOWN_OFFSET).
"""

# Degrees added to the upper-arm Z rotation in every clip: turns the A-pose arms
# (30 degrees below horizontal) to hang at the sides. Sign is mirrored for L.
ARM_DOWN_OFFSET = {"upper_arm_R": 58.0, "upper_arm_L": -58.0}

# Shared aim poses (phase 2). Arm Z is relative to the arms-down offset, see ARM_DOWN_OFFSET.
PISTOL_AIM = {
    "spine": (0, -4, 0),
    "chest": (-2, -4, 0),
    "neck": (0, 4, 0),
    "head": (-2, 4, 0),
    "upper_arm_R": (86, 0, 0),
    "forearm_R": (0, 0, 0),
    "upper_arm_L": (84, 0, -40),
    "forearm_L": (30, 0, 0),
}
RIFLE_AIM = {
    "spine": (-3, -6, 0),
    "chest": (-4, -6, 0),
    "neck": (-2, 4, 0),
    "head": (0, 4, 0),
    "upper_arm_R": (84, 0, -6),
    "forearm_R": (30, 0, 0),
    "upper_arm_L": (88, 0, -40),
    "forearm_L": (16, 0, 0),
}
