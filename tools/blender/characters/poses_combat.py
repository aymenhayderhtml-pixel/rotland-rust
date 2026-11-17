"""Player combat clip tables: melee, aim, fire, reload and hit reaction.

Keyframes, bone entries and the rules are described in poses.py.
"""
from poses_shared import PISTOL_AIM, RIFLE_AIM

CLIPS = {
    # Right hand: wind up over the right shoulder, cut across the body to the left, recover.
    "melee_1h": {
        "length": 0.5,
        "loop": False,
        "frames": {
            0.0: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "neck": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (10, 0, 0),
                "forearm_R": (40, 0, 0),
                "hand_R": (0, 0, 0),
                "upper_arm_L": (10, 0, 0),
                "forearm_L": (30, 0, 0),
            },
            0.15: {
                "spine": (6, -24, 0),
                "chest": (4, -30, 0),
                "neck": (0, -10, 0),
                "head": (0, 10, 0),
                "upper_arm_R": (130, 0, -20),
                "forearm_R": (80, 0, 0),
                "hand_R": (-20, 0, 0),
                "upper_arm_L": (30, 0, 0),
                "forearm_L": (60, 0, 0),
            },
            0.30: {
                "spine": (-6, 28, 0),
                "chest": (-8, 34, 0),
                "neck": (0, 12, 0),
                "head": (-6, -8, 0),
                "upper_arm_R": (55, 0, 40),
                "forearm_R": (25, 0, 0),
                "hand_R": (10, 0, 0),
                "upper_arm_L": (-10, 0, 0),
                "forearm_L": (30, 0, 0),
            },
            0.40: {
                "spine": (-8, 36, 0),
                "chest": (-10, 40, 0),
                "neck": (0, 14, 0),
                "head": (-6, -10, 0),
                "upper_arm_R": (25, 0, 34),
                "forearm_R": (20, 0, 0),
                "hand_R": (10, 0, 0),
                "upper_arm_L": (-14, 0, 0),
                "forearm_L": (30, 0, 0),
            },
            0.50: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "neck": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (10, 0, 0),
                "forearm_R": (40, 0, 0),
                "hand_R": (0, 0, 0),
                "upper_arm_L": (10, 0, 0),
                "forearm_L": (30, 0, 0),
            },
        },
    },
    # Both hands on the haft: raise overhead with a big lean back, then chop forward and down.
    "melee_2h": {
        "length": 0.7,
        "loop": False,
        "frames": {
            0.0: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (30, 0, 20),
                "forearm_R": (90, 0, 0),
                "upper_arm_L": (30, 0, -20),
                "forearm_L": (90, 0, 0),
            },
            0.25: {
                "spine": (14, 0, 0),
                "chest": (16, 0, 0),
                "neck": (-4, 0, 0),
                "head": (10, 0, 0),
                "upper_arm_R": (165, 0, 12),
                "forearm_R": (35, 0, 0),
                "upper_arm_L": (165, 0, -12),
                "forearm_L": (35, 0, 0),
            },
            0.40: {
                "spine": (-18, 0, 0),
                "chest": (-20, 0, 0),
                "neck": (-6, 0, 0),
                "head": (-10, 0, 0),
                "upper_arm_R": (45, 0, 15),
                "forearm_R": (25, 0, 0),
                "upper_arm_L": (45, 0, -15),
                "forearm_L": (25, 0, 0),
            },
            0.55: {
                "spine": (-12, 0, 0),
                "chest": (-14, 0, 0),
                "neck": (-4, 0, 0),
                "head": (-6, 0, 0),
                "upper_arm_R": (60, 0, 25),
                "forearm_R": (40, 0, 0),
                "upper_arm_L": (60, 0, -25),
                "forearm_L": (40, 0, 0),
            },
            0.70: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (30, 0, 20),
                "forearm_R": (90, 0, 0),
                "upper_arm_L": (30, 0, -20),
                "forearm_L": (90, 0, 0),
            },
        },
    },
    # Aim and fire clips share one pose each, so a fire clip starts exactly on its aim pose.
    # Pistol: right arm out in front at shoulder height, left forearm under the grip.
    "pistol_aim": {
        "length": 1.0,
        "loop": True,
        "frames": {
            0.0: dict(PISTOL_AIM),
            0.5: {**PISTOL_AIM, "chest": (-3, -4, 0)},
            1.0: dict(PISTOL_AIM),
        },
    },
    # Recoil: forearm jumps up and the torso jolts back, then settles on the aim pose.
    "pistol_fire": {
        "length": 0.15,
        "loop": False,
        "frames": {
            0.0: dict(PISTOL_AIM),
            0.04: {**PISTOL_AIM, "chest": (2, -4, 0), "head": (2, 4, 0),
                   "upper_arm_R": (84, 0, 0), "forearm_R": (26, 0, 0)},
            0.15: dict(PISTOL_AIM),
        },
    },
    # Rifle shouldered: right elbow out with the trigger hand at chest height, left hand out to the stock.
    "rifle_aim": {
        "length": 1.0,
        "loop": True,
        "frames": {
            0.0: dict(RIFLE_AIM),
            0.5: {**RIFLE_AIM, "chest": (-3, -6, 0), "upper_arm_L": (88, 0, -36)},
            1.0: dict(RIFLE_AIM),
        },
    },
    # Rifle kick: the shoulder is shoved back and the head rises, then back to the aim pose.
    "rifle_fire": {
        "length": 0.2,
        "loop": False,
        "frames": {
            0.0: dict(RIFLE_AIM),
            0.05: {**RIFLE_AIM, "spine": (0, -6, 0), "chest": (4, -6, 0), "head": (3, 4, 0),
                   "upper_arm_R": (78, 0, -6), "forearm_R": (40, 0, 0)},
            0.20: dict(RIFLE_AIM),
        },
    },
    # Generic reload: hands meet at the waist, a short fiddle with the magazine, then back.
    "reload": {
        "length": 1.4,
        "loop": False,
        "frames": {
            0.0: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (0, 0, 0),
                "forearm_R": (10, 0, 0),
                "upper_arm_L": (0, 0, 0),
                "forearm_L": (10, 0, 0),
            },
            0.35: {
                "spine": (-3, 0, 0),
                "chest": (-4, 0, 0),
                "head": (-10, 0, 0),
                "upper_arm_R": (45, 0, 25),
                "forearm_R": (95, 0, 0),
                "upper_arm_L": (45, 0, -25),
                "forearm_L": (95, 0, 0),
            },
            0.90: {
                "spine": (-3, 0, 0),
                "chest": (-4, 0, 0),
                "head": (-14, 0, 0),
                "upper_arm_R": (50, 0, 35),
                "forearm_R": (95, 0, 0),
                "upper_arm_L": (50, 0, -20),
                "forearm_L": (100, 0, 0),
            },
            1.15: {
                "spine": (-3, 0, 0),
                "chest": (-4, 0, 0),
                "head": (-10, 0, 0),
                "upper_arm_R": (45, 0, 25),
                "forearm_R": (95, 0, 0),
                "upper_arm_L": (45, 0, -25),
                "forearm_L": (95, 0, 0),
            },
            1.40: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (0, 0, 0),
                "forearm_R": (10, 0, 0),
                "upper_arm_L": (0, 0, 0),
                "forearm_L": (10, 0, 0),
            },
        },
    },
    # Quick flinch: chest and head snap back, arms curl in, then back to neutral.
    "hit": {
        "length": 0.3,
        "loop": False,
        "frames": {
            0.0: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "neck": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (0, 0, 0),
                "forearm_R": (10, 0, 0),
                "upper_arm_L": (0, 0, 0),
                "forearm_L": (10, 0, 0),
            },
            0.06: {
                "spine": (12, 0, 0),
                "chest": (16, 0, 0),
                "neck": (8, 0, 0),
                "head": (14, 0, 6),
                "upper_arm_R": (20, 0, 20),
                "forearm_R": (70, 0, 0),
                "upper_arm_L": (20, 0, -20),
                "forearm_L": (70, 0, 0),
            },
            0.15: {
                "spine": (8, 0, 0),
                "chest": (10, 0, 0),
                "neck": (5, 0, 0),
                "head": (10, 0, 3),
                "upper_arm_R": (14, 0, 16),
                "forearm_R": (50, 0, 0),
                "upper_arm_L": (14, 0, -16),
                "forearm_L": (50, 0, 0),
            },
            0.30: {
                "spine": (0, 0, 0),
                "chest": (0, 0, 0),
                "neck": (0, 0, 0),
                "head": (0, 0, 0),
                "upper_arm_R": (0, 0, 0),
                "forearm_R": (10, 0, 0),
                "upper_arm_L": (0, 0, 0),
                "forearm_L": (10, 0, 0),
            },
        },
    },
}

