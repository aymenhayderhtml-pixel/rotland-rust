"""Player melee shove and block clips (A13b).

Keyframes, bone entries and the rules are described in poses.py.
"""

_MELEE_1H_REST = {
    "spine": (0, 0, 0), "chest": (0, 0, 0), "neck": (0, 0, 0), "head": (0, 0, 0),
    "upper_arm_R": (10, 0, 0), "forearm_R": (40, 0, 0), "hand_R": (0, 0, 0),
    "upper_arm_L": (10, 0, 0), "forearm_L": (30, 0, 0),
}

_BLOCK_1H = {
    "spine": (-2, 0, 0), "chest": (-6, -8, 0), "neck": (-4, 4, 0), "head": (-8, 6, 0),
    "upper_arm_R": (70, 0, 28), "forearm_R": (110, 0, 0), "hand_R": (0, 0, 70),
    "upper_arm_L": (50, 0, -24), "forearm_L": (120, 0, 0),
}

_BLOCK_2H = {
    "spine": (-2, 0, 0), "chest": (-6, 0, 0), "neck": (-4, 0, 0), "head": (-6, 0, 0),
    "upper_arm_R": (75, 0, 26), "forearm_R": (100, 0, 0), "hand_R": (0, 0, 20),
    "upper_arm_L": (75, 0, -26), "forearm_L": (100, 0, 0),
}

CLIPS = {
    # Both arms pull in, then shove straight forward at chest height and recover to melee_1h rest.
    "shove": {
        "length": 0.45,
        "loop": False,
        "frames": {
            0.00: {"spine": (2, 0, 0), "chest": (4, 0, 0), "neck": (0, 0, 0), "head": (0, 0, 0), "upper_arm_R": (25, 0, 18), "forearm_R": (100, 0, 0), "hand_R": (0, 0, 0), "upper_arm_L": (25, 0, -18), "forearm_L": (100, 0, 0)},
            0.15: {"spine": (-6, 0, 0), "chest": (-10, 0, 0), "neck": (2, 0, 0), "head": (4, 0, 0), "upper_arm_R": (80, 0, 8), "forearm_R": (15, 0, 0), "hand_R": (-15, 0, 0), "upper_arm_L": (80, 0, -8), "forearm_L": (15, 0, 0)},
            0.25: {"spine": (-4, 0, 0), "chest": (-6, 0, 0), "neck": (0, 0, 0), "head": (2, 0, 0), "upper_arm_R": (60, 0, 8), "forearm_R": (25, 0, 0), "hand_R": (-8, 0, 0), "upper_arm_L": (60, 0, -8), "forearm_L": (25, 0, 0)},
            0.45: dict(_MELEE_1H_REST),
        },
    },
    # Held 1H block: right forearm and weapon raised across the face, left arm up as a guard.
    "block_1h": {
        "length": 0.5,
        "loop": True,
        "frames": {
            0.00: dict(_BLOCK_1H),
            0.25: {**_BLOCK_1H, "chest": (-7, -8, 0), "upper_arm_R": (72, 0, 28), "upper_arm_L": (52, 0, -24)},
            0.50: dict(_BLOCK_1H),
        },
    },
    # Held 2H block: both hands hold the weapon horizontally in front of the face.
    "block_2h": {
        "length": 0.5,
        "loop": True,
        "frames": {
            0.00: dict(_BLOCK_2H),
            0.25: {**_BLOCK_2H, "chest": (-7, 0, 0), "upper_arm_R": (77, 0, 26), "upper_arm_L": (77, 0, -26)},
            0.50: dict(_BLOCK_2H),
        },
    },
    # Blocked hit jolt: from block_2h, arms and chest kick back then recover.
    "block_hit": {
        "length": 0.3,
        "loop": False,
        "frames": {
            0.00: dict(_BLOCK_2H),
            0.08: {**_BLOCK_2H, "spine": (4, 0, 0), "chest": (8, 0, 0), "head": (4, 0, 0), "upper_arm_R": (60, 0, 26), "upper_arm_L": (60, 0, -26)},
            0.30: dict(_BLOCK_2H),
        },
    },
}
