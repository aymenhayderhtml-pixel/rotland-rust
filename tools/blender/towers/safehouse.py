"""R44 safe house props: tarp shack, camp cot, storage chest, travel sign, rope ladder (segment + coil).

Blender frame: X = width, Y = depth, Z = up. The front (door side) is +Y, which is Godot -Z.
Every model is built centred on X and Y with its base on Z = 0, and is exported with the grip
given in SPECS, so the origin is the spot the task table names.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "traces"))

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import graffiti_font  # noqa: E402

PROPS = "props"
TARP_TILT = math.atan2(-0.4, 3.0)  # the tarp falls 0.4 m from back (-Y) to front (+Y) over 3.0 m


def stick(p0, p1, thick, color):
    """Thin square rod from p0 to p1 (rope, wire, guy line, pole)."""
    a, b = Vector(p0), Vector(p1)
    d = b - a
    bm = common.make_box((thick, thick, d.length), color=color)
    quat = d.normalized().to_track_quat("Z", "Y")
    matrix = Matrix.Translation((a + b) / 2) @ quat.to_matrix().to_4x4()
    bmesh.ops.transform(bm, matrix=matrix, verts=list(bm.verts))
    return bm


def _tarp_surface_z(y):
    """Height of the tarp's middle line at depth y (the front edge is at y = +1.5)."""
    return 2.85 - (0.4 / 3.0) * y


def _shack_tarp(model):
    """Three tarp strips (the middle one sags), dirty patches on top, and two guy ropes to the ground."""
    length = math.hypot(3.0, 0.4)
    for x, dip in ((-1.36, 0.0), (0.0, 0.12), (1.36, 0.0)):
        bm = common.make_box((1.36, length, 0.03), color="tarp_blue")
        common.transform(bm, rot=(TARP_TILT, 0.0, 0.0), loc=(x, 0.0, 2.85 - dip))
        model.add(bm)
    for x, y, col in ((-1.0, -0.9, "soot"), (0.9, 0.4, "soot"), (-0.3, 1.1, "tarp_blue"),
                      (0.5, -0.4, "grime_green"), (-1.2, 0.2, "soot")):
        z = _tarp_surface_z(y) + 0.02
        bm = common.make_box((0.3, 0.4, 0.02), color=col)
        common.transform(bm, rot=(TARP_TILT, 0.0, 0.0), loc=(x, y, z))
        model.add(bm)
    # Grommets along the back and front edges, and two rope lines lashed across the tarp
    for i in range(4):
        x = -1.5 + i
        for y in (-1.5, 1.5):
            z = _tarp_surface_z(y) + 0.03
            bm = common.make_box((0.06, 0.06, 0.03), color="metal")
            common.transform(bm, rot=(TARP_TILT, 0.0, 0.0), loc=(x, y, z))
            model.add(bm)
    for x in (-0.7, 0.7):
        model.add(stick((x, -1.5, _tarp_surface_z(-1.5) + 0.03), (x, 1.5, _tarp_surface_z(1.5) + 0.03),
                        0.025, "canvas_khaki"))
    for x in (-1.9, 1.9):
        model.add(stick((x, 1.55, 2.62), (x * 1.35, 2.3, 0.02), 0.03, "canvas_khaki"))
        model.add(common.make_box((0.05, 0.05, 0.12), loc=(x * 1.35, 2.3, 0.06), color="metal"))


def _shack_walls(model):
    """Plank back wall (-Y), two low side walls, the door side (+Y) stays open. Colliders: back and sides."""
    for i in range(8):
        x = -1.9 + 0.2375 + i * 0.475
        h = 2.1 + 0.2 * ((i * 7) % 3) / 2.0
        col = "bark_light" if i % 2 else "bark_dark"
        model.add(common.make_box((0.46, 0.04, h), loc=(x, -1.46, h / 2), color=col))
    for z in (0.5, 1.8):
        model.add(common.make_box((3.9, 0.04, 0.07), loc=(0.0, -1.51, z), color="bark_dark"))
    model.add_collider("back", common.make_box((4.0, 0.1, 2.3), loc=(0.0, -1.46, 1.15), color="white"))
    for sx in (-1.9, 1.9):
        for i in range(6):
            y = -1.25 + i * 0.5
            col = "bark_light" if i % 2 else "bark_dark"
            model.add(common.make_box((0.06, 0.46, 1.1), loc=(sx, y, 0.55), color=col))
        model.add(common.make_box((0.08, 2.8, 0.06), loc=(sx, 0.0, 1.12), color="bark_dark"))
    model.add_collider("side_l", common.make_box((0.08, 2.8, 1.1), loc=(-1.9, 0.0, 0.55), color="white"))
    model.add_collider("side_r", common.make_box((0.08, 2.8, 1.1), loc=(1.9, 0.0, 0.55), color="white"))


