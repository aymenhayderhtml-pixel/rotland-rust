"""Dog clips (V11-08): in place, no root motion. Angles are degrees in each bone's own axes.

A pose is {bone: (rx, ry, rz)}. The key "hips_loc" holds the hips' position offset (metres).
Loop clips are functions of phase (0..1). One-shot clips are key lists [(time, pose), ...]
that are linearly blended. The rest of the time they hold the last key (die stays down).
Rest pose is all zeros. Keyed at 60 fps (see FPS).
"""
import math

FPS = 60
TAU = 2.0 * math.pi
LEGS = ["FL", "FR", "BL", "BR"]


def _rot(x=0.0, y=0.0, z=0.0):
    return (x, y, z)


def _leg(leg, upper, lower=0.0, paw=0.0):
    return {f"upper_{leg}": _rot(upper), f"lower_{leg}": _rot(lower), f"paw_{leg}": _rot(paw)}


def _merge(*poses):
    out = {}
    for pose in poses:
        out.update(pose)
    return out


def _gait(phase_of, upper_amp, flex_amp):
    """Each leg swings from its own phase: forward swing lifts the paw (knee flexes)."""
    out = {}
    for leg in LEGS:
        ph = phase_of[leg]
        swing = upper_amp * math.sin(ph)
        flex = -flex_amp * max(0.0, math.cos(ph))
        out.update(_leg(leg, swing, flex, paw=-0.25 * swing))
    return out


def _walk(p):
    th = TAU * p
    phases = {"FL": th, "BR": th, "FR": th + math.pi, "BL": th + math.pi}  # diagonal pairs
    pose = _gait(phases, upper_amp=24.0, flex_amp=28.0)
    pose["hips_loc"] = (0.0, 0.0, 0.012 * math.cos(2 * th))
    pose["tail_1"] = _rot(-10.0)
    return pose


def _run(p):
    th = TAU * p
    # Gallop: the hind pair leads, the front pair follows a little later
    phases = {"BL": th, "BR": th, "FL": th - 0.45 * math.pi, "FR": th - 0.45 * math.pi}
    pose = _gait(phases, upper_amp=38.0, flex_amp=55.0)
    pose["spine"] = _rot(8.0 * math.sin(th))
    pose["chest"] = _rot(-6.0 * math.sin(th))
    # Bounce lifts the body only: the paws must not sink into the ground
    pose["hips_loc"] = (0.0, 0.0, 0.03 + 0.02 * math.cos(2 * th))
    pose["tail_1"] = _rot(-25.0)
    return pose


def _idle(p):
    th = TAU * p
    return {
        "chest": _rot(1.2 * math.sin(th)),
        "head": _rot(1.5 * math.sin(th + 1.0)),
        "tail_1": _rot(0.0, 0.0, 22.0 * math.sin(2 * th)),  # two wags per loop
        "tail_2": _rot(0.0, 0.0, 14.0 * math.sin(2 * th - 0.6)),
        "ear_L": _rot(3.0 * math.sin(2 * th)),
        "ear_R": _rot(3.0 * math.sin(2 * th + 0.5)),
    }


def _sit(p):
    th = TAU * p
    pose = _merge(
        {"hips": _rot(48.0), "hips_loc": (0.0, 0.0, -0.22)},
        {"neck": _rot(-28.0), "head": _rot(-20.0), "chest": _rot(1.0 * math.sin(th))},
        _leg("FL", -48.0), _leg("FR", -48.0),  # front legs stay straight under the chest
        _leg("BL", 40.0, -85.0), _leg("BR", 40.0, -85.0),  # haunches folded on the ground
        {"tail_1": _rot(0.0, 0.0, 10.0 * math.sin(2 * th))},
    )
    return pose


def _lie(p):
    th = TAU * p
    return _merge(
        {"hips": _rot(0.0), "hips_loc": (0.0, 0.0, -0.36)},
        {"chest": _rot(0.8 * math.sin(th)), "neck": _rot(-10.0), "head": _rot(-8.0)},
        # Legs folded under and forward so the paws rest on the ground (checked in CLIPCHECK)
        _leg("FL", 96.0, 0.0), _leg("FR", 96.0, 0.0),
        _leg("BL", 96.0, -170.0, paw=95.0), _leg("BR", 96.0, -170.0, paw=95.0),
        {"tail_1": _rot(0.0, 0.0, 6.0 * math.sin(th))},
    )


