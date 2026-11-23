"""Directional lower-body movement and turn-in-place clips (A04).

Legs and hips only (hips, thigh_*, shin_*, foot_*, toe_*). All clips loop.
Phase rule: every walk-family (1.0 s) and run-family (0.7 s) clip matches the
contact/swing phase of walk and run in poses_locomotion.py:
  0.00 cycle: right foot contact, left foot trailing/push-off
  0.25 cycle: right stance, left swing
  0.50 cycle: left foot contact, right foot trailing/push-off
  0.75 cycle: left stance, right swing
Right-side clips (_r) are true mirrors of left-side clips (_l) shifted by half
a cycle so right contact stays at time 0.0.
"""


def _mirror_bone(name):
    if name.endswith("_L"):
        return name[:-2] + "_R"
    if name.endswith("_R"):
        return name[:-2] + "_L"
    return name


def _mirror_entry(entry):
    if isinstance(entry, dict):
        rx, ry, rz = entry["rot"]
        dx, dy, dz = entry["loc"]
        return {"rot": (rx, -ry, -rz), "loc": (-dx, dy, dz)}
    rx, ry, rz = entry
    return (rx, -ry, -rz)


def _mirror_frame(frame):
    return {_mirror_bone(bone): _mirror_entry(val) for bone, val in frame.items()}


def _mirror_clip(clip, shift_half=True):
    length = clip["length"]
    half = length * 0.5
    out_frames = {}
    for t, pose in clip["frames"].items():
        if t == length:
            continue
        new_t = round((t + half) % length, 4) if shift_half else t
        if new_t >= length:
            new_t = 0.0
        out_frames[new_t] = _mirror_frame(pose)
    out_frames = dict(sorted(out_frames.items()))
    out_frames[length] = out_frames[0.0]
    return {"length": length, "loop": clip["loop"], "frames": out_frames}


