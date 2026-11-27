"""R41-04 crow models (crow_perched, crow_peck, crow_fly_f0..f3) in category 'animals'."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import common  # noqa: E402

CROW_BUDGET = 400


def _crow_body(pitch_rad, loc=(0.0, 0.0, 0.0), legs=True):
    """Core crow body (~0.45 m long with tail & beak), facing +Y."""
    parts = []
    # Torso (ellipsoid)
    torso = common.make_icosphere(subdiv=1, radius=(0.068, 0.135, 0.072), color="gun_black")
    parts.append(torso)
    # Head + beak
    head = common.make_icosphere(subdiv=1, radius=(0.044, 0.052, 0.046), loc=(0.0, 0.13, 0.055), color="gun_black")
    parts.append(head)
    beak = common.make_frustum(0.018, 0.002, 0.0, 0.068, segments=5, color="canvas_khaki")
    common.transform(beak, rot=(-math.pi * 0.5, 0.0, 0.0), loc=(0.0, 0.165, 0.052))
    parts.append(beak)
    # Tail fan
    tail = common.make_box((0.08, 0.15, 0.018), color="tire_black")
    common.transform(tail, rot=(math.radians(10.0), 0.0, 0.0), loc=(0.0, -0.17, -0.015))
    parts.append(tail)

    body = common.combine(parts)
    common.transform(body, rot=(pitch_rad, 0.0, 0.0), loc=loc)

    out = [body]
    if legs:
        # Folded wings on sides for perched/peck
        for sx in (-1.0, 1.0):
            wing = common.make_box((0.024, 0.19, 0.068), color="glass_dark")
            common.transform(wing, rot=(pitch_rad - 0.1, 0.0, sx * 0.08),
                             loc=(sx * 0.068, loc[1] - 0.02, loc[2] + 0.005))
            out.append(wing)
        # Two thin legs down to z=0
        for sx in (-1.0, 1.0):
            leg = common.make_frustum(0.008, 0.006, 0.0, loc[2] - 0.03, segments=4,
                                      center=(sx * 0.032, 0.0), color="pants_dark")
            foot = common.make_box((0.026, 0.048, 0.010), loc=(sx * 0.032, 0.012, 0.005), color="pants_dark")
            out.extend([leg, foot])
    return out


def _build_perched(model):
    for bm in _crow_body(pitch_rad=math.radians(24.0), loc=(0.0, 0.0, 0.155), legs=True):
        model.add(bm)


def _build_peck(model):
    # Pitched forward/down so beak reaches near ground (z ~ 0.02)
    for bm in _crow_body(pitch_rad=math.radians(-42.0), loc=(0.0, 0.02, 0.145), legs=True):
        model.add(bm)


def _build_fly_frame(wing_deg):
    """Flying crow with 0.90 m wingspan, pivot at body centre (0, 0, 0)."""
    def build(model):
        for bm in _crow_body(pitch_rad=0.0, loc=(0.0, 0.0, 0.0), legs=False):
            model.add(bm)
        # Tucked legs
        for sx in (-1.0, 1.0):
            leg = common.make_box((0.012, 0.065, 0.012), loc=(sx * 0.026, -0.08, -0.055), color="pants_dark")
            model.add(leg)
        # Left & right wings (0.45 m each -> 0.90 m span)
        ang = math.radians(wing_deg)
        for sx in (-1.0, 1.0):
            inner = common.make_box((0.23, 0.14, 0.016), color="glass_dark")
            common.transform(inner, loc=(sx * 0.14, 0.02, 0.0))
            common.transform(inner, rot=(0.0, -sx * ang, 0.0))
            outer = common.make_box((0.21, 0.10, 0.012), color="gun_black")
            # Outer tip bends slightly further
            tip_x = sx * (0.25 * math.cos(ang) + 0.10 * math.cos(ang * 1.25))
            tip_z = 0.25 * math.sin(ang) + 0.10 * math.sin(ang * 1.25)
            common.transform(outer, rot=(0.0, -sx * ang * 1.25, -sx * 0.12), loc=(tip_x, -0.01, tip_z))
            model.add(inner)
            model.add(outer)
    return build


def build_all():
    """Exports crow_perched, crow_peck, and crow_fly_f0..f3 into animals/."""
    specs = [
        ("crow_perched", _build_perched),
        ("crow_peck", _build_peck),
        ("crow_fly_f0", _build_fly_frame(38.0)),   # wings up
        ("crow_fly_f1", _build_fly_frame(6.0)),    # mid downstroke
        ("crow_fly_f2", _build_fly_frame(-34.0)),  # wings down
        ("crow_fly_f3", _build_fly_frame(-6.0)),   # mid upstroke
    ]
    return [
        (name, common.run_model(name, "animals", CROW_BUDGET, fn, grip=(0.0, 0.0, 0.0)))
        for name, fn in specs
    ]
