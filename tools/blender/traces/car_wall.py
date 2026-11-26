"""car_wall_stack: stacked car-wreck street barricade (10-12 m wide, ~3.5 m tall, 4 m deep) with ladder ramp.

Reuses jam_cars/jam_car_shell for the wrecked shells, joined by planks, sheet metal, and a climbable
ladder-ramp on the +X side (<= 40 deg slope). Budget: 8000 tris.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import common  # noqa: E402
import jam_car_shell  # noqa: E402
import jam_cars  # noqa: E402

BUDGET = 8000  # new: DECISIONS 2026-11-26
WALL_VIEW = (0.45, 1.0, 0.42)  # new: DECISIONS 2026-11-26 (+Y front 3/4 view)


def _simple_wheels(spec):
    """Low-poly 10-sided wheels so 3 stacked wrecks stay well inside the 8000 tri budget."""
    x, y = spec["W"] - 0.04, spec["wheel_y"]
    out = []
    for i, (sx, sy) in enumerate([(-x, y), (x, y), (-x, -y), (x, -y)]):
        flat = 0.72 if i in spec["flat"] else 1.0  # new: DECISIONS 2026-11-26
        tyre = common.make_frustum(0.34, 0.34, -0.12, 0.12, segments=10, color="tire_black")
        hub = common.make_frustum(0.20, 0.20, -0.13, 0.13, segments=8, color="metal")
        wheel = common.combine([tyre, hub])
        common.transform(wheel, rot=(0.0, math.pi / 2.0, 0.0), scale=(1.0, 1.0, flat),
                         loc=(sx, sy, 0.34 * flat))
        out.append(wheel)
    return out


def _wreck_bm(key, body_color, loc, rot, burnt=False, strip_doors=False):
    """Builds one car wreck shell using jam_car_shell and places it at (loc, rot)."""
    spec = dict(jam_cars.CARS[key])
    if strip_doors:
        # Solid side panels without door cutouts save ~1,200 tris per buried/stacked wreck
        spec["doors"] = []
    parts = jam_car_shell.lower_parts(spec, body_color, burnt) + jam_car_shell.greenhouse_parts(spec, body_color, burnt)
    parts += _simple_wheels(spec)
    bm = common.combine(parts)
    common.transform(bm, rot=rot, loc=loc)
    return bm


def _beam_box(p0, p1, w, h, color):
    """Oriented plank/rail connecting 3D points p0 -> p1."""
    dx, dy, dz = p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]
    length = math.sqrt(dx * dx + dy * dy + dz * dz)
    yaw = math.atan2(dy, dx)
    horiz = math.hypot(dx, dy)
    pitch = -math.atan2(dz, horiz)
    bm = common.make_box((length, w, h), color=color)
    common.transform(bm, rot=(0.0, pitch, yaw),
                     loc=((p0[0] + p1[0]) / 2, (p0[1] + p1[1]) / 2, (p0[2] + p1[2]) / 2))
    return bm


def _ramp_wedge_bm(x_bot, x_top, y0, y1, height):
    """Convex wedge rising from (x_bot, z=0) up to (x_top, z=height)."""
    bm = common.new_bm()
    b0 = bm.verts.new((x_bot, y0, 0.0))
    b1 = bm.verts.new((x_bot, y1, 0.0))
    b2 = bm.verts.new((x_top, y1, 0.0))
    b3 = bm.verts.new((x_top, y0, 0.0))
    t0 = bm.verts.new((x_top, y0, height))
    t1 = bm.verts.new((x_top, y1, height))
    bm.faces.new([b3, b2, b1, b0])
    bm.faces.new([b0, b1, t1, t0])
    bm.faces.new([b2, b3, t0, t1])
    bm.faces.new([b0, t0, b3])
    bm.faces.new([b1, b2, t1])
    common.paint(bm, "white")
    return bm


def _build_car_wall(model):
    """11 m wide x 3.5 m tall x 3.6 m deep stacked car barricade with climbable ladder-ramp on +X."""
    # Two ground wrecks spanning X across the street (car length runs along X via rot_z = +-pi/2)
    model.add(_wreck_bm("pickup", "car_red", loc=(-2.45, -0.25, 0.0), rot=(0.0, 0.0, math.pi / 2 + 0.07)))
    model.add(_wreck_bm("sedan", "car_blue", loc=(2.20, -0.35, 0.0), rot=(0.0, 0.0, -math.pi / 2 - 0.06),
                        strip_doors=True))

    # Third tilted wreck wedged on top of the two lower cars (reaches ~3.45 m)
    model.add(_wreck_bm("hatch", "rust", loc=(-0.20, -0.30, 1.42), rot=(0.14, -0.22, math.pi / 2 - 0.18),
                        burnt=True, strip_doors=True))

    # Sheet metal panels & vertical posts closing the -X end (x = -5.5..-4.6) and gaps
    wing_sheets = [
        ((-5.05, 0.25, 1.15), (1.10, 0.06, 2.20), (0.04, 0.0, 0.08), "rust_dark"),
        ((-4.50, 0.45, 1.25), (1.35, 0.06, 2.20), (0.04, 0.0, -0.06), "metal"),
        ((-4.85, -0.75, 1.05), (1.20, 0.06, 1.95), (-0.05, 0.0, 0.08), "rust_dark"),
        ((-1.15, 0.68, 1.45), (1.60, 0.05, 1.15), (0.06, 0.10, 0.0), "metal"),
        ((1.35, 0.62, 1.30), (1.50, 0.05, 1.05), (-0.05, -0.08, 0.04), "rust_dark"),
        ((4.45, -0.45, 1.05), (1.85, 0.06, 2.05), (0.0, 0.04, -0.06), "metal"),
        ((5.30, -0.40, 0.95), (0.06, 1.55, 1.90), (0.0, -0.04, -0.10), "rust_dark"),
    ]
    for loc, size, rot, col in wing_sheets:
        bm = common.make_box(size, color=col)
        common.transform(bm, rot=rot, loc=loc)
        model.add(bm)

    # Heavy timber posts & cross planks tying the wrecks together
    for px, py, h in ((-5.45, 0.35, 2.4), (-3.80, 0.72, 2.5), (-0.25, 0.76, 2.8),
                      (1.90, 0.70, 2.5), (4.10, 0.55, 2.2), (5.35, 0.25, 2.0)):
        model.add(common.make_box((0.14, 0.14, h), loc=(px, py, h / 2), color="bark_dark"))

    planks = [
        ((-5.45, 0.42, 1.55), (-1.40, 0.78, 1.75), 0.05, 0.22, "wood_crate"),
        ((-5.45, 0.44, 0.88), (-0.90, 0.75, 0.85), 0.05, 0.20, "bark_light"),
        ((-1.90, 0.76, 2.25), (1.60, 0.70, 2.10), 0.05, 0.22, "wood_crate"),
        ((1.30, 0.68, 1.60), (5.35, 0.32, 1.45), 0.05, 0.20, "bark_light"),
        ((-5.35, -0.80, 1.40), (0.20, -1.00, 1.50), 0.05, 0.22, "wood_crate"),
    ]
    for p0, p1, w, h, col in planks:
        model.add(_beam_box(p0, p1, w, h, col))

    # Climbable ladder-ramp on the +X side (+Y street front): rises from x=+5.4 (z=0) up to x=+1.5 (z=2.35)
    # Run = 3.9 m, rise = 2.35 m -> slope = atan2(2.35, 3.9) = 31.1 deg (well under 40 deg)
    x_bot, x_top, ramp_h = 5.40, 1.50, 2.35  # new: DECISIONS 2026-11-26
    y_mid, rail_half = 1.25, 0.44  # new: DECISIONS 2026-11-26
    for sy in (-rail_half, rail_half):
        model.add(_beam_box((x_bot, y_mid + sy, 0.04), (x_top, y_mid + sy, ramp_h + 0.04),
                            0.10, 0.14, "bark_dark"))
    # Support posts & diagonal braces under the ladder-ramp
    for sx, sz in ((3.45, ramp_h * 0.50), (1.65, ramp_h * 0.96)):
        for sy in (-rail_half, rail_half):
            model.add(common.make_box((0.12, 0.12, sz), loc=(sx, y_mid + sy, sz / 2), color="bark_dark"))
    # Rungs spaced every 0.30 m along the horizontal/slope run
    ramp_len = math.hypot(x_bot - x_top, ramp_h)
    num_rungs = int(ramp_len / 0.30)  # new: DECISIONS 2026-11-26 (0.3 m rung spacing)
    for i in range(1, num_rungs + 1):
        t = i / (num_rungs + 0.5)
        rx = x_bot + (x_top - x_bot) * t
        rz = ramp_h * t + 0.06
        model.add(common.make_box((0.18, rail_half * 2.05, 0.06), loc=(rx, y_mid, rz), color="bark_light"))
    # Small top landing deck over the right-hand car roof
    model.add(common.make_box((1.35, 1.05, 0.08), loc=(0.85, y_mid, ramp_h), color="wood_crate"))

    # Colliders: one solid wall box blocking the street, one convex ramp wedge on the ladder side
    model.add_collider("wall", common.make_box((11.0, 2.1, 3.4), loc=(0.0, -0.35, 1.7), color="white"))
    model.add_collider("ramp", _ramp_wedge_bm(x_bot, x_top, y_mid - rail_half - 0.06,
                                              y_mid + rail_half + 0.06, ramp_h))


def build_all():
    ok = common.run_model("car_wall_stack", "props", BUDGET, _build_car_wall, view=WALL_VIEW)
    return [("car_wall_stack", ok)]
