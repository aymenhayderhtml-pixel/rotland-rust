"""Clips for the wild animals (V12-01): in place, no root motion. Angles are degrees in each bone's own axes.

Same conventions as dog_poses.py: +X swings a leg forward and lifts the head, -X lowers the head,
Z turns the head or wags the tail sideways. Loop clips are periodic functions of phase (0..1), so
the last frame equals the first. One-shot clips are key lists, blended linearly, that hold their
last key (a die stays down). Keyed at 60 fps.

CLIPS[style] = {clip: (length s, loop, phase function or None, key list or None)}.
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
    """Each leg swings from its own phase: a forward swing lifts the paw (the knee flexes)."""
    out = {}
    for leg in LEGS:
        ph = phase_of[leg]
        swing = upper_amp * math.sin(ph)
        flex = -flex_amp * max(0.0, math.cos(ph))
        out.update(_leg(leg, swing, flex, paw=-0.25 * swing))
    return out


def _diagonal_walk(p, upper_amp, flex_amp):
    th = TAU * p
    phases = {"FL": th, "BR": th, "FR": th + math.pi, "BL": th + math.pi}
    pose = _gait(phases, upper_amp, flex_amp)
    pose["hips_loc"] = (0.0, 0.0, 0.01 * math.cos(2 * th))
    pose["tail_1"] = _rot(-8.0)
    return pose


def _bound(p, upper_amp, flex_amp, lift):
    """Gallop: the hind pair leads, the front pair follows a little later."""
    th = TAU * p
    phases = {"BL": th, "BR": th, "FL": th - 0.45 * math.pi, "FR": th - 0.45 * math.pi}
    pose = _gait(phases, upper_amp, flex_amp)
    pose["spine"] = _rot(7.0 * math.sin(th))
    pose["chest"] = _rot(-5.0 * math.sin(th))
    pose["hips_loc"] = (0.0, 0.0, lift + 0.4 * lift * math.cos(2 * th))  # bounce; paws stay near the floor
    pose["tail_1"] = _rot(-20.0)
    return pose


def _breathe(th, amp=1.0):
    return {"chest": _rot(amp * math.sin(th))}


def _hop(p, hind_amp, hind_flex, front_amp, lift):
    """One hop per loop. Airborne in the first half (hind legs tuck), landing and pushing in the second."""
    th = TAU * p
    air = max(0.0, math.sin(th))
    pose = _merge(
        _leg("BL", -hind_amp * math.cos(th), -hind_flex * air),
        _leg("BR", -hind_amp * math.cos(th), -hind_flex * air),
        _leg("FL", -front_amp * math.cos(th - 0.3), -0.4 * front_amp * air),
        _leg("FR", -front_amp * math.cos(th - 0.3), -0.4 * front_amp * air),
    )
    pose["hips_loc"] = (0.0, 0.0, lift * air)
    pose["hips"] = _rot(4.0 * math.sin(th))  # the body pitches with the hop
    return pose


# ---- deer (doe and buck share these) ----

def _deer_idle(p):
    th = TAU * p
    return _merge(
        _breathe(th, 1.0),
        {"head": _rot(2.0 * math.sin(th + 1.0)), "tail_1": _rot(0.0, 0.0, 14.0 * math.sin(2 * th)),
         "ear_L": _rot(4.0 * math.sin(2 * th)), "ear_R": _rot(4.0 * math.sin(2 * th + 0.5))},
    )


def _deer_graze(p):
    """Head down to browse: neck and head folded forward, with a slow chew."""
    th = TAU * p
    return _merge(
        {"neck": _rot(-125.0 + 3.0 * math.sin(2 * th)), "head": _rot(65.0 + 5.0 * math.sin(2 * th + 0.4)),
         "jaw": _rot(-8.0 * max(0.0, math.sin(4 * th))), "chest": _rot(-4.0),
         "tail_1": _rot(0.0, 0.0, 6.0 * math.sin(th))},
        _leg("FL", 6.0), _leg("FR", 6.0),
    )


def _deer_walk(p):
    return _diagonal_walk(p, upper_amp=20.0, flex_amp=26.0)


def _deer_run(p):
    return _bound(p, upper_amp=38.0, flex_amp=56.0, lift=0.04)


DEER_ALERT_KEYS = [
    (0.00, {}),
    (0.30, {"neck": _rot(25.0), "head": _rot(35.0), "ear_L": _rot(-12.0), "ear_R": _rot(-12.0)}),
    (1.10, {"neck": _rot(25.0), "head": _rot(35.0), "ear_L": _rot(-12.0), "ear_R": _rot(-12.0)}),
    (1.50, {}),
]


def _die_keys(hips_drop, hips_turn, legs_pose):
    """Falls onto its side and stays there. It rolls first and drops after, so the legs clear the floor."""
    steps = [  # time, share of the roll, share of the drop, share of the limp legs
        (0.30, 0.34, 0.0, 0.3),
        (0.60, 0.68, 0.3, 0.7),
        (0.90, 1.00, 1.0, 1.0),
        (1.20, 1.00, 1.0, 1.0),
    ]
    keys = [(0.00, {})]
    for t, roll, drop, limp in steps:
        keys.append((t, _merge({"hips": _rot(0.0, hips_turn * roll),
                                "hips_loc": (0.0, 0.0, hips_drop * drop),
                                "neck": _rot(-10.0 * limp), "head": _rot(10.0 * limp)},
                               legs_pose(limp))))
    return keys


def _legs_limp(amount):
    """Legs fold and go slack as the animal drops."""
    return _merge(_leg("FL", 30.0 * amount, -20.0 * amount), _leg("FR", 30.0 * amount, -20.0 * amount),
                  _leg("BL", 20.0 * amount, -30.0 * amount), _leg("BR", 20.0 * amount, -30.0 * amount))


DEER_DIE_KEYS = _die_keys(hips_drop=-1.10, hips_turn=88.0, legs_pose=_legs_limp)


DEER = {
    "idle": (2.0, True, _deer_idle, None),
    "graze": (3.0, True, _deer_graze, None),
    "walk": (1.0, True, _deer_walk, None),
    "run": (0.6, True, _deer_run, None),
    "alert": (1.5, False, None, DEER_ALERT_KEYS),
    "die": (1.2, False, None, DEER_DIE_KEYS),
}


# ---- rabbit ----

def _rabbit_idle(p):
    th = TAU * p
    return _merge(
        _breathe(th, 0.8),
        {"head": _rot(1.5 * math.sin(3 * th)), "ear_L": _rot(0.0, 0.0, 5.0 * math.sin(2 * th)),
         "ear_R": _rot(0.0, 0.0, -5.0 * math.sin(2 * th + 0.6))},
    )


def _rabbit_graze(p):
    th = TAU * p
    return _merge(
        # The ears counter-turn the head's pitch so they stay upright (the ear bones turn the other way)
        {"neck": _rot(-93.0 + 2.0 * math.sin(th)), "head": _rot(24.0 + 4.0 * math.sin(2 * th)),
         "ear_L": _rot(-60.0), "ear_R": _rot(-60.0),
         "jaw": _rot(-6.0 * max(0.0, math.sin(4 * th)))},
    )


def _rabbit_walk(p):
    return _hop(p, hind_amp=20.0, hind_flex=25.0, front_amp=18.0, lift=0.02)


def _rabbit_run(p):
    return _hop(p, hind_amp=42.0, hind_flex=60.0, front_amp=30.0, lift=0.07)


RABBIT_ALERT_KEYS = [
    (0.00, {}),
    (0.25, {"neck": _rot(20.0), "head": _rot(25.0), "ear_L": _rot(0.0, 0.0, 6.0), "ear_R": _rot(0.0, 0.0, -6.0)}),
    (0.90, {"neck": _rot(20.0), "head": _rot(25.0), "ear_L": _rot(0.0, 0.0, 6.0), "ear_R": _rot(0.0, 0.0, -6.0)}),
    (1.20, {}),
]


def _rabbit_legs_limp(amount):
    return _merge(_leg("FL", 25.0 * amount), _leg("FR", 25.0 * amount),
                  _leg("BL", 30.0 * amount, -20.0 * amount), _leg("BR", 30.0 * amount, -20.0 * amount))


RABBIT_DIE_KEYS = _die_keys(hips_drop=-0.12, hips_turn=88.0, legs_pose=_rabbit_legs_limp)

RABBIT = {
    "idle": (2.0, True, _rabbit_idle, None),
    "graze": (2.0, True, _rabbit_graze, None),
    "walk": (0.8, True, _rabbit_walk, None),
    "run": (0.4, True, _rabbit_run, None),
    "alert": (1.2, False, None, RABBIT_ALERT_KEYS),
    "die": (1.0, False, None, RABBIT_DIE_KEYS),
}


# ---- boar ----

def _boar_idle(p):
    th = TAU * p
    return _merge(
        _breathe(th, 1.0),
        {"head": _rot(2.0 * math.sin(th + 0.8)), "tail_1": _rot(0.0, 0.0, 16.0 * math.sin(2 * th))},
    )


def _boar_graze(p):
    """Rooting: the snout goes down to the ground and shoves side to side."""
    th = TAU * p
    return _merge(
        {"neck": _rot(-12.0 + 2.0 * math.sin(th)), "head": _rot(-35.0 + 4.0 * math.sin(2 * th)),
         "jaw": _rot(-6.0 * max(0.0, math.sin(2 * th))),
         "tail_1": _rot(0.0, 0.0, 10.0 * math.sin(th))},
    )


def _boar_walk(p):
    return _diagonal_walk(p, upper_amp=18.0, flex_amp=22.0)


def _boar_run(p):
    return _bound(p, upper_amp=30.0, flex_amp=44.0, lift=0.02)


def _boar_charge(p):
    """Head low, body lowered, legs driving: the same gallop, faster and heavier."""
    th = TAU * p
    pose = _bound(p, upper_amp=36.0, flex_amp=50.0, lift=0.0)
    pose["neck"] = _rot(-12.0)
    pose["head"] = _rot(-12.0 + 3.0 * math.sin(2 * th))
    pose["hips_loc"] = (0.0, 0.0, -0.03 + 0.01 * math.cos(2 * th))
    return pose


BOAR_ALERT_KEYS = [
    (0.00, {}),
    (0.30, {"neck": _rot(30.0), "head": _rot(38.0), "ear_L": _rot(-20.0), "ear_R": _rot(-20.0)}),
    (1.20, {"neck": _rot(30.0), "head": _rot(38.0), "ear_L": _rot(-20.0), "ear_R": _rot(-20.0)}),
    (1.50, {}),
]

BOAR_ATTACK_KEYS = [
    (0.00, {}),
    (0.15, {"head": _rot(0.0, 0.0, 18.0), "jaw": _rot(-12.0), "neck": _rot(-6.0)}),
    (0.30, {"head": _rot(0.0, 0.0, -18.0), "jaw": _rot(-22.0), "neck": _rot(-6.0)}),
    (0.45, {"head": _rot(0.0, 0.0, 8.0), "jaw": _rot(-8.0)}),
    (0.60, {}),
]

BOAR_DIE_KEYS = _die_keys(hips_drop=-0.72, hips_turn=88.0, legs_pose=_legs_limp)

BOAR = {
    "idle": (2.0, True, _boar_idle, None),
    "graze": (3.0, True, _boar_graze, None),
    "walk": (1.0, True, _boar_walk, None),
    "run": (0.6, True, _boar_run, None),
    "charge": (0.7, True, _boar_charge, None),
    "alert": (1.5, False, None, BOAR_ALERT_KEYS),
    "attack": (0.6, False, None, BOAR_ATTACK_KEYS),
    "die": (1.2, False, None, BOAR_DIE_KEYS),
}

CLIPS = {"deer": DEER, "rabbit": RABBIT, "boar": BOAR}


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
                zero = (0.0, 0.0, 0.0)
                out[bone] = _blend(p0.get(bone, zero), p1.get(bone, zero), w)
            return out
    return dict(keys[-1][1])


def pose_at(clips, clip, t):
    """Pose of one clip at time t (seconds). Loops wrap by phase, so t = length equals t = 0."""
    length, loop, fn, keys = clips[clip]
    if fn is not None:
        return fn((t / length) % 1.0 if loop else min(t / length, 1.0))
    return _pose_at_keys(keys, t)


def sample_times(clips, clip):
    """Frame times (seconds) for keying: one per frame, the last one is the clip length."""
    length = clips[clip][0]
    count = round(length * FPS)
    return [i / FPS for i in range(count + 1)]
