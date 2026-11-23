"""Player weapon aim and reload clips (upper body only).

Arm Z values are relative to ARM_DOWN_OFFSET in poses_shared.py.
Keyframes and bone entry conventions are described in poses.py.
In aim poses, hand_R cancels forearm_R bend so aimHold.rot = [0, 0, 0] keeps the barrel level.
"""


def _breathing_loop(base, breath):
    """Build a 2.0 s seamless looping aim clip with tiny breathing at mid-cycle."""
    mid = {**base, **breath}
    return {
        "length": 2.0,
        "loop": True,
        "frames": {
            0.0: dict(base),
            1.0: mid,
            2.0: dict(base),
        },
    }


# Rifle shouldered aim: torso bladed so left shoulder leads, right elbow bent sharply so the
# right hand sits near the right side of the chest and the stock meets the shoulder;
# hand_R cancels forearm_R bend to keep the barrel level; left arm reaches under the fore-end.
_AIM_RIFLE_SHOULDER = {
    "spine": (-3, -8, 0),
    "chest": (-4, -10, 0),
    "neck": (0, 8, 0),
    "head": (-4, 10, 0),
    "upper_arm_R": (24, 0, -22),
    "forearm_R": (108, 0, 0),
    "hand_R": (-40, 0, 0),
    "upper_arm_L": (78, 0, -32),
    "forearm_L": (26, 0, 0),
}

# Shotgun shouldered aim: same bladed shouldered family with heavier forward lean.
_AIM_SHOTGUN_SHOULDER = {
    "spine": (-5, -10, 0),
    "chest": (-6, -12, 0),
    "neck": (2, 10, 0),
    "head": (-3, 12, 0),
    "upper_arm_R": (22, 0, -24),
    "forearm_R": (112, 0, 0),
    "hand_R": (-38, 0, 0),
    "upper_arm_L": (82, 0, -30),
    "forearm_L": (22, 0, 0),
}

# Pistol two-handed aim: both arms forward in a triangle with slight elbow bend and hands together.
_AIM_PISTOL_TWO_HAND = {
    "spine": (-1, -3, 0),
    "chest": (-2, -3, 0),
    "neck": (0, 3, 0),
    "head": (-2, 3, 0),
    "upper_arm_R": (68, 0, 12),
    "forearm_R": (22, 0, 0),
    "hand_R": (-2, 0, 0),
    "upper_arm_L": (66, 0, -24),
    "forearm_L": (26, 0, 0),
}

# Bow full draw: bow is held in the right hand in this game, so right arm extends straight
# forward holding the bow and left hand draws the string back to the cheek.
_AIM_BOW_DRAW = {
    "spine": (-1, 12, 0),
    "chest": (-2, 16, 0),
    "neck": (0, -12, 0),
    "head": (-1, -16, 0),
    "upper_arm_R": (88, 0, -16),
    "forearm_R": (4, 0, 0),
    "hand_R": (-4, 0, 0),
    "upper_arm_L": (32, 0, 26),
    "forearm_L": (124, 0, 0),
}

