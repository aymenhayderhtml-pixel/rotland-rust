"""Camp-life clip tables (Phase 3): sit, sit_ground, talk, listen, wave, carry, chop.

Player and settlement NPCs get these (zombies do not). Keyframes, bone entries and the rules are
described in poses.py. Arm Z is relative to the arms-down offset: for the right arm a negative Z
moves it out to the side, for the left arm a positive Z moves it out.

Sitting: hips drop with `loc` Y (hips about 0.5 m on a 0.45 m seat). Thighs point forward (+X).
"""
import copy

from poses_locomotion import CLIPS as _LOCOMOTION

CLIPS = {
    # Seat at 0.45 m: hips about 0.50 m, thighs flat forward, shins down, feet on the floor.
    "sit": {
        "length": 2.0,
        "loop": True,
        "frames": {
            0.0: {
                "hips": {"rot": (0, 0, 0), "loc": (0, -0.38, 0)},
                "spine": (4, 0, 0),
                "chest": (2, 0, 0),
                "neck": (-2, 0, 0),
                "head": (-1, 0, 0),
                "thigh_R": (90, 0, 2),
                "shin_R": (-90, 0, 0),
                "thigh_L": (90, 0, -2),
                "shin_L": (-90, 0, 0),
                "upper_arm_R": (20, 0, -4),
                "forearm_R": (35, 0, 0),
                "upper_arm_L": (20, 0, 4),
                "forearm_L": (35, 0, 0),
            },
            1.0: {
                "hips": {"rot": (0, 0, 0), "loc": (0, -0.37, 0)},
                "spine": (6, 0, 0),
                "chest": (3, 0, 0),
                "neck": (-2, 0, 0),
                "head": (-1, 0, 0),
                "thigh_R": (90, 0, 2),
                "shin_R": (-90, 0, 0),
                "thigh_L": (90, 0, -2),
                "shin_L": (-90, 0, 0),
                "upper_arm_R": (21, 0, -4),
                "forearm_R": (36, 0, 0),
                "upper_arm_L": (21, 0, 4),
                "forearm_L": (36, 0, 0),
            },
            2.0: {
                "hips": {"rot": (0, 0, 0), "loc": (0, -0.38, 0)},
                "spine": (4, 0, 0),
                "chest": (2, 0, 0),
                "neck": (-2, 0, 0),
                "head": (-1, 0, 0),
                "thigh_R": (90, 0, 2),
                "shin_R": (-90, 0, 0),
                "thigh_L": (90, 0, -2),
                "shin_L": (-90, 0, 0),
                "upper_arm_R": (20, 0, -4),
                "forearm_R": (35, 0, 0),
                "upper_arm_L": (20, 0, 4),
                "forearm_L": (35, 0, 0),
            },
        },
    },
    # On the ground by a fire: butt on the floor (hips about 0.04 m), knees up, arms back to lean.
    "sit_ground": {
        "length": 2.0,
        "loop": True,
        "frames": {
            0.0: {
                "hips": {"rot": (0, 0, 0), "loc": (0, -0.86, 0)},
                "spine": (10, 0, 0),
                "chest": (6, 0, 0),
                "neck": (-4, 0, 0),
                "head": (-2, 0, 0),
                "thigh_R": (150, 0, 6),
                "shin_R": (-110, 0, 0),
                "foot_R": (-40, 0, 0),
                "thigh_L": (150, 0, -6),
                "shin_L": (-110, 0, 0),
                "foot_L": (-40, 0, 0),
                "upper_arm_R": (-30, 0, -15),
                "forearm_R": (10, 0, 0),
                "upper_arm_L": (-30, 0, 15),
                "forearm_L": (10, 0, 0),
            },
            1.0: {
                "hips": {"rot": (0, 0, 0), "loc": (0, -0.86, 0)},
                "spine": (12, 0, 0),
                "chest": (7, 0, 0),
                "neck": (-4, 0, 0),
                "head": (-2, 0, 0),
                "thigh_R": (150, 0, 6),
                "shin_R": (-110, 0, 0),
                "foot_R": (-40, 0, 0),
                "thigh_L": (150, 0, -6),
                "shin_L": (-110, 0, 0),
                "foot_L": (-40, 0, 0),
                "upper_arm_R": (-31, 0, -15),
                "forearm_R": (11, 0, 0),
                "upper_arm_L": (-31, 0, 15),
                "forearm_L": (11, 0, 0),
            },
            2.0: {
                "hips": {"rot": (0, 0, 0), "loc": (0, -0.86, 0)},
                "spine": (10, 0, 0),
                "chest": (6, 0, 0),
                "neck": (-4, 0, 0),
                "head": (-2, 0, 0),
                "thigh_R": (150, 0, 6),
                "shin_R": (-110, 0, 0),
                "foot_R": (-40, 0, 0),
                "thigh_L": (150, 0, -6),
                "shin_L": (-110, 0, 0),
                "foot_L": (-40, 0, 0),
                "upper_arm_R": (-30, 0, -15),
                "forearm_R": (10, 0, 0),
                "upper_arm_L": (-30, 0, 15),
                "forearm_L": (10, 0, 0),
            },
        },
    },
    # Standing, right hand gesturing, head nods every 0.6 s, left arm relaxed.
    "talk": {
        "length": 2.4,
        "loop": True,
        "frames": {
            0.0: {
                "head": (0, 0, 0),
                "neck": (0, 0, 0),
                "upper_arm_R": (8, 0, -6),
                "forearm_R": (35, 0, 0),
                "upper_arm_L": (-3, 0, 3),
                "forearm_L": (12, 0, 0),
            },
            0.6: {
                "spine": (2, -3, 0),
                "head": (-6, 0, 3),
                "neck": (-3, 0, 0),
                "upper_arm_R": (22, 0, -10),
                "forearm_R": (70, 0, 0),
                "hand_R": (-10, 0, 0),
                "upper_arm_L": (-3, 0, 3),
                "forearm_L": (12, 0, 0),
            },
            1.2: {
                "spine": (0, 0, 0),
                "head": (1, 0, -2),
                "neck": (0, 0, 0),
                "upper_arm_R": (14, 0, -8),
                "forearm_R": (40, 0, 0),
                "upper_arm_L": (0, 0, 3),
                "forearm_L": (20, 0, 0),
            },
            1.8: {
                "spine": (-2, 3, 0),
                "head": (-6, 0, -3),
                "neck": (-3, 0, 0),
                "upper_arm_R": (24, 0, -12),
                "forearm_R": (75, 0, 0),
                "hand_R": (-10, 0, 0),
                "upper_arm_L": (0, 0, 3),
                "forearm_L": (20, 0, 0),
            },
            2.4: {
                "head": (0, 0, 0),
                "neck": (0, 0, 0),
                "upper_arm_R": (8, 0, -6),
                "forearm_R": (35, 0, 0),
                "upper_arm_L": (-3, 0, 3),
                "forearm_L": (12, 0, 0),
            },
        },
    },
    # Standing, arms crossed, head tilting side to side.
    "listen": {
        "length": 2.4,
        "loop": True,
        "frames": {
            0.0: {
                "spine": (-2, 0, 0),
                "chest": (-2, 0, 0),
                "head": (2, 0, -6),
                "upper_arm_R": (45, 0, 35),
                "forearm_R": (75, 0, 0),
                "upper_arm_L": (45, 0, -35),
                "forearm_L": (75, 0, 0),
            },
            1.2: {
                "spine": (-2, 0, 0),
                "chest": (-2, 0, 0),
                "head": (2, 0, 6),
                "upper_arm_R": (45, 0, 35),
                "forearm_R": (75, 0, 0),
                "upper_arm_L": (45, 0, -35),
                "forearm_L": (75, 0, 0),
            },
            2.4: {
                "spine": (-2, 0, 0),
                "chest": (-2, 0, 0),
                "head": (2, 0, -6),
                "upper_arm_R": (45, 0, 35),
                "forearm_R": (75, 0, 0),
                "upper_arm_L": (45, 0, -35),
                "forearm_L": (75, 0, 0),
            },
        },
    },
    # One-shot: right arm up and out, forearm waving, back to rest.
    "wave": {
        "length": 1.2,
        "loop": False,
        "frames": {
            0.0: {
                "upper_arm_R": (0, 0, 0),
                "forearm_R": (0, 0, 0),
            },
            0.25: {
                "upper_arm_R": (5, 0, -95),
                "forearm_R": (0, 80, -75),
                "head": (0, -6, 0),
            },
            0.45: {
                "upper_arm_R": (5, 0, -95),
                "forearm_R": (0, 80, -60),
                "head": (0, -6, 0),
            },
            0.65: {
                "upper_arm_R": (5, 0, -95),
                "forearm_R": (0, 80, -90),
                "head": (0, -6, 0),
            },
            0.85: {
                "upper_arm_R": (5, 0, -95),
                "forearm_R": (0, 80, -60),
                "head": (0, -6, 0),
            },
            1.0: {
                "upper_arm_R": (5, 0, -95),
                "forearm_R": (0, 80, -75),
                "head": (0, -6, 0),
            },
            1.2: {
                "upper_arm_R": (0, 0, 0),
                "forearm_R": (0, 0, 0),
                "head": (0, 0, 0),
            },
        },
    },
    # Filled in below: the walk cycle's legs and torso, arms held still at waist height.
    "carry": {
        "length": 1.0,
        "loop": True,
        "frames": {},
    },
    # Woodcutter: two hands on the haft, raised overhead, then chopped down. Loops.
    "chop": {
        "length": 1.2,
        "loop": True,
        "frames": {
            0.0: {
                "spine": (2, 0, 0),
                "chest": (2, 0, 0),
                "upper_arm_R": (40, 0, 12),
                "forearm_R": (80, 0, 0),
                "upper_arm_L": (40, 0, -12),
                "forearm_L": (80, 0, 0),
            },
            0.35: {
                "spine": (14, 0, 0),
                "chest": (16, 0, 0),
                "neck": (-4, 0, 0),
                "head": (6, 0, 0),
                "upper_arm_R": (165, 0, 12),
                "forearm_R": (35, 0, 0),
                "upper_arm_L": (165, 0, -12),
                "forearm_L": (35, 0, 0),
            },
            0.6: {
                "spine": (-18, 0, 0),
                "chest": (-20, 0, 0),
                "neck": (-6, 0, 0),
                "head": (-8, 0, 0),
                "upper_arm_R": (40, 0, 12),
                "forearm_R": (30, 0, 0),
                "upper_arm_L": (40, 0, -12),
                "forearm_L": (30, 0, 0),
            },
            0.85: {
                "spine": (-20, 0, 0),
                "chest": (-22, 0, 0),
                "neck": (-6, 0, 0),
                "head": (-8, 0, 0),
                "upper_arm_R": (32, 0, 12),
                "forearm_R": (40, 0, 0),
                "upper_arm_L": (32, 0, -12),
                "forearm_L": (40, 0, 0),
            },
            1.2: {
                "spine": (2, 0, 0),
                "chest": (2, 0, 0),
                "upper_arm_R": (40, 0, 12),
                "forearm_R": (80, 0, 0),
                "upper_arm_L": (40, 0, -12),
                "forearm_L": (80, 0, 0),
            },
        },
    },
}

# carry: the walk cycle's legs and torso, with both arms bent forward in front of the body.
_CARRY_ARMS = {
    "upper_arm_R": (10, 0, 22),
    "forearm_R": (85, 0, 0),
    "upper_arm_L": (10, 0, -22),
    "forearm_L": (85, 0, 0),
    "shoulder_R": (0, 0, 0),
    "shoulder_L": (0, 0, 0),
}
_ARM_BONES = ("shoulder_R", "upper_arm_R", "forearm_R", "shoulder_L", "upper_arm_L", "forearm_L")
for _t, _pose in _LOCOMOTION["walk"]["frames"].items():
    _frame = copy.deepcopy(_pose)
    for _bone in _ARM_BONES:
        _frame.pop(_bone, None)
    _frame.update(_CARRY_ARMS)
    CLIPS["carry"]["frames"][_t] = _frame