# Gun clips for the player only (V12-04). NPCs never get them, so their glbs stay unchanged.
# poses.ALL_CLIPS merges them in, and build_characters.clips_for gives them to human_player only.
PLAYER_ONLY_CLIPS = {
    # Automatic fire loop: the shoulder jolts up and back on each round, then settles (loops, 0.1 s).
    "rifle_auto_fire": {
        "length": 0.1,
        "loop": True,
        "frames": {
            0.0: dict(RIFLE_AIM),
            0.03: {**RIFLE_AIM, "spine": (0, -6, 0), "chest": (3, -6, 0), "head": (2, 4, 0),
                   "upper_arm_R": (80, 0, -8), "forearm_R": (36, 0, 0)},
            0.10: dict(RIFLE_AIM),
        },
    },
    # Magazine change, 2.2 s: hands at the waist, right hand drops the empty mag (hidden by the code
    # from 0.7 s to 1.35 s), takes a new one from the belt, slaps it in, then pulls the charging handle.
    "rifle_reload_mag": {
        "length": 2.2,
        "loop": False,
        "frames": {
            0.0: {"spine": (0, 0, 0), "chest": (0, 0, 0), "head": (0, 0, 0),
                  "upper_arm_R": (0, 0, 0), "forearm_R": (10, 0, 0),
                  "upper_arm_L": (0, 0, 0), "forearm_L": (10, 0, 0)},
            0.35: {"spine": (-3, 0, 0), "chest": (-4, 0, 0), "head": (-10, 0, 0),
                   "upper_arm_R": (45, 0, 25), "forearm_R": (95, 0, 0),
                   "upper_arm_L": (45, 0, -25), "forearm_L": (95, 0, 0)},
            0.70: {"spine": (-4, 0, 0), "chest": (-6, 0, 0), "head": (-12, 0, 0),
                   "upper_arm_R": (60, 0, 40), "forearm_R": (70, 0, 0),
                   "upper_arm_L": (50, 0, -20), "forearm_L": (100, 0, 0)},
            1.00: {"spine": (-6, 0, 0), "chest": (-8, 0, 0), "head": (-16, 0, 0),
                   "upper_arm_R": (70, 0, 45), "forearm_R": (100, 0, 0),
                   "upper_arm_L": (60, 0, -25), "forearm_L": (110, 0, 0)},
            1.35: {"spine": (-3, 0, 0), "chest": (-4, 0, 0), "head": (-10, 0, 0),
                   "upper_arm_R": (50, 0, 15), "forearm_R": (95, 0, 0),
                   "upper_arm_L": (40, 0, -25), "forearm_L": (95, 0, 0)},
            1.70: {"spine": (-2, 0, 0), "chest": (-3, 0, 0), "head": (-8, 0, 0),
                   "upper_arm_R": (70, 0, 20), "forearm_R": (40, 0, 0),
                   "upper_arm_L": (45, 0, -25), "forearm_L": (90, 0, 0)},
            2.00: {"spine": (-1, 0, 0), "chest": (-2, 0, 0), "head": (-4, 0, 0),
                   "upper_arm_R": (20, 0, 10), "forearm_R": (40, 0, 0),
                   "upper_arm_L": (20, 0, -10), "forearm_L": (40, 0, 0)},
            2.20: {"spine": (0, 0, 0), "chest": (0, 0, 0), "head": (0, 0, 0),
                   "upper_arm_R": (0, 0, 0), "forearm_R": (10, 0, 0),
                   "upper_arm_L": (0, 0, 0), "forearm_L": (10, 0, 0)},
        },
    },
    # Gun bash, 0.6 s: the rifle is pulled back under the arm, then butt-stroked forward (impact at 0.30 s).
    "gun_bash": {
        "length": 0.6,
        "loop": False,
        "frames": {
            0.0: dict(RIFLE_AIM),
            0.15: {"spine": (6, 0, 0), "chest": (8, 0, 0), "neck": (-2, 0, 0), "head": (-4, 0, 0),
                   "upper_arm_R": (60, 0, -10), "forearm_R": (110, 0, 0),
                   "upper_arm_L": (60, 0, -30), "forearm_L": (100, 0, 0)},
            0.30: {"spine": (-8, -6, 0), "chest": (-12, -6, 0), "neck": (0, 4, 0), "head": (-6, 4, 0),
                   "upper_arm_R": (88, 0, -6), "forearm_R": (0, 0, 0),
                   "upper_arm_L": (88, 0, -36), "forearm_L": (0, 0, 0)},
            0.45: {"spine": (-4, -4, 0), "chest": (-5, -5, 0), "neck": (0, 4, 0), "head": (-2, 4, 0),
                   "upper_arm_R": (84, 0, -6), "forearm_R": (30, 0, 0),
                   "upper_arm_L": (88, 0, -40), "forearm_L": (16, 0, 0)},
            0.60: dict(RIFLE_AIM),
        },
    },
}