# Pistol reload (1.0 s, non-looping): tilt gun in, left hand to belt, up into grip, slap.
_RELOAD_PISTOL = {
    "length": 1.0,
    "loop": False,
    "frames": {
        0.0: {
            "spine": (-1, -3, 0),
            "chest": (-2, -3, 0),
            "neck": (0, 3, 0),
            "head": (-2, 3, 0),
            "upper_arm_R": (58, 0, 12),
            "forearm_R": (36, 0, 0),
            "hand_R": (-8, 0, 0),
            "upper_arm_L": (54, 0, -24),
            "forearm_L": (38, 0, 0),
        },
        0.2: {
            "spine": (-2, -4, 0),
            "chest": (-4, -5, 0),
            "neck": (-2, 4, 0),
            "head": (-6, 5, 0),
            "upper_arm_R": (40, 0, 16),
            "forearm_R": (66, 0, 0),
            "hand_R": (-20, 22, 0),
            "upper_arm_L": (22, 0, -10),
            "forearm_L": (38, 0, 0),
        },
        0.45: {
            "spine": (-2, -5, 0),
            "chest": (-5, -6, 0),
            "neck": (-3, 5, 0),
            "head": (-8, 6, 0),
            "upper_arm_R": (38, 0, 16),
            "forearm_R": (68, 0, 0),
            "hand_R": (-22, 24, 0),
            "upper_arm_L": (-8, 0, 8),
            "forearm_L": (46, 0, 0),
        },
        0.72: {
            "spine": (-2, -4, 0),
            "chest": (-3, -4, 0),
            "neck": (-1, 4, 0),
            "head": (-5, 4, 0),
            "upper_arm_R": (42, 0, 15),
            "forearm_R": (62, 0, 0),
            "hand_R": (-16, 18, 0),
            "upper_arm_L": (50, 0, -28),
            "forearm_L": (60, 0, 0),
        },
        0.85: {
            "spine": (-1, -3, 0),
            "chest": (-2, -3, 0),
            "neck": (0, 3, 0),
            "head": (-3, 3, 0),
            "upper_arm_R": (46, 0, 14),
            "forearm_R": (56, 0, 0),
            "hand_R": (-12, 10, 0),
            "upper_arm_L": (58, 0, -30),
            "forearm_L": (52, 0, 0),
        },
        1.0: {
            "spine": (-1, -3, 0),
            "chest": (-2, -3, 0),
            "neck": (0, 3, 0),
            "head": (-2, 3, 0),
            "upper_arm_R": (58, 0, 12),
            "forearm_R": (36, 0, 0),
            "hand_R": (-8, 0, 0),
            "upper_arm_L": (54, 0, -24),
            "forearm_L": (38, 0, 0),
        },
    },
}

# Rifle reload (1.0 s, non-looping): pull mag down, to belt, insert mag, pull bolt.
_RELOAD_RIFLE = {
    "length": 1.0,
    "loop": False,
    "frames": {
        0.0: {
            "spine": (-3, -8, 0),
            "chest": (-4, -10, 0),
            "neck": (0, 8, 0),
            "head": (-4, 10, 0),
            "upper_arm_R": (26, 0, -18),
            "forearm_R": (102, 0, 0),
            "hand_R": (-34, 0, 0),
            "upper_arm_L": (62, 0, -32),
            "forearm_L": (46, 0, 0),
        },
        0.18: {
            "spine": (-3, -9, 0),
            "chest": (-5, -11, 0),
            "neck": (-2, 9, 0),
            "head": (-7, 10, 0),
            "upper_arm_R": (24, 0, -16),
            "forearm_R": (100, 0, 0),
            "hand_R": (-28, 12, 0),
            "upper_arm_L": (28, 0, -22),
            "forearm_L": (42, 0, 0),
        },
        0.38: {
            "spine": (-4, -10, 0),
            "chest": (-6, -12, 0),
            "neck": (-3, 10, 0),
            "head": (-9, 11, 0),
            "upper_arm_R": (22, 0, -16),
            "forearm_R": (102, 0, 0),
            "hand_R": (-26, 14, 0),
            "upper_arm_L": (-10, 0, 6),
            "forearm_L": (44, 0, 0),
        },
        0.62: {
            "spine": (-3, -9, 0),
            "chest": (-5, -10, 0),
            "neck": (-1, 9, 0),
            "head": (-6, 10, 0),
            "upper_arm_R": (24, 0, -16),
            "forearm_R": (100, 0, 0),
            "hand_R": (-28, 10, 0),
            "upper_arm_L": (52, 0, -30),
            "forearm_L": (64, 0, 0),
        },
        0.82: {
            "spine": (-2, -8, 0),
            "chest": (-3, -9, 0),
            "neck": (0, 8, 0),
            "head": (-4, 9, 0),
            "upper_arm_R": (26, 0, -20),
            "forearm_R": (104, 0, 0),
            "hand_R": (-36, 4, 0),
            "upper_arm_L": (42, 0, -34),
            "forearm_L": (86, 0, 0),
        },
        1.0: {
            "spine": (-3, -8, 0),
            "chest": (-4, -10, 0),
            "neck": (0, 8, 0),
            "head": (-4, 10, 0),
            "upper_arm_R": (24, 0, -22),
            "forearm_R": (108, 0, 0),
            "hand_R": (-40, 0, 0),
            "upper_arm_L": (76, 0, -32),
            "forearm_L": (28, 0, 0),
        },
    },
}