def _shack_inside(model):
    """Nail heads on the back planks, sacks and debris on the floor, a pot hung on the back-left post."""
    for i in range(8):
        x = -1.9 + 0.2375 + i * 0.475
        for z in (0.6, 1.6):
            model.add(common.make_box((0.05, 0.02, 0.05), loc=(x, -1.43, z), color="metal"))
    for x, y, r, h in ((-1.4, -1.0, 0.16, 0.30), (-1.1, -0.8, 0.14, 0.26), (1.45, -1.1, 0.15, 0.28)):
        model.add(common.make_frustum(r, r * 0.8, 0.05, 0.05 + h, segments=7, center=(x, y), color="canvas_khaki"))
        model.add(common.make_box((0.05, 0.05, 0.04), loc=(x, y, 0.05 + h + 0.02), color="bark_dark"))
    model.add(common.make_frustum(0.09, 0.08, 1.5, 1.62, segments=8, center=(-1.9, -1.1), color="soot"))
    model.add(common.make_box((0.02, 0.02, 0.2), loc=(-1.9, -1.1, 1.7), color="metal"))
    for x, y, col, size in ((0.4, -0.6, "rust", 0.14), (0.9, -1.2, "bark_light", 0.18),
                            (-0.6, 0.5, "soot", 0.12), (0.2, 0.9, "rock_mid", 0.10)):
        model.add(common.make_box((size, size * 0.8, size * 0.6), loc=(x, y, 0.05 + size * 0.3), color=col))


def _shack_clutter(model):
    """Rope coil on the left front post, a tin can on a wire, floor clutter."""
    for k in range(4):
        z = 1.9 + k * 0.05
        model.add(common.make_frustum(0.05, 0.05, z, z + 0.035, segments=8,
                                      center=(-1.9, 1.52), color="canvas_khaki"))
    model.add(stick((1.9, 1.5, 2.55), (1.9, 1.56, 1.34), 0.012, "metal"))
    model.add(common.make_frustum(0.08, 0.08, 1.2, 1.34, segments=8, center=(1.9, 1.56), color="metal"))
    model.add(common.make_box((0.16, 0.02, 0.02), loc=(1.9, 1.56, 1.34), color="rust"))
    model.add(common.make_box((0.25, 0.15, 0.35), loc=(-0.3, -0.9, 0.225), color="red_paint"))
    model.add(common.make_box((0.06, 0.04, 0.06), loc=(-0.3, -0.9, 0.42), color="gun_black"))
    model.add(common.make_frustum(0.12, 0.10, 0.05, 0.30, segments=8, center=(-1.2, -0.7), color="metal"))
    model.add(common.make_box((0.5, 0.4, 0.35), loc=(1.2, -0.9, 0.225), color="wood_crate"))
    model.add(common.make_box((0.3, 0.3, 0.25), loc=(1.25, -0.85, 0.525), color="bark_light"))
    model.add(common.make_box((0.2, 0.25, 0.08), loc=(0.6, -1.0, 0.09), color="canvas_khaki"))


def build_shack(model):
    """Tarp shack 4.0 x 3.0 x 3.2 m: floor, four posts, sagging tarp, plank back, half side walls, clutter."""
    for i in range(5):
        y = -1.2 + i * 0.6
        col = "bark_dark" if i % 2 else "bark_light"
        model.add(common.make_box((3.9, 0.52, 0.05), loc=(0.0, y, 0.025), color=col))
    for x, y, top in ((-1.9, -1.4, 3.0), (1.9, -1.4, 3.0), (-1.9, 1.4, 2.6), (1.9, 1.4, 2.6)):
        model.add(common.make_box((0.12, 0.12, top), loc=(x, y, top / 2), color="bark_dark"))
        model.add(common.make_box((0.17, 0.17, 0.05), loc=(x, y, top - 0.02), color="rust_dark"))
    _shack_tarp(model)
    _shack_walls(model)
    _shack_inside(model)
    _shack_clutter(model)


