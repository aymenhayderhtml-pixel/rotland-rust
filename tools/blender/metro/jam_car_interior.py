"""Car interior for the R38 jam cars: seats with headrests (bare spring frames when burnt), a dashboard with
instrument cluster, a 16-segment steering wheel, a rear shelf and bags on some seats. Visible through open doors
and broken glass, and from the top preview.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import car_parts  # noqa: E402
import common  # noqa: E402
from jam_util import box, pbox, to_build  # noqa: E402

WHEEL_SEG = 16  # new: DECISIONS 2026-11-26 (steering ring segments)
BAG_COLORS = ["soda_purple", "leather", "canvas_khaki", "car_blue"]


def _seat(p, burnt, bag_color=None):
    """Seat with its cushion centre at p, facing +Y. Burnt: a steel frame with springs and no cushion."""
    x, y, z = p
    parts = [box((x - 0.24, y - 0.26, z - 0.1), (x + 0.24, y + 0.26, z - 0.06), "metal")]  # seat frame
    if burnt:
        for k in range(4):
            sx = x - 0.18 + k * 0.12
            parts.append(box((sx - 0.01, y - 0.24, z - 0.06), (sx + 0.01, y + 0.24, z - 0.02), "metal"))  # springs
        parts.append(box((x - 0.24, y - 0.31, z - 0.06), (x + 0.24, y - 0.27, z + 0.52), "metal"))  # back frame
        return parts
    parts.append(pbox((x - 0.22, y - 0.24, z - 0.06), (x + 0.22, y + 0.24, z + 0.06), "leather", bevel=0.03))
    parts.append(pbox((x - 0.21, y - 0.31, z + 0.04), (x + 0.21, y - 0.23, z + 0.58), "leather", bevel=0.03))
    parts.append(pbox((x - 0.1, y - 0.3, z + 0.6), (x + 0.1, y - 0.24, z + 0.74), "gun_black", bevel=0.02))  # headrest
    if bag_color:
        parts.append(pbox((x - 0.16, y - 0.1, z + 0.06), (x + 0.16, y + 0.18, z + 0.28), bag_color, bevel=0.04))
    return parts


def _steering_wheel(p, dash_y, burnt):
    """16-segment ring with three spokes and a hub, tilted toward the driver. p = driver seat cushion centre."""
    x, y, z = p
    hub = (x, y + 0.22, z + 0.40)
    radius, tube = 0.18, 0.024
    parts = []
    for k in range(WHEEL_SEG):
        a = 2 * math.pi * k / WHEEL_SEG
        seg_len = 2 * math.pi * radius / WHEEL_SEG * 1.15
        seg = common.make_box((seg_len, 0.03, tube), color="gun_black")
        common.transform(seg, rot=(0.0, 0.0, a), loc=(math.cos(a) * radius, math.sin(a) * radius, 0.0))
        parts.append(seg)
    for k in range(3):
        a = 2 * math.pi * k / 3 + 0.5
        spoke = common.make_box((radius * 2, 0.02, 0.02), color="gun_black")
        common.transform(spoke, rot=(0.0, 0.0, a), loc=(0.0, 0.0, 0.0))
        parts.append(spoke)
    parts.append(common.make_frustum(0.04, 0.04, -0.02, 0.02, segments=10, color="gun_black"))
    for bm in parts:
        common.transform(bm, rot=(1.57 - 0.5, 0.0, 0.0), loc=hub)
    column = car_parts._beam((x, dash_y + 0.05, z + 0.14), hub, 0.025, color="metal", segments=6)
    return parts + [column]


def _dash(spec, dash_y, burnt):
    w, belt = spec["W"], spec["belt"]
    parts = [pbox((-w + 0.1, dash_y, belt - 0.26), (w - 0.1, dash_y + 0.1, belt - 0.04), "gun_black", bevel=0.03)]
    if not burnt:
        parts.append(box((-0.22, dash_y + 0.1, belt - 0.2), (-0.02, dash_y + 0.11, belt - 0.1), "glass_tint"))
        parts.append(pbox((0.12, dash_y + 0.1, belt - 0.18), (0.24, dash_y + 0.16, belt - 0.1), "metal", bevel=0.01))
    return parts


def build_interior(spec, seats, offset, burnt):
    """Dash, seats, steering wheel (driver) and rear shelf. `seats` = [(name, exported point)]."""
    dash_y = spec["cab"]["bf"] - 0.10
    parts = _dash(spec, dash_y, burnt)
    for i, (name, point) in enumerate(seats):
        p = to_build(point, offset)
        bag = BAG_COLORS[i % len(BAG_COLORS)] if (i % 2 == 1 and not burnt) else None
        parts += _seat(p, burnt, bag)
        if name == "driver":
            parts += _steering_wheel(p, dash_y, burnt)
    if spec.get("shelf") and not burnt:
        rear_y = spec["cab"]["bb"] + 0.25
        parts.append(pbox((-spec["W"] + 0.08, rear_y - 0.2, spec["belt"] - 0.28),
                          (spec["W"] - 0.08, rear_y + 0.2, spec["belt"] - 0.22), "gun_black", bevel=0.01))
    return parts