# Shotgun reload (1.0 s, non-looping): barrel tilted up, left hand loads 3 shells from belt.
_SHOTGUN_TILT_HOLD = {
    "spine": (-4, -10, 0),
    "chest": (-5, -12, 0),
    "neck": (-1, 10, 0),
    "head": (-6, 11, 0),
    "upper_arm_R": (28, 0, -18),
    "forearm_R": (98, 0, 0),
    "hand_R": (-16, 12, 0),
}
_SHOTGUN_SHELL_BELT = {
    **_SHOTGUN_TILT_HOLD,
    "upper_arm_L": (-6, 0, 4),
    "forearm_L": (42, 0, 0),
}
_SHOTGUN_SHELL_INSERT = {
    **_SHOTGUN_TILT_HOLD,
    "chest": (-4, -11, 0),
    "upper_arm_L": (46, 0, -32),
    "forearm_L": (68, 0, 0),
}

_RELOAD_SHOTGUN = {
    "length": 1.0,
    "loop": False,
    "frames": {
        0.0: {
            **_SHOTGUN_TILT_HOLD,
            "upper_arm_L": (56, 0, -30),
            "forearm_L": (48, 0, 0),
        },
        0.12: dict(_SHOTGUN_SHELL_BELT),
        0.26: dict(_SHOTGUN_SHELL_INSERT),
        0.40: dict(_SHOTGUN_SHELL_BELT),
        0.54: dict(_SHOTGUN_SHELL_INSERT),
        0.68: dict(_SHOTGUN_SHELL_BELT),
        0.82: dict(_SHOTGUN_SHELL_INSERT),
        1.0: {
            "spine": (-5, -10, 0),
            "chest": (-6, -12, 0),
            "neck": (2, 10, 0),
            "head": (-3, 12, 0),
            "upper_arm_R": (22, 0, -24),
            "forearm_R": (112, 0, 0),
            "hand_R": (-38, 0, 0),
            "upper_arm_L": (80, 0, -30),
            "forearm_L": (24, 0, 0),
        },
    },
}

CLIPS = {
    "aim_rifle_shoulder": _breathing_loop(
        _AIM_RIFLE_SHOULDER,
        {"chest": (-3, -10, 0), "upper_arm_R": (25, 0, -22), "upper_arm_L": (79, 0, -32)},
    ),
    "aim_shotgun_shoulder": _breathing_loop(
        _AIM_SHOTGUN_SHOULDER,
        {"chest": (-5, -12, 0), "upper_arm_R": (23, 0, -24), "upper_arm_L": (83, 0, -30)},
    ),
    "aim_pistol_two_hand": _breathing_loop(
        _AIM_PISTOL_TWO_HAND,
        {"chest": (-1, -3, 0), "upper_arm_R": (69, 0, 12), "upper_arm_L": (67, 0, -24)},
    ),
    "aim_bow_draw": _breathing_loop(
        _AIM_BOW_DRAW,
        {"chest": (-1, 16, 0), "upper_arm_R": (89, 0, -16), "forearm_L": (126, 0, 0)},
    ),
    "reload_pistol": _RELOAD_PISTOL,
    "reload_rifle": _RELOAD_RIFLE,
    "reload_shotgun": _RELOAD_SHOTGUN,
}