BARK_KEYS = [
    (0.00, {}),
    (0.10, {"jaw": _rot(-28.0), "head": _rot(14.0)}),
    (0.20, {"jaw": _rot(-4.0), "head": _rot(8.0)}),
    (0.30, {"jaw": _rot(-28.0), "head": _rot(14.0)}),
    (0.40, {"jaw": _rot(-4.0), "head": _rot(8.0)}),
    (0.50, {"jaw": _rot(-28.0), "head": _rot(14.0)}),
    (0.60, {}),
]

BITE_KEYS = [
    (0.00, {}),
    (0.15, _merge({"spine": _rot(-3.0), "chest": _rot(-6.0), "neck": _rot(-12.0),
                   "head": _rot(-6.0), "jaw": _rot(-30.0), "hips_loc": (0.0, 0.0, 0.03)},
                  _leg("FL", 15.0), _leg("FR", 15.0), _leg("BL", -12.0), _leg("BR", -12.0))),
    (0.28, {"spine": _rot(-3.0), "chest": _rot(-6.0), "neck": _rot(-12.0), "hips_loc": (0.0, 0.0, 0.03),
            "head": _rot(-6.0), "jaw": _rot(-30.0), **_leg("FL", 15.0), **_leg("FR", 15.0)}),
    (0.32, {"spine": _rot(-3.0), "chest": _rot(-6.0), "neck": _rot(-12.0), "head": _rot(-6.0)}),
    (0.50, {}),
]

DIE_KEYS = [
    (0.00, {}),
    (0.25, _merge({"hips": _rot(0.0, 20.0), "hips_loc": (0.0, 0.0, -0.06), "neck": _rot(-10.0)},
                  _leg("FL", 30.0), _leg("FR", 30.0), _leg("BL", 20.0), _leg("BR", 20.0))),
    (0.55, _merge({"hips": _rot(0.0, 62.0), "hips_loc": (0.0, 0.0, -0.18), "neck": _rot(-10.0),
                   "head": _rot(10.0)},
                  _leg("FL", 55.0, -30.0), _leg("FR", 55.0, -30.0),
                  _leg("BL", 50.0, -40.0), _leg("BR", 50.0, -40.0))),
    (0.85, _merge({"hips": _rot(0.0, 88.0), "hips_loc": (0.0, 0.0, -0.30), "neck": _rot(-10.0),
                   "head": _rot(10.0)},
                  _leg("FL", 70.0, -40.0), _leg("FR", 70.0, -40.0),
                  _leg("BL", 60.0, -40.0), _leg("BR", 60.0, -40.0))),
    (1.20, _merge({"hips": _rot(0.0, 88.0), "hips_loc": (0.0, 0.0, -0.30), "neck": _rot(-10.0),
                   "head": _rot(10.0)},
                  _leg("FL", 70.0, -40.0), _leg("FR", 70.0, -40.0),
                  _leg("BL", 60.0, -40.0), _leg("BR", 60.0, -40.0))),
]

# name -> (length in seconds, loop, phase function or None, key list or None)
CLIPS = {
    "idle": (2.0, True, _idle, None),
    "walk": (0.8, True, _walk, None),
    "run": (0.45, True, _run, None),
    "sit": (2.0, True, _sit, None),
    "lie": (2.0, True, _lie, None),
    "bark": (0.6, False, None, BARK_KEYS),
    "bite": (0.5, False, None, BITE_KEYS),
    "die": (1.2, False, None, DIE_KEYS),
}


def _blend(a, b, w):
    return tuple(x + (y - x) * w for x, y in zip(a, b))


def _pose_at_keys(keys, t):
    """Linear blend between the two keys around time t (holds the first and last key)."""
    if t <= keys[0][0]:
        return dict(keys[0][1])
    for (t0, p0), (t1, p1) in zip(keys, keys[1:]):
        if t <= t1:
            w = (t - t0) / (t1 - t0) if t1 > t0 else 1.0
            out = {}
            for bone in set(p0) | set(p1):
                a = p0.get(bone, _rot() if bone != "hips_loc" else (0.0, 0.0, 0.0))
                b = p1.get(bone, _rot() if bone != "hips_loc" else (0.0, 0.0, 0.0))
                out[bone] = _blend(a, b, w)
            return out
    return dict(keys[-1][1])


def pose_at(clip, t):
    """Pose of one clip at time t (seconds). Loops wrap by phase, so t = length equals t = 0."""
    length, loop, fn, keys = CLIPS[clip]
    if fn is not None:
        return fn((t / length) % 1.0 if loop else min(t / length, 1.0))
    return _pose_at_keys(keys, t)


def sample_times(clip):
    """Frame times (seconds) for keying: one per frame, the last one is the clip length."""
    length = CLIPS[clip][0]
    count = round(length * FPS)
    return [i / FPS for i in range(count + 1)]
