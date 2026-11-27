"""Fire-station props (R42-05): hose_rack, gear_rack, bay_door_torn, kitchen_counter, traffic_cone, hose_line.

Build coordinates are metres: +X right, +Y front, +Z up, ground at Z = 0. The Model origin is the bottom
centre of the bounding box (the default, no grip), so a placed prop stands on the ground at its own footprint.

`torn_door_parts` is shared with `fire_station.py`: the bay-3 roller door of the building is the same torn
shutter, hanging off its rail.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER = os.path.dirname(HERE)
for _p in (HERE, BLENDER,
           os.path.join(BLENDER, "buildings"),
           os.path.join(BLENDER, "metro"),
           os.path.join(BLENDER, "traces"),
           os.path.join(BLENDER, "vehicles")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import bmesh  # noqa: E402

import car_parts  # noqa: E402
import common  # noqa: E402
from graffiti_font import FONT as _STROKE_FONT  # noqa: E402
from jam_car_interior import _seat, _steering_wheel  # noqa: E402
from jam_util import add_bbox_collider, box, ground_rim, pbox, rim_bm, tyre_bm  # noqa: E402

# data: budgets.json models.props (4000 tris each); the vehicles use the 10k vehicle budget
PROP_BUDGET = 4000
VEHICLE_BUDGET = 10000   # data: budgets.json models.vehicles is 25000; R42-03 sets 10k for the place vehicles
VEHICLE_BUDGET_LOD1 = 1000

TURNOUT = "cloth_olive"     # turnout coat cloth
TURNOUT_DARK = "leather"    # cuffs, collars, boots
REFLECT = "wheat_gold"      # reflective stripe
HELMET = "wheat_gold"
HELMET_DARK = "rock_dark"
HOSE_RED = "red_paint"
HOSE_GREEN = "cloth_olive"


# ---------------------------------------------------------------- small shared shapes


def ring_boxes(cx, cy, cz, radius, seg=12, thick=0.07, depth=0.07, color="red_paint",
               plane="xz", closed=True):
    """A ring made of short boxes, the cheap way to read as a coiled hose.

    plane "xz": the ring stands up (hanging on a wall, facing +Y). plane "xy": it lies flat on the floor.
    """
    parts = []
    step = 2.0 * math.pi / seg
    count = seg if closed else seg
    for i in range(count):
        a = i * step
        length = 2.0 * radius * math.tan(step / 2.0) * 1.25
        bm = common.make_box((length, depth, thick), color=color)
        if plane == "xz":
            common.transform(bm, rot=(0.0, -(a + math.pi / 2.0), 0.0),
                             loc=(cx + math.cos(a) * radius, cy, cz + math.sin(a) * radius))
        else:
            common.transform(bm, rot=(0.0, 0.0, a + math.pi / 2.0),
                             loc=(cx + math.cos(a) * radius, cy + math.sin(a) * radius, cz))
        parts.append(bm)
    return parts


def _hook(x, y, z, color="metal"):
    """Coat hook: a short arm off the wall plus an upturned tip."""
    return [box((x - 0.03, y, z), (x + 0.03, y + 0.16, z + 0.04), color),
            box((x - 0.03, y + 0.13, z + 0.04), (x + 0.03, y + 0.16, z + 0.12), color)]


def coat_parts(cx, cy, z_top, color=TURNOUT, width=0.52, burnt=False):
    """A turnout coat hanging on a hook: shoulders, body, two sleeves, two reflective bands, a collar."""
    parts = []
    shoulder_z = z_top - 0.12
    bottom_z = z_top - 0.95
    body = pbox((cx - width / 2, cy - 0.10, bottom_z), (cx + width / 2, cy + 0.10, shoulder_z),
                "soot" if burnt else color, bevel=0.03)
    parts.append(body)
    # Shoulders sloping out to the sleeves
    parts.append(pbox((cx - width / 2 - 0.13, cy - 0.10, shoulder_z - 0.10),
                      (cx + width / 2 + 0.13, cy + 0.10, shoulder_z + 0.02),
                      "soot" if burnt else color, bevel=0.04))
    for side in (-1, 1):
        sx = cx + side * (width / 2 + 0.09)
        parts.append(pbox((sx - 0.07, cy - 0.08, bottom_z + 0.18), (sx + 0.07, cy + 0.08, shoulder_z - 0.04),
                          "soot" if burnt else color, bevel=0.02))
    if not burnt:
        for frac in (0.34, 0.62):
            z = bottom_z + (shoulder_z - bottom_z) * frac
            parts.append(box((cx - width / 2 - 0.005, cy - 0.105, z), (cx + width / 2 + 0.005, cy + 0.105, z + 0.06),
                             REFLECT))
        parts.append(box((cx - width / 2 + 0.04, cy - 0.105, shoulder_z - 0.06),
                         (cx + width / 2 - 0.04, cy + 0.105, shoulder_z + 0.03), TURNOUT_DARK))
    # Helmet resting on the shoulders of the coat
    dome = common.make_frustum(0.135, 0.085, shoulder_z + 0.02, shoulder_z + 0.20, segments=10,
                              color="soot" if burnt else HELMET)
    common.transform(dome, loc=(cx, cy, 0.0))
    parts.append(dome)
    parts.append(box((cx - 0.19, cy - 0.14, shoulder_z + 0.01), (cx + 0.19, cy + 0.14, shoulder_z + 0.05),
                     HELMET_DARK))
    return parts


def helmet_parts(cx, cy, cz, burnt=False, yaw=0.0):
    """A fire helmet sitting on a shelf: dome, brim, front shield."""
    parts = []
    dome = common.make_frustum(0.135, 0.08, 0.0, 0.16, segments=10,
                               color="soot" if burnt else HELMET)
    brim = common.make_frustum(0.18, 0.18, -0.02, 0.02, segments=12, color=HELMET_DARK)
    shield = box((-0.06, 0.13, 0.02), (0.06, 0.16, 0.10), "brass" if not burnt else "soot")
    for bm in (dome, brim, shield):
        common.transform(bm, rot=(0.0, 0.0, yaw), loc=(cx, cy, cz))
        parts.append(bm)
    return parts


def boot_parts(cx, cy, burnt=False):
    parts = [box((cx - 0.09, cy - 0.28, 0.0), (cx + 0.09, cy + 0.14, 0.30), "soot" if burnt else TURNOUT_DARK),
             box((cx - 0.09, cy + 0.02, 0.0), (cx + 0.09, cy + 0.22, 0.07), "gun_black")]
    return parts


# ---------------------------------------------------------------- bay_door_torn


def torn_door_parts(width=5.0, height=4.6, seed=11):
    """Roller shutter torn off its rail: the rail stub at z = 0, slats hanging below it.

    The left two-thirds still hangs from the rail; the right third has torn loose, sagged ~0.4 m and
    leans out of the wall plane. The bottom edge is ragged.
    """
    rng = random.Random(seed)
    parts = []
    half = width / 2.0
    tear_x = 0.75  # where the panel split
    # Rail stub and its two brackets
    parts.append(box((-half - 0.22, -0.07, -0.05), (half + 0.22, 0.07, 0.05), "rust"))
    for bx in (-half + 0.1, tear_x, half - 0.1):
        parts.append(box((bx - 0.05, -0.12, 0.0), (bx + 0.05, 0.12, 0.16), "metal"))

    slat_h = 0.135
    rows = int(height / slat_h)

    def slat(x0, x1, z, y0=-0.045, y1=0.045, color="metal"):
        return box((x0, y0, z), (x1, y1, z + slat_h * 0.82), color)

    # Left part: still on the rail, ragged at the bottom
    for i in range(rows):
        z = -0.12 - i * slat_h
        left = -half + rng.uniform(0.0, 0.09)
        right = tear_x - rng.uniform(0.0, 0.05)
        if i > rows * 0.55:  # ragged bottom: the panel is torn shorter at random columns
            keep = rng.random()
            if keep < 0.18:
                continue
            if keep < 0.45:
                right -= rng.uniform(0.5, 1.4)
        col = "metal" if i % 2 == 0 else "rust"
        parts.append(slat(left, right, z, color=col))
        parts.append(box((left, 0.045, z), (right, 0.055, z + slat_h * 0.7), "rust_dark"))

    # Right part: torn loose, hanging lower and skewed outward
    loose = []
    for i in range(rows):
        z = -0.12 - i * slat_h
        left = tear_x + rng.uniform(0.02, 0.12)
        right = half - rng.uniform(0.0, 0.12)
        if i > rows * 0.42:
            keep = rng.random()
            if keep < 0.22:
                continue
            if keep < 0.5:
                left += rng.uniform(0.4, 1.2)
        col = "metal" if i % 2 == 0 else "rust"
        loose.append(slat(left, right, z, color=col))
        loose.append(box((left, 0.045, z), (right, 0.055, z + slat_h * 0.7), "rust_dark"))
    hinge = common.combine(loose)
    common.transform(hinge, rot=(0.0, -0.20, 0.16), loc=(tear_x * 0.06, 0.30, -0.42))
    parts.append(hinge)
    # A bent slat still hooked over the rail above the loose part
    parts.append(box((tear_x - 0.02, -0.06, 0.0), (half - 0.2, 0.02, 0.05), "rust"))
    return common.combine(parts)


# ---------------------------------------------------------------- the four place props


def build_hose_rack(model):
    """1.8 x 0.55 x 1.9 m wall rack: three hanging hose coils, a shelf of nozzles, two brackets."""
    parts = []
    w, d, h = 1.8, 0.55, 1.9
    for sx in (-w / 2, w / 2):
        parts.append(pbox((sx - 0.045, -d / 2, 0.0), (sx + 0.045, d / 2, h), "metal", bevel=0.008))
        parts.append(box((sx - 0.07, -d / 2 - 0.06, 0.0), (sx + 0.07, d / 2 + 0.02, 0.06), "rust_dark"))
    # Back frame rails
    for z in (0.35, 1.25, h - 0.06):
        parts.append(box((-w / 2, -d / 2, z), (w / 2, -d / 2 + 0.05, z + 0.07), "rust_dark"))
    # Top shelf
    parts.append(pbox((-w / 2 - 0.06, -d / 2, h - 0.06), (w / 2 + 0.06, d / 2, h), "metal", bevel=0.01))
    # Three coils hanging on pegs, at slightly different sizes
    for i, (cx, r, col) in enumerate(((-0.58, 0.30, HOSE_RED), (0.0, 0.34, HOSE_GREEN), (0.58, 0.27, HOSE_RED))):
        cz = 1.10 - i * 0.02
        parts += ring_boxes(cx, 0.02, cz, r, seg=12, thick=0.075, depth=0.14, color=col)
        parts += ring_boxes(cx, 0.02, cz, r * 0.62, seg=10, thick=0.07, depth=0.13, color=col)
        parts += _hook(cx, -d / 2 + 0.05, cz + r + 0.02)
        # Hanging tail
        parts.append(box((cx - 0.04, -0.06, cz - r - 0.45), (cx + 0.04, 0.06, cz - r + 0.05), col))
        parts.append(box((cx - 0.05, -0.08, cz - r - 0.5), (cx + 0.05, 0.08, cz - r - 0.42), "brass"))
    # Nozzles and couplings on the top shelf
    for i, cx in enumerate((-0.62, -0.2, 0.24, 0.62)):
        parts.append(common.make_frustum(0.045, 0.03, h, h + 0.22, segments=8, center=(cx, 0.0), color="brass"))
        parts.append(box((cx - 0.05, -0.05, h + 0.2), (cx + 0.05, 0.05, h + 0.28), "metal"))
    model.add(common.combine(parts))
    model.add_collider("rack", box((-w / 2, -d / 2, 0.0), (w / 2, d / 2, h), "white"))


def build_gear_rack(model):
    """2.1 x 0.7 x 2.2 m rack: four turnout coats on hooks, three helmets on the top shelf, boots below."""
    parts = []
    w, d, h = 2.1, 0.7, 2.2
    for sx in (-w / 2, w / 2):
        parts.append(pbox((sx - 0.05, -d / 2, 0.0), (sx + 0.05, d / 2, h), "metal", bevel=0.008))
    for z in (0.06, 1.30, h - 0.05):
        parts.append(box((-w / 2, -d / 2, z), (w / 2, -d / 2 + 0.06, z + 0.08), "rust_dark"))
    # Top shelf and the hanging bar
    parts.append(pbox((-w / 2 - 0.06, -d / 2, h - 0.05), (w / 2 + 0.06, d / 2, h + 0.03), "metal", bevel=0.01))
    parts.append(box((-w / 2, -d / 2 + 0.08, h - 0.42), (w / 2, -d / 2 + 0.12, h - 0.36), "metal"))
    # Four coats on hooks along the bar (one is missing: only its empty hook shows)
    for i, cx in enumerate((-0.74, -0.25, 0.25)):
        parts += _hook(cx, -d / 2 + 0.10, h - 0.40)
        parts += coat_parts(cx, -d / 2 + 0.26, h - 0.30, burnt=(i == 2))
    parts += _hook(0.74, -d / 2 + 0.10, h - 0.40)
    # Helmets on the top shelf
    for i, cx in enumerate((-0.66, -0.12, 0.44)):
        parts += helmet_parts(cx, 0.02, h + 0.03, burnt=(i == 2), yaw=0.4 * i)
    # Boots on the floor
    for cx in (-0.74, -0.58, 0.28, 0.44):
        parts += boot_parts(cx, 0.06)
    # A spare air cylinder leaning at the end
    cyl = common.make_frustum(0.10, 0.10, 0.0, 0.72, segments=10, center=(0.92, 0.16), color="gun_black")
    parts.append(cyl)
    parts.append(common.make_frustum(0.035, 0.035, 0.72, 0.86, segments=8, center=(0.92, 0.16), color="brass"))
    model.add(common.combine(parts))
    model.add_collider("rack", box((-w / 2, -d / 2, 0.0), (w / 2, d / 2, h), "white"))


def build_bay_door_torn(model):
    """5 m x 4.6 m torn roller shutter, leaning where it was dragged into the yard."""
    bm = torn_door_parts()
    common.transform(bm, rot=(-0.30, 0.0, 0.22), loc=(0.0, 0.0, 4.6))
    model.add(bm)
    model.add_collider("door", box((-2.4, -0.5, 0.0), (2.4, 1.4, 1.4), "white"))


def build_kitchen_counter(model):
    """2.4 x 0.7 x 1.55 m counter run: sink, two burners, cupboard doors, a wall shelf of jars."""
    parts = []
    w, d = 2.4, 0.7
    top = 0.94
    parts.append(pbox((-w / 2, -d / 2, 0.0), (w / 2, d / 2, top - 0.06), "white", bevel=0.012))
    parts.append(box((-w / 2 - 0.03, -d / 2 - 0.03, top - 0.06), (w / 2 + 0.03, d / 2 + 0.03, top), "wood_crate"))
    parts.append(box((-w / 2 + 0.04, -d / 2 + 0.04, 0.0), (w / 2 - 0.04, d / 2 - 0.02, 0.08), "soot"))
    # Cupboard doors with handles and a kick line
    for i in range(4):
        x0 = -w / 2 + 0.06 + i * 0.58
        parts.append(box((x0, d / 2 - 0.03, 0.10), (x0 + 0.54, d / 2 - 0.005, top - 0.10), "white"))
        parts.append(box((x0 + 0.02, d / 2 - 0.05, 0.12), (x0 + 0.52, d / 2 - 0.03, top - 0.12), "bone_white"))
        parts.append(box((x0 + 0.44, d / 2, 0.62), (x0 + 0.50, d / 2 + 0.04, 0.66), "metal"))
    # Sink: recessed bowl + tap
    parts.append(box((-1.02, -0.24, top - 0.02), (-0.42, 0.24, top + 0.005), "metal"))
    parts.append(box((-0.96, -0.19, top - 0.14), (-0.48, 0.19, top - 0.02), "rock_dark"))
    parts.append(common.make_frustum(0.028, 0.028, top, top + 0.30, segments=8, center=(-0.72, -0.24), color="metal"))
    parts.append(box((-0.76, -0.30, top + 0.26), (-0.68, -0.10, top + 0.32), "metal"))
    # Two burner rings
    for cx in (0.22, 0.78):
        parts.append(common.make_frustum(0.16, 0.16, top, top + 0.03, segments=12, center=(cx, 0.0), color="gun_black"))
        parts.append(common.make_frustum(0.07, 0.07, top + 0.03, top + 0.05, segments=10, center=(cx, 0.0),
                                         color="metal"))
    # Wall shelf above with jars, tins, a kettle
    parts.append(box((-w / 2, -d / 2, 1.42), (w / 2, -d / 2 + 0.26, 1.47), "wood_crate"))
    for i in range(6):
        cx = -1.0 + i * 0.42
        parts.append(common.make_frustum(0.055, 0.05, 1.47, 1.68, segments=8, center=(cx, -d / 2 + 0.13),
                                         color=("bone_white", "leaf_dark", "brass")[i % 3]))
        parts.append(common.make_frustum(0.06, 0.06, 1.66, 1.70, segments=8, center=(cx, -d / 2 + 0.13),
                                         color="metal"))
    kettle = common.make_frustum(0.10, 0.08, 1.47, 1.62, segments=10, center=(1.0, -d / 2 + 0.13), color="metal")
    parts.append(kettle)
    # A dead microwave on the counter end
    parts.append(pbox((0.78, -0.26, top), (1.16, 0.24, top + 0.28), "metal", bevel=0.01))
    parts.append(box((0.84, 0.24, top + 0.05), (1.10, 0.26, top + 0.23), "soot"))
    model.add(common.combine(parts))
    model.add_collider("counter", box((-w / 2, -d / 2, 0.0), (w / 2, d / 2, top), "white"))


def build_traffic_cone(model):
    """0.36 m square base, 0.7 m tall cone with two white bands."""
    base = box((-0.18, -0.18, 0.0), (0.18, 0.18, 0.035), "rock_dark")
    body = common.make_frustum(0.135, 0.03, 0.035, 0.70, segments=10, color="carrot_orange")
    parts = [base, body]
    for z0, z1, r0, r1 in ((0.24, 0.32, 0.105, 0.09), (0.42, 0.50, 0.078, 0.062)):
        parts.append(common.make_frustum(r0, r1, z0, z1, segments=10, color="white"))
    model.add(common.combine(parts))
    model.add_collider("cone", box((-0.16, -0.16, 0.0), (0.16, 0.16, 0.6), "white"))


def build_hose_line(model):
    """6 m of 65 mm hose laid out in a lazy S on the ground, with a coupling and a nozzle."""
    rng = random.Random(23)
    pts = []
    for i in range(15):
        t = i / 14.0
        x = -2.9 + 5.8 * t
        y = math.sin(t * 4.2) * 0.55 + rng.uniform(-0.05, 0.05)
        pts.append((x, y, 0.055))
    parts = []
    for i in range(len(pts) - 1):
        parts.append(car_parts._beam(pts[i], pts[i + 1], 0.055, color=HOSE_RED, segments=8))
    # Two flat fold-overs (the hose was dragged)
    for i in (4, 9):
        p = pts[i]
        parts.append(box((p[0] - 0.10, p[1] - 0.10, 0.0), (p[0] + 0.10, p[1] + 0.10, 0.10), HOSE_RED))
    # Brass couplings at both ends and a nozzle
    for p in (pts[0], pts[-1]):
        parts.append(common.make_frustum(0.065, 0.065, 0.0, 0.13, segments=10, center=(p[0], p[1]), color="brass"))
    parts.append(common.make_frustum(0.03, 0.045, 0.0, 0.34, segments=8, center=(pts[-1][0] + 0.12, pts[-1][1]),
                                     color="brass"))
    model.add(common.combine(parts))
    model.add_collider("hose", box((-3.0, -0.7, 0.0), (3.0, 0.7, 0.12), "white"))


BUILDINGS_ORDER = ("hose_rack", "gear_rack", "bay_door_torn", "kitchen_counter",
                   "traffic_cone", "hose_line")

# ================================================================ lettering

# The R42 signs need FIRE DEPT and FIRE STATION; graffiti_font has no P (new: DECISIONS 2026-11-27 glyph).
GLYPHS = dict(_STROKE_FONT)
GLYPHS["P"] = [(0, 0, 0, 1), (0, 1, 0.78, 1), (0.78, 1, 1, 0.74), (1, 0.74, 0.78, 0.46), (0.78, 0.46, 0, 0.46)]


def letters_bm(text, letter_w, letter_h, color="white", depth=0.02, thick_scale=1.0, mirror=False):
    """Stroke letters as flat bars in the XZ plane, reading left to right seen from +Y.

    Centred on x = 0, sitting on z = 0. The caller moves and turns the result onto a face.
    `mirror` flips the reading direction: the side faces (+X / -X) need it, because a -90 degree
    turn about Z puts the letters the wrong way round (see `letters_on`).
    """
    gap = letter_w * 0.35
    space = letter_w * 0.62
    total = sum(space if ch == " " else letter_w for ch in text) + gap * (len(text) - 1)
    thick = letter_h * 0.13 * thick_scale
    sign = 1.0 if mirror else -1.0
    parts = []
    x = -total / 2.0
    for ch in text:
        if ch == " ":
            x += space + gap
            continue
        for u0, v0, u1, v1 in GLYPHS[ch]:
            p0 = (sign * (x + u0 * letter_w), 0.0, v0 * letter_h)
            p1 = (sign * (x + u1 * letter_w), 0.0, v1 * letter_h)
            dx, dz = p1[0] - p0[0], p1[2] - p0[2]
            length = math.hypot(dx, dz)
            if length < 1e-5:
                continue
            bar_bm = common.make_box((length * 1.06, depth, thick), color=color)
            common.transform(bar_bm, rot=(0.0, -math.atan2(dz, dx), 0.0),
                             loc=((p0[0] + p1[0]) / 2.0, 0.0, (p0[2] + p1[2]) / 2.0))
            parts.append(bar_bm)
        x += letter_w + gap
    return common.combine(parts) if parts else common.new_bm()


# Which way each face is turned, and whether its letters must be mirrored to read correctly
FACE_TURN = {"+Y": (0.0, False), "-Y": (math.pi, False), "+X": (-math.pi / 2.0, True),
             "-X": (math.pi / 2.0, True)}


def letters_on(bm, loc, facing):
    """Turn a `letters_bm` result onto a wall face: facing is one of "+Y", "-Y", "+X", "-X"."""
    rot, _ = FACE_TURN[facing]
    common.transform(bm, rot=(0.0, 0.0, rot), loc=loc)
    return bm


# ================================================================ fire trucks

# new: DECISIONS 2026-11-27 (truck sizes in metres, R38 car quality at the R42 10k vehicle budget)
PUMPER = dict(half_l=3.9, w=1.22, sill=0.62, belt=1.63, roof=3.02, cab_back=1.45, cab_front=3.74,
              wheel_front=2.85, wheel_rear=-2.15, wheel_r=0.52, wheel_half=0.22, dual=True)
LADDER = dict(half_l=5.5, w=1.24, sill=0.66, belt=1.66, roof=3.05, cab_back=2.85, cab_front=5.34,
              wheel_front=4.35, wheel_rear=-1.55, wheel_r=0.55, wheel_half=0.23, dual=True)

BODY_RED = "car_red"
STRIPE_WHITE = "car_white"
CHROME = "chrome"
GLASS = "glass_tint"


def _shell_colors(burnt):
    return ("soot", "gun_black") if burnt else (BODY_RED, "red_paint")


def _chassis(sp, burnt):
    out = []
    for x in (-0.62, 0.62):
        out.append(box((x - 0.09, -sp["half_l"], 0.36), (x + 0.09, sp["half_l"] - 0.1, 0.60), "gun_black"))
    # Rear step bumper and tow eye
    out.append(pbox((-sp["w"] - 0.04, -sp["half_l"] - 0.16, 0.30), (sp["w"] + 0.04, -sp["half_l"] + 0.02, 0.52),
                    "gun_black", bevel=0.03))
    out.append(box((-0.12, -sp["half_l"] - 0.22, 0.34), (0.12, -sp["half_l"] - 0.14, 0.46), "metal"))
    # Fuel tank and air tanks under the frame
    tank = common.make_frustum(0.22, 0.22, -0.85, 0.85, segments=10, color="metal")
    common.transform(tank, rot=(math.pi / 2.0, 0.0, 0.0), loc=(-0.95, -0.6, 0.48))
    out.append(tank)
    return out


TRUCK_TYRE_SEG = 16   # new: DECISIONS 2026-11-27 (truck tyre segments: round enough, keeps the 10k budget)


def truck_wheel(x, y, radius, half_w, flat=1.0):
    """Tyre plus rim for a truck wheel at (x, y), axle along X. `flat` squashes the tyre (0.78 = flat)."""
    tyre = tyre_bm(radius, half_w, seg=TRUCK_TYRE_SEG)
    rim = rim_bm(radius, half_w, spokes=5)
    out = [tyre, rim]
    for bm in out:
        common.transform(bm, rot=(0.0, math.pi / 2.0, 0.0))
        common.transform(bm, scale=(1.0, 1.0, flat), loc=(x, y, radius * flat))
    return out


def _wheels(sp, burnt, rng):
    out = []
    xs = [(-sp["w"] + 0.16, sp["w"] - 0.16)]
    if sp["dual"]:
        xs.append((-sp["w"] + 0.55, sp["w"] - 0.55))
    if burnt:
        for i, (x0, x1) in enumerate(xs):
            for sx in (x0, x1):
                y = sp["wheel_front"] if i else sp["wheel_rear"]
                out.append(box((sx - 0.2, y - 0.4, 0.0), (sx + 0.2, y + 0.4, 0.5), "gun_black"))
        for k, (dx, dy) in enumerate(((-1.9, 1.2), (1.7, -0.6), (-1.4, -2.4), (2.2, 0.4))):
            out.append(ground_rim(dx, dy, sp["wheel_r"], sp["wheel_half"], yaw=0.4 * k))
        return out
    for i, (x0, x1) in enumerate(xs):
        for sx in (x0, x1):
            y = sp["wheel_front"] if i else sp["wheel_rear"]
            flat = 0.78 if (i == 1 and sx < 0) else 1.0  # one flat rear tyre
            out += truck_wheel(sx, y, sp["wheel_r"], sp["wheel_half"], flat=flat)
    # Mudflaps behind the rear wheels
    for sx in (-sp["w"] + 0.16, sp["w"] - 0.16):
        out.append(box((sx - 0.26, sp["wheel_rear"] - 0.72, 0.10), (sx + 0.26, sp["wheel_rear"] - 0.66, 0.70),
                       "gun_black"))
    return out


def _arches(sp, color):
    """Fender flares over each wheel: a thin lip plus the dark arch liner behind it."""
    out = []
    r = sp["wheel_r"]
    for y in (sp["wheel_front"], sp["wheel_rear"]):
        for side in (-1, 1):
            x0, x1 = sorted((side * (sp["w"] - 0.01), side * (sp["w"] + 0.055)))
            # Arch: five flat segments stepping over the tyre (the R38 jam-car fender trick)
            for (cy, cz), (sy, sz), rx in (((y - 0.52, 0.62), (0.10, 0.40), 0.0),
                                           ((y - 0.38, 0.98), (0.10, 0.26), -0.80),
                                           ((y, 1.10), (0.56, 0.10), 0.0),
                                           ((y + 0.38, 0.98), (0.10, 0.26), 0.80),
                                           ((y + 0.52, 0.62), (0.10, 0.40), 0.0)):
                seg = common.make_box((x1 - x0, sy, sz), color=color)
                common.transform(seg, rot=(rx, 0.0, 0.0), loc=(side * (sp["w"] + 0.02), cy, cz))
                out.append(seg)
            # Dark arch liner behind the flare
            out.append(box((x0, y - 0.46, r * 0.55), (x1, y + 0.46, sp["sill"] - 0.02), "gun_black"))
    return out


def _cab(sp, burnt):
    """Cab box with windscreen, side windows, doors, mirrors and steps."""
    red, dark = _shell_colors(burnt)
    w, top = sp["w"], sp["roof"]
    cb, cf, sill = sp["cab_back"], sp["cab_front"], sp["sill"]
    out = []
    bl = [(-w, cb, sill), (w, cb, sill), (w, cf, sill), (-w, cf, sill)]
    tp = [(-w + 0.06, cb + 0.10, top), (w - 0.06, cb + 0.10, top),
          (w - 0.06, cf - 0.14, top), (-w + 0.06, cf - 0.14, top)]
    out.append(car_parts._closed_loft(bl, tp, [red] * 4, red, "gun_black"))
    # Windscreen: glass pane plus frame bars, slightly proud of the front face. It sits high so the
    # FIRE DEPT band has room on the front panel above the grille.
    z0, z1 = sill + 1.08, top - 0.26
    for side in (-1, 1):
        x0, x1 = sorted((side * 0.06, side * (w - 0.16)))
        out.append(box((x0, cf - 0.16, z0), (x1, cf - 0.11, z1), "gun_black" if burnt else GLASS))
    out.append(box((-w + 0.12, cf - 0.155, (z0 + z1) / 2.0 - 0.03), (w - 0.12, cf - 0.115, (z0 + z1) / 2.0 + 0.03),
                   "gun_black"))
    out.append(box((-w + 0.12, cf - 0.155, z0), (w - 0.12, cf - 0.115, z0 + 0.05), "gun_black"))
    out.append(box((-w + 0.12, cf - 0.155, z1 - 0.05), (w - 0.12, cf - 0.115, z1), "gun_black"))
    # Wipers
    for wx in (-0.55, 0.15):
        out.append(box((wx, cf - 0.10, z0 + 0.02), (wx + 0.42, cf - 0.07, z0 + 0.07), "gun_black"))
    # Side windows and doors on both flanks
    for side in (-1, 1):
        x0, x1 = sorted((side * (w - 0.04), side * (w + 0.02)))
        out.append(box((x0, cb + 0.20, z0), (x1, cb + 1.24, z1 - 0.10), "gun_black" if burnt else GLASS))
        out.append(box((x0, cb + 0.16, z0 - 0.05), (x1, cb + 1.28, z0), "metal"))
        out.append(box((x0, cb + 1.24, z0 - 0.05), (x1, cb + 1.30, z1 - 0.05), "metal"))
        # Door seam, handle, step below
        out.append(box((sorted((side * (w + 0.005), side * (w + 0.02)))[0], cb + 0.14, sill),
                       (sorted((side * (w + 0.005), side * (w + 0.02)))[1], cf - 0.16, sill + 0.04), "gun_black"))
        out.append(box((sorted((side * (w + 0.01), side * (w + 0.045)))[0], cb + 0.30, sill + 0.80),
                       (sorted((side * (w + 0.01), side * (w + 0.045)))[1], cb + 0.58, sill + 0.86), "chrome"))
        out.append(box((sorted((side * (w - 0.02), side * (w + 0.22)))[0], cb + 0.55, sill - 0.34),
                       (sorted((side * (w - 0.02), side * (w + 0.22)))[1], cf - 0.30, sill - 0.26), "metal"))
        # Mirror arm and head
        out.append(car_parts._beam((side * w, cf - 0.30, top - 0.30), (side * (w + 0.30), cf - 0.34, top - 0.16),
                                   0.03, color="metal", segments=6))
        xm0, xm1 = sorted((side * (w + 0.24), side * (w + 0.36)))
        out.append(pbox((xm0, cf - 0.40, top - 0.32), (xm1, cf - 0.22, top + 0.12), "gun_black", bevel=0.01))
        xg0, xg1 = sorted((side * (w + 0.355), side * (w + 0.37)))
        out.append(box((xg0, cf - 0.38, top - 0.30), (xg1, cf - 0.24, top + 0.08), "glass_tint"))
    return out


def _body(sp, burnt, ladder):
    """Equipment body behind the cab: compartment doors, white stripe, chevrons, lettering."""
    red, dark = _shell_colors(burnt)
    w, sill, belt, top = sp["w"], sp["sill"], sp["belt"], sp["roof"]
    cb, cf, half = sp["cab_back"], sp["cab_front"], sp["half_l"]
    out = []
    # Main body block
    out.append(pbox((-w, -half, sill), (w, cb, top), red, bevel=0.02))
    # Roof over the body, slightly proud
    out.append(pbox((-w - 0.02, -half - 0.02, top - 0.06), (w + 0.02, cb + 0.02, top), dark, bevel=0.015))
    # Compartment doors on both flanks
    span = cb - (-half) - 0.2
    count = 4 if not ladder else 5
    for side in (-1, 1):
        x0, x1 = sorted((side * (w - 0.01), side * (w + 0.02)))
        for i in range(count):
            y0 = -half + 0.12 + i * span / count
            y1 = y0 + span / count - 0.07
            col = "soot" if burnt else "bone_white"
            out.append(pbox((x0, y0, sill + 0.12), (x1, y1, belt - 0.06), col, bevel=0.008))
            # Roll-up door slats
            z = sill + 0.22
            while z < belt - 0.12:
                out.append(box((x0, y0 + 0.03, z), (x1, y1 - 0.03, z + 0.025), "metal"))
                z += 0.13
            # Handle and hinge
            xh0, xh1 = sorted((side * (w + 0.02), side * (w + 0.06)))
            out.append(box((xh0, (y0 + y1) / 2 - 0.12, sill + 0.30), (xh1, (y0 + y1) / 2 + 0.12, belt - 0.24), "chrome"))
            if burnt and i in (1, 3):  # a door torn off and hanging
                out.append(pbox((sorted((side * (w + 0.02), side * (w + 0.30)))[0], y0, sill - 0.30),
                                (sorted((side * (w + 0.02), side * (w + 0.30)))[1], y1, sill + 0.10),
                                "rust", bevel=0.01))
    # Belt moulding along the top of the body, under the roof cap
    out.append(box((-w - 0.03, -half - 0.01, belt - 0.06), (w + 0.03, cb + 0.01, belt), "metal"))
    if not burnt:
        # The white stripe: a bold band across the top of the compartment doors, and a thin one below
        for side in (-1, 1):
            x0, x1 = sorted((side * (w + 0.008), side * (w + 0.036)))
            out.append(box((x0, -half - 0.01, belt - 0.44), (x1, cb - 0.02, belt - 0.16), STRIPE_WHITE))
            out.append(box((x0, -half - 0.01, belt - 0.62), (x1, cb - 0.02, belt - 0.56), STRIPE_WHITE))
        # FIRE DEPT on the flank, inside the broad white band
        for side in (-1, 1):
            facing = "+X" if side > 0 else "-X"
            letters = letters_bm("FIRE DEPT", 0.135, 0.22, color="red_paint", depth=0.02)
            letters_on(letters, (side * (w + 0.044), -half + 1.9, belt - 0.40), facing)
            out.append(letters)
    # A ladder strapped flat on the flank, in the rail above the compartment doors
    if not ladder:
        side = 1
        x0, x1 = sorted((side * (w + 0.03), side * (w + 0.10)))
        out.append(box((x0, -half + 0.6, belt + 0.36), (x1, cb - 0.4, belt + 0.44), "metal"))
        y = -half + 0.7
        while y < cb - 0.5:
            out.append(box((x0, y - 0.03, belt + 0.24), (x1, y + 0.03, belt + 0.40), "metal"))
            y += 0.36
    # A white band round the cab too, tying the stripe to the doors
    for side in (-1, 1):
        x0, x1 = sorted((side * (w + 0.005), side * (w + 0.025)))
        out.append(box((x0, cb + 0.05, belt + 0.05), (x1, cf - 0.20, belt + 0.28), "white" if not burnt else "soot"))
    return out


def _front(sp, burnt, ladder):
    """Chrome bumper, grille, headlights, indicators, FIRE DEPT letters, front number plate."""
    w, cf, sill, top = sp["w"], sp["cab_front"], sp["sill"], sp["roof"]
    out = []
    out.append(pbox((-w - 0.06, cf - 0.06, 0.42), (w + 0.06, cf + 0.14, 0.76), CHROME if not burnt else "rust",
                    bevel=0.03))
    for sx in (-1, 1):
        out.append(box((sx * 0.42 - 0.10, cf + 0.10, 0.44), (sx * 0.42 + 0.10, cf + 0.20, 0.66), "metal"))
    # Number plate
    out.append(box((-0.26, cf + 0.14, 0.50), (0.26, cf + 0.17, 0.68), "white"))
    # Grille: a dark recess with seven chrome slats
    out.append(box((-w + 0.06, cf - 0.02, 0.78), (w - 0.06, cf + 0.06, 1.44), "gun_black"))
    for k in range(7):
        x = -w + 0.22 + k * (2 * (w - 0.22) / 6.0)
        out.append(box((x - 0.035, cf + 0.05, 0.80), (x + 0.035, cf + 0.095, 1.42), CHROME if not burnt else "rust"))
    # Grille surround and a chrome edge round it
    out.append(box((-w + 0.04, cf + 0.02, 1.44), (w - 0.04, cf + 0.10, 1.50), CHROME if not burnt else "rust"))
    # Headlights and indicators
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 0.60, sx * 1.02))
        out.append(pbox((x0, cf + 0.02, 1.05), (x1, cf + 0.10, 1.32), "white" if not burnt else "soot", bevel=0.015))
        out.append(box((x0 + 0.04, cf + 0.10, 1.10), (x1 - 0.04, cf + 0.12, 1.27), CHROME if not burnt else "soot"))
        out.append(pbox((x0, cf + 0.02, 0.90), (x1, cf + 0.09, 1.02), "carrot_orange", bevel=0.01))
    # Bumper mounted winch and tow hooks
    out.append(pbox((-0.30, cf + 0.14, 0.78), (0.30, cf + 0.34, 1.04), "metal", bevel=0.02))
    for sx in (-0.55, 0.55):
        out.append(box((sx - 0.06, cf + 0.12, 0.46), (sx + 0.06, cf + 0.24, 0.58), "red_paint"))
    # FIRE DEPT on the front panel above the grille, under the windscreen
    if not burnt:
        out.append(box((-w + 0.06, cf + 0.045, sill + 0.96), (w - 0.06, cf + 0.075, sill + 1.06), "gun_black"))
        letters = letters_bm("FIRE DEPT", 0.165, 0.26, color="white", depth=0.02)
        letters_on(letters, (0.0, cf + 0.085, sill + 0.60), "+Y")
        out.append(letters)
    # Sun visor over the windscreen
    out.append(pbox((-w + 0.04, cf - 0.24, top - 0.10), (w - 0.04, cf + 0.16, top - 0.02), BODY_RED if not burnt
                    else "soot", bevel=0.02))
    # Roof marker lights
    for sx in (-0.75, -0.35, 0.35, 0.75):
        out.append(box((sx - 0.07, cf - 0.22, top - 0.02), (sx + 0.07, cf - 0.08, top + 0.06),
                       "carrot_orange" if not burnt else "soot"))
    return out


def _rear(sp, burnt, ladder):
    """Rear chevrons, tail lights, hose reel, ladder turntable overhang."""
    w, half, sill, top = sp["w"], sp["half_l"], sp["sill"], sp["roof"]
    out = []
    # Chevron panel
    out.append(box((-w + 0.06, -half - 0.03, sill + 0.10), (w - 0.06, -half - 0.01, sill + 0.66),
                   "white" if not burnt else "soot"))
    for k in range(6):
        x = -w + 0.20 + k * 0.42
        if x + 0.16 > w - 0.14:
            break
        bar_bm = common.make_box((0.16, 0.02, 0.58), color="red_paint")
        common.transform(bar_bm, rot=(0.0, 0.55, 0.0), loc=(x, -half - 0.045, sill + 0.38))
        out.append(bar_bm)
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 0.62, sx * (w - 0.12)))
        out.append(pbox((x0, -half - 0.06, sill + 0.72), (x1, -half - 0.01, sill + 0.96), "red_paint", bevel=0.01))
        out.append(box((x0 + 0.03, -half - 0.07, sill + 0.76), (x1 - 0.03, -half - 0.05, sill + 0.92),
                       "carrot_orange"))
    # Rear body lettering
    if not burnt:
        letters = letters_bm("FIRE DEPT", 0.16, 0.26, color="white", depth=0.02)
        letters_on(letters, (0.0, -half - 0.055, top - 0.52), "-Y")
        out.append(letters)
    # Hose reel hanging on the back of the body
    reel = common.make_frustum(0.26, 0.26, -0.10, 0.10, segments=14, color="metal")
    common.transform(reel, rot=(0.0, math.pi / 2.0, 0.0), loc=(-0.55, -half - 0.22, sill + 0.30))
    out.append(reel)
    for i in range(10):
        a = 2.0 * math.pi * i / 10
        seg = common.make_box((0.14, 0.05, 0.06), color="red_paint")
        common.transform(seg, rot=(0.0, math.pi / 2.0, a), loc=(-0.55, -half - 0.22 + math.cos(a) * 0.28,
                                                                sill + 0.30 + math.sin(a) * 0.28))
        out.append(seg)
    return out


def _roof_gear(sp, burnt, ladder, rng):
    """Hose bed with coiled hose and a roof ladder (pumper) or the aerial ladder turntable (ladder truck)."""
    w, top, half = sp["w"], sp["roof"], sp["half_l"]
    out = []
    if not ladder:
        # Hose bed: low walls around a coiled load
        y0, y1 = -half + 0.20, sp["cab_back"] - 0.35
        for sx in (-1, 1):
            x0, x1 = sorted((sx * (w - 0.16), sx * (w - 0.02)))
            out.append(box((x0, y0, top), (x1, y1, top + 0.26), "metal"))
        out.append(box((-w + 0.02, y0, top), (w - 0.02, y0 + 0.08, top + 0.26), "metal"))
        for i in range(9):
            cx = -w + 0.34 + (i % 3) * 0.72
            cy = y0 + 0.55 + (i // 3) * 1.0
            hose_col = "soot" if burnt else ("cloth_olive" if i % 2 else "red_paint")
            out.append(common.make_frustum(0.26, 0.26, top + 0.02, top + 0.24, segments=10,
                                           center=(cx, cy), color=hose_col))
        # Roof ladder strapped over the hose bed
        for sx in (-0.34, 0.34):
            out.append(box((sx - 0.05, y0 + 0.10, top + 0.26), (sx + 0.05, y1 - 0.10, top + 0.36), "metal"))
        y = y0 + 0.2
        while y < y1 - 0.2:
            out.append(box((-0.42, y - 0.03, top + 0.28), (0.42, y + 0.03, top + 0.34), "metal"))
            y += 0.34
        # Deck gun folded flat at the front of the hose bed
        gun = "rust" if burnt else "brass"
        out.append(common.make_frustum(0.09, 0.07, top + 0.02, top + 0.40, segments=8, center=(0.6, y1 - 0.4),
                                       color=gun))
        out.append(car_parts._beam((0.6, y1 - 0.4, top + 0.40), (0.6, y1 + 0.30, top + 0.52), 0.05,
                                   color=gun, segments=6))
    else:
        # Turntable base at the rear of the body
        ty = -half + 1.95
        out.append(common.make_frustum(0.95, 0.95, top, top + 0.22, segments=16, center=(0.0, ty), color="metal"))
        out.append(common.make_frustum(0.62, 0.62, top + 0.22, top + 0.46, segments=14, center=(0.0, ty),
                                       color="soot" if burnt else "red_paint"))
        # Folded aerial: three nested sections lying forward along the roof, each smaller than the last
        base_z = top + 0.46
        sections = ((ty - 0.1, ty + 2.6, 0.54, 0.00, "metal"),
                    (ty + 2.5, ty + 4.9, 0.44, 0.16, "metal"),
                    (ty + 4.8, ty + 7.2, 0.34, 0.32, "rust"))
        for y_start, y_end, half_w, lift, col in sections:
            for sx in (-half_w, half_w):
                out.append(box((sx - 0.07, y_start, base_z + lift), (sx + 0.07, y_end, base_z + lift + 0.18), col))
            # Cross rungs between the rails
            y = y_start + 0.20
            while y < y_end - 0.08:
                out.append(box((-half_w + 0.05, y - 0.035, base_z + lift + 0.03),
                               (half_w - 0.05, y + 0.035, base_z + lift + 0.13), "metal"))
                y += 0.28
            # Diagonal truss braces on the near side
            for k in range(int((y_end - y_start) / 0.56)):
                y0 = y_start + 0.2 + k * 0.56
                out.append(car_parts._beam((half_w, y0, base_z + lift + 0.04),
                                           (half_w, y0 + 0.56, base_z + lift + 0.16), 0.018,
                                           color="rust", segments=4))
        # Waterway pipe, nozzle and two stabiliser jacks at the tip
        out.append(car_parts._beam((0.0, ty + 7.1, base_z + 0.44), (0.0, ty + 7.6, base_z + 0.48), 0.06,
                                   color="brass", segments=8))
        out.append(common.make_frustum(0.10, 0.06, base_z + 0.42, base_z + 0.66, segments=8, color="brass"))
        for sx in (-0.34, 0.34):
            out.append(box((sx - 0.05, ty + 6.6, base_z + 0.40), (sx + 0.05, ty + 7.3, base_z + 0.50), "metal"))
        # Outrigger pads folded up along the flanks
        for side in (-1, 1):
            x0, x1 = sorted((side * (w + 0.02), side * (w + 0.24)))
            out.append(box((x0, -half + 1.1, top - 0.55), (x1, -half + 2.6, top - 0.40), "gun_black"))
    return out


def _lights(sp, burnt):
    """Light bar on the cab roof plus two red beacons and a siren speaker."""
    w, top, cb, cf = sp["w"], sp["roof"], sp["cab_back"], sp["cab_front"]
    out = []
    yc = cf - 0.50
    out.append(pbox((-0.85, yc - 0.11, top + 0.02), (0.85, yc + 0.11, top + 0.20), "gun_black", bevel=0.015))
    for sx in (-0.52, 0.52):
        out.append(box((sx - 0.22, yc - 0.09, top + 0.14), (sx + 0.22, yc + 0.09, top + 0.30),
                       "red_paint" if not burnt else "soot"))
    out.append(box((-0.08, yc - 0.09, top + 0.14), (0.08, yc + 0.09, top + 0.26), "carrot_orange"))
    # Siren speaker on the roof behind the bar
    out.append(common.make_frustum(0.14, 0.14, 0.0, 0.26, segments=10, color="metal"))
    common.transform(out[-1], rot=(math.pi / 2.0, 0.0, 0.0), loc=(0.0, yc - 0.70, top + 0.13))
    # Side repeaters on the cab shoulders
    for side in (-1, 1):
        x0, x1 = sorted((side * (w - 0.02), side * (w + 0.03)))
        out.append(box((x0, cb + 0.30, top - 0.55), (x1, cb + 0.55, top - 0.40),
                       "carrot_orange" if not burnt else "soot"))
    return out


def _interior(sp, burnt):
    """Two cab seats, dash and steering wheel, visible through the windscreen and the top preview."""
    w, cb, cf, sill, belt = sp["w"], sp["cab_back"], sp["cab_front"], sp["sill"], sp["belt"]
    out = []
    seat_y = cf - 1.05
    for sx in (-0.52, 0.52):
        out += _seat((sx, seat_y, sill + 0.55), burnt)
    out += _steering_wheel((-0.52, seat_y, sill + 0.55), cf - 1.85, burnt)
    # Dash and engine cover
    out.append(pbox((-w + 0.10, cf - 1.95, belt - 0.34), (w - 0.10, cf - 1.75, belt + 0.02), "gun_black",
                    bevel=0.02))
    out.append(pbox((-0.30, seat_y - 0.55, sill), (0.30, seat_y + 0.30, sill + 0.62), "gun_black", bevel=0.02))
    return out


def _truck_parts(sp, burnt, ladder, rng):
    parts = [(bm, False) for bm in _chassis(sp, burnt)]
    parts += [(bm, False) for bm in _wheels(sp, burnt, rng)]
    parts += [(bm, False) for bm in _arches(sp, "gun_black" if burnt else BODY_RED)]
    parts += [(bm, True) for bm in _cab(sp, burnt)]   # roof-flagged so the top preview sees the seats
    parts += [(bm, False) for bm in _body(sp, burnt, ladder)]
    parts += [(bm, False) for bm in _front(sp, burnt, ladder)]
    parts += [(bm, False) for bm in _rear(sp, burnt, ladder)]
    parts += [(bm, False) for bm in _roof_gear(sp, burnt, ladder, rng)]
    parts += [(bm, False) for bm in _lights(sp, burnt)]
    parts += [(bm, False) for bm in _interior(sp, burnt)]
    return parts


def _build_truck(sp, burnt, ladder):
    def build(model):
        rng = random.Random(41 + (7 if ladder else 0) + (13 if burnt else 0))
        parts = _truck_parts(sp, burnt, ladder, rng)
        add_bbox_collider(model, [bm for bm, _ in parts], inset=0.12)
        for bm, roof in parts:
            model.add(bm, roof=roof)
    return build


def _build_truck_lod1(sp, burnt, ladder):
    """Chunky LOD: one body block, one cab block, wheels as octagon discs, the stripe and the ladder as bars."""
    def build(model):
        w, half, sill, top = sp["w"], sp["half_l"], sp["sill"], sp["roof"]
        cb, cf = sp["cab_back"], sp["cab_front"]
        red = "soot" if burnt else BODY_RED
        model.add(pbox((-w, -half, sill), (w, cb, top), red, bevel=0.02))
        model.add(pbox((-w + 0.05, cb, sill), (w - 0.05, cf, top - 0.10), red, bevel=0.02))
        model.add(box((-w - 0.04, -half - 0.04, 0.30), (w + 0.04, cf - 0.05, sill), "gun_black"))
        for sx in (-w + 0.16, w - 0.16):
            for y in (sp["wheel_front"], sp["wheel_rear"]):
                disc = common.make_frustum(sp["wheel_r"], sp["wheel_r"], -sp["wheel_half"], sp["wheel_half"],
                                           segments=8, color="gun_black")
                common.transform(disc, rot=(0.0, math.pi / 2.0, 0.0), loc=(sx, y, sp["wheel_r"]))
                model.add(disc)
        if not burnt:
            for side in (-1, 1):
                x0, x1 = sorted((side * (w + 0.01), side * (w + 0.03)))
                model.add(box((x0, -half + 0.1, sp["belt"] + 0.02), (x1, cf - 0.2, sp["belt"] + 0.20), STRIPE_WHITE))
        # Windscreen and side glass as flat panes
        model.add(box((-w + 0.18, cf - 0.05, sill + 1.0), (w - 0.18, cf - 0.01, top - 0.16),
                      "gun_black" if burnt else GLASS))
        for side in (-1, 1):
            x0, x1 = sorted((side * (w - 0.02), side * (w + 0.01)))
            model.add(box((x0, cb + 0.2, sill + 1.0), (x1, cb + 1.2, top - 0.26),
                          "gun_black" if burnt else GLASS))
        # Roof kit: hose bed walls and roof ladder, or the turntable and folded aerial
        if not ladder:
            model.add(box((-w + 0.02, -half + 0.2, top), (w - 0.02, cb - 0.35, top + 0.26), "metal"))
            for sx in (-0.34, 0.34):
                model.add(box((sx - 0.05, -half + 0.3, top + 0.24), (sx + 0.05, cb - 0.4, top + 0.34), "metal"))
        else:
            ty = -half + 1.95
            model.add(common.make_frustum(0.95, 0.95, top, top + 0.46, segments=10, center=(0.0, ty),
                                          color="metal"))
            model.add(box((-0.55, ty, top + 0.46), (0.55, ty + 6.6, top + 0.64), "metal"))
        # Light bar
        model.add(box((-0.85, cf - 0.62, top + 0.02), (0.85, cf - 0.40, top + 0.22),
                      "red_paint" if not burnt else "soot"))
        model.add_collider("shell", box((-w - 0.05, -half - 0.12, 0.0), (w + 0.05, cf + 0.08, top), "white"))
    return build


def build_all():
    """Builds the fire-station props and trucks. Returns [(name, ok)]."""
    front = (1.0, 1.0, 0.6)
    specs = [
        ("hose_rack", build_hose_rack),
        ("gear_rack", build_gear_rack),
        ("bay_door_torn", build_bay_door_torn),
        ("kitchen_counter", build_kitchen_counter),
        ("traffic_cone", build_traffic_cone),
        ("hose_line", build_hose_line),
    ]
    results = [(name, common.run_model(name, "props", PROP_BUDGET, fn, view=front)) for name, fn in specs]
    results += build_vehicles()
    return results


def build_vehicles():
    """The three R42-05 fire trucks and their `_lod1` models. Returns [(name, ok)]."""
    view = (1.0, 1.0, 0.5)
    specs = [
        ("fire_truck", PUMPER, False, False),
        ("ladder_truck", LADDER, False, True),
        ("fire_truck_burnt", PUMPER, True, False),
    ]
    out = []
    for name, sp, burnt, ladder in specs:
        out.append((name, common.run_model(name, "vehicles", VEHICLE_BUDGET,
                                           _build_truck(sp, burnt, ladder), top_preview=True, view=view)))
        out.append((name + "_lod1", common.run_model(name + "_lod1", "vehicles", VEHICLE_BUDGET_LOD1,
                                                     _build_truck_lod1(sp, burnt, ladder), view=view)))
    return out


if __name__ == "__main__":
    failed = [n for n, ok in build_all() if not ok]
    if failed:
        print("FIRE PROPS FAILED:", ", ".join(failed))
        sys.exit(1)
    print("FIRE PROPS OK")
