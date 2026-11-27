"""Shared kit for the R42 big places (hospital, police station, fire station).

Build coordinates are metres: +X right, +Y front (the street side), +Z up, ground at Z = 0.
Every model is exported with `grip=(0, 0, 0)`, so the origin stays the bottom centre of the
footprint instead of the bounding-box centre (canopies and yards stick out, that would shift it).

Ground floors are walk-in: `floor_slab` lays the raised slab, `wall` cuts door and window gaps and
puts one convex collider on every solid wall piece and none in the gaps.

Function names are stable: R42-04 (police) and R42-05 (fire station) import this module.
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
           os.path.join(BLENDER, "overgrowth"),
           os.path.join(BLENDER, "towers"),
           os.path.join(BLENDER, "traces")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import kit  # noqa: E402
import og_util  # noqa: E402
import roof_parts  # noqa: E402
from graffiti_font import FONT  # noqa: E402

# Shared dimensions, R42-common "every R42 building"
WALL_T = 0.3        # real wall thickness
FLOOR_Z = 0.15      # raised walkable slab top
DOOR_H = 2.4        # door gap height
DOOR_W = 1.8        # smallest walkable door gap width
CEIL_Z = 4.2        # ground floor ceiling

WALL_COLOR = "white"
BAND_COLOR = "rock_light"
PLINTH_COLOR = "concrete_grey"
FLOOR_COLOR = "concrete_grey"
FRAME_COLOR = "metal"
GLASS_DARK = "glass_dark"

WINDOW_STATES = ("dark", "broken", "boarded")

# Painted letters beyond the graffiti set, in the same unit cell (new: DECISIONS 2026-11-27, R42 signs).
FONT_EXTRA = {
    "Y": [(0.0, 1.0, 0.5, 0.45), (1.0, 1.0, 0.5, 0.45), (0.5, 0.45, 0.5, 0.0)],
    "P": [(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.72, 1.0), (0.72, 1.0, 1.0, 0.74),
          (1.0, 0.74, 0.72, 0.5), (0.72, 0.5, 0.0, 0.5)],
    "X": [(0.0, 1.0, 1.0, 0.0), (1.0, 1.0, 0.0, 0.0)],
    "J": [(1.0, 1.0, 1.0, 0.28), (1.0, 0.28, 0.66, 0.0), (0.66, 0.0, 0.28, 0.0),
          (0.28, 0.0, 0.0, 0.3)],
}
LETTERS = dict(FONT)
LETTERS.update(FONT_EXTRA)


# ---------------------------------------------------------------- primitives


def opening(x0, x1, y0, y1, z0, z1):
    """Opposite corners of a gap, ready for `wall(..., openings=[...])`."""
    return ((min(x0, x1), min(y0, y1), min(z0, z1)),
            (max(x0, x1), max(y0, y1), max(z0, z1)))


def box(k, lo, hi, color, roof=False):
    """Visual-only box (trim, decal, prop baked into the building)."""
    k.box(lo, hi, color, roof=roof)


def box_bm(lo, hi, color):
    """A standalone box bmesh between two opposite corners (for combining small sub-assemblies)."""
    size = [hi[i] - lo[i] for i in range(3)]
    loc = [(hi[i] + lo[i]) / 2.0 for i in range(3)]
    return common.make_box(size, loc=loc, color=color)


def solid(k, lo, hi, color, base="blk"):
    """Visual box plus one collider of the same size."""
    k.solid(lo, hi, color, base=base)


def collider(k, lo, hi, base="blk"):
    k.collider(lo, hi, base)


def wall(k, lo, hi, openings=(), color=WALL_COLOR, base="wall"):
    """Wall box with gaps: one collider per solid piece, none inside a gap."""
    k.wall(lo, hi, openings=list(openings), color=color, base=base)


def frame(k, opening, face, sign, color=FRAME_COLOR, t=0.09, depth=0.05, sill=True):
    """Visual trim round a door or window opening (kit.Kit.frame)."""
    k.frame(opening, face, sign, color=color, t=t, depth=depth, sill=sill)


def floor_slab(k, x0, y0, x1, y1, z_top=FLOOR_Z, color=FLOOR_COLOR, base="floor"):
    """Raised walkable slab over the footprint, with a floor collider on top of the ground."""
    k.solid((min(x0, x1), min(y0, y1), 0.0), (max(x0, x1), max(y0, y1), z_top), color, base=base)


def floor_tiles(k, x0, y0, x1, y1, z, tile=2.0, color_a="bone_white", color_b="rock_light", inset=0.03,
                variety=0.0, seed=0):
    """Flat floor pattern (2 tris per tile, no colliders). `z` should be a few mm above the slab.
    With `variety` above 0 only that fraction of tiles (chosen by `seed`) use `color_b`, so the
    floor reads as worn vinyl rather than a checkerboard."""
    rng = random.Random(seed)
    sides = {}
    nx = max(1, int(round((x1 - x0) / tile)))
    ny = max(1, int(round((y1 - y0) / tile)))
    tw = (x1 - x0) / nx
    th = (y1 - y0) / ny
    for i in range(nx):
        for j in range(ny):
            if variety > 0.0:
                col = color_b if rng.random() < variety else color_a
            else:
                col = color_a if (i + j) % 2 == 0 else color_b
            if col not in sides:
                sides[col] = common.new_bm()
            ax, bx = x0 + i * tw, x0 + (i + 1) * tw
            ay, by = y0 + j * th, y0 + (j + 1) * th
            common.add_quad(sides[col], [(ax + inset, ay + inset, z), (bx - inset, ay + inset, z),
                                         (bx - inset, by - inset, z), (ax + inset, by - inset, z)],
                            (0.0, 0.0, 1.0), col)
    for bm in sides.values():
        k.model.add(bm)


def floor_decal(k, points, z, color):
    """One flat quad decal on the floor (a stain, a painted line, a parking mark)."""
    bm = common.new_bm()
    rounded = [(p[0], p[1], z) for p in points]
    common.add_quad(bm, rounded, (0.0, 0.0, 1.0), color)
    k.model.add(bm)


def patch(k, cx, cy, rx, ry, z, color, seed=0, jitter=0.35, sides=8):
    """Irregular flat blob decal (a dark stain, a moss patch) from a ring of jittered points."""
    rng = random.Random(seed)
    bm = common.new_bm()
    verts = []
    for i in range(sides):
        a = 2.0 * math.pi * i / sides
        f = 1.0 + rng.uniform(-jitter, jitter)
        verts.append(bm.verts.new((cx + math.cos(a) * rx * f, cy + math.sin(a) * ry * f, z)))
    bm.faces.new(verts)
    common.paint(bm, color)
    k.model.add(bm)


# ---------------------------------------------------------------- facade


def _face_point(face_coord, axis, sign, u, z, d=0.0):
    """Point on a wall face. axis 1 = a +-Y wall (so `u` is the world x), axis 0 = a +-X wall
    (so `u` is the world y); `d` sticks out of the wall along `sign`."""
    if axis == 1:
        return (u, face_coord + sign * d, z)
    return (face_coord + sign * d, u, z)


def face_box(k, face_coord, axis, sign, u0, u1, z0, z1, d0, d1, color, roof=False):
    """Box drawn in wall-face coordinates (see `_face_point`)."""
    a = _face_point(face_coord, axis, sign, u0, z0, d0)
    b = _face_point(face_coord, axis, sign, u1, z1, d1)
    lo = tuple(min(a[i], b[i]) for i in range(3))
    hi = tuple(max(a[i], b[i]) for i in range(3))
    k.box(lo, hi, color, roof=roof)


def window(k, face_coord, axis, sign, u_center, z0, z1, state="dark", width=2.4,
           roof=False, frame_color=FRAME_COLOR):
    """One window on a closed facade: recessed pane, proud frame, sill, and its damage state.

    States: "dark" (dirty glass), "broken" (empty soot hole with shards), "boarded" (planks nailed over).
    """
    hw = width / 2.0
    pane = "soot" if state == "broken" else GLASS_DARK
    t = 0.10
    # Pane, slightly recessed into the wall
    face_box(k, face_coord, axis, sign, u_center - hw, u_center + hw, z0, z1, -0.06, 0.02, pane, roof)
    # Proud surround: jambs, head, sill
    face_box(k, face_coord, axis, sign, u_center - hw - t, u_center - hw, z0, z1, 0.0, 0.06, frame_color, roof)
    face_box(k, face_coord, axis, sign, u_center + hw, u_center + hw + t, z0, z1, 0.0, 0.06, frame_color, roof)
    face_box(k, face_coord, axis, sign, u_center - hw - t, u_center + hw + t, z1, z1 + t, 0.0, 0.07, frame_color, roof)
    face_box(k, face_coord, axis, sign, u_center - hw - t * 1.3, u_center + hw + t * 1.3,
             z0 - t, z0, 0.0, 0.10, "rock_light", roof)
    # Mullion and transom split the opening into four panes
    face_box(k, face_coord, axis, sign, u_center - 0.035, u_center + 0.035, z0, z1, 0.0, 0.045, frame_color, roof)
    mid = (z0 + z1) * 0.5
    face_box(k, face_coord, axis, sign, u_center - hw, u_center + hw, mid - 0.035, mid + 0.035,
             0.0, 0.045, frame_color, roof)
    if state == "boarded":
        for row, frac in enumerate((0.20, 0.50, 0.80)):
            z = z0 + (z1 - z0) * frac
            shift = 0.05 if row % 2 else -0.05
            face_box(k, face_coord, axis, sign, u_center - hw - 0.10 + shift, u_center + hw + 0.10 + shift,
                     z - 0.07, z + 0.07, 0.04, 0.10, "bark_light", roof)
    elif state == "broken":
        # Two crossing crack bars and jagged corner shards left in the frame
        diag = math.hypot(width, z1 - z0) * 0.42
        thick = z1 - z0
        for s in (-1.0, 1.0):
            p0 = _face_point(face_coord, axis, sign, u_center - s * width * 0.32,
                             (z0 + z1) * 0.5 - thick * 0.32, 0.03)
            p1 = _face_point(face_coord, axis, sign, u_center + s * width * 0.32,
                             (z0 + z1) * 0.5 + thick * 0.32, 0.03)
            normal = (0.0, sign, 0.0) if axis == 1 else (sign, 0.0, 0.0)
            _stroke(k, p0, p1, normal, 0.05, diag * 0.02, GLASS_DARK, roof=roof)
        face_box(k, face_coord, axis, sign, u_center - hw, u_center - hw * 0.34, z0, z0 + (z1 - z0) * 0.22,
                 0.02, 0.05, GLASS_DARK, roof)
        face_box(k, face_coord, axis, sign, u_center + hw * 0.40, u_center + hw, z1 - (z1 - z0) * 0.18,
                 z1, 0.02, 0.05, GLASS_DARK, roof)
    return


def window_band(k, face_coord, axis, sign, u_centres, z0, z1, states=None, width=2.4, roof=False):
    """A row of windows; `states` is a list (or None for dark/broken/boarded in a repeating rhythm)."""
    rhythm = ("dark", "dark", "broken", "dark", "boarded", "dark", "broken")
    for i, u in enumerate(u_centres):
        state = states[i] if states else rhythm[i % len(rhythm)]
        window(k, face_coord, axis, sign, u, z0, z1, state=state, width=width, roof=roof)


def bars_panel(k, face_coord, axis, sign, u0, u1, z0, z1, spacing=0.22, color="metal",
               thick=0.03, depth=0.05, roof=False):
    """Barred panel (mesh cage wall, jail bars): uprights every `spacing` plus two rails."""
    u = u0 + spacing
    while u < u1 - 0.02:
        face_box(k, face_coord, axis, sign, u - thick / 2, u + thick / 2, z0, z1, 0.0, depth, color, roof)
        u += spacing
    for zr in (z0 + (z1 - z0) * 0.5, z1 - 0.06):
        face_box(k, face_coord, axis, sign, u0, u1, zr - thick / 2, zr + thick / 2, 0.0, depth, color, roof)


# ---------------------------------------------------------------- letters


def _stroke(k, p0, p1, normal, thick, depth, color, roof=False):
    """One lettering bar between two 3D points, `thick` tall and `depth` deep along the wall normal."""
    d = Vector(p1) - Vector(p0)
    if d.length < 1e-5:
        return
    xd = d.normalized()
    yd = Vector(normal).normalized()
    zd = xd.cross(yd)
    if zd.length < 1e-5:
        return
    bm = common.make_box((d.length, depth, thick), color=color)
    mat = (Matrix.Translation((Vector(p0) + Vector(p1)) / 2)
           @ Matrix((xd, yd, zd.normalized())).transposed().to_4x4())
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    k.model.add(bm, roof=roof)


def letters_bms(text, center, letter_w, letter_h, normal, color="red_paint", depth=0.04,
                thick_scale=1.0, mirror=False):
    """Lettering as free-standing parts: `center` is the middle of the text on the plane,
    `normal` the direction the text faces. Returns a list of bmeshes (use for a prop's own signs)."""
    gap = letter_w * 0.35
    space = letter_w * 0.6
    total = sum(space if ch == " " else letter_w for ch in text) + gap * (len(text) - 1)
    thick = letter_h * 0.12 * thick_scale
    n = Vector(normal).normalized()
    across = Vector((0.0, 0.0, 1.0)).cross(n).normalized()   # the reader's left-to-right
    flip = -1.0 if mirror else 1.0
    out = []
    u = -total / 2.0
    base = Vector(center)
    for ch in text:
        if ch == " ":
            u += space + gap
            continue
        strokes = LETTERS.get(ch.upper())
        if strokes is None:
            u += letter_w + gap
            continue
        for s0, t0, s1, t1 in strokes:
            p0 = base + across * (flip * (u + s0 * letter_w)) + Vector((0.0, 0.0, t0 * letter_h))
            p1 = base + across * (flip * (u + s1 * letter_w)) + Vector((0.0, 0.0, t1 * letter_h))
            out.append(_bar_bm(p0, p1, n, thick, depth, color))
        u += letter_w + gap
    return out


def _bar_bm(p0, p1, normal, thick, depth, color):
    d = Vector(p1) - Vector(p0)
    if d.length < 1e-5:
        return common.new_bm()
    xd = d.normalized()
    yd = Vector(normal).normalized()
    zd = xd.cross(yd)
    if zd.length < 1e-5:
        return common.new_bm()
    bm = common.make_box((d.length, depth, thick), color=color)
    mat = (Matrix.Translation((Vector(p0) + Vector(p1)) / 2)
           @ Matrix((xd, yd, zd.normalized())).transposed().to_4x4())
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    return bm


def sign_text(k, text, face_coord, axis, sign, u_center, z_base, letter_w, letter_h,
              color="red_paint", depth=0.04, thick_scale=1.0, roof=False, d=0.03):
    """Painted letters on a wall face, reading left to right as seen from outside."""
    normal = (0.0, sign, 0.0) if axis == 1 else (sign, 0.0, 0.0)
    base = _face_point(face_coord, axis, sign, u_center, z_base, d)
    for bm in letters_bms(text, base, letter_w, letter_h, normal, color=color, depth=depth,
                          thick_scale=thick_scale):
        k.model.add(bm, roof=roof)


def sign_plate(k, face_coord, axis, sign, u_center, u_half, z0, z1, color="white", depth=0.08, roof=False):
    """Backing board for a sign, standing `depth` proud of the wall."""
    face_box(k, face_coord, axis, sign, u_center - u_half, u_center + u_half, z0, z1, 0.0, depth, color, roof)


def cross_symbol(k, face_coord, axis, sign, u_center, z_center, size, color="red_paint",
                 depth=0.05, roof=False):
    """Red cross: two overlapping bars, the horizontal one a hair prouder so they never z-fight."""
    arm = size * 0.34
    face_box(k, face_coord, axis, sign, u_center - arm, u_center + arm, z_center - size, z_center + size,
             0.02, 0.02 + depth, color, roof)
    face_box(k, face_coord, axis, sign, u_center - size, u_center + size, z_center - arm, z_center + arm,
             0.02, 0.02 + depth + 0.006, color, roof)


# ---------------------------------------------------------------- overgrowth


def grime(k, face_coord, axis, sign, u_center, z0, z1, seed=0, width=1.6, streaks=5, roof=False):
    """Dirty drips and moss runs on a wall, 2-4 cm proud so they never z-fight.
    Narrow and tall, so they read as water streaks, not as a mosaic."""
    rng = random.Random(seed)
    colors = ("concrete_stain", "concrete_stain", "soot", "concrete_stain", "grime_green", "rust_dark")
    for i in range(streaks):
        u = u_center + rng.uniform(-width / 2.0, width / 2.0)
        zt = z1 - rng.uniform(0.0, (z1 - z0) * 0.30)
        zb = max(z0, zt - rng.uniform(0.6, (z1 - z0) * 1.05))
        w = rng.uniform(0.05, 0.17)
        face_box(k, face_coord, axis, sign, u - w / 2, u + w / 2, zb, zt,
                 rng.uniform(0.015, 0.022), rng.uniform(0.026, 0.034),
                 colors[i % len(colors)], roof)
        # A wider, paler wash at the top where the water starts
        if i % 2 == 0:
            g = w * rng.uniform(1.6, 2.4)
            face_box(k, face_coord, axis, sign, u - g / 2, u + g / 2, zt - (z1 - z0) * 0.28, zt,
                     0.012, 0.022, "concrete_stain", roof)


def vines(k, face_coord, axis, sign, u_center, width, z0, z1, seed=0, leaves=110, roof=False):
    """Climbing vine patch on a wall: woody stems plus a leaf mat 2-5 cm off the face."""
    rng = random.Random(seed)
    # Climbing stems
    for i in range(4):
        u = u_center + (i / 3.0 - 0.5) * width * 0.7
        z = z0
        while z < z1 - 0.3:
            step = rng.uniform(0.55, 0.9)
            u2 = u + rng.uniform(-0.14, 0.14)
            p0 = _face_point(face_coord, axis, sign, u, z, 0.025)
            p1 = _face_point(face_coord, axis, sign, u2, min(z1, z + step), 0.03)
            normal = (0.0, sign, 0.0) if axis == 1 else (sign, 0.0, 0.0)
            _stroke(k, p0, p1, normal, 0.045, 0.03, "bark_dark", roof=roof)
            u, z = u2, z + step
    # Leaf mat, thicker near the bottom
    for i in range(leaves):
        t = rng.random() ** 0.8
        u = u_center + rng.uniform(-width / 2.0, width / 2.0)
        z = z0 + (z1 - z0) * (1.0 - t)
        d = rng.uniform(0.02, 0.05)
        col = ("leaf_dark", "leaf_mid", "leaf_mid", "leaf_light", "grime_green")[i % 5]
        leaf = og_util.make_leaf_diamond(rng.uniform(0.16, 0.30), rng.uniform(0.18, 0.34), 0.02, color=col)
        loc = _face_point(face_coord, axis, sign, u, z, d)
        yaw = rng.uniform(-0.5, 0.5) + (0.0 if axis == 1 else -sign * math.pi / 2.0)
        common.transform(leaf, rot=(rng.uniform(-0.45, 0.45), rng.uniform(-0.45, 0.45), yaw), loc=loc)
        k.model.add(leaf, roof=roof)


# ---------------------------------------------------------------- roof


def ac_unit(k, x, y, z0):
    """Rooftop HVAC unit with louvres and a fan ring."""
    roof_parts.ac_unit(k, x, y, z0)


def roof_vent(k, x, y, z0, h=1.3):
    roof_parts.vent_stack(k, x, y, z0, h)


def roof_antenna(k, x, y, z0, h=3.4):
    roof_parts.antenna(k, x, y, z0, h)


def helipad(k, cx, cy, z, radius=5.0, color="white", roof=False):
    """Painted helipad: a dashed ring, an inner circle and a big H (flat quads, 2 tris each)."""
    bm = common.new_bm()
    seg = 24
    for i in range(seg):
        if i % 2:
            continue
        a0 = 2.0 * math.pi * i / seg
        a1 = 2.0 * math.pi * (i + 2) / seg
        inner = radius * 0.90
        pts = [(cx + math.cos(a0) * inner, cy + math.sin(a0) * inner, z),
               (cx + math.cos(a1) * inner, cy + math.sin(a1) * inner, z),
               (cx + math.cos(a1) * radius, cy + math.sin(a1) * radius, z),
               (cx + math.cos(a0) * radius, cy + math.sin(a0) * radius, z)]
        common.add_quad(bm, pts, (0.0, 0.0, 1.0), color)
    k.model.add(bm, roof=roof)
    arm_h, bar_w, gap = radius * 0.55, radius * 0.13, radius * 0.30
    for sx in (-1.0, 1.0):
        box(k, (cx + sx * gap - bar_w / 2, cy - arm_h, z), (cx + sx * gap + bar_w / 2, cy + arm_h, z + 0.02),
            color, roof=roof)
    box(k, (cx - gap - bar_w / 2, cy - bar_w / 2, z), (cx + gap + bar_w / 2, cy + bar_w / 2, z + 0.02),
        color, roof=roof)


# ---------------------------------------------------------------- markers


def spawn(k, index, x, y, z=FLOOR_Z):
    """A `spawn_N` empty: a zombie spawn point inside a room or at a door."""
    k.empty(f"spawn_{index}", (x, y, z))


def marker(k, label, x, y, z=FLOOR_Z):
    k.empty(label, (x, y, z))