def build_bed(model):
    """Camp cot 2.0 x 0.6 x 0.5 m: legs, rails, slats, canvas, grey blanket, pillow. Top at 0.5 m."""
    for x in (-0.92, 0.92):
        for y in (-0.25, 0.25):
            model.add(common.make_box((0.05, 0.05, 0.28), loc=(x, y, 0.14), color="metal"))
    for y in (-0.27, 0.27):
        model.add(common.make_box((2.0, 0.06, 0.05), loc=(0.0, y, 0.305), color="bark_dark"))
    for i in range(11):
        x = -0.95 + i * 0.19
        model.add(common.make_box((0.05, 0.6, 0.04), loc=(x, 0.0, 0.30), color="bark_light"))
    model.add(common.make_box((1.9, 0.56, 0.03), loc=(0.0, 0.0, 0.34), color="canvas_khaki"))
    model.add(common.make_box((0.5, 0.2, 0.01), loc=(0.6, 0.05, 0.362), color="soil_dark"))
    model.add(common.make_box((1.1, 0.5, 0.06), loc=(-0.3, 0.0, 0.385), color="concrete_grey"))
    model.add(common.make_box((0.18, 0.5, 0.08), loc=(0.27, 0.0, 0.395), color="rock_mid"))
    model.add(common.make_box((1.1, 0.06, 0.062), loc=(-0.3, 0.25, 0.386), color="rock_dark"))
    model.add(common.make_box((0.5, 0.36, 0.2), loc=(0.2, 0.0, 0.1), color="canvas_khaki"))  # kit bag under the cot
    model.add(common.make_box((0.04, 0.38, 0.03), loc=(0.2, 0.0, 0.21), color="bark_dark"))
    for x in (-0.85, -0.45, 0.45, 0.85):
        for y in (-0.3, 0.3):
            model.add(common.make_box((0.04, 0.04, 0.03), loc=(x, y, 0.315), color="rock_light"))  # rivets
    model.add(common.make_box((0.42, 0.32, 0.14), loc=(-0.72, 0.0, 0.43), color="bone_white"))
    model.add(common.make_box((0.3, 0.05, 0.02), loc=(-0.72, 0.0, 0.5), color="rock_light"))
    model.add_collider("cot", common.make_box((2.0, 0.6, 0.30), loc=(0.0, 0.0, 0.15), color="white"))


