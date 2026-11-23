"""Player weapon-holding ready stance clips (upper body only, 2.0 s subtle breathing loops).

Arm Z values are relative to ARM_DOWN_OFFSET in poses_shared.py.
Keyframes and bone entry conventions are described in poses.py.
"""


def _breathing_loop(base, breath):
    """Build a 2.0 s seamless looping stance clip with subtle breathing at mid-cycle."""
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


# Pistol low ready: gun angled down and forward, right elbow slightly bent, left arm relaxed near it.
_PISTOL_READY = {
    "spine": (-1, -3, 0),
    "chest": (-2, -3, 0),
    "neck": (0, 3, 0),
    "head": (-1, 3, 0),
    "upper_arm_R": (42, 0, 10),
    "forearm_R": (28, 0, 0),
    "hand_R": (12, 0, 0),
    "upper_arm_L": (28, 0, -18),
    "forearm_L": (42, 0, 0),
}

# Rifle low ready: across the body, muzzle low to the left, right hand near hip/lower chest,
# left hand forward supporting the barrel.
_RIFLE_READY = {
    "spine": (-2, -6, 0),
    "chest": (-3, -6, 0),
    "neck": (0, 5, 0),
    "head": (-1, 5, 0),
    "upper_arm_R": (26, 0, 8),
    "forearm_R": (68, 0, 0),
    "hand_R": (8, 22, 0),
    "upper_arm_L": (52, 0, -24),
    "forearm_L": (38, 0, 0),
}

# Shotgun low ready: like the rifle, slightly lower and heavier.
_SHOTGUN_READY = {
    "spine": (-3, -7, 0),
    "chest": (-4, -7, 0),
    "neck": (0, 6, 0),
    "head": (-1, 6, 0),
    "upper_arm_R": (20, 0, 6),
    "forearm_R": (62, 0, 0),
    "hand_R": (12, 20, 0),
    "upper_arm_L": (44, 0, -20),
    "forearm_L": (34, 0, 0),
}

# One-handed melee ready: weapon low at the side, elbow bent, relaxed and ready.
_MELEE1H_READY = {
    "spine": (-1, -4, 0),
    "chest": (-2, -4, 0),
    "neck": (0, 4, 0),
    "head": (0, 4, 0),
    "upper_arm_R": (14, 0, -8),
    "forearm_R": (48, 0, 0),
    "hand_R": (0, 0, 0),
    "upper_arm_L": (12, 0, 4),
    "forearm_L": (26, 0, 0),
}

# Two-handed melee ready: both hands on the handle, weapon resting up toward the right shoulder.
_MELEE2H_READY = {
    "spine": (-2, -10, 0),
    "chest": (-3, -12, 0),
    "neck": (0, 10, 0),
    "head": (0, 10, 0),
    "upper_arm_R": (46, 0, -10),
    "forearm_R": (88, 0, 0),
    "hand_R": (-12, 0, 0),
    "upper_arm_L": (42, 0, -34),
    "forearm_L": (78, 0, 0),
}

# Spear ready: both hands, spear held diagonally with point forward and up.
_SPEAR_READY = {
    "spine": (-2, -12, 0),
    "chest": (-3, -14, 0),
    "neck": (0, 12, 0),
    "head": (-1, 12, 0),
    "upper_arm_R": (22, 0, -6),
    "forearm_R": (58, 0, 0),
    "hand_R": (-18, 0, 0),
    "upper_arm_L": (58, 0, -22),
    "forearm_L": (42, 0, 0),
}

# Bow ready: bow held low in front in the hand, other arm relaxed.
_BOW_READY = {
    "spine": (-1, -4, 0),
    "chest": (-2, -4, 0),
    "neck": (0, 4, 0),
    "head": (0, 4, 0),
    "upper_arm_R": (34, 0, 12),
    "forearm_R": (36, 0, 0),
    "hand_R": (0, 0, 0),
    "upper_arm_L": (10, 0, 4),
    "forearm_L": (22, 0, 0),
}

# Fishing rod ready: rod angled ~45 deg forward/up, right hand near waist, left hand supporting.
_ROD_READY = {
    "spine": (1, -4, 0),
    "chest": (0, -4, 0),
    "neck": (0, 4, 0),
    "head": (0, 4, 0),
    "upper_arm_R": (24, 0, 10),
    "forearm_R": (76, 0, 0),
    "hand_R": (-16, 0, 0),
    "upper_arm_L": (40, 0, -26),
    "forearm_L": (62, 0, 0),
}

CLIPS = {
    "stance_pistol_ready": _breathing_loop(
        _PISTOL_READY,
        {"chest": (-1, -3, 0), "upper_arm_R": (43, 0, 10), "upper_arm_L": (29, 0, -18)},
    ),
    "stance_rifle_ready": _breathing_loop(
        _RIFLE_READY,
        {"chest": (-2, -6, 0), "upper_arm_R": (27, 0, 8), "upper_arm_L": (53, 0, -24)},
    ),
    "stance_shotgun_ready": _breathing_loop(
        _SHOTGUN_READY,
        {"chest": (-3, -7, 0), "upper_arm_R": (21, 0, 6), "upper_arm_L": (45, 0, -20)},
    ),
    "stance_melee1h_ready": _breathing_loop(
        _MELEE1H_READY,
        {"chest": (-1, -4, 0), "upper_arm_R": (15, 0, -8), "forearm_R": (50, 0, 0)},
    ),
    "stance_melee2h_ready": _breathing_loop(
        _MELEE2H_READY,
        {"chest": (-2, -12, 0), "upper_arm_R": (47, 0, -10), "upper_arm_L": (43, 0, -34)},
    ),
    "stance_spear_ready": _breathing_loop(
        _SPEAR_READY,
        {"chest": (-2, -14, 0), "upper_arm_R": (23, 0, -6), "upper_arm_L": (59, 0, -22)},
    ),
    "stance_bow_ready": _breathing_loop(
        _BOW_READY,
        {"chest": (-1, -4, 0), "upper_arm_R": (35, 0, 12), "upper_arm_L": (11, 0, 4)},
    ),
    "stance_rod_ready": _breathing_loop(
        _ROD_READY,
        {"chest": (1, -4, 0), "upper_arm_R": (25, 0, 10), "upper_arm_L": (41, 0, -26)},
    ),
}