_WALK_BACK = {
    "length": 1.0,
    "loop": True,
    "frames": {
        0.0: {
            "hips": {"rot": (3, 3, 0), "loc": (0, -0.02, 0)},
            "thigh_R": (-20, 0, 1),
            "shin_R": (-10, 0, 0),
            "foot_R": (-12, 0, 0),
            "toe_R": (8, 0, 0),
            "thigh_L": (18, 0, -1),
            "shin_L": (-14, 0, 0),
            "foot_L": (10, 0, 0),
            "toe_L": (4, 0, 0),
        },
        0.25: {
            "hips": {"rot": (2, 0, -2), "loc": (0, 0.015, 0)},
            "thigh_R": (-2, 0, 1),
            "shin_R": (-6, 0, 0),
            "foot_R": (0, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (-4, 0, -1),
            "shin_L": (-44, 0, 0),
            "foot_L": (-10, 0, 0),
            "toe_L": (0, 0, 0),
        },
        0.5: {
            "hips": {"rot": (3, -3, 0), "loc": (0, -0.02, 0)},
            "thigh_R": (18, 0, 1),
            "shin_R": (-14, 0, 0),
            "foot_R": (10, 0, 0),
            "toe_R": (4, 0, 0),
            "thigh_L": (-20, 0, -1),
            "shin_L": (-10, 0, 0),
            "foot_L": (-12, 0, 0),
            "toe_L": (8, 0, 0),
        },
        0.75: {
            "hips": {"rot": (2, 0, 2), "loc": (0, 0.015, 0)},
            "thigh_R": (-4, 0, 1),
            "shin_R": (-44, 0, 0),
            "foot_R": (-10, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (-2, 0, -1),
            "shin_L": (-6, 0, 0),
            "foot_L": (0, 0, 0),
            "toe_L": (0, 0, 0),
        },
        1.0: {
            "hips": {"rot": (3, 3, 0), "loc": (0, -0.02, 0)},
            "thigh_R": (-20, 0, 1),
            "shin_R": (-10, 0, 0),
            "foot_R": (-12, 0, 0),
            "toe_R": (8, 0, 0),
            "thigh_L": (18, 0, -1),
            "shin_L": (-14, 0, 0),
            "foot_L": (10, 0, 0),
            "toe_L": (4, 0, 0),
        },
    },
}

_RUN_BACK = {
    "length": 0.7,
    "loop": True,
    "frames": {
        0.0: {
            "hips": {"rot": (4, 4, 0), "loc": (0, -0.025, 0)},
            "thigh_R": (-28, 0, 2),
            "shin_R": (-14, 0, 0),
            "foot_R": (-18, 0, 0),
            "toe_R": (12, 0, 0),
            "thigh_L": (24, 0, -2),
            "shin_L": (-48, 0, 0),
            "foot_L": (12, 0, 0),
            "toe_L": (4, 0, 0),
        },
        0.175: {
            "hips": {"rot": (4, 0, -2), "loc": (0, 0.02, 0)},
            "thigh_R": (-4, 0, 2),
            "shin_R": (-26, 0, 0),
            "foot_R": (-4, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (-14, 0, -2),
            "shin_L": (-76, 0, 0),
            "foot_L": (-16, 0, 0),
            "toe_L": (0, 0, 0),
        },
        0.35: {
            "hips": {"rot": (4, -4, 0), "loc": (0, -0.025, 0)},
            "thigh_R": (24, 0, 2),
            "shin_R": (-48, 0, 0),
            "foot_R": (12, 0, 0),
            "toe_R": (4, 0, 0),
            "thigh_L": (-28, 0, -2),
            "shin_L": (-14, 0, 0),
            "foot_L": (-18, 0, 0),
            "toe_L": (12, 0, 0),
        },
        0.525: {
            "hips": {"rot": (4, 0, 2), "loc": (0, 0.02, 0)},
            "thigh_R": (-14, 0, 2),
            "shin_R": (-76, 0, 0),
            "foot_R": (-16, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (-4, 0, -2),
            "shin_L": (-26, 0, 0),
            "foot_L": (-4, 0, 0),
            "toe_L": (0, 0, 0),
        },
        0.7: {
            "hips": {"rot": (4, 4, 0), "loc": (0, -0.025, 0)},
            "thigh_R": (-28, 0, 2),
            "shin_R": (-14, 0, 0),
            "foot_R": (-18, 0, 0),
            "toe_R": (12, 0, 0),
            "thigh_L": (24, 0, -2),
            "shin_L": (-48, 0, 0),
            "foot_L": (12, 0, 0),
            "toe_L": (4, 0, 0),
        },
    },
}

_STRAFE_L = {
    "length": 1.0,
    "loop": True,
    "frames": {
        0.0: {
            "hips": {"rot": (2, 3, -2), "loc": (0.015, -0.02, 0)},
            "thigh_R": (8, 2, 4),
            "shin_R": (-14, 0, 0),
            "foot_R": (4, 0, -4),
            "toe_R": (0, 0, 0),
            "thigh_L": (10, -2, -10),
            "shin_L": (-18, 0, 0),
            "foot_L": (-6, 0, 8),
            "toe_L": (0, 0, 0),
        },
        0.25: {
            "hips": {"rot": (1, 0, 1), "loc": (-0.005, 0.015, 0)},
            "thigh_R": (4, 0, 3),
            "shin_R": (-8, 0, 0),
            "foot_R": (0, 0, -2),
            "toe_R": (0, 0, 0),
            "thigh_L": (16, -4, -22),
            "shin_L": (-42, 0, 0),
            "foot_L": (-10, 0, 12),
            "toe_L": (0, 0, 0),
        },
        0.5: {
            "hips": {"rot": (2, -3, 3), "loc": (-0.02, -0.025, 0)},
            "thigh_R": (8, 4, 14),
            "shin_R": (-16, 0, 0),
            "foot_R": (-8, 0, -10),
            "toe_R": (-4, 0, 0),
            "thigh_L": (10, -4, -18),
            "shin_L": (-14, 0, 0),
            "foot_L": (4, 0, 12),
            "toe_L": (2, 0, 0),
        },
        0.75: {
            "hips": {"rot": (1, 0, -1), "loc": (0.005, 0.015, 0)},
            "thigh_R": (14, 2, 3),
            "shin_R": (-38, 0, 0),
            "foot_R": (-10, 0, -4),
            "toe_R": (0, 0, 0),
            "thigh_L": (4, -2, -6),
            "shin_L": (-8, 0, 0),
            "foot_L": (0, 0, 4),
            "toe_L": (0, 0, 0),
        },
        1.0: {
            "hips": {"rot": (2, 3, -2), "loc": (0.015, -0.02, 0)},
            "thigh_R": (8, 2, 4),
            "shin_R": (-14, 0, 0),
            "foot_R": (4, 0, -4),
            "toe_R": (0, 0, 0),
            "thigh_L": (10, -2, -10),
            "shin_L": (-18, 0, 0),
            "foot_L": (-6, 0, 8),
            "toe_L": (0, 0, 0),
        },
    },
}

_RUN_STRAFE_L = {
    "length": 0.7,
    "loop": True,
    "frames": {
        0.0: {
            "hips": {"rot": (-2, 4, -3), "loc": (0.02, -0.03, 0)},
            "thigh_R": (14, 2, 5),
            "shin_R": (-24, 0, 0),
            "foot_R": (4, 0, -5),
            "toe_R": (0, 0, 0),
            "thigh_L": (16, -4, -14),
            "shin_L": (-30, 0, 0),
            "foot_L": (-10, 0, 10),
            "toe_L": (-4, 0, 0),
        },
        0.175: {
            "hips": {"rot": (-2, 0, 2), "loc": (-0.01, 0.022, 0)},
            "thigh_R": (8, 0, 4),
            "shin_R": (-18, 0, 0),
            "foot_R": (-4, 0, -4),
            "toe_R": (0, 0, 0),
            "thigh_L": (24, -6, -28),
            "shin_L": (-64, 0, 0),
            "foot_L": (-14, 0, 16),
            "toe_L": (0, 0, 0),
        },
        0.35: {
            "hips": {"rot": (-2, -4, 4), "loc": (-0.025, -0.03, 0)},
            "thigh_R": (12, 6, 18),
            "shin_R": (-26, 0, 0),
            "foot_R": (-14, 0, -14),
            "toe_R": (-8, 0, 0),
            "thigh_L": (16, -6, -24),
            "shin_L": (-22, 0, 0),
            "foot_L": (6, 0, 16),
            "toe_L": (4, 0, 0),
        },
        0.525: {
            "hips": {"rot": (-2, 0, -2), "loc": (0.01, 0.022, 0)},
            "thigh_R": (22, 4, 4),
            "shin_R": (-60, 0, 0),
            "foot_R": (-14, 0, -4),
            "toe_R": (0, 0, 0),
            "thigh_L": (8, -2, -8),
            "shin_L": (-18, 0, 0),
            "foot_L": (-4, 0, 6),
            "toe_L": (0, 0, 0),
        },
        0.7: {
            "hips": {"rot": (-2, 4, -3), "loc": (0.02, -0.03, 0)},
            "thigh_R": (14, 2, 5),
            "shin_R": (-24, 0, 0),
            "foot_R": (4, 0, -5),
            "toe_R": (0, 0, 0),
            "thigh_L": (16, -4, -14),
            "shin_L": (-30, 0, 0),
            "foot_L": (-10, 0, 10),
            "toe_L": (-4, 0, 0),
        },
    },
}

_TURN_L = {
    "length": 0.6,
    "loop": True,
    "frames": {
        0.0: {
            "hips": {"rot": (1, -5, -1), "loc": (0.008, -0.01, 0)},
            "thigh_R": (5, 4, 3),
            "shin_R": (-10, 0, 0),
            "foot_R": (2, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (2, -4, -4),
            "shin_L": (-12, 0, 0),
            "foot_L": (-4, 0, 0),
            "toe_L": (0, 0, 0),
        },
        0.15: {
            "hips": {"rot": (1, 0, 1), "loc": (-0.004, 0.008, 0)},
            "thigh_R": (3, 2, 2),
            "shin_R": (-8, 0, 0),
            "foot_R": (0, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (12, -10, -6),
            "shin_L": (-34, 0, 0),
            "foot_L": (-10, 0, 0),
            "toe_L": (0, 0, 0),
        },
        0.3: {
            "hips": {"rot": (1, 5, 1), "loc": (-0.008, -0.01, 0)},
            "thigh_R": (2, 6, 4),
            "shin_R": (-12, 0, 0),
            "foot_R": (-4, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (5, -6, -4),
            "shin_L": (-10, 0, 0),
            "foot_L": (2, 0, 0),
            "toe_L": (0, 0, 0),
        },
        0.45: {
            "hips": {"rot": (1, 0, -1), "loc": (0.004, 0.008, 0)},
            "thigh_R": (12, 10, 5),
            "shin_R": (-32, 0, 0),
            "foot_R": (-10, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (3, -2, -3),
            "shin_L": (-8, 0, 0),
            "foot_L": (0, 0, 0),
            "toe_L": (0, 0, 0),
        },
        0.6: {
            "hips": {"rot": (1, -5, -1), "loc": (0.008, -0.01, 0)},
            "thigh_R": (5, 4, 3),
            "shin_R": (-10, 0, 0),
            "foot_R": (2, 0, 0),
            "toe_R": (0, 0, 0),
            "thigh_L": (2, -4, -4),
            "shin_L": (-12, 0, 0),
            "foot_L": (-4, 0, 0),
            "toe_L": (0, 0, 0),
        },
    },
}

def _spread_legs(clip):
    """Thigh and foot Z were keyed inward, which crossed the legs; flip them so side steps open outward."""
    for frame in clip["frames"].values():
        for bone in ("thigh_L", "thigh_R", "foot_L", "foot_R"):
            rx, ry, rz = frame[bone]
            frame[bone] = (rx, ry, -rz)
    return clip


_spread_legs(_STRAFE_L)
_spread_legs(_RUN_STRAFE_L)

CLIPS = {
    "move_walk_back": _WALK_BACK,
    "move_run_back": _RUN_BACK,
    "move_strafe_l": _STRAFE_L,
    "move_strafe_r": _mirror_clip(_STRAFE_L),
    "move_run_strafe_l": _RUN_STRAFE_L,
    "move_run_strafe_r": _mirror_clip(_RUN_STRAFE_L),
    "move_turn_l": _TURN_L,
    "move_turn_r": _mirror_clip(_TURN_L),
}