def build_chest(model):
    """Dented olive footlocker 0.9 x 0.6 x 0.6 m: skids, body, lid, metal corners, hasp and padlock, rope wrap, tarp corner."""
    for y in (-0.22, 0.22):  # skids on the floor
        model.add(common.make_box((0.9, 0.05, 0.06), loc=(0.0, y, 0.03), color="gun_black"))
    model.add(common.make_box((0.84, 0.54, 0.38), loc=(0.0, 0.0, 0.25), color="cloth_olive"))
    model.add(common.make_box((0.88, 0.58, 0.16), loc=(0.0, 0.0, 0.52), color="cloth_olive"))
    for sx in (-0.43, 0.43):  # metal corner strips
        for sy in (-0.29, 0.29):
            model.add(common.make_box((0.04, 0.04, 0.54), loc=(sx, sy, 0.33), color="metal"))
    for sx in (-1.0, 1.0):  # side handles: two stands and a grip bar
        for y in (-0.1, 0.1):
            model.add(common.make_box((0.03, 0.03, 0.05), loc=(sx * 0.44, y, 0.47), color="metal"))
        model.add(common.make_box((0.03, 0.26, 0.03), loc=(sx * 0.46, 0.0, 0.51), color="metal"))
        model.add(common.make_box((0.01, 0.12, 0.08), loc=(sx * 0.44, -0.15, 0.36), color="rust"))  # side rust patch
    for x in (-0.36, -0.12, 0.12, 0.36):  # lid rivets on the front
        model.add(common.make_box((0.03, 0.02, 0.03), loc=(x, 0.32, 0.50), color="rock_light"))
    model.add(common.make_box((0.14, 0.02, 0.12), loc=(0.0, 0.30, 0.36), color="metal"))  # hasp plate
    model.add(common.make_box((0.07, 0.05, 0.09), loc=(0.0, 0.335, 0.30), color="brass"))  # padlock body
    for x in (-0.025, 0.025):
        model.add(stick((x, 0.335, 0.345), (x, 0.335, 0.39), 0.012, "metal"))
    model.add(stick((-0.025, 0.335, 0.39), (0.025, 0.335, 0.39), 0.012, "metal"))
    model.add(common.make_box((0.2, 0.01, 0.02), loc=(-0.25, 0.30, 0.16), color="white"))  # stencil marks
    model.add(common.make_box((0.14, 0.01, 0.06), loc=(0.18, 0.30, 0.32), color="white"))
    model.add(common.make_box((0.04, 0.01, 0.04), loc=(0.30, 0.30, 0.16), color="white"))
    for p0, p1 in (((0.45, -0.30, 0.2), (0.45, 0.30, 0.2)), ((0.45, 0.30, 0.2), (-0.45, 0.30, 0.2)),
                   ((-0.45, 0.30, 0.2), (-0.45, -0.30, 0.2)), ((-0.45, -0.30, 0.2), (0.45, -0.30, 0.2))):
        model.add(stick(p0, p1, 0.03, "canvas_khaki"))  # rope wrap around the body
    model.add(common.make_box((0.06, 0.06, 0.06), loc=(0.45, 0.30, 0.2), color="canvas_khaki"))  # knot
    model.add(common.make_box((0.24, 0.2, 0.01), loc=(-0.32, -0.19, 0.625), color="tarp_blue"))  # tarp corner
    model.add(common.make_box((0.12, 0.1, 0.01), loc=(0.15, 0.05, 0.625), color="rust"))  # lid rust
    model.add_collider("body", common.make_box((0.9, 0.6, 0.6), loc=(0.0, 0.0, 0.3), color="white"))


def build_travel_sign(model):
    """Painted board 0.9 x 1.6 x 0.1 m on two posts, a white up arrow and two bars. No letters."""
    for x in (-0.36, 0.36):
        model.add(common.make_box((0.07, 0.07, 1.6), loc=(x, 0.0, 0.8), color="bark_dark"))
    model.add(common.make_box((0.9, 0.1, 0.8), loc=(0.0, 0.0, 1.2), color="wood_crate"))
    model.add(common.make_box((0.84, 0.02, 0.05), loc=(0.0, 0.0, 1.58), color="rust_dark"))
    model.add(common.make_box((0.9, 0.12, 0.04), loc=(0.0, 0.0, 0.8), color="bark_dark"))
    y = 0.052  # just in front of the +Y face
    model.add(graffiti_font.quad_bar(0.0, 0.98, 0.0, 1.46, y, 0.09, "white"))
    model.add(graffiti_font.quad_bar(0.0, 1.5, -0.22, 1.26, y, 0.09, "white"))
    model.add(graffiti_font.quad_bar(0.0, 1.5, 0.22, 1.26, y, 0.09, "white"))
    model.add(graffiti_font.quad_bar(-0.3, 0.9, 0.3, 0.9, y, 0.05, "white"))
    model.add(graffiti_font.quad_bar(-0.3, 0.84, 0.3, 0.84, y, 0.05, "white"))
    for x in (-0.36, 0.36):
        model.add_collider(f"post_{'l' if x < 0 else 'r'}", common.make_box((0.07, 0.07, 1.6), loc=(x, 0.0, 0.8), color="white"))


