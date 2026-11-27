"""R42-06 the failed army line: barrier props (`sandbag_wall`, `sandbag_corner`, `razor_wire`,
`concrete_barrier`), the shared army helpers (stencil lettering, sandbags) and the group `build_all()`.

The vehicles live in `army_vehicles.py` and the camp in `army_camp.py`; both import this module for the
shared helpers, so `build_all()` imports them here (not at the top) to keep the import order simple.
Every stencil is plain text or plain numbers: no insignia, no stars.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))
sys.path.insert(0, os.path.join(HERE, "..", "traces"))

import bmesh  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402

import common  # noqa: E402
import graffiti_font  # noqa: E402
from car_parts import _beam  # noqa: E402
from graffiti_font import quad_bar  # noqa: E402
from jam_util import box, extrude_sections, pbox  # noqa: E402

OLIVE = "cloth_olive"
KHAKI = "canvas_khaki"
METAL = "metal"
DARK = "gun_black"

# new: DECISIONS 2026-11-26 (R42 budgets: props 4000, vehicles 12000 / _lod1 1200)
WALL_BUDGET = 4000
WALL_LOD1_BUDGET = 1200
CORNER_BUDGET = 4000
WIRE_BUDGET = 4000
CONCRETE_BUDGET = 4000
CONCRETE_LOD1_BUDGET = 1200

WALL_LEN = 6.0  # new: DECISIONS 2026-11-26 (sandbag_wall is 6 m straight)
WALL_HIGH = 1.3  # new: DECISIONS 2026-11-26 (1.3 m high)

# --- stencil lettering -------------------------------------------------------------------------
# graffiti_font.FONT plus the letters the army stencils need and the digits for unit numbers.
DIGITS = {
    "0": [(0, 0, 0, 1), (0, 1, 1, 1), (1, 1, 1, 0), (1, 0, 0, 0)],
    "1": [(0.3, 0.78, 0.5, 1), (0.5, 1, 0.5, 0), (0.28, 0, 0.72, 0)],
    "2": [(0, 1, 1, 1), (1, 1, 1, 0.5), (1, 0.5, 0, 0.5), (0, 0.5, 0, 0), (0, 0, 1, 0)],
    "3": [(0, 1, 1, 1), (1, 1, 1, 0), (1, 0, 0, 0), (0, 0.45, 1, 0.45)],
    "4": [(0, 1, 0, 0.42), (0, 0.42, 1, 0.42), (1, 1, 1, 0)],
    "5": [(1, 1, 0, 1), (0, 1, 0, 0.5), (0, 0.5, 1, 0.5), (1, 0.5, 1, 0), (1, 0, 0, 0)],
    "6": [(1, 1, 0, 1), (0, 1, 0, 0), (0, 0, 1, 0), (1, 0, 1, 0.5), (1, 0.5, 0, 0.5)],
    "7": [(0, 1, 1, 1), (1, 1, 0.35, 0)],
    "8": [(0, 0.5, 0, 1), (0, 1, 1, 1), (1, 1, 1, 0), (1, 0, 0, 0), (0, 0, 0, 0.5),
          (0, 0.5, 1, 0.5), (1, 0.5, 1, 0)],
    "9": [(1, 0, 1, 1), (1, 1, 0, 1), (0, 1, 0, 0.5), (0, 0.5, 1, 0.5), (1, 0.5, 1, 0)],
    # extra letters (graffiti_font stops at W)
    "P": [(0, 0, 0, 1), (0, 1, 0.85, 1), (0.85, 1, 1, 0.78), (1, 0.78, 0.85, 0.55), (0.85, 0.55, 0, 0.55)],
    "Y": [(0, 1, 0.5, 0.45), (1, 1, 0.5, 0.45), (0.5, 0.45, 0.5, 0)],
    "-": [(0.12, 0.45, 0.88, 0.45)],
    ".": [(0.42, 0, 0.58, 0.14)],
}

FONT = dict(graffiti_font.FONT)
FONT.update(DIGITS)


def stencil_text(text, centre_x, base_z, letter_w, letter_h, y, color):
    """Flat quads (2 tris per stroke) spelling `text` on the XZ plane at height `y`.

    Letters are laid out mirrored in X so the word reads left-to-right for a viewer standing on the
    +Y side of that plane (a face looking at +Y). `stencil_on_side` turns the result onto a side
    face: a rotation keeps the handedness, so the same layout reads correctly from outside there too.
    """
    gap = letter_w * 0.36  # new: DECISIONS 2026-11-26 (stencil letter spacing)
    total = len(text) * letter_w + gap * (len(text) - 1)
    thick = letter_h * 0.13
    x = centre_x - total / 2
    out = []
    for ch in text:
        for u0, v0, u1, v1 in FONT[ch]:
            out.append(quad_bar(-(x + u0 * letter_w), base_z + v0 * letter_h,
                                -(x + u1 * letter_w), base_z + v1 * letter_h, y, thick, color))
        x += letter_w + gap
    return out


def stencil_on_side(text, letter_w, letter_h, face_x, base_z, centre_y, color, outward=1):
    """Stencil letters lying on a vertical side face at x = face_x, reading from outside."""
    bm = common.combine(stencil_text(text, 0.0, base_z, letter_w, letter_h, 0.0, color))
    # the plane normal +Y turns to +X for a -90 deg turn about Z (and to -X for +90)
    common.transform(bm, rot=(0.0, 0.0, -math.pi / 2.0 if outward > 0 else math.pi / 2.0),
                     loc=(face_x, centre_y, 0.0))
    return bm


# --- sandbags ----------------------------------------------------------------------------------
_RING = [(math.cos(2 * math.pi * k / 5 + math.pi / 10), math.sin(2 * math.pi * k / 5 + math.pi / 10))
         for k in range(5)]


def pivot(bm, rot=(0.0, 0.0, 0.0), loc=(0.0, 0.0, 0.0)):
    """Turn a bmesh about its OWN bounding-box centre (then move it by `loc`).

    `common.transform` rotates about the world origin, which throws a part built at its final
    build coordinates off into space; every angled part in the army models goes through here.
    """
    xs = [v.co.x for v in bm.verts]
    ys = [v.co.y for v in bm.verts]
    zs = [v.co.z for v in bm.verts]
    centre = Vector(((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2, (min(zs) + max(zs)) / 2))
    bmesh.ops.translate(bm, vec=-centre, verts=list(bm.verts))
    if rot != (0.0, 0.0, 0.0):
        bmesh.ops.rotate(bm, cent=(0.0, 0.0, 0.0), matrix=Euler(rot, "XYZ").to_matrix(),
                         verts=list(bm.verts))
    bmesh.ops.translate(bm, vec=centre + Vector(loc), verts=list(bm.verts))
    bm.normal_update()
    return bm


def pbox_at(lo, hi, color, rot=(0.0, 0.0, 0.0), loc=(0.0, 0.0, 0.0), bevel=0.015):
    """Bevelled box between two corners, turned about its own centre, then moved by `loc`."""
    return pivot(pbox(lo, hi, color, bevel=bevel), rot=rot, loc=loc)


def beam_at(p0, p1, rot, radius, color=METAL, segments=5):
    """A beam from p0 to p1, then turned about its own midpoint by `rot`."""
    return pivot(_beam(p0, p1, radius, color=color, segments=segments), rot=rot)


def sandbag(length, width, height, loc, rot_z=0.0, color=KHAKI, seed=0):
    """Low-poly plump sandbag (~36 tris): a pinched 5-sided pillow centred on `loc`."""
    hl, hw, hh = length / 2, width / 2, height / 2
    rng = random.Random(seed)
    sections = []
    for fx, sy, sz in ((-1.0, 0.52, 0.30), (-0.7, 0.96, 0.92), (0.0, 1.0, 1.0),
                       (0.7, 0.96, 0.92), (1.0, 0.52, 0.30)):
        jitter = 1.0 + rng.uniform(-0.06, 0.06)
        sections.append((fx * hl, [(y * hw * sy * jitter, z * hh * sz) for y, z in _RING]))
    bm = extrude_sections(sections, color)
    common.transform(bm, rot=(0.0, 0.0, rot_z), loc=loc)
    return bm


def sandbag_row(a0, a1, offset, z, course, rng, colors=(KHAKI, KHAKI, OLIVE), bag_len=0.5, axis="x"):
    """One course of sandbags running along `axis` from a0 to a1, at `offset` on the other axis, height z."""
    out = []
    n = max(1, int(round((a1 - a0) / bag_len)))
    step = (a1 - a0) / n
    for i in range(n):
        a = a0 + step * (i + 0.5) + rng.uniform(-0.02, 0.02)
        b = offset + rng.uniform(-0.02, 0.02)
        col = colors[rng.randrange(len(colors))]
        yaw = rng.uniform(-0.13, 0.13) + (math.pi / 2.0 if axis == "y" else 0.0)
        loc = (a, b, z) if axis == "x" else (b, a, z)
        out.append(sandbag(step * 1.06, 0.36, 0.24, loc, yaw, col, seed=rng.randrange(9999) + course))
    return out


# --- sandbag_wall ------------------------------------------------------------------------------
def _build_sandbag_wall(model, lod=False):
    """6 m straight sandbag wall, 1.3 m high, with a horizontal firing slit through the top third."""
    rng = random.Random(4201)
    courses = 6  # new: DECISIONS 2026-11-26 (6 courses x 0.22 m = 1.32 m)
    dz = 0.22
    parts = []
    if lod:
        for c in range(courses):
            z = 0.02 + c * dz
            parts.append(box((-WALL_LEN / 2, -0.28, z), (WALL_LEN / 2, 0.28, z + 0.24), KHAKI))
    else:
        for c in range(courses):
            z = 0.01 + c * dz
            off = 0.0 if c % 2 == 0 else 0.25
            if c == 4:  # firing slit: skip the middle of this course
                parts += sandbag_row(-WALL_LEN / 2 + off, -1.5, 0.0, z, c, rng)
                parts += sandbag_row(1.5, WALL_LEN / 2, 0.0, z, c, rng)
            else:
                parts += sandbag_row(-WALL_LEN / 2 + off * 0.0, WALL_LEN / 2, 0.0, z, c, rng)
        # slit sill and lintel bags, so the gap reads as a firing slit and not a hole
        for x in (-1.35, 1.35):
            parts.append(sandbag(0.5, 0.34, 0.2, (x, 0.0, 0.01 + 4 * dz + 0.11), 0.0, KHAKI, seed=int(x * 10) + 7))
        # loose spill bags and a couple of splits at the feet
        for k in range(6):
            x = rng.uniform(-2.9, 2.9)
            parts.append(sandbag(0.44, 0.32, 0.16, (x, rng.choice((-1, 1)) * rng.uniform(0.34, 0.48), 0.07),
                                 rng.uniform(-0.5, 0.5), rng.choice((KHAKI, OLIVE)), seed=900 + k))
        # torn bag showing sand, plus a wooden stake bracing the wall
        for x in (-2.2, 1.7):
            parts.append(pbox((x - 0.22, -0.34, 0.28), (x + 0.22, -0.28, 0.5), "dirt", bevel=0.02))
        parts.append(_beam((-2.6, -0.42, 0.0), (-2.6, -0.05, 0.95), 0.035, color="bark_dark", segments=5))
        parts.append(_beam((2.4, -0.42, 0.0), (2.4, -0.05, 0.95), 0.035, color="bark_dark", segments=5))
    for bm in parts:
        model.add(bm)
    model.add_collider("wall", box((-WALL_LEN / 2, -0.3, 0.0), (WALL_LEN / 2, 0.3, WALL_HIGH), "white"))


# --- sandbag_corner ----------------------------------------------------------------------------
def _build_sandbag_corner(model):
    """L-shaped sandbag nest (4 m front arm, 3.4 m side arm) with a machine-gun mount, no gun."""
    rng = random.Random(4202)
    parts = []
    arm_x = 4.0  # new: DECISIONS 2026-11-26 (nest arms 4 m and 3.4 m, 1.1 m high)
    arm_y = 3.4
    courses = 5
    dz = 0.22
    for c in range(courses):
        z = 0.01 + c * dz
        parts += sandbag_row(-arm_x / 2, arm_x / 2, -arm_y / 2, z, c, rng, axis="x", bag_len=0.62)
        parts += sandbag_row(-arm_y / 2, arm_y / 2, -arm_x / 2, z, c + 20, rng, axis="y", bag_len=0.6)
    # machine-gun mount: a steel pintle on a plate, a cradle and an empty mount ring (no gun)
    parts.append(pbox((-0.9, -0.9, 0.0), (0.9, 0.9, 0.05), METAL, bevel=0.01))
    parts.append(_beam((0.0, 0.0, 0.05), (0.0, 0.0, 0.95), 0.05, color=METAL, segments=6))
    parts.append(common.make_frustum(0.16, 0.13, 0.95, 1.02, segments=8, color="rock_dark"))
    parts.append(_beam((-0.26, 0.0, 1.02), (0.26, 0.0, 1.02), 0.035, color=METAL, segments=6))  # cradle yoke
    for sx in (-1, 1):
        parts.append(_beam((sx * 0.24, 0.0, 1.02), (sx * 0.24, 0.0, 1.2), 0.03, color=METAL, segments=5))
    parts.append(pbox((-0.13, -0.1, 1.18), (0.13, 0.1, 1.26), "rock_dark", bevel=0.01))
    # ammo cans and a barrel bag inside the nest
    for x, y, rot in ((-1.0, -0.35, 0.2), (-0.45, -0.55, -0.1), (0.55, 1.1, 0.5)):
        parts.append(pbox_at((x - 0.17, y - 0.1, 0.0), (x + 0.17, y + 0.1, 0.24), OLIVE,
                             rot=(0.0, 0.0, rot), bevel=0.02))
        parts.append(pbox_at((x - 0.1, y - 0.11, 0.1), (x + 0.1, y + 0.11, 0.14), METAL,
                             rot=(0.0, 0.0, rot), bevel=0.0))
    parts.append(pbox((0.1, 1.0, 0.0), (0.95, 1.3, 0.26), KHAKI, bevel=0.05))  # barrel bag
    for bm in parts:
        model.add(bm)
    model.add_collider("arm_x", box((-arm_x / 2, -arm_y / 2 - 0.3, 0.0), (arm_x / 2, -arm_y / 2 + 0.3, 1.1),
                                    "white"))
    model.add_collider("arm_y", box((-arm_x / 2 - 0.3, -arm_y / 2, 0.0), (-arm_x / 2 + 0.3, arm_y / 2, 1.1),
                                    "white"))
    model.add_collider("mount", box((-0.9, -0.9, 0.0), (0.9, 0.9, 1.26), "white"))


# --- razor_wire --------------------------------------------------------------------------------
def _coil(y0, y1, radius, centre_x, centre_z, color, turns=26, samples=7):
    """One concertina coil: a thin ribbon swept along a helix (2 tris per segment, no caps)."""
    bm = common.new_bm()
    pts = []
    n = turns * samples
    for i in range(n + 1):
        t = i / n
        a = t * turns * 2 * math.pi
        y = y0 + (y1 - y0) * t
        r = radius * (1.0 + 0.10 * math.sin(a * 0.5))  # slight wobble so it is not a perfect tube
        pts.append((centre_x + math.cos(a) * r, y, centre_z + math.sin(a) * r))
    prev = None
    for i, (px, py, pz) in enumerate(pts):
        # ribbon is one quad wide along Y so it reads as a wire from any angle
        w = 0.035
        ring = [bm.verts.new((px, py - w, pz)), bm.verts.new((px, py + w, pz))]
        if prev is not None:
            f = bm.faces.new([prev[0], prev[1], ring[1], ring[0]])
            common.paint_faces(bm, [f], color)
        prev = ring
    return bm


def _build_razor_wire(model):
    """8 m concertina run on pickets: 3 coils stacked like a pyramid, no collider (it is only wire)."""
    length = 8.0  # new: DECISIONS 2026-11-26 (razor_wire run 8 m, 3 coils, pyramid stack)
    parts = []
    for cx, cz in ((-0.42, 0.42), (0.42, 0.42), (0.0, 1.02)):
        parts.append(_coil(-length / 2, length / 2, 0.42, cx, cz, METAL))
    for cx, cz in ((-0.42, 0.42), (0.42, 0.42), (0.0, 1.02)):
        parts.append(_coil(-length / 2 + 0.1, length / 2 - 0.1, 0.40, cx, cz, "chrome", turns=24))
    # pickets every 2 m with a hooked top, plus the tie wire through them
    for k in range(5):
        y = -length / 2 + k * (length / 4)
        parts.append(_beam((0.0, y, 0.0), (0.0, y, 1.55), 0.032, color="rust", segments=5))
        parts.append(_beam((0.0, y, 1.55), (0.0, y + 0.16, 1.62), 0.03, color="rust", segments=5))
        parts.append(pbox((-0.09, y - 0.02, 0.0), (0.09, y + 0.02, 0.06), "rust_dark", bevel=0.01))
    parts.append(_beam((0.0, -length / 2, 1.42), (0.0, length / 2, 1.42), 0.012, color="chrome", segments=4))
    # tangled wire lying on the ground at the ends and a few cut loops
    for k, (x, y) in enumerate(((-0.9, -3.4), (0.7, 3.1), (0.2, -1.2), (-0.6, 2.0))):
        parts.append(_coil(y - 0.35, y + 0.35, 0.3, x, 0.06, METAL, turns=5, samples=6))
    combined = common.combine(parts)
    model.add(combined)


# --- concrete_barrier ---------------------------------------------------------------------------
def _build_concrete_barrier(model, lod=False):
    """Precast T-wall: 2 m wide, 3 m tall, a broad base flange, chipped edges and stencilled numbers."""
    w = 2.0  # new: DECISIONS 2026-11-26 (T-wall 2 m wide, 3 m tall, 0.7 m base)
    height = 3.0
    parts = []
    if lod:
        parts.append(box((-w / 2, -0.35, 0.0), (w / 2, 0.35, 0.3), "concrete_grey"))
        parts.append(box((-w / 2, -0.25, 0.3), (w / 2, 0.25, height), "concrete_grey"))
    else:
        # base flange and the tapered stem
        parts.append(pbox((-w / 2, -0.35, 0.0), (w / 2, 0.35, 0.3), "concrete_grey", bevel=0.03))
        parts.append(pbox((-w / 2 + 0.02, -0.27, 0.28), (w / 2 - 0.02, 0.27, 0.42), "concrete_stain", bevel=0.02))
        parts.append(pbox((-w / 2 + 0.03, -0.25, 0.42), (w / 2 - 0.03, 0.25, 2.2), "concrete_grey", bevel=0.03))
        parts.append(pbox((-w / 2 + 0.06, -0.2, 2.2), (w / 2 - 0.06 + 0.0, 0.2, height), "concrete_grey",
                          bevel=0.03))
        # chamfered top cap and lifting loops
        parts.append(pbox((-w / 2 + 0.1, -0.16, height - 0.04), (w / 2 - 0.1, 0.16, height + 0.02),
                          "limestone_dark", bevel=0.02))
        for x in (-0.62, 0.62):
            parts.append(_beam((x - 0.09, 0.0, height), (x, 0.0, height + 0.13), 0.018, color="rust", segments=4))
            parts.append(_beam((x, 0.0, height + 0.13), (x + 0.09, 0.0, height), 0.018, color="rust", segments=4))
        # chips knocked out of the edges, damp streaks, weeds at the foot
        rng = random.Random(4203)
        for _ in range(7):
            x = rng.uniform(-0.85, 0.85)
            z = rng.uniform(0.4, 2.7)
            parts.append(box((x - 0.09, 0.24, z), (x + 0.09, 0.26, z + 0.13), "rock_dark"))
        for _ in range(5):
            x = rng.uniform(-0.8, 0.8)
            parts.append(box((x, 0.245, 0.32), (x + 0.11, 0.255, rng.uniform(1.0, 2.6)), "concrete_stain"))
        for k in range(4):
            x = -0.8 + k * 0.55
            parts.append(_beam((x, 0.3, 0.0), (x + 0.05, 0.36, 0.34), 0.02, color="grass", segments=4))
        # stencilled unit number, plain digits only
        for bm in stencil_text("07", 0.0, 1.05, 0.34, 0.5, 0.26, "soot"):
            parts.append(bm)
        for bm in stencil_text("42", 0.0, 0.55, 0.22, 0.32, 0.26, "soot"):
            parts.append(bm)
    for bm in parts:
        model.add(bm)
    model.add_collider("wall", box((-w / 2, -0.34, 0.0), (w / 2, 0.34, height), "white"))


# --- group -------------------------------------------------------------------------------------
def build_all():
    """Builds every R42-06 army model (barriers + vehicles + camp). Returns [(name, ok)]."""
    import army_camp  # noqa: E402  (lazy: army_camp imports this module for the helpers)
    import army_vehicles  # noqa: E402

    results = []
    results += army_vehicles.build_all()
    results += army_camp.build_all()
    results += [
        ("sandbag_wall", common.run_model("sandbag_wall", "props", WALL_BUDGET, _build_sandbag_wall,
                                          view=(0.7, -1.0, 0.5))),
        ("sandbag_wall_lod1", common.run_model("sandbag_wall_lod1", "props", WALL_LOD1_BUDGET,
                                               lambda m: _build_sandbag_wall(m, lod=True))),
        ("sandbag_corner", common.run_model("sandbag_corner", "props", CORNER_BUDGET,
                                            _build_sandbag_corner, top_preview=True)),
        ("razor_wire", common.run_model("razor_wire", "props", WIRE_BUDGET, _build_razor_wire,
                                        view=(0.9, -1.0, 0.45))),
        ("concrete_barrier", common.run_model("concrete_barrier", "props", CONCRETE_BUDGET,
                                              _build_concrete_barrier, view=(0.7, -1.0, 0.4))),
        ("concrete_barrier_lod1", common.run_model("concrete_barrier_lod1", "props", CONCRETE_LOD1_BUDGET,
                                                   lambda m: _build_concrete_barrier(m, lod=True))),
    ]
    return results
