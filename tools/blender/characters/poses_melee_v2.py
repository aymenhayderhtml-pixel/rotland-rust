"""Additional melee combo clips (A13a): light stabs, 1H/2H combo steps, heavy overhead, spear thrust.

Upper body only. Keyframe format and bone sign conventions match poses_combat.py.
"""

_REST_1H = {
    "spine": (0, 0, 0), "chest": (0, 0, 0), "neck": (0, 0, 0), "head": (0, 0, 0),
    "upper_arm_R": (10, 0, 0), "forearm_R": (40, 0, 0), "hand_R": (0, 0, 0),
    "upper_arm_L": (10, 0, 0), "forearm_L": (30, 0, 0),
}

_REST_2H = {
    "spine": (0, 0, 0), "chest": (0, 0, 0), "neck": (0, 0, 0), "head": (0, 0, 0),
    "upper_arm_R": (30, 0, 20), "forearm_R": (90, 0, 0), "hand_R": (0, 0, 0),
    "upper_arm_L": (30, 0, -20), "forearm_L": (90, 0, 0),
}

CLIPS = {
    # Short right-hand jab straight forward at chest height, chest twists slightly left.
    "stab_a": {
        "length": 0.35,
        "loop": False,
        "frames": {
            0.00: dict(_REST_1H),
            0.06: {"spine": (2, -8, 0), "chest": (2, -12, 0), "neck": (0, 4, 0), "head": (0, 6, 0), "upper_arm_R": (25, 0, -8), "forearm_R": (95, 0, 0), "hand_R": (-10, 0, 0), "upper_arm_L": (15, 0, 0), "forearm_L": (45, 0, 0)},
            0.12: {"spine": (-4, 12, 0), "chest": (-6, 18, 0), "neck": (0, -6, 0), "head": (0, -8, 0), "upper_arm_R": (88, 0, 6), "forearm_R": (12, 0, 0), "hand_R": (8, 0, 0), "upper_arm_L": (-8, 0, 0), "forearm_L": (35, 0, 0)},
            0.22: {"spine": (-2, 8, 0), "chest": (-3, 10, 0), "neck": (0, -3, 0), "head": (0, -4, 0), "upper_arm_R": (50, 0, 4), "forearm_R": (35, 0, 0), "hand_R": (4, 0, 0), "upper_arm_L": (4, 0, 0), "forearm_L": (30, 0, 0)},
            0.35: dict(_REST_1H),
        },
    },
    # Second jab, slightly lower, chest twists right.
    "stab_b": {
        "length": 0.35,
        "loop": False,
        "frames": {
            0.00: dict(_REST_1H),
            0.06: {"spine": (0, 10, 0), "chest": (0, 14, 0), "neck": (0, -4, 0), "head": (0, -6, 0), "upper_arm_R": (20, 0, 14), "forearm_R": (100, 0, 0), "hand_R": (-8, 0, 0), "upper_arm_L": (12, 0, -6), "forearm_L": (40, 0, 0)},
            0.12: {"spine": (-5, -12, 0), "chest": (-7, -18, 0), "neck": (0, 6, 0), "head": (0, 8, 0), "upper_arm_R": (72, 0, -6), "forearm_R": (14, 0, 0), "hand_R": (10, 0, 0), "upper_arm_L": (20, 0, -10), "forearm_L": (50, 0, 0)},
            0.22: {"spine": (-2, -6, 0), "chest": (-3, -9, 0), "neck": (0, 3, 0), "head": (0, 4, 0), "upper_arm_R": (42, 0, -2), "forearm_R": (35, 0, 0), "hand_R": (4, 0, 0), "upper_arm_L": (14, 0, -4), "forearm_L": (35, 0, 0)},
            0.35: dict(_REST_1H),
        },
    },
    # One-hand backhand: wind up across the body to the left (yaw +), cut out to the right (yaw -).
    "melee_1h_b": {
        "length": 0.5,
        "loop": False,
        "frames": {
            0.00: dict(_REST_1H),
            0.12: {"spine": (4, 24, 0), "chest": (4, 30, 0), "neck": (0, -8, 0), "head": (0, -10, 0), "upper_arm_R": (75, 0, 52), "forearm_R": (85, 0, 0), "hand_R": (-16, 0, 0), "upper_arm_L": (-8, 0, 0), "forearm_L": (40, 0, 0)},
            0.25: {"spine": (-6, -26, 0), "chest": (-8, -32, 0), "neck": (0, 10, 0), "head": (-4, 12, 0), "upper_arm_R": (65, 0, -38), "forearm_R": (20, 0, 0), "hand_R": (12, 0, 0), "upper_arm_L": (24, 0, -10), "forearm_L": (50, 0, 0)},
            0.36: {"spine": (-6, -32, 0), "chest": (-8, -36, 0), "neck": (0, 12, 0), "head": (-4, 12, 0), "upper_arm_R": (35, 0, -42), "forearm_R": (22, 0, 0), "hand_R": (10, 0, 0), "upper_arm_L": (18, 0, -8), "forearm_L": (42, 0, 0)},
            0.50: dict(_REST_1H),
        },
    },
    # One-hand overhead chop: right arm high (155 deg), chop down, spine bends forward (-12) at impact.
    "melee_1h_c": {
        "length": 0.6,
        "loop": False,
        "frames": {
            0.00: dict(_REST_1H),
            0.20: {"spine": (8, -10, 0), "chest": (10, -14, 0), "neck": (-4, 4, 0), "head": (6, 6, 0), "upper_arm_R": (155, 0, -14), "forearm_R": (55, 0, 0), "hand_R": (-20, 0, 0), "upper_arm_L": (25, 0, -10), "forearm_L": (55, 0, 0)},
            0.35: {"spine": (-12, 10, 0), "chest": (-14, 12, 0), "neck": (-4, -4, 0), "head": (-8, -4, 0), "upper_arm_R": (42, 0, 18), "forearm_R": (20, 0, 0), "hand_R": (14, 0, 0), "upper_arm_L": (-14, 0, 0), "forearm_L": (35, 0, 0)},
            0.46: {"spine": (-10, 12, 0), "chest": (-12, 14, 0), "neck": (-2, -4, 0), "head": (-6, -4, 0), "upper_arm_R": (20, 0, 14), "forearm_R": (25, 0, 0), "hand_R": (10, 0, 0), "upper_arm_L": (-8, 0, 0), "forearm_L": (32, 0, 0)},
            0.60: dict(_REST_1H),
        },
    },
    # Two-hand return swing: wind up to the left (yaw +24/+30), strike to the right (yaw -28/-34).
    "melee_2h_b": {
        "length": 0.7,
        "loop": False,
        "frames": {
            0.00: dict(_REST_2H),
            0.18: {"spine": (6, 24, 0), "chest": (6, 30, 0), "neck": (0, -8, 0), "head": (0, -12, 0), "upper_arm_R": (85, 0, 48), "forearm_R": (85, 0, 0), "hand_R": (-12, 0, 0), "upper_arm_L": (75, 0, 26), "forearm_L": (75, 0, 0)},
            0.35: {"spine": (-8, -28, 0), "chest": (-10, -34, 0), "neck": (0, 10, 0), "head": (-6, 14, 0), "upper_arm_R": (70, 0, -28), "forearm_R": (25, 0, 0), "hand_R": (10, 0, 0), "upper_arm_L": (75, 0, -42), "forearm_L": (30, 0, 0)},
            0.50: {"spine": (-8, -32, 0), "chest": (-10, -36, 0), "neck": (0, 12, 0), "head": (-4, 14, 0), "upper_arm_R": (48, 0, -30), "forearm_R": (35, 0, 0), "hand_R": (8, 0, 0), "upper_arm_L": (52, 0, -40), "forearm_L": (42, 0, 0)},
            0.70: dict(_REST_2H),
        },
    },
    # Two-hand overhead slam: both arms up (~150), lean back, then slam down hard (spine -20 at impact).
    "melee_2h_c": {
        "length": 0.8,
        "loop": False,
        "frames": {
            0.00: dict(_REST_2H),
            0.28: {"spine": (10, 0, 0), "chest": (12, 0, 0), "neck": (-4, 0, 0), "head": (8, 0, 0), "upper_arm_R": (150, 0, 14), "forearm_R": (40, 0, 0), "hand_R": (-16, 0, 0), "upper_arm_L": (150, 0, -14), "forearm_L": (40, 0, 0)},
            0.45: {"spine": (-20, 0, 0), "chest": (-22, 0, 0), "neck": (-6, 0, 0), "head": (-10, 0, 0), "upper_arm_R": (38, 0, 16), "forearm_R": (22, 0, 0), "hand_R": (14, 0, 0), "upper_arm_L": (38, 0, -16), "forearm_L": (22, 0, 0)},
            0.60: {"spine": (-14, 0, 0), "chest": (-16, 0, 0), "neck": (-4, 0, 0), "head": (-6, 0, 0), "upper_arm_R": (45, 0, 20), "forearm_R": (45, 0, 0), "hand_R": (8, 0, 0), "upper_arm_L": (45, 0, -20), "forearm_L": (45, 0, 0)},
            0.80: dict(_REST_2H),
        },
    },
    # Heavy overhead slam: longer windup (arms 165, spine +10, chest +12 by 0.35s), slam to spine -25, slow recover.
    "heavy_overhead": {
        "length": 0.9,
        "loop": False,
        "frames": {
            0.00: dict(_REST_2H),
            0.35: {"spine": (10, 0, 0), "chest": (12, 0, 0), "neck": (-6, 0, 0), "head": (10, 0, 0), "upper_arm_R": (165, 0, 12), "forearm_R": (35, 0, 0), "hand_R": (-20, 0, 0), "upper_arm_L": (165, 0, -12), "forearm_L": (35, 0, 0)},
            0.50: {"spine": (-25, 0, 0), "chest": (-26, 0, 0), "neck": (-6, 0, 0), "head": (-12, 0, 0), "upper_arm_R": (30, 0, 16), "forearm_R": (20, 0, 0), "hand_R": (16, 0, 0), "upper_arm_L": (30, 0, -16), "forearm_L": (20, 0, 0)},
            0.68: {"spine": (-18, 0, 0), "chest": (-18, 0, 0), "neck": (-4, 0, 0), "head": (-8, 0, 0), "upper_arm_R": (36, 0, 18), "forearm_R": (45, 0, 0), "hand_R": (10, 0, 0), "upper_arm_L": (36, 0, -18), "forearm_L": (45, 0, 0)},
            0.90: dict(_REST_2H),
        },
    },
    # Spear thrust: both hands on shaft, pull back (forearms 110), thrust forward (forearms 10, upper arms 85, chest -8).
    "spear_thrust": {
        "length": 0.55,
        "loop": False,
        "frames": {
            0.00: dict(_REST_2H),
            0.10: {"spine": (4, -14, 0), "chest": (4, -18, 0), "neck": (0, 6, 0), "head": (0, 8, 0), "upper_arm_R": (20, 0, 14), "forearm_R": (110, 0, 0), "hand_R": (-10, 0, 0), "upper_arm_L": (40, 0, -16), "forearm_L": (110, 0, 0)},
            0.22: {"spine": (-6, 12, 0), "chest": (-8, 16, 0), "neck": (0, -6, 0), "head": (-4, -8, 0), "upper_arm_R": (85, 0, 10), "forearm_R": (10, 0, 0), "hand_R": (8, 0, 0), "upper_arm_L": (85, 0, -14), "forearm_L": (10, 0, 0)},
            0.38: {"spine": (-3, 6, 0), "chest": (-4, 8, 0), "neck": (0, -3, 0), "head": (-2, -4, 0), "upper_arm_R": (55, 0, 16), "forearm_R": (55, 0, 0), "hand_R": (4, 0, 0), "upper_arm_L": (55, 0, -18), "forearm_L": (55, 0, 0)},
            0.55: dict(_REST_2H),
        },
    },
}