def build_rope_ladder_seg(model):
    """One 0.5 x 1.0 x 0.1 m rope ladder segment, hung flat. Origin at top centre (z = 1.0)."""
    for x in (-0.2, 0.2):
        for dx in (-0.012, 0.012):
            model.add(stick((x + dx, 0.0, 0.0), (x + dx, 0.0, 1.0), 0.022, "canvas_khaki"))
    for z in (0.17, 0.5, 0.83):
        model.add(common.make_box((0.46, 0.035, 0.035), loc=(0.0, 0.0, z), color="bark_light"))
        for x in (-0.2, 0.2):
            model.add(common.make_box((0.05, 0.05, 0.05), loc=(x, 0.0, z), color="bark_dark"))
    model.add(common.make_box((0.46, 0.1, 0.03), loc=(0.0, 0.0, 0.985), color="rust_dark"))
    model.add(common.make_box((0.06, 0.06, 0.04), loc=(0.0, 0.0, 0.94), color="metal"))


def _unit(x, y, z):
    n = math.sqrt(x * x + y * y + z * z) or 1.0
    return (x / n, y / n, z / n)


def _cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def build_rope_ladder_coil(model):
    """Rope ladder pulled up and rolled: loose rope loops, short rungs sticking out, a hook clamp on the
    parapet with a tie rope. 0.7 x 0.4 x 0.7 m, origin at the base centre, no collider."""
    rng = random.Random(4421)  # fixed seed: the same bundle every build
    for _ in range(12):
        u = _unit(rng.uniform(-1, 1), rng.uniform(-0.3, 0.3), rng.uniform(-1, 1))
        v = _unit(*_cross(_unit(rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)), u))
        r = rng.uniform(0.08, 0.2)
        cx, cy, cz = rng.uniform(-0.1, 0.1), rng.uniform(-0.12, 0.12), 0.3 + rng.uniform(-0.12, 0.12)
        pts = []
        for k in range(7):
            t = 2.0 * math.pi * k / 7
            x = cx + r * (math.cos(t) * u[0] + math.sin(t) * v[0])
            y = cy + r * (math.cos(t) * u[1] + math.sin(t) * v[1])
            z = cz + r * (math.cos(t) * u[2] + math.sin(t) * v[2])
            pts.append((min(0.3, max(-0.3, x)), min(0.16, max(-0.16, y)), min(0.6, max(0.04, z))))
        for k in range(7):
            model.add(stick(pts[k], pts[(k + 1) % 7], 0.04, "canvas_khaki"))
    for _ in range(6):  # wooden rungs poking out of the bundle
        d = _unit(rng.uniform(-1, 1), rng.uniform(-0.3, 0.3), rng.uniform(-0.5, 0.5))
        a = (0.14 * d[0], 0.14 * d[1], 0.3 + 0.14 * d[2])
        b = (0.34 * d[0], 0.34 * d[1], 0.3 + 0.34 * d[2])
        model.add(stick(a, b, 0.03, "bark_light"))
    model.add(common.make_box((0.05, 0.14, 0.08), loc=(0.33, 0.0, 0.6), color="metal"))  # parapet clamp
    model.add(stick((0.33, 0.0, 0.64), (0.33, 0.0, 0.68), 0.02, "metal"))  # hook
    model.add(stick((0.33, 0.0, 0.68), (0.26, 0.0, 0.68), 0.02, "metal"))
    model.add(stick((0.22, 0.0, 0.6), (0.3, 0.0, 0.6), 0.025, "canvas_khaki"))  # tie rope to the hook


# name, budget, build function, grip (origin), preview view (None = default)
SPECS = (
    ("safehouse_shack", 2500, build_shack, (0.0, 0.0, 0.0), (1.0, 1.0, 0.7)),
    ("safehouse_bed", 500, build_bed, (0.0, 0.0, 0.0), None),
    ("safehouse_chest", 500, build_chest, (0.0, 0.0, 0.0), (-1.0, 1.0, 0.6)),
    ("travel_sign", 700, build_travel_sign, (0.0, 0.0, 0.0), (0.5, 1.0, 0.35)),
    ("rope_ladder_seg", 400, build_rope_ladder_seg, (0.0, 0.0, 1.0), (0.8, 1.0, 0.4)),
    ("rope_ladder_coil", 1500, build_rope_ladder_coil, (0.0, 0.0, 0.0), None),
)


def build_all():
    """Returns [(name, ok)] for every safe house model."""
    results = []
    for name, budget, fn, grip, view in SPECS:
        ok = common.run_model(name, PROPS, budget, fn, grip=grip, view=view)
        results.append((name, ok))
    return results
