"""Player drowning death clip (A16).

Keyframes, bone entries and rules are described in poses.py.
Sign rules:
- hips loc Y negative = lower body (sinking).
- spine/chest/head X negative = bend forward/droop; positive = tilt back.
- upper_arm_* X positive = arm forward/up; Z negative on R = out, positive on L = out.
- thigh_* X positive = leg forward; shin_* X negative = knee bends.
"""

CLIPS = {
    # Drowned in deep water: brief gasp/reach, then goes limp and slowly sinks.
    "death_drown": {
        "length": 2.0,
        "loop": False,
        "frames": {
            0.0: {
                "hips": {"rot": (-6, 0, 0), "loc": (0, 0, 0)},
                "spine": (-4, 0, 0),
                "chest": (-4, 0, 0),
                "neck": (0, 0, 0),
                "head": (4, 0, 0),
                "upper_arm_R": (36, 0, 28),
                "forearm_R": (42, 0, 0),
                "upper_arm_L": (36, 0, -28),
                "forearm_L": (42, 0, 0),
                "thigh_R": (16, 0, -4),
                "shin_R": (-24, 0, 0),
                "thigh_L": (12, 0, 4),
                "shin_L": (-20, 0, 0),
            },
            0.45: {
                # Last gasp: head tilts up/back, arms reach weakly outward
                "hips": {"rot": (4, 0, 2), "loc": (0, -0.14, 0)},
                "spine": (6, 0, 2),
                "chest": (8, 0, 2),
                "neck": (6, 0, 0),
                "head": (14, 0, 4),
                "upper_arm_R": (64, 0, -34),
                "forearm_R": (28, 0, 0),
                "upper_arm_L": (58, 0, 34),
                "forearm_L": (32, 0, 0),
                "thigh_R": (20, 0, -6),
                "shin_R": (-34, 0, 0),
                "thigh_L": (14, 0, 6),
                "shin_L": (-28, 0, 0),
            },
            1.05: {
                # Strength fades: head drops forward, arms go slack, body sinks
                "hips": {"rot": (-8, 0, -4), "loc": (0, -0.44, 0)},
                "spine": (-16, 0, -3),
                "chest": (-14, 0, -2),
                "neck": (-10, 0, 0),
                "head": (-18, 8, -4),
                "upper_arm_R": (18, 0, -22),
                "forearm_R": (16, 0, 0),
                "upper_arm_L": (16, 0, 22),
                "forearm_L": (18, 0, 0),
                "thigh_R": (14, 0, -5),
                "shin_R": (-28, 0, 0),
                "thigh_L": (10, 0, 5),
                "shin_L": (-24, 0, 0),
            },
            1.60: {
                # Sinking limp
                "hips": {"rot": (-12, 0, -6), "loc": (0, -0.68, 0)},
                "spine": (-22, 0, -4),
                "chest": (-18, 0, -3),
                "neck": (-12, 0, 0),
                "head": (-22, 12, -6),
                "upper_arm_R": (10, 0, -16),
                "forearm_R": (12, 0, 0),
                "upper_arm_L": (12, 0, 16),
                "forearm_L": (14, 0, 0),
                "thigh_R": (10, 0, -4),
                "shin_R": (-20, 0, 0),
                "thigh_L": (8, 0, 4),
                "shin_L": (-18, 0, 0),
            },
            2.00: {
                # Settled limp and still
                "hips": {"rot": (-12, 0, -6), "loc": (0, -0.75, 0)},
                "spine": (-24, 0, -4),
                "chest": (-18, 0, -3),
                "neck": (-12, 0, 0),
                "head": (-24, 14, -6),
                "upper_arm_R": (8, 0, -14),
                "forearm_R": (10, 0, 0),
                "upper_arm_L": (10, 0, 14),
                "forearm_L": (12, 0, 0),
                "thigh_R": (8, 0, -4),
                "shin_R": (-16, 0, 0),
                "thigh_L": (6, 0, 4),
                "shin_L": (-14, 0, 0),
            },
        },
    },
}
