"""Directional zombie death clips (A17): fall forward and fall backward.

Keyframes, bone entries and rules are described in poses.py.
Sign rules:
- root rot X negative = whole body tips forward about the feet (positive = backward).
- hips rot X negative = lean forward; hips loc Y negative = lower.
- spine/chest/head X negative = bend forward.
- upper_arm_* X positive = arm forward/up (0 = hanging); forearm_* X positive = elbow bends.
- thigh_* X positive = leg forward; shin_* X negative = knee bends.
"""
CLIPS = {
    # Pushed from behind: stumbles a step, folds forward and lands flat on the face.
    # Like `die`, the whole body tips about the feet with `root` (negative X = forward).
    "zombie_die_forward": {
        "length": 1.2,
        "loop": False,
        "frames": {
            0.0: {"root": {"rot": (0, 0, 0), "loc": (0, 0, 0)}},
            0.15: {
                "root": {"rot": (-8, 0, 0), "loc": (0, 0, 0)},
                "spine": (-10, 0, 0),
                "head": (8, 0, 0),
                "upper_arm_R": (20, 0, 0),
                "upper_arm_L": (20, 0, 0),
                "thigh_R": (14, 0, 0),
                "shin_R": (-20, 0, 0),
            },
            0.40: {
                "root": {"rot": (-34, 0, 0), "loc": (0, 0, 0.03)},
                "spine": (-14, 0, 0),
                "head": (12, 0, 0),
                "upper_arm_R": (55, 0, 0),
                "upper_arm_L": (50, 0, 0),
                "forearm_R": (25, 0, 0),
                "forearm_L": (25, 0, 0),
                "thigh_R": (22, 0, 0),
                "shin_R": (-50, 0, 0),
                "thigh_L": (16, 0, 0),
                "shin_L": (-40, 0, 0),
            },
            0.75: {
                "root": {"rot": (-72, 0, 0), "loc": (0, 0, 0.07)},
                "spine": (-4, 0, 0),
                "head": (14, 0, 0),
                "upper_arm_R": (95, 0, 0),
                "upper_arm_L": (85, 0, 0),
                "forearm_R": (20, 0, 0),
                "forearm_L": (20, 0, 0),
                "thigh_R": (6, 0, 0),
                "shin_R": (-20, 0, 0),
                "thigh_L": (4, 0, 0),
                "shin_L": (-16, 0, 0),
            },
            1.00: {
                "root": {"rot": (-88, 0, 0), "loc": (0, 0, 0.10)},
                "spine": (2, 0, 0),
                "head": (20, 12, 0),
                "upper_arm_R": (120, 0, -10),
                "upper_arm_L": (100, 0, 10),
                "forearm_R": (12, 0, 0),
                "forearm_L": (16, 0, 0),
                "thigh_R": (2, 0, 0),
                "shin_R": (-6, 0, 0),
                "thigh_L": (0, 0, 0),
                "shin_L": (-4, 0, 0),
            },
            1.20: {
                "root": {"rot": (-88, 0, 0), "loc": (0, 0, 0.10)},
                "spine": (2, 0, 0),
                "head": (20, 12, 0),
                "upper_arm_R": (120, 0, -10),
                "upper_arm_L": (100, 0, 10),
                "forearm_R": (10, 0, 0),
                "forearm_L": (14, 0, 0),
                "thigh_R": (2, 0, 0),
                "shin_R": (-6, 0, 0),
                "thigh_L": (0, 0, 0),
                "shin_L": (-4, 0, 0),
            },
        },
    },
    # Hit from the front: head snaps back, knees buckle, lands flat on the back, arms out.
    # Like `die`, the whole body tips about the feet with `root` (positive X = backward).
    "zombie_die_back": {
        "length": 1.2,
        "loop": False,
        "frames": {
            0.0: {"root": {"rot": (0, 0, 0), "loc": (0, 0, 0)}},
            0.12: {
                "root": {"rot": (8, 0, 0), "loc": (0, 0, 0)},
                "spine": (8, 0, 0),
                "head": (-18, 0, 0),
                "upper_arm_R": (-15, 0, -20),
                "upper_arm_L": (-15, 0, 20),
                "forearm_R": (30, 0, 0),
                "forearm_L": (30, 0, 0),
            },
            0.40: {
                "root": {"rot": (34, 0, 0), "loc": (0, 0, 0.03)},
                "spine": (6, 0, 0),
                "head": (-10, 0, 0),
                "upper_arm_R": (-20, 0, -45),
                "upper_arm_L": (-20, 0, 45),
                "forearm_R": (35, 0, 0),
                "forearm_L": (35, 0, 0),
                "thigh_R": (24, 0, 0),
                "shin_R": (-62, 0, 0),
                "thigh_L": (20, 0, 0),
                "shin_L": (-55, 0, 0),
            },
            0.75: {
                "root": {"rot": (72, 0, 0), "loc": (0, 0, 0.07)},
                "head": (-4, 0, 0),
                "upper_arm_R": (-10, 0, -60),
                "upper_arm_L": (-10, 0, 60),
                "forearm_R": (20, 0, 0),
                "forearm_L": (20, 0, 0),
                "thigh_R": (10, 0, 0),
                "shin_R": (-30, 0, 0),
                "thigh_L": (8, 0, 0),
                "shin_L": (-26, 0, 0),
            },
            1.00: {
                "root": {"rot": (88, 0, 0), "loc": (0, 0, 0.10)},
                "head": (0, -15, 0),
                "upper_arm_R": (-4, 0, -70),
                "upper_arm_L": (-4, 0, 65),
                "forearm_R": (10, 0, 0),
                "forearm_L": (12, 0, 0),
                "thigh_R": (2, 0, -6),
                "shin_R": (-6, 0, 0),
                "thigh_L": (2, 0, 6),
                "shin_L": (-4, 0, 0),
            },
            1.20: {
                "root": {"rot": (88, 0, 0), "loc": (0, 0, 0.10)},
                "head": (0, -15, 0),
                "upper_arm_R": (-4, 0, -70),
                "upper_arm_L": (-4, 0, 65),
                "forearm_R": (8, 0, 0),
                "forearm_L": (10, 0, 0),
                "thigh_R": (2, 0, -6),
                "shin_R": (-6, 0, 0),
                "thigh_L": (2, 0, 6),
                "shin_L": (-4, 0, 0),
            },
        },
    },
}
