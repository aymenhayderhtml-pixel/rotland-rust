"""R42-04 `metro_police`: the big walk-in police station of New Halden, 36 x 28 m, 3 floors.

Build coordinates: +X right, +Y front (the street side), +Z up, ground at Z = 0.
Footprint x -18..18, y -14..14. Ground floor (0 .. 4.2 m) is walk-in: a raised 0.15 m slab,
0.3 m walls, one collider per solid wall piece and none in the door gaps.

Outside: wide steps to the double doors under a POLICE sign, two blue lamp posts, barred lower
windows all round, an open sally-port gate at the back, a flagpole without a flag, roof antennas.
Inside: entrance lobby with the front desk behind a broken glass screen, a waiting room, an open
office with six desks, an evidence room, a cell corridor with three barred cells, a locker room
and the armoury behind a forced mesh cage.

Upper floors are a closed facade (dark / broken / boarded windows, grime, vines). All of them are
added with `roof=True`, so the top preview is the cut-away of the ground floor.

Run: `flock /tmp/rotland-blender.lock blender -b --python-expr
      "import sys; sys.path.insert(0,'tools/blender'); from places import police; police.build_all()"`
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
for _p in (HERE, os.path.join(HERE, ".."), os.path.join(HERE, "..", "buildings"),
           os.path.join(HERE, "..", "metro"), os.path.join(HERE, "..", "traces")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import bpy  # noqa: E402
import common  # noqa: E402
import kit  # noqa: E402
import place_kit as pk  # noqa: E402
from jam_util import pbox  # noqa: E402
import police_props as pp  # noqa: E402

NAME, CATEGORY, BUDGET, LOD_BUDGET = "metro_police", "buildings", 90000, 6000

# Footprint and storey heights (task file: 36 x 28 m, ground 4.2 m + 2 x 3.8 m)
X0, X1, Y0, Y1 = -18.0, 18.0, -14.0, 14.0
T = pk.WALL_T                 # 0.3 m wall thickness
FZ = pk.FLOOR_Z               # 0.15 m walkable slab
CEIL = 4.05                   # ground-floor ceiling underside
G_TOP = 4.2                   # top of the ground storey
F2_TOP = 8.0                  # top of the second storey
F3_TOP = 11.8                 # top of the third storey
ROOF_TOP = 12.0
PARAPET = 12.75               # parapet top

IX0, IX1, IY0, IY1 = X0 + T, X1 - T, Y0 + T, Y1 - T  # inner faces -17.7 .. 17.7 / -13.7 .. 13.7

# Room plan (see the Worker notes for the walk-through)
SPINE_Y0, SPINE_Y1 = 2.0, 5.0          # east-west spine corridor
C2_X0, C2_X1 = 2.0, 5.0                # north-south cell corridor
OFF_X1 = 2.0                           # open office: IX0 .. 2.0, IY0 .. 2.0
CELL_X0, CELL_X1 = 5.0, 9.4            # cell block (three cells)
LOCKER_Y0, LOCKER_Y1 = -4.5, 1.7       # locker room: 9.4 .. 17.7
EVID_X0 = 9.0                          # evidence room in the north band
WAIT_X1 = -9.0                         # waiting room in the north band
CELL_ENDS = (1.7, -2.43, -6.56, -10.7)  # three cells between consecutive values (high y first)

# Facades as (face coordinate, axis, sign, u -> build coordinate). u runs left to right seen from outside.
FRONT = (Y1, 1, 1, lambda a: -a)
BACK = (Y0, 1, -1, lambda a: a)
RIGHT = (X1, 0, 1, lambda a: a)
LEFT = (X0, 0, -1, lambda a: -a)

WALL_C = "limestone"          # heavy stone body
BAND_C = "limestone_dark"     # string courses, plinth, pilasters
BRICK_C = "brick_red"         # brick dressings round the openings
TRIM_C = "rock_dark"
FLOOR_C = "concrete_grey"
INNER_C = "rock_light"        # plaster lining inside

FRONT_VIEW = (1.0, 1.0, 0.55)     # preview camera: front right, above
BACK_VIEW = (-0.55, -1.0, 0.60)   # extra preview from behind: the sally port and the back wall


# ---------------------------------------------------------------- facade helpers


def _letters(k, text, facade, u_center, z_base, letter_w, letter_h, color="white", depth=0.13, roof=False):
    """Stroke letters on a facade, reading left to right from outside. `police_props.text_bm` supplies
    the letters the shared graffiti font is missing (P)."""
    face, axis, sign, _ = facade
    bm = pp.text_bm(text, letter_w, letter_h, color)
    if axis == 1:
        rot = 0.0 if sign > 0 else math.pi
        loc = (-sign * u_center, face + sign * depth, z_base)
    else:
        rot = sign * math.pi / 2.0
        loc = (face + sign * depth, sign * u_center, z_base)
    common.transform(bm, rot=(0.0, 0.0, rot), loc=loc)
    k.model.add(bm, roof=roof)


def _bars(k, facade, u0, u1, z0, z1, spacing=0.20, color="gun_black", thick=0.035, depth=0.09, roof=False):
    """A barred panel on a facade (security bars over a lower window, a cell front)."""
    if u1 - u0 < 0.05:
        return
    pk.bars_panel(k, facade[0], facade[1], facade[2], u0, u1, z0, z1, spacing=spacing, color=color,
                  thick=thick, depth=depth, roof=roof)


def _paper(k, x, y, z, rot, color="white"):
    """A sheet of paper on the floor (2 tris)."""
    bm = pbox((-0.11, -0.15, -0.002), (0.11, 0.15, 0.002), color, bevel=0.0)
    common.transform(bm, rot=(0.0, 0.0, rot), loc=(x, y, z))
    k.model.add(bm)


def _debris(rng, k, x, y, z, n=4, spread=0.5):
    """Scattered small boxes, folders and broken board (debris on the floor)."""
    for _ in range(n):
        sx, sy, sz = rng.uniform(0.20, 0.42), rng.uniform(0.16, 0.34), rng.uniform(0.05, 0.16)
        cx, cy = x + rng.uniform(-spread, spread), y + rng.uniform(-spread, spread)
        col = rng.choice(("wood_crate", "bone_white", "rock_light", "bark_light"))
        box = pbox((cx - sx / 2, cy - sy / 2, z), (cx + sx / 2, cy + sy / 2, z + sz), col, bevel=0.008)
        k.model.add(box)
    for _ in range(n):
        _paper(k, x + rng.uniform(-spread, spread), y + rng.uniform(-spread, spread), z + 0.006,
               rng.uniform(0.0, 3.1), rng.choice(("white", "bone_white")))


def _shelf_unit(k, x0, x1, y, depth, z0, z1, shelves=4, color="metal"):
    """A shelving unit against a wall (visual only): uprights and `shelves` boards."""
    for x in (x0, x1 - 0.05):
        for yy in (y, y + depth - 0.05):
            k.box((x, yy, z0), (x + 0.05, yy + 0.05, z1), "rock_dark")
    for i in range(shelves + 1):
        z = z0 + (z1 - z0) * i / shelves
        k.box((x0, y, z), (x1, y + depth, z + 0.04), color)


def _desk(k, x, y, rng, tipped=False):
    """A steel office desk (1.7 x 0.85 m) with a monitor, a keyboard, papers and a chair."""
    parts = []
    top = 0.74
    parts.append(pbox((x - 0.85, y - 0.42, FZ + top), (x + 0.85, y + 0.42, FZ + top + 0.05), "bark_light",
                      bevel=0.012))
    for px0, px1 in ((x - 0.83, x - 0.40), (x + 0.40, x + 0.83)):
        parts.append(pbox((px0, y - 0.38, FZ), (px1, y + 0.36, FZ + top), "metal", bevel=0.01))
        for z in (0.10, 0.30, 0.50):
            parts.append(pbox((px0 - 0.006, y + 0.30, FZ + z), (px1 + 0.006, y + 0.36, FZ + z + 0.16),
                              "rock_light", bevel=0.005))
    parts.append(pbox((x - 0.82, y - 0.40, FZ + 0.10), (x + 0.82, y - 0.36, FZ + top - 0.02), "metal",
                      bevel=0.006))
    lie = rng.random() < 0.35
    if lie:
        parts.append(pbox((x - 0.16, y - 0.20, FZ + top + 0.05), (x + 0.26, y + 0.16, FZ + top + 0.10),
                          "gun_black", bevel=0.01))
    else:
        parts.append(pbox((x - 0.02, y - 0.06, FZ + top + 0.05), (x + 0.24, y + 0.12, FZ + top + 0.09),
                          "gun_black", bevel=0.006))
        parts.append(pbox((x + 0.08, y - 0.03, FZ + top + 0.09), (x + 0.13, y + 0.03, FZ + top + 0.24),
                          "gun_black", bevel=0.004))
        parts.append(pbox((x - 0.10, y - 0.10, FZ + top + 0.24), (x + 0.36, y + 0.04, FZ + top + 0.52),
                          "gun_black", bevel=0.012))
        parts.append(pbox((x - 0.07, y - 0.11, FZ + top + 0.27), (x + 0.33, y - 0.09, FZ + top + 0.49),
                          "soot", bevel=0.004))
    parts.append(pbox((x - 0.24, y + 0.16, FZ + top + 0.05), (x + 0.12, y + 0.30, FZ + top + 0.075),
                      "gun_black", bevel=0.004))
    for _ in range(rng.randint(2, 4)):
        px, py = x + rng.uniform(-0.7, 0.3), y + rng.uniform(-0.28, 0.28)
        parts.append(pbox((px, py, FZ + top + 0.052), (px + 0.22, py + 0.30, FZ + top + 0.057), "white",
                          bevel=0.0))
    chair = [pbox((-0.22, -0.22, 0.40), (0.22, 0.22, 0.48), "gun_black", bevel=0.02),
             pbox((-0.22, -0.26, 0.48), (-0.14, 0.22, 0.94), "gun_black", bevel=0.02),
             pbox((-0.03, -0.03, 0.05), (0.03, 0.03, 0.42), "metal", bevel=0.0),
             pbox((-0.26, -0.26, 0.0), (0.26, 0.26, 0.05), "metal", bevel=0.01)]
    tipped_chair = rng.random() < 0.7
    for bm in chair:
        if tipped_chair:
            common.transform(bm, rot=(math.radians(94.0), 0.0, rng.uniform(0.0, 3.0)),
                             loc=(x + rng.uniform(-0.5, 0.5), y - 1.0, FZ + 0.22))
        else:
            common.transform(bm, rot=(0.0, 0.0, rng.uniform(-0.5, 0.5)),
                             loc=(x + rng.uniform(-0.4, 0.4), y - 0.95, FZ))
        parts.append(bm)
    if tipped:
        for bm in parts:
            common.transform(bm, rot=(0.0, math.radians(-96.0), 0.0), loc=(x, y, FZ + 0.43))
    for bm in parts:
        k.model.add(bm)


# ---------------------------------------------------------------- exterior


def _site(k):
    """The paved apron, the wide front steps with cheek walls, and the kerb."""
    k.box((X0 - 1.2, Y0 - 1.2, -0.06), (X1 + 1.2, Y1 + 3.8, 0.02), "concrete_stain")
    for xx in (-13.0, -8.0, 8.0, 13.0):
        k.box((xx - 0.06, Y1 + 1.7, 0.02), (xx + 0.06, Y1 + 3.6, 0.026), "bone_white")
    k.box((X0 - 0.4, Y1 + 3.60, 0.02), (X1 + 0.4, Y1 + 3.75, 0.10), "concrete_grey")  # the kerb

    # wide steps: three shallow treads straight up to the door, 10 m wide
    for i, (ya, yb, zt) in enumerate(((Y1 + 1.10, Y1 + 1.70, 0.05),
                                      (Y1 + 0.55, Y1 + 1.10, 0.10),
                                      (Y1, Y1 + 0.55, FZ))):
        k.solid((-5.0, ya, 0.0), (5.0, yb, zt), "rock_light", base="step")
    for sx in (-1, 1):
        xa, xb = sorted((sx * 5.0, sx * 5.5))
        k.solid((xa, Y1, 0.0), (xb, Y1 + 1.70, 0.66), "limestone_dark", base="step_cheek")
        k.box((xa - 0.05, Y1 - 0.02, 0.66), (xb + 0.05, Y1 + 1.75, 0.74), "rock_light")


def _front_facade(k):
    """Front wall: double doors, six barred lower windows, the POLICE sign, grime and vines."""
    face, axis, sign, u = FRONT
    door = pk.opening(-1.6, 1.6, Y1 - T - 0.05, Y1 + 0.05, FZ, FZ + 2.4)
    win_x = (-15.0, -10.5, -6.2, 6.2, 10.5, 15.0)
    wins = [(pk.opening(cx - 1.3, cx + 1.3, Y1 - T - 0.05, Y1 + 0.05, 1.2, 3.1), cx) for cx in win_x]
    pk.wall(k, (X0, Y1 - T, FZ), (X1, Y1, CEIL), openings=[door] + [w[0] for w in wins],
            color=WALL_C, base="wall_front")

    # brick door surround, dark reveal, two leaves (one ajar)
    for xa, xb in ((-2.02, -1.6), (1.6, 2.02)):
        k.box((xa, Y1 - 0.02, FZ), (xb, Y1 + 0.07, FZ + 2.78), BRICK_C)
    k.box((-2.02, Y1 - 0.02, FZ + 2.42), (2.02, Y1 + 0.07, FZ + 2.78), BRICK_C)
    k.box((-1.62, Y1 - T - 0.01, FZ), (1.62, Y1 - T + 0.02, FZ + 2.42), "soot")
    k.box((-1.55, Y1 - 0.22, FZ), (-0.05, Y1 - 0.12, FZ + 2.36), "limestone")
    k.box((-1.35, Y1 - 0.24, FZ + 0.90), (-0.25, Y1 - 0.16, FZ + 2.20), "glass_dark")
    k.box((-0.34, Y1 - 0.26, FZ + 1.00), (-0.20, Y1 - 0.10, FZ + 1.24), "brass")
    for dx, dz, sw, sh, col in ((0.0, 0.0, 1.50, 2.36, "limestone"), (0.16, 0.90, 1.20, 1.30, "glass_dark")):
        bm = pbox((-sw / 2, -0.05, FZ + dz), (sw / 2, 0.05, FZ + dz + sh), col, bevel=0.015)
        common.transform(bm, rot=(0.0, 0.0, math.radians(-62.0)), loc=(0.80, Y1 - 0.20, 0.0))
        k.model.add(bm)

    # barred lower windows
    for opening, cx in wins:
        pk.window(k, Y1, axis, sign, u(cx), 1.2, 3.1,
                  state="broken" if cx in (-10.5, 10.5) else "dark", width=2.6)
        _bars(k, FRONT, u(cx) - 1.36, u(cx) + 1.36, 1.14, 3.16, spacing=0.20, depth=0.10)
        k.box((cx - 1.52, Y1 + 0.05, 1.04), (cx + 1.52, Y1 + 0.17, 1.18), "rock_light")
        k.box((cx - 1.52, Y1 + 0.02, 3.10), (cx + 1.52, Y1 + 0.13, 3.30), BAND_C)

    # POLICE sign: a dark plate above the doors with white letters
    k.box((-3.5, Y1 - 0.02, 3.00), (3.5, Y1 + 0.10, 4.00), "gun_black")
    k.box((-3.6, Y1 - 0.02, 2.94), (3.6, Y1 + 0.15, 3.03), TRIM_C)
    k.box((-3.6, Y1 - 0.02, 3.97), (3.6, Y1 + 0.15, 4.06), TRIM_C)
    _letters(k, "POLICE", FRONT, 0.0, 3.14, 0.52, 0.70, "white", depth=0.14)

    for i, cx in enumerate((-16.0, -12.2, -8.0, -3.2, 2.8, 8.0, 12.6, 16.4)):
        pk.grime(k, Y1, axis, sign, u(cx), 0.4, 3.95, seed=i, width=1.8, streaks=5)
    pk.vines(k, Y1, axis, sign, u(-8.6), 4.0, 0.2, 5.4, seed=4, leaves=90)
    pk.vines(k, Y1, axis, sign, u(13.8), 3.4, 0.2, 4.2, seed=9, leaves=70)


def _back_and_sides(k):
    """Ground-floor back and side walls with barred windows; the back carries the sally-port gate."""
    gate = pk.opening(5.6, 8.2, Y0 - 0.05, Y0 + T + 0.05, FZ, FZ + 2.7)
    back_wins = [(pk.opening(cx - 1.1, cx + 1.1, Y0 - 0.05, Y0 + T + 0.05, 1.2, 3.0), cx)
                 for cx in (-15.0, -9.5, -3.5, 2.0, 12.5, 16.0)]
    pk.wall(k, (X0, Y0, FZ), (X1, Y0 + T, CEIL), openings=[gate] + [w[0] for w in back_wins],
            color=WALL_C, base="wall_back")

    left_wins = [(pk.opening(yv - 1.1, yv + 1.1, X0 - 0.05, X0 + T + 0.05, 1.2, 3.0), yv)
                 for yv in (11.0, 7.0, 3.0, -2.0, -7.0, -11.0)]
    pk.wall(k, (X0, Y0 + T, FZ), (X0 + T, Y1 - T, CEIL), openings=[w[0] for w in left_wins],
            color=WALL_C, base="wall_left")
    right_wins = [(pk.opening(yv - 1.1, yv + 1.1, X1 - T - 0.05, X1 + 0.05, 1.2, 3.0), yv)
                  for yv in (10.0, 6.0, 1.0, -3.5, -8.0, -12.2)]
    pk.wall(k, (X1 - T, Y0 + T, FZ), (X1, Y1 - T, CEIL), openings=[w[0] for w in right_wins],
            color=WALL_C, base="wall_right")

    for facade, wins in ((BACK, back_wins), (LEFT, left_wins), (RIGHT, right_wins)):
        for opening, c in wins:
            pk.window(k, facade[0], facade[1], facade[2], facade[3](c), 1.2, 3.0,
                      state="boarded" if c in (3.0, -8.0) else "dark", width=2.2)
            _bars(k, facade, facade[3](c) - 1.18, facade[3](c) + 1.18, 1.14, 3.06, spacing=0.20, depth=0.10)

    # sally-port gate: brick surround, a lintel and one steel leaf hanging open outward
    k.box((5.35, Y0 - 0.08, FZ), (5.6, Y0 + T, FZ + 2.85), BRICK_C)
    k.box((8.2, Y0 - 0.08, FZ), (8.45, Y0 + T, FZ + 2.85), BRICK_C)
    k.box((5.35, Y0 - 0.08, FZ + 2.70), (8.45, Y0 + T, FZ + 2.95), BRICK_C)
    k.box((5.62, Y0 + T - 0.02, FZ), (8.18, Y0 + T, FZ + 2.72), "soot")
    leaf = [pbox((-1.3, -0.05, FZ), (0.0, 0.05, FZ + 2.62), "metal", bevel=0.015)]
    for i in range(4):
        leaf.append(pbox((-1.24, -0.07, FZ + 0.34 + i * 0.58), (-0.06, 0.07, FZ + 0.46 + i * 0.58),
                         "rock_dark", bevel=0.005))
    leaf.append(pbox((-0.30, -0.09, FZ + 1.05), (-0.14, 0.09, FZ + 1.30), "metal", bevel=0.01))
    for bm in leaf:
        common.transform(bm, rot=(0.0, 0.0, math.radians(-66.0)), loc=(8.20, Y0 - 0.12, 0.0))
        k.model.add(bm)
    k.collider((7.30, Y0 - 0.62, FZ), (7.55, Y0 - 0.34, FZ + 2.6), "sally_leaf")
    k.box((5.5, Y0 - 0.12, FZ - 0.02), (8.3, Y0 + 0.02, FZ + 0.03), "soot")

    for i, (facade, cs) in enumerate(((BACK, (-16.5, -12.0, -5.0, 11.0, 15.5)),
                                      (LEFT, (12.6, 8.6, 4.6, -4.0, -9.0)),
                                      (RIGHT, (11.6, 7.6, 2.6, -6.0, -10.6)))):
        for j, c in enumerate(cs):
            pk.grime(k, facade[0], facade[1], facade[2], facade[3](c), 0.4, 3.9, seed=i * 10 + j, width=1.6)
    pk.vines(k, LEFT[0], LEFT[1], LEFT[2], LEFT[3](9.0), 3.2, 0.2, 3.6, seed=12, leaves=60)


def _lamps_pole(k):
    """The two blue lamp posts either side of the doors, and the flagpole with no flag."""
    for sx in (-1, 1):
        x, y = sx * 6.4, Y1 + 2.0
        k.solid((x - 0.24, y - 0.24, 0.0), (x + 0.24, y + 0.24, 0.30), "limestone_dark", base="lamp_base")
        k.box((x - 0.10, y - 0.10, 0.30), (x + 0.10, y + 0.10, 3.10), "gun_black")
        k.box((x - 0.17, y - 0.17, 3.10), (x + 0.17, y + 0.17, 3.18), "gun_black")
        k.box((x - 0.15, y - 0.15, 3.18), (x + 0.15, y + 0.15, 3.72), "water_blue")
        for z in (3.32, 3.56):
            k.box((x - 0.19, y - 0.19, z), (x + 0.19, y + 0.19, z + 0.06), "gun_black")
        k.box((x - 0.20, y - 0.20, 3.72), (x + 0.20, y + 0.20, 3.82), "gun_black")
        k.box((x - 0.13, y - 0.13, 3.82), (x + 0.13, y + 0.13, 4.00), "gun_black")
        k.box((x - 0.05, y - 0.05, 4.00), (x + 0.05, y + 0.05, 4.14), "gun_black")
        k.collider((x - 0.11, y - 0.11, 0.0), (x + 0.11, y + 0.11, 3.10), "lamp_post")

    fx, fy = -8.4, Y1 + 2.6
    k.solid((fx - 0.44, fy - 0.44, 0.0), (fx + 0.44, fy + 0.44, 0.34), "limestone_dark", base="pole_base")
    k.box((fx - 0.30, fy - 0.30, 0.34), (fx + 0.30, fy + 0.30, 0.50), "rock_light")
    k.model.add(common.make_frustum(0.075, 0.045, 0.50, 8.6, segments=8, center=(fx, fy), color="metal"))
    k.collider((fx - 0.09, fy - 0.09, 0.0), (fx + 0.09, fy + 0.09, 8.6), "flagpole")
    k.box((fx - 0.05, fy - 0.05, 8.60), (fx + 0.05, fy + 0.05, 8.78), "brass")
    k.box((fx - 0.03, fy - 0.03, 7.80), (fx + 0.03, fy + 0.03, 7.86), "brass")  # the cleat, no flag


def _upper_floors(k):
    """Closed upper storeys, the roof and everything on it. All `roof=True` (the cut-away hides them)."""
    pk.box(k, (X0, Y0, CEIL), (X1, Y1, G_TOP), "concrete_grey", roof=True)  # the ground-floor ceiling
    for z0, z1 in ((G_TOP, F2_TOP), (F2_TOP, F3_TOP)):
        for lo, hi in (((X0, Y1 - T, z0), (X1, Y1, z1)), ((X0, Y0, z0), (X1, Y0 + T, z1)),
                       ((X0, Y0 + T, z0), (X0 + T, Y1 - T, z1)), ((X1 - T, Y0 + T, z0), (X1, Y1 - T, z1))):
            pk.box(k, lo, hi, WALL_C, roof=True)
        for cx in (-15.0, -10.0, -5.0, 0.0, 5.0, 10.0, 15.0):
            pk.box(k, (cx - 0.22, Y1, z0), (cx + 0.22, Y1 + 0.08, z1), BAND_C, roof=True)
            pk.box(k, (cx - 0.22, Y0 - 0.08, z0), (cx + 0.22, Y0, z1), BAND_C, roof=True)
        for cy in (-11.0, -6.0, -1.0, 4.0, 9.0):
            pk.box(k, (X0 - 0.08, cy - 0.22, z0), (X0, cy + 0.22, z1), BAND_C, roof=True)
            pk.box(k, (X1, cy - 0.22, z0), (X1 + 0.08, cy + 0.22, z1), BAND_C, roof=True)

    for z in (G_TOP - 0.06, F2_TOP - 0.06):
        pk.box(k, (X0 - 0.12, Y1, z), (X1 + 0.12, Y1 + 0.16, z + 0.20), BAND_C, roof=True)
        pk.box(k, (X0 - 0.12, Y0 - 0.16, z), (X1 + 0.12, Y0, z + 0.20), BAND_C, roof=True)
        pk.box(k, (X0 - 0.16, Y0, z), (X0, Y1, z + 0.20), BAND_C, roof=True)
        pk.box(k, (X1, Y0, z), (X1 + 0.16, Y1, z + 0.20), BAND_C, roof=True)

    front_w = (-15.5, -11.5, -7.5, -3.5, 3.5, 7.5, 11.5, 15.5)
    back_w = (-15.5, -11.0, -6.5, -2.0, 2.5, 7.0, 11.5, 15.5)
    side_w = (-11.0, -6.5, -2.0, 2.5, 7.0, 11.0)
    for z0, z1 in ((G_TOP + 0.70, G_TOP + 2.90), (F2_TOP + 0.70, F2_TOP + 2.90)):
        pk.window_band(k, Y1, 1, 1, [FRONT[3](c) for c in front_w], z0, z1, width=2.3, roof=True)
        pk.window_band(k, Y0, 1, -1, [BACK[3](c) for c in back_w], z0, z1, width=2.3, roof=True)
        pk.window_band(k, X1, 0, 1, [RIGHT[3](c) for c in side_w], z0, z1, width=2.2, roof=True)
        pk.window_band(k, X0, 0, -1, [LEFT[3](c) for c in side_w], z0, z1, width=2.2, roof=True)

    for i, cx in enumerate((-13.0, -4.0, 5.0, 13.0)):
        pk.grime(k, Y1, 1, 1, FRONT[3](cx), G_TOP + 0.3, F3_TOP - 0.3, seed=20 + i, width=2.6, streaks=7,
                 roof=True)
    pk.vines(k, Y1, 1, 1, FRONT[3](-7.0), 4.6, G_TOP, 9.6, seed=31, leaves=110, roof=True)
    pk.vines(k, X0, 0, -1, LEFT[3](-2.0), 3.4, G_TOP + 0.2, 8.4, seed=32, leaves=70, roof=True)

    pk.box(k, (X0 - 0.12, Y0 - 0.12, F3_TOP), (X1 + 0.12, Y1 + 0.12, ROOF_TOP), "concrete_grey", roof=True)
    for lo, hi in (((X0 - 0.12, Y1 - T, ROOF_TOP), (X1 + 0.12, Y1 + 0.12, PARAPET)),
                   ((X0 - 0.12, Y0 - 0.12, ROOF_TOP), (X1 + 0.12, Y0 + T, PARAPET)),
                   ((X0 - 0.12, Y0 + T, ROOF_TOP), (X0 + T, Y1 - T, PARAPET)),
                   ((X1 - T, Y0 + T, ROOF_TOP), (X1 + 0.12, Y1 - T, PARAPET))):
        pk.box(k, lo, hi, WALL_C, roof=True)
    for lo, hi in (((X0 - 0.22, Y1 - 0.10, PARAPET), (X1 + 0.22, Y1 + 0.22, PARAPET + 0.12)),
                   ((X0 - 0.22, Y0 - 0.22, PARAPET), (X1 + 0.22, Y0 + 0.10, PARAPET + 0.12)),
                   ((X0 - 0.22, Y0 + 0.10, PARAPET), (X0 + 0.10, Y1 - 0.10, PARAPET + 0.12)),
                   ((X1 - 0.10, Y0 + 0.10, PARAPET), (X1 + 0.22, Y1 - 0.10, PARAPET + 0.12))):
        pk.box(k, lo, hi, TRIM_C, roof=True)

    pk.ac_unit(k, -8.0, -4.0, ROOF_TOP)
    pk.ac_unit(k, -3.0, -6.5, ROOF_TOP)
    pk.ac_unit(k, 8.0, -8.0, ROOF_TOP)
    pk.roof_vent(k, 13.0, 2.0, ROOF_TOP, h=1.6)
    pk.roof_vent(k, -13.0, 8.0, ROOF_TOP, h=1.3)
    pk.box(k, (2.0, 6.0, ROOF_TOP), (6.0, 10.0, ROOF_TOP + 2.6), WALL_C, roof=True)
    pk.box(k, (1.88, 5.88, ROOF_TOP + 2.6), (6.12, 10.12, ROOF_TOP + 2.80), TRIM_C, roof=True)
    pk.box(k, (3.2, 9.98, ROOF_TOP), (4.8, 10.04, ROOF_TOP + 2.05), "gun_black", roof=True)
    pk.roof_antenna(k, -14.0, -9.5, ROOF_TOP, h=4.6)
    pk.roof_antenna(k, 15.0, -10.0, ROOF_TOP, h=3.8)

    # a lattice mast with guy wires and a radio dish
    mx, my = -8.0, 9.0
    pk.box(k, (mx - 0.60, my - 0.60, ROOF_TOP), (mx + 0.60, my + 0.60, ROOF_TOP + 0.30), TRIM_C, roof=True)
    for dx, dy in ((-0.34, -0.34), (0.34, -0.34), (-0.34, 0.34), (0.34, 0.34)):
        k.box((mx + dx - 0.05, my + dy - 0.05, ROOF_TOP + 0.30),
              (mx + dx + 0.05, my + dy + 0.05, ROOF_TOP + 7.60), "metal", roof=True)
    for i in range(9):
        z = ROOF_TOP + 0.50 + i * 0.78
        k.box((mx - 0.40, my - 0.02, z), (mx + 0.40, my + 0.02, z + 0.05), "metal", roof=True)
        k.box((mx - 0.02, my - 0.40, z + 0.36), (mx + 0.02, my + 0.40, z + 0.41), "metal", roof=True)
    for ex, ey in ((-14.0, 4.0), (-2.0, 4.0), (-14.0, 13.0), (-2.0, 13.0)):
        k.model.add(pp._beam((mx, my, ROOF_TOP + 6.2), (ex, ey, ROOF_TOP + 0.1), 0.02, color="gun_black"),
                    roof=True)
        k.box((ex - 0.16, ey - 0.16, ROOF_TOP), (ex + 0.16, ey + 0.16, ROOF_TOP + 0.20), TRIM_C, roof=True)
    dish = common.make_frustum(0.80, 0.80, -0.10, 0.0, segments=12, color="white")
    common.transform(dish, rot=(math.radians(-35.0), 0.0, math.radians(40.0)),
                     loc=(mx + 1.3, my - 0.4, ROOF_TOP + 2.4))
    k.model.add(dish, roof=True)
    k.box((mx + 1.6, my - 0.5, ROOF_TOP + 1.2), (mx + 1.75, my - 0.35, ROOF_TOP + 2.7), "metal", roof=True)


# ---------------------------------------------------------------- interior shell


def _door(y0, y1, xa, xb, top=FZ + 2.4):
    """An opening box that cuts right through a 0.3 m wall."""
    return pk.opening(xa, xb, y0 - 0.02, y1 + 0.02, FZ, top)


def _interior_shell(k):
    """The slab and every interior wall, with a collider per solid piece and none in the gaps."""
    pk.floor_slab(k, X0, Y0, X1, Y1, z_top=FZ, color=FLOOR_C)
    pk.floor_tiles(k, -9.0, 5.3, 9.0, 13.7, FZ + 0.004, tile=1.4, color_a="bone_white",
                   color_b="limestone_dark")
    pk.floor_tiles(k, C2_X0, IY0, C2_X1, 1.7, FZ + 0.004, tile=1.0, color_a="concrete_stain",
                   color_b="concrete_grey")

    # the inner plaster lining of the exterior walls, 2 cm proud
    for lo, hi in (((IX0, Y1 - 0.02, FZ), (IX1, Y1, CEIL)), ((IX0, Y0, FZ), (IX1, Y0 + 0.02, CEIL)),
                   ((IX0, IY0, FZ), (IX0 + 0.02, IY1, CEIL)), ((IX1 - 0.02, IY0, FZ), (IX1, IY1, CEIL))):
        k.box(lo, hi, INNER_C)

    # spine corridor: north wall (waiting / lobby / evidence) and south wall (office / cells / lockers)
    pk.wall(k, (IX0, SPINE_Y1, FZ), (IX1, SPINE_Y1 + T, CEIL), color=INNER_C, base="wall_spine_n",
            openings=[_door(SPINE_Y1, SPINE_Y1 + T, -14.0, -12.4),
                      _door(SPINE_Y1, SPINE_Y1 + T, -4.2, 4.2, FZ + 3.0),
                      _door(SPINE_Y1, SPINE_Y1 + T, 11.0, 12.6)])
    pk.wall(k, (IX0, SPINE_Y0 - T, FZ), (IX1, SPINE_Y0, CEIL), color=INNER_C, base="wall_spine_s",
            openings=[_door(SPINE_Y0 - T, SPINE_Y0, -6.0, -4.4),
                      _door(SPINE_Y0 - T, SPINE_Y0, C2_X0, C2_X1, FZ + 2.9),
                      _door(SPINE_Y0 - T, SPINE_Y0, 11.0, 12.6)])

    # partitions in the north band
    pk.wall(k, (WAIT_X1, SPINE_Y1 + T, FZ), (WAIT_X1 + T, IY1, CEIL), color=INNER_C, base="wall_wait",
            openings=[pk.opening(WAIT_X1 - 0.02, WAIT_X1 + T + 0.02, 8.0, 9.8, FZ, FZ + 2.4)])
    pk.wall(k, (EVID_X0 - T, SPINE_Y1 + T, FZ), (EVID_X0, IY1, CEIL), color=INNER_C, base="wall_evid",
            openings=[pk.opening(EVID_X0 - T - 0.02, EVID_X0 + 0.02, 8.0, 9.8, FZ, FZ + 2.4)])

    # the office east wall: the corridor door is barricaded from inside, so the only way in is the
    # narrow back door at its north end (they walled themselves in, then left)
    pk.wall(k, (OFF_X1 - T, IY0, FZ), (OFF_X1, 0.30, CEIL), color=INNER_C, base="wall_office_e")
    pk.wall(k, (OFF_X1 - T, 1.60, FZ), (OFF_X1, SPINE_Y0 - T, CEIL), color=INNER_C,
            base="wall_office_e2")
    pk.wall(k, (OFF_X1 - T, 0.30, FZ + 2.40), (OFF_X1, 1.60, CEIL), color=INNER_C,
            base="wall_office_e3")
    # the cell block east wall, which doubles as the armoury / locker room west wall
    pk.wall(k, (CELL_X1 - T, Y0 + T, FZ), (CELL_X1, CELL_ENDS[0], CEIL), color=INNER_C,
            base="wall_cells_e")
    pk.wall(k, (C2_X1, CELL_ENDS[3] - T, FZ), (CELL_X1, CELL_ENDS[3], CEIL), color=INNER_C,
            base="wall_cells_s")
    for i, yv in enumerate(CELL_ENDS[1:3]):
        pk.wall(k, (C2_X1, yv - T / 2, FZ), (CELL_X1, yv + T / 2, CEIL), color=INNER_C,
                base=f"wall_cell_div{i}")
    # the locker room | armoury divider
    pk.wall(k, (CELL_X1, LOCKER_Y0 - T, FZ), (IX1, LOCKER_Y0, CEIL), color=INNER_C, base="wall_armoury",
            openings=[_door(LOCKER_Y0 - T, LOCKER_Y0, 11.0, 12.6)])

    # steel door jambs round every interior opening
    for xa, xb, ya, yb in ((-6.0, -4.4, SPINE_Y0 - T, SPINE_Y0), (11.0, 12.6, SPINE_Y0 - T, SPINE_Y0),
                           (-14.0, -12.4, SPINE_Y1, SPINE_Y1 + T), (11.0, 12.6, SPINE_Y1, SPINE_Y1 + T),
                           (11.0, 12.6, LOCKER_Y0 - T, LOCKER_Y0)):
        for x in (xa - 0.06, xb):
            k.box((x, ya - 0.02, FZ), (x + 0.06, yb + 0.02, FZ + 2.46), TRIM_C)
    for ya, yb in ((8.0, 9.8),):
        for x in (WAIT_X1 - 0.06, WAIT_X1 + T, EVID_X0 - T - 0.06, EVID_X0):
            k.box((x, ya - 0.02, FZ), (x + 0.06, yb + 0.02, FZ + 2.46), TRIM_C)

    # ceiling fittings: one fallen panel and dead strip lights
    k.box((6.5, 2.6, CEIL - 0.06), (9.0, 4.6, CEIL), "bone_white")
    k.box((7.0, 3.0, CEIL - 0.24), (8.2, 3.7, CEIL - 0.06), "metal")
    k.box((7.4, 3.5, FZ + 0.004), (8.6, 4.2, FZ + 0.02), "bone_white")  # the fallen panel on the floor
    for lx, ly in ((-5.0, 3.5), (6.0, 3.5), (13.0, 3.5), (-12.0, 3.5), (3.5, -6.0), (-9.0, -6.0)):
        k.box((lx - 0.55, ly - 0.14, CEIL - 0.10), (lx + 0.55, ly + 0.14, CEIL - 0.02), "gun_black")
        k.box((lx - 0.45, ly - 0.10, CEIL - 0.145), (lx + 0.45, ly + 0.10, CEIL - 0.10), "bone_white")


# ---------------------------------------------------------------- rooms


def _lobby(k, rng):
    """Entrance lobby: front desk behind a smashed glass screen, cabinets, bench, notices, clutter."""
    k.solid((-3.6, 6.9, FZ), (3.6, 7.5, FZ + 1.12), "wood_crate", base="desk_front")
    k.box((-3.7, 6.85, FZ + 1.12), (3.7, 7.55, FZ + 1.19), "bark_light")
    k.solid((3.0, 3.4, FZ), (3.6, 6.9, FZ + 1.12), "wood_crate", base="desk_return")
    k.box((2.95, 3.35, FZ + 1.12), (3.65, 6.95, FZ + 1.19), "bark_light")

    # the glass screen, with two panes smashed out
    k.box((-3.55, 7.05, FZ + 1.19), (0.90, 7.15, FZ + 2.45), "glass_dark")
    k.box((1.55, 7.05, FZ + 1.19), (3.55, 7.15, FZ + 2.45), "glass_dark")
    for gx in (-3.55, 0.90, 1.55, 3.55):
        k.box((gx - 0.05, 7.02, FZ + 1.19), (gx + 0.05, 7.18, FZ + 2.52), "metal")
    k.box((-3.55, 7.02, FZ + 2.45), (3.55, 7.18, FZ + 2.55), "metal")
    k.box((0.95, 7.04, FZ + 1.19), (1.50, 7.16, FZ + 2.45), "soot")   # a smashed pane
    for i in range(5):                                                 # glass teeth on the frame
        x = 0.97 + i * 0.108
        k.box((x, 7.05, FZ + 1.19), (x + 0.07, 7.15, FZ + 1.19 + rng.uniform(0.10, 0.32)), "glass_dark")
    for _ in range(7):                                                 # shards on the floor
        shard = pbox((-0.06, -0.08, -0.004), (0.06, 0.08, 0.004), "glass_dark", bevel=0.0)
        common.transform(shard, rot=(0.0, 0.0, rng.uniform(0.0, 3.0)),
                         loc=(rng.uniform(-1.6, 0.8), rng.uniform(6.2, 7.0), FZ + 0.012))
        k.model.add(shard)

    # the officer's side of the desk
    k.solid((-3.2, 4.4, FZ), (3.2, 5.6, FZ + 0.76), "metal", base="desk_back")
    k.box((-3.25, 4.35, FZ + 0.76), (3.25, 5.65, FZ + 0.81), "bark_light")
    k.box((-2.62, 4.95, FZ + 0.81), (-1.78, 5.20, FZ + 0.86), "gun_black")
    k.box((-2.28, 5.00, FZ + 0.86), (-2.12, 5.10, FZ + 1.14), "gun_black")
    k.box((-2.56, 5.02, FZ + 1.14), (-1.84, 5.18, FZ + 1.48), "gun_black")
    k.box((-1.40, 4.85, FZ + 0.81), (-0.90, 5.15, FZ + 0.85), "gun_black")
    k.box((0.60, 4.90, FZ + 0.81), (0.95, 5.20, FZ + 0.87), "white")
    for _ in range(7):
        px, py = rng.uniform(-3.0, 2.8), rng.uniform(4.45, 5.55)
        k.box((px, py, FZ + 0.815), (px + 0.22, py + 0.30, FZ + 0.822), "white")
    for cx, tipped in ((-1.6, False), (1.2, True)):
        if tipped:
            chair = pbox((-0.24, -0.24, 0.0), (0.24, 0.24, 0.9), "gun_black", bevel=0.02)
            common.transform(chair, rot=(math.radians(92.0), 0.0, 0.8), loc=(2.2, 6.2, FZ + 0.24))
            k.model.add(chair)
        else:
            k.box((cx - 0.24, 3.72, FZ + 0.40), (cx + 0.24, 4.18, FZ + 0.48), "gun_black")
            k.box((cx - 0.24, 3.72, FZ + 0.48), (cx - 0.16, 4.18, FZ + 0.94), "gun_black")
            k.box((cx - 0.03, 3.90, FZ + 0.05), (cx + 0.03, 3.96, FZ + 0.42), "metal")
            k.box((cx - 0.26, 3.68, FZ), (cx + 0.26, 4.20, FZ + 0.05), "metal")

    # filing cabinets, one drawn open with its folders on the floor
    for i in range(3):
        x = -7.7 + i * 1.25
        k.solid((x, 5.6, FZ), (x + 1.15, 6.3, FZ + 1.45), "metal", base=f"cabinet{i}")
        for j in range(4):
            k.box((x + 0.06, 6.30, FZ + 0.16 + j * 0.32), (x + 1.09, 6.35, FZ + 0.42 + j * 0.32), "rock_light")
            k.box((x + 0.48, 6.35, FZ + 0.24 + j * 0.32), (x + 0.68, 6.38, FZ + 0.30 + j * 0.32), "chrome")
    k.box((-6.45, 6.30, FZ + 0.48), (-5.45, 6.80, FZ + 0.74), "metal")
    _debris(rng, k, -5.6, 7.9, FZ, n=4, spread=0.6)

    # the waiting bench along the lobby's west wall and a dead plant
    k.solid((-8.4, 8.0, FZ), (-8.1, 11.6, FZ + 0.45), "wood_crate", base="bench_lobby")
    k.box((-8.42, 8.0, FZ + 0.45), (-8.05, 11.6, FZ + 0.52), "bark_light")
    k.box((-8.42, 8.0, FZ + 0.52), (-8.20, 11.6, FZ + 1.05), "wood_crate")
    k.model.add(common.make_frustum(0.22, 0.26, FZ, FZ + 0.50, segments=8, center=(-7.9, 12.6),
                                    color="bark_dark"))
    for i in range(5):
        k.model.add(common.make_icosphere(subdiv=1, radius=(0.15, 0.15, 0.06),
                                          loc=(-7.9, 12.6, FZ + 0.50 + i * 0.07),
                                          color="leaf_dark" if i % 2 else "dirt"))

    # notice board and a big wall map on the lobby's inner wall
    k.box((-8.2, 5.30, FZ + 1.40), (-4.2, 5.36, FZ + 2.60), "bark_light")
    for i in range(6):
        k.box((-8.0 + i * 0.62, 5.36, FZ + 1.55), (-7.7 + i * 0.62, 5.375, FZ + 2.35), "white")
    k.box((5.2, 5.30, FZ + 1.30), (8.2, 5.36, FZ + 2.50), "bone_white")
    k.box((5.3, 5.36, FZ + 1.40), (8.1, 5.375, FZ + 2.40), "tarp_blue")

    for i, (cx, cy, r) in enumerate(((-2.0, 9.6, 1.1), (1.6, 11.0, 0.8), (-5.4, 6.4, 0.9))):
        pk.patch(k, cx, cy, r, r * 0.7, FZ + 0.006, "blood_dark" if i == 0 else "concrete_stain", seed=40 + i)
    for _ in range(14):
        _paper(k, rng.uniform(-7.0, 8.0), rng.uniform(5.6, 13.0), FZ + 0.006, rng.uniform(0, 3.1),
               rng.choice(("white", "bone_white")))
    _debris(rng, k, 6.8, 12.0, FZ, n=3, spread=0.5)


def _waiting(k, rng):
    """Waiting room: three rows of benches, bags left behind, papers."""
    for i, yy in enumerate((7.0, 10.0, 12.6)):
        k.solid((IX0 + 0.10, yy, FZ), (IX0 + 0.55, yy + 2.6, FZ + 0.45), "wood_crate", base=f"bench_wait{i}")
        k.box((IX0 + 0.08, yy, FZ + 0.45), (IX0 + 0.58, yy + 2.6, FZ + 0.52), "bark_light")
        k.box((IX0 + 0.08, yy, FZ + 0.52), (IX0 + 0.20, yy + 2.6, FZ + 1.05), "wood_crate")
    for x, y in ((-16.4, 9.0), (-15.2, 12.0), (-12.0, 6.6)):
        k.box((x - 0.22, y - 0.30, FZ), (x + 0.22, y + 0.30, FZ + 0.44), "leather")
    for i in range(3):
        pk.patch(k, -14.0 + i * 1.6, 9.5 + i * 0.7, 0.9, 0.6, FZ + 0.006, "concrete_stain", seed=50 + i)
    for _ in range(10):
        _paper(k, rng.uniform(-17.4, -9.4), rng.uniform(5.6, 13.4), FZ + 0.006, rng.uniform(0, 3.1))
    _debris(rng, k, -11.0, 11.6, FZ, n=3, spread=0.5)


def _evidence(k, rng):
    """Evidence room: three tall shelf runs of boxes, a tagged-bag table, spilled files."""
    for yy in (6.0, 9.4, 12.4):
        _shelf_unit(k, 10.0, 16.9, yy, 0.55, FZ, FZ + 2.2, shelves=4)
        for j in range(4):
            z = FZ + 0.04 + j * 0.54
            for bx in range(7):
                if rng.random() < 0.25:
                    continue
                x = 10.15 + bx * 0.95
                w, h = rng.uniform(0.50, 0.80), rng.uniform(0.22, 0.42)
                k.box((x, yy + 0.06, z + 0.04), (x + w, yy + 0.50, z + 0.04 + h),
                      rng.choice(("wood_crate", "bone_white", "rock_light", "cloth_olive", "bark_light")))
    k.solid((12.0, 7.4, FZ), (15.0, 8.4, FZ + 0.75), "metal", base="evid_table")
    k.box((11.9, 7.3, FZ + 0.75), (15.1, 8.5, FZ + 0.81), "bark_light")
    for i in range(5):
        k.box((12.2 + i * 0.55, 7.6, FZ + 0.81), (12.6 + i * 0.55, 8.2, FZ + 0.91), "bone_white")
        k.box((12.3 + i * 0.55, 7.62, FZ + 0.82), (12.5 + i * 0.55, 8.18, FZ + 0.86), "rock_light")
    _debris(rng, k, 13.6, 11.4, FZ, n=4, spread=0.7)
    # a tipped shelf run and its boxes across the floor
    k.box((10.6, 11.0, FZ), (16.4, 11.4, FZ + 0.10), "metal")
    _debris(rng, k, 13.0, 10.4, FZ, n=5, spread=1.2)


def _office(k, rng):
    """The open office: six desks, the barricade of desks against the corridor door, debris and stains."""
    for yy in (-2.6, -8.8):
        for xx in (-14.5, -9.5, -4.5):
            _desk(k, xx, yy, rng)
    for i in range(2):
        x = -17.2 + i * 1.3
        k.solid((x, -13.5, FZ), (x + 1.2, -12.8, FZ + 1.4), "metal", base=f"office_cab{i}")
    k.box((1.10, -13.4, FZ), (1.55, -12.9, FZ + 0.90), "white")
    k.box((1.15, -13.3, FZ + 0.90), (1.50, -13.0, FZ + 1.20), "water_blue")

    # the pinboards on the west wall
    for i in range(3):
        k.box((X0 + T + 0.02, -11.2 + i * 3.2, FZ + 1.40), (X0 + T + 0.07, -9.4 + i * 3.2, FZ + 2.50),
              "bark_light")
        k.box((X0 + T + 0.07, -11.1 + i * 3.2, FZ + 1.50), (X0 + T + 0.08, -9.5 + i * 3.2, FZ + 2.40),
              "white")

    # the barricade: three desks on their sides plus a cabinet, jammed against the corridor door
    for bx, by, br in ((-5.3, 1.20, 0.06), (-4.9, 0.62, -0.10), (-5.5, 0.10, 0.14)):
        pile = [pbox((-0.85, -0.42, 0.0), (0.85, 0.42, 0.05), "bark_light", bevel=0.012),
                pbox((-0.83, -0.38, 0.05), (-0.40, 0.36, 0.74), "metal", bevel=0.01),
                pbox((0.40, -0.38, 0.05), (0.83, 0.36, 0.74), "metal", bevel=0.01),
                pbox((-0.82, -0.40, 0.10), (0.82, -0.36, 0.72), "metal", bevel=0.006)]
        for bm in pile:
            # tipped onto its long side: 1.7 m long, 0.85 m tall, then turned a little
            common.transform(bm, rot=(math.radians(-90.0), 0.0, br), loc=(bx, by, FZ + 0.43))
            k.model.add(bm)
        k.collider((bx - 0.5, by - 0.9, FZ), (bx + 0.5, by + 0.9, FZ + 0.95), "barricade_desk")
    k.solid((-6.9, 0.85, FZ), (-6.1, 1.45, FZ + 1.30), "metal", base="barricade_cabinet")
    # the barricade closes the corridor door for good: a collider across the whole gap
    k.collider((-6.0, 1.62, FZ), (-4.4, 2.08, FZ + 1.35), "barricade_door")
    for i in range(4):
        k.box((-6.95, 0.95 + i * 0.13, FZ + 0.20), (-6.90, 1.35 + i * 0.13, FZ + 1.15), "rock_light")
    for _ in range(18):
        _paper(k, rng.uniform(-7.6, -4.0), rng.uniform(-0.8, 1.7), FZ + 0.006, rng.uniform(0, 3.1))
    _debris(rng, k, -6.2, -0.8, FZ, n=4, spread=0.6)

    for i in range(6):
        pk.patch(k, rng.uniform(-16.0, 0.0), rng.uniform(-12.5, 0.5), rng.uniform(0.5, 1.3),
                 rng.uniform(0.4, 0.9), FZ + 0.006,
                 rng.choice(("concrete_stain", "blood_dark", "soot")), seed=60 + i)
    for _ in range(34):
        _paper(k, rng.uniform(-17.2, 1.4), rng.uniform(-13.2, 1.4), FZ + 0.006, rng.uniform(0, 3.1),
               rng.choice(("white", "bone_white")))
    for bx, by in ((-11.6, -11.6), (-1.0, -5.4), (-15.6, -0.4), (-7.8, -12.4), (-2.4, -0.6)):
        _debris(rng, k, bx, by, FZ, n=3, spread=0.5)
    k.box((-12.4, -6.2, FZ + 0.02), (-10.2, -4.4, FZ + 0.06), "bone_white")  # a fallen ceiling panel
    cab = pbox((-0.6, -0.7, 0.0), (0.6, 0.7, 1.3), "metal", bevel=0.02)
    common.transform(cab, rot=(math.radians(94.0), 0.0, 0.5), loc=(-2.6, -12.6, FZ + 0.70))
    k.model.add(cab)


def _cells(k, rng):
    """The cell corridor and three barred cells: fronts, doors (one hanging open), bunks, toilets."""
    z0, z1 = FZ, FZ + 3.15
    for i in range(3):
        yn, ys = CELL_ENDS[i], CELL_ENDS[i + 1]
        gap0, gap1 = ys + 0.55, ys + 1.50
        _bars(k, (CELL_X0, 0, 1, lambda a: a), ys + 0.10, gap0, z0, z1, spacing=0.16, depth=0.10)
        _bars(k, (CELL_X0, 0, 1, lambda a: a), gap1, yn - 0.10, z0, z1, spacing=0.16, depth=0.10)
        # the bar panels block the player (a barred wall, not a visual)
        k.collider((CELL_X0 - 0.06, ys + 0.10, z0), (CELL_X0 + 0.10, gap0, z1), "cell_bars_a")
        k.collider((CELL_X0 - 0.06, gap1, z0), (CELL_X0 + 0.10, yn - 0.10, z1), "cell_bars_b")
        k.box((CELL_X0 - 0.06, gap0 - 0.07, z0), (CELL_X0 + 0.06, gap0, z0 + 2.58), TRIM_C)
        k.box((CELL_X0 - 0.06, gap1, z0), (CELL_X0 + 0.06, gap1 + 0.07, z0 + 2.58), TRIM_C)
        k.box((CELL_X0 - 0.06, gap0 - 0.07, z0 + 2.48), (CELL_X0 + 0.06, gap1 + 0.07, z0 + 2.62), TRIM_C)

        # the barred door leaf. The middle one has come off its track and hangs open into the cell.
        bars = []
        for j in range(4):
            yy = j * 0.225
            bars.append(pbox((-0.025, yy, 0.10), (0.025, yy + 0.05, 2.42), "gun_black", bevel=0.0))
        bars.append(pbox((-0.03, 0.0, 0.10), (0.03, 0.95, 0.17), "gun_black", bevel=0.0))
        bars.append(pbox((-0.03, 0.0, 2.34), (0.03, 0.95, 2.42), "gun_black", bevel=0.0))
        bars.append(pbox((-0.04, 0.74, 1.10), (0.04, 0.86, 1.30), "metal", bevel=0.0))
        if i == 1:
            # hanging open: the leaf swung 95 degrees into the cell, resting against the bunk end
            for bm in bars:
                common.transform(bm, rot=(0.0, 0.0, math.radians(-95.0)), loc=(CELL_X0 + 0.06, gap0, 0.0))
                k.model.add(bm)
            k.collider((CELL_X0 + 0.06, gap0 - 0.10, z0), (CELL_X0 + 1.02, gap0 + 0.06, z0 + 2.42),
                       "cell_door_open")
        else:
            for bm in bars:
                k.model.add(bm)
            k.collider((CELL_X0 + 0.05, gap0, z0), (CELL_X0 + 0.20, gap1, z0 + 2.42), "cell_door")

        # inside: a steel bunk, mattress and blanket, a toilet, a bench, scratches and a stain
        ym = (yn + ys) / 2.0
        for lx in (CELL_X1 - 0.94, CELL_X1 - 0.26):
            for ly in (yn - 0.94, yn - 0.16):
                k.box((lx, ly, z0), (lx + 0.06, ly + 0.06, z0 + 0.46), "metal")
        k.solid((CELL_X1 - 0.96, yn - 0.94, z0 + 0.40), (CELL_X1 - 0.18, yn - 0.10, z0 + 0.50), "metal",
                base="bunk")
        k.box((CELL_X1 - 0.93, yn - 0.90, z0 + 0.50), (CELL_X1 - 0.20, yn - 0.14, z0 + 0.62), "rock_light")
        k.box((CELL_X1 - 0.91, yn - 0.72, z0 + 0.62), (CELL_X1 - 0.22, yn - 0.16, z0 + 0.68), "cloth_olive")
        k.box((CELL_X1 - 0.88, yn - 0.86, z0 + 0.62), (CELL_X1 - 0.56, yn - 0.60, z0 + 0.72), "bone_white")
        k.solid((CELL_X1 - 0.58, ys + 0.24, z0), (CELL_X1 - 0.16, ys + 0.62, z0 + 0.42), "metal",
                base="cell_toilet")
        k.box((CELL_X1 - 0.58, ys + 0.54, z0 + 0.42), (CELL_X1 - 0.16, ys + 0.70, z0 + 0.80), "metal")
        k.solid((CELL_X1 - 0.92, ym - 0.32, z0), (CELL_X1 - 0.36, ym + 0.32, z0 + 0.44), "metal",
                base="cell_bench")
        for s in range(4):
            k.box((CELL_X0 + 0.21, yn - 0.50 + s * 0.28, z0 + 0.90 + s * 0.12),
                  (CELL_X0 + 0.235, yn - 0.42 + s * 0.28, z0 + 0.93 + s * 0.12), "rock_dark")
        pk.patch(k, CELL_X0 + 1.2, ym - 0.60, 0.55, 0.36, z0 + 0.006, "blood_dark", seed=70 + i)
        _paper(k, CELL_X0 + 0.95, ym + 0.40, z0 + 0.006, 0.7)

    for i, (cx, cy) in enumerate(((3.4, -1.0), (3.0, -6.6), (4.2, -9.8))):
        pk.patch(k, cx, cy, 0.7, 0.5, FZ + 0.006, "blood_dark" if i == 1 else "concrete_stain", seed=80 + i)
    k.box((2.40, -3.40, FZ), (2.80, -3.00, FZ + 0.06), "metal")  # a dropped tray
    k.box((2.50, -3.35, FZ + 0.06), (2.60, -3.25, FZ + 0.14), "white")
    for _ in range(12):
        _paper(k, rng.uniform(2.2, 4.9), rng.uniform(-10.4, 1.4), FZ + 0.006, rng.uniform(0, 3.1))


def _lockers(k, rng):
    """Locker room: five lockers on the east wall, a bench, duffels, hooks with jackets, a notice board."""
    for i in range(5):
        _locker(k, IX1, LOCKER_Y0 + 0.65 + i * 1.05, 0.50, 1.00, FZ, FZ + 1.90, open_door=(i in (1, 3)))
    k.solid((12.0, -0.4, FZ), (12.6, 1.2, FZ + 0.45), "wood_crate", base="locker_bench")
    k.box((11.95, -0.45, FZ + 0.45), (12.65, 1.25, FZ + 0.52), "bark_light")
    for by in (-0.20, 0.45):
        k.box((12.05, by, FZ), (12.20, by + 0.20, FZ + 0.24), "pants_dark")
    for bx, by in ((13.4, -3.4), (14.6, -2.2)):
        k.box((bx - 0.30, by - 0.22, FZ), (bx + 0.30, by + 0.22, FZ + 0.42), "cloth_olive")
        k.box((bx - 0.16, by - 0.06, FZ + 0.42), (bx + 0.16, by + 0.10, FZ + 0.52), "leather")

    k.box((11.0, LOCKER_Y1 - 0.06, FZ + 1.40), (14.0, LOCKER_Y1 - 0.02, FZ + 2.40), "bark_light")
    for i in range(5):
        k.box((11.15 + i * 0.56, LOCKER_Y1 - 0.02, FZ + 1.50), (11.50 + i * 0.56, LOCKER_Y1 - 0.01,
                                                               FZ + 2.30), "white")
    for i in range(4):
        hx = 15.0 + i * 0.45
        k.box((hx, LOCKER_Y1 - 0.10, FZ + 1.75), (hx + 0.05, LOCKER_Y1 - 0.06, FZ + 1.85), "metal")
    k.box((15.3, LOCKER_Y1 - 0.32, FZ + 1.00), (15.8, LOCKER_Y1 - 0.06, FZ + 1.75), "pants_denim")
    k.box((16.2, LOCKER_Y1 - 0.32, FZ + 1.10), (16.7, LOCKER_Y1 - 0.06, FZ + 1.80), "gun_black")
    for i in range(3):
        pk.patch(k, 11.0 + i * 2.2, -1.8 + i * 1.4, 0.8, 0.6, FZ + 0.006, "concrete_stain", seed=90 + i)
    for _ in range(8):
        _paper(k, rng.uniform(9.8, 17.2), rng.uniform(-4.2, 1.4), FZ + 0.006, rng.uniform(0, 3.1))
    _debris(rng, k, 16.2, -3.6, FZ, n=3, spread=0.5)


def _locker(k, x, y, depth, width, z0, z1, open_door=False):
    """One steel locker: a louvred door with a handle, a shelf inside; the door can hang open."""
    x0, x1 = x - depth, x
    k.box((x0, y, z0), (x1, y + width, z1), "shirt_blue")
    k.box((x0 + 0.02, y + 0.03, z0), (x0 + 0.06, y + width - 0.03, z1 - 0.06), "gun_black")
    k.box((x0 + 0.02, y + 0.04, z0 + 1.25), (x1 - 0.04, y + width - 0.04, z0 + 1.30), "metal")
    if open_door:
        door = [pbox((-0.015, 0.0, 0.05), (0.015, width - 0.05, z1 - z0 - 0.04), "metal", bevel=0.008)]
        for j in range(4):
            door.append(pbox((-0.025, 0.09, 0.20 + j * 0.09), (-0.005, width - 0.14, 0.245 + j * 0.09),
                             "gun_black", bevel=0.0))
        door.append(pbox((-0.035, width - 0.20, 0.85), (-0.005, width - 0.11, 1.05), "chrome", bevel=0.004))
        for bm in door:
            common.transform(bm, rot=(0.0, 0.0, math.radians(-74.0)), loc=(x0 - 0.02, y + 0.02, z0))
            k.model.add(bm)
    else:
        k.box((x0 - 0.03, y + 0.03, z0 + 0.05), (x0, y + width - 0.03, z1 - 0.05), "metal")
        for j in range(4):
            k.box((x0 - 0.045, y + 0.10, z1 - 0.30 - j * 0.08),
                  (x0 - 0.030, y + width - 0.14, z1 - 0.25 - j * 0.08), "gun_black")
        k.box((x0 - 0.065, y + width - 0.22, z0 + 0.85), (x0 - 0.030, y + width - 0.12, z0 + 1.05), "chrome")


def _armoury(k, rng):
    """The armoury: a heavy mesh cage with a forced door, empty gun racks, benches, evidence shelves,
    and the back processing area by the sally port."""
    cx0, cx1, cy0, cy1 = 11.6, 17.6, -12.9, -6.4
    z0, z1 = FZ, FZ + 2.55
    for x in (cx0, cx1):
        for y in (cy0, cy1):
            k.box((x - 0.06, y - 0.06, z0), (x + 0.06, y + 0.06, z1), TRIM_C)
    for x in (cx0, cx1):
        k.box((x - 0.06, cy0, z1 - 0.08), (x + 0.06, cy1, z1), TRIM_C)
        k.box((x - 0.06, cy0, z0), (x + 0.06, cy1, z0 + 0.10), TRIM_C)
    for y in (cy0, cy1):
        k.box((cx0, y - 0.06, z1 - 0.08), (cx1, y + 0.06, z1), TRIM_C)
        k.box((cx0, y - 0.06, z0), (cx1, y + 0.06, z0 + 0.10), TRIM_C)

    def mesh_along_y(x, y0a, y1a, gaps=()):
        y = y0a + 0.11
        while y < y1a - 0.03:
            if not any(g0 <= y <= g1 for g0, g1 in gaps):
                k.box((x - 0.022, y - 0.012, z0 + 0.10), (x + 0.022, y + 0.012, z1 - 0.08), "metal")
            y += 0.145
        for z in (z0 + 0.72, z0 + 1.58, z1 - 0.13):
            k.box((x - 0.026, y0a, z - 0.012), (x + 0.026, y1a, z + 0.012), "metal")

    def mesh_along_x(x0a, x1a, y):
        x = x0a + 0.11
        while x < x1a - 0.03:
            k.box((x - 0.012, y - 0.022, z0 + 0.10), (x + 0.012, y + 0.022, z1 - 0.08), "metal")
            x += 0.145
        for z in (z0 + 0.72, z0 + 1.58, z1 - 0.13):
            k.box((x0a, y - 0.026, z - 0.012), (x1a, y + 0.026, z + 0.012), "metal")

    mesh_along_y(cx0, cy0, cy1, gaps=((-7.45, -6.35),))
    mesh_along_x(cx0, cx1, cy1)
    mesh_along_x(cx0, cx1, cy0)

    # the forced door, bent out of its frame
    leaf = []
    for j in range(6):
        leaf.append(pbox((-0.022, j * 0.175, 0.10), (0.022, j * 0.175 + 0.05, 1.85), "metal", bevel=0.0))
    leaf.append(pbox((-0.03, 0.0, 0.15), (0.03, 1.06, 0.22), "metal", bevel=0.0))
    leaf.append(pbox((-0.03, 0.0, 1.78), (0.03, 1.06, 1.86), "metal", bevel=0.0))
    leaf.append(pbox((-0.05, 0.80, 0.90), (0.05, 0.96, 1.10), "metal", bevel=0.006))
    for bm in leaf:
        common.transform(bm, rot=(0.0, 0.0, math.radians(-118.0)), loc=(cx0 - 0.02, -7.45, z0))
        k.model.add(bm)
    k.box((cx0 - 0.06, -6.44, z0 + 1.00), (cx0 + 0.06, -6.20, z0 + 1.12), "metal")

    # empty racks inside the cage
    for y in (-11.6, -8.4):
        k.box((cx0 + 0.20, y - 0.06, z0 + 0.10), (cx1 - 0.20, y + 0.06, z0 + 1.75), "bark_dark")
        k.box((cx0 + 0.20, y - 0.09, z0 + 1.62), (cx1 - 0.20, y + 0.09, z0 + 1.72), "bark_dark")
        k.box((cx0 + 0.20, y - 0.09, z0 + 0.14), (cx1 - 0.20, y + 0.09, z0 + 0.24), "bark_dark")
        for i in range(11):
            x = cx0 + 0.36 + i * 0.50
            k.box((x - 0.02, y - 0.09, z0 + 0.26), (x + 0.02, y + 0.09, z0 + 1.66), "gun_black")
    k.box((cx0 + 0.16, -12.8, z0 + 0.10), (cx1 - 0.16, -12.0, z0 + 0.14), "metal")
    for i in range(4):
        x = cx0 + 0.24 + i * 0.62
        k.box((x, -12.7, z0 + 0.14), (x + 0.42, -12.2, z0 + 0.36), "cloth_olive")
    k.box((12.4, -10.6, z0), (12.8, -10.2, z0 + 0.20), "cloth_olive")

    # outside the cage: the workbench, a cabinet, papers and the evidence shelves
    k.solid((10.4, -9.0, FZ), (11.5, -7.0, FZ + 0.85), "bark_dark", base="armoury_bench")
    k.box((10.35, -9.1, FZ + 0.85), (11.55, -6.9, FZ + 0.92), "metal")
    for i in range(4):
        k.box((10.5, -8.8 + i * 0.45, FZ + 0.92), (10.9, -8.5 + i * 0.45, FZ + 1.02), "gun_black")
    k.solid((9.9, -13.3, FZ), (11.2, -12.2, FZ + 1.50), "metal", base="armoury_cabinet")
    k.box((9.85, -13.35, FZ + 1.50), (11.25, -12.15, FZ + 1.56), TRIM_C)
    k.box((9.85, -12.45, FZ + 0.40), (9.92, -12.15, FZ + 0.52), "chrome")
    for i in range(3):
        pk.patch(k, 11.0 + i * 1.6, -6.2 - i * 1.8, 0.9, 0.6, FZ + 0.006, "concrete_stain", seed=100 + i)
    for _ in range(10):
        _paper(k, rng.uniform(9.6, 17.2), rng.uniform(-13.2, -5.0), FZ + 0.006, rng.uniform(0, 3.1))
    _shelf_unit(k, 12.4, 17.2, -13.6, 0.55, FZ, FZ + 2.10, shelves=4)
    for i in range(14):
        x = 12.5 + (i % 7) * 0.68
        z = FZ + 0.05 + (i // 7) * 1.02
        k.box((x, -13.5, z), (x + 0.56, -13.1, z + rng.uniform(0.22, 0.40)),
              rng.choice(("wood_crate", "bone_white", "cloth_olive")))

    # the back processing area by the sally port: a counter, a bench, papers
    k.solid((5.8, -13.4, FZ), (8.6, -12.9, FZ + 0.80), "metal", base="sally_desk")
    k.box((5.7, -13.5, FZ + 0.80), (8.7, -12.8, FZ + 0.86), "bark_light")
    k.box((6.20, -13.2, FZ + 0.86), (6.70, -13.0, FZ + 1.05), "white")
    k.box((7.30, -13.25, FZ + 0.86), (7.80, -13.05, FZ + 0.90), "white")
    k.solid((2.2, -13.4, FZ), (4.6, -13.1, FZ + 0.45), "wood_crate", base="sally_bench")
    pk.patch(k, 6.5, -11.0, 1.1, 0.7, FZ + 0.006, "blood_dark", seed=110)
    _debris(rng, k, 3.6, -11.8, FZ, n=4, spread=0.7)


# ---------------------------------------------------------------- markers


def _empties(k):
    """Fourteen spawn points: mostly inside the rooms, a few at the doors."""
    spots = ((0.0, 10.4, FZ), (6.4, 8.6, FZ), (-14.0, 9.0, FZ), (13.2, 9.2, FZ),
             (-12.0, -2.2, FZ), (-4.6, -8.6, FZ), (-15.0, -10.0, FZ), (3.5, -2.0, FZ),
             (7.2, -4.4, FZ), (13.0, -1.4, FZ), (13.4, -8.6, FZ), (0.0, 13.0, FZ),
             (0.0, Y1 + 2.6, 0.12), (7.0, Y0 - 2.2, 0.12))
    for i, (x, y, z) in enumerate(spots):
        pk.spawn(k, i + 1, x, y, z)
    pk.marker(k, "lot_size", 0.0, 0.0, 0.0)


# ---------------------------------------------------------------- models


def build_metro_police(model):
    k = kit.Kit(model)
    rng = random.Random(42)
    _site(k)
    _front_facade(k)
    _back_and_sides(k)
    _interior_shell(k)
    _lobby(k, rng)
    _waiting(k, rng)
    _evidence(k, rng)
    _office(k, rng)
    _cells(k, rng)
    _lockers(k, rng)
    _armoury(k, rng)
    _lamps_pole(k)
    _upper_floors(k)
    _empties(k)


def build_metro_police_lod1(model):
    """The outside only: massing, openings painted flat, the tiered facade and the roof silhouettes."""
    k = kit.Kit(model)
    k.box((X0 - 1.0, Y0 - 1.0, -0.02), (X1 + 1.0, Y1 + 3.6, 0.02), "concrete_stain")
    for i, (ya, yb, zt) in enumerate(((Y1 + 1.10, Y1 + 1.70, 0.05), (Y1 + 0.55, Y1 + 1.10, 0.10),
                                      (Y1, Y1 + 0.55, FZ))):
        k.box((-5.0, ya, 0.0), (5.0, yb, zt), "rock_light")
    # ground storey as one box with a door gap and flat window panes
    k.solid((X0, Y1 - T, 0.0), (-1.6, Y1, G_TOP), WALL_C, base="lod_body_f")
    k.solid((1.6, Y1 - T, 0.0), (X1, Y1, G_TOP), WALL_C, base="lod_body_f2")
    k.solid((X0, Y1 - T, FZ + 2.4), (1.6, Y1, G_TOP), WALL_C, base="lod_body_fl")
    k.solid((X0, Y0, 0.0), (X1, Y0 + T, G_TOP), WALL_C, base="lod_body_b")
    k.solid((X0, Y0 + T, 0.0), (X0 + T, Y1 - T, G_TOP), WALL_C, base="lod_body_l")
    k.solid((X1 - T, Y0 + T, 0.0), (X1, Y1 - T, G_TOP), WALL_C, base="lod_body_r")
    for cx in (-15.0, -10.5, -6.2, 6.2, 10.5, 15.0):
        k.box((cx - 1.3, Y1 - T, 1.2), (cx + 1.3, Y1 - T + 0.05, 3.1), "glass_dark")
    for cx in (-15.0, -10.5, -6.2, 6.2, 10.5, 15.0):
        k.box((cx - 1.3, Y1 - 0.02, 1.2), (cx + 1.3, Y1 + 0.02, 3.05), "gun_black")
    k.box((-3.5, Y1 - 0.02, 3.00), (3.5, Y1 + 0.10, 4.00), "gun_black")
    k.box((-1.55, Y1 - 0.22, FZ), (1.55, Y1 - 0.12, FZ + 2.36), "metal")
    for yv in (11.0, 7.0, 3.0, -2.0, -7.0, -11.0):
        k.box((X0, yv - 1.1, 1.2), (X0 + 0.05, yv + 1.1, 3.0), "glass_dark")
    for yv in (10.0, 6.0, 1.0, -3.5, -8.0, -12.2):
        k.box((X1 - 0.05, yv - 1.1, 1.2), (X1, yv + 1.1, 3.0), "glass_dark")
    k.box((5.6, Y0, FZ), (8.2, Y0 + 0.05 + T, FZ + 2.6), "soot")
    for cx in (-15.0, -9.5, -3.5, 2.0, 12.5, 16.0):
        k.box((cx - 1.1, Y0, 1.2), (cx + 1.1, Y0 + 0.05, 3.0), "glass_dark")

    # upper storeys, string courses and flat window bands
    for z0, z1 in ((G_TOP, F2_TOP), (F2_TOP, F3_TOP)):
        k.solid((X0, Y0, z0), (X1, Y1, z1), WALL_C, base="lod_upper")
        for cx in (-15.5, -11.5, -7.5, -3.5, 3.5, 7.5, 11.5, 15.5):
            k.box((cx - 1.15, Y1, z0 + 0.70), (cx + 1.15, Y1 + 0.05, z0 + 2.90), "glass_dark")
        for cx in (-15.5, -11.0, -6.5, -2.0, 2.5, 7.0, 11.5, 15.5):
            k.box((cx - 1.15, Y0 - 0.05, z0 + 0.70), (cx + 1.15, Y0, z0 + 2.90), "glass_dark")
        for cy in (-11.0, -6.5, -2.0, 2.5, 7.0, 11.0):
            k.box((X0 - 0.05, cy - 1.1, z0 + 0.70), (X0, cy + 1.1, z0 + 2.90), "glass_dark")
            k.box((X1, cy - 1.1, z0 + 0.70), (X1 + 0.05, cy + 1.1, z0 + 2.90), "glass_dark")
    for z in (G_TOP - 0.06, F2_TOP - 0.06):
        k.box((X0 - 0.12, Y1, z), (X1 + 0.12, Y1 + 0.16, z + 0.20), BAND_C)
        k.box((X0 - 0.12, Y0 - 0.16, z), (X1 + 0.12, Y0, z + 0.20), BAND_C)
        k.box((X0 - 0.16, Y0, z), (X0, Y1, z + 0.20), BAND_C)
        k.box((X1, Y0, z), (X1 + 0.16, Y1, z + 0.20), BAND_C)

    k.box((X0 - 0.12, Y0 - 0.12, F3_TOP), (X1 + 0.12, Y1 + 0.12, ROOF_TOP), "concrete_grey")
    for lo, hi in (((X0 - 0.12, Y1 - T, ROOF_TOP), (X1 + 0.12, Y1 + 0.12, PARAPET)),
                   ((X0 - 0.12, Y0 - 0.12, ROOF_TOP), (X1 + 0.12, Y0 + T, PARAPET)),
                   ((X0 - 0.12, Y0 + T, ROOF_TOP), (X0 + T, Y1 - T, PARAPET)),
                   ((X1 - T, Y0 + T, ROOF_TOP), (X1 + 0.12, Y1 - T, PARAPET))):
        k.box(lo, hi, WALL_C)
    # rooftop silhouettes
    k.box((2.0, 6.0, ROOF_TOP), (6.0, 10.0, ROOF_TOP + 2.6), WALL_C)
    for ax, ay, ah in ((-14.0, -9.5, 4.6), (15.0, -10.0, 3.8), (-8.0, 9.0, 7.6)):
        k.box((ax - 0.10, ay - 0.10, ROOF_TOP), (ax + 0.10, ay + 0.10, ROOF_TOP + ah), "metal")
    for ax, ay in ((-8.0, -4.0), (-3.0, -6.5), (8.0, -8.0)):
        k.box((ax - 1.2, ay - 1.2, ROOF_TOP), (ax + 1.2, ay + 1.2, ROOF_TOP + 0.95), "metal")
    # the steps, lamp posts and the flagpole (small but they shape the street)
    for sx in (-1, 1):
        k.box((sx * 6.4 - 0.10, Y1 + 1.99, 0.0), (sx * 6.4 + 0.10, Y1 + 2.01 + 0.20, 3.10), "gun_black")
        k.box((sx * 6.4 - 0.15, Y1 + 1.85, 3.18), (sx * 6.4 + 0.15, Y1 + 2.15, 3.72), "water_blue")
    k.box((-8.48, Y1 + 2.52, 0.50), (-8.32, Y1 + 2.68, 8.60), "metal")


def _render_extra(obj_name, view, suffix):
    """A second preview from another camera. `finalize_and_export` hides the visual after the
    top-down render, so unhide it first (this view wants the roof on, like the front preview)."""
    obj = bpy.data.objects.get(obj_name)
    if obj is None:
        return
    obj.hide_render = False
    common.render_preview(obj_name + suffix, obj, view=view)


def _render_cutaway(size=900):
    """The cut-away previews: the building is built again and only the ground-floor parts are kept
    (everything marked `roof=True` — the upper storeys, the roof, the ceiling — is dropped), then
    rendered from straight above and from the front-right corner."""
    common.reset_scene()
    model = common.Model(NAME, CATEGORY, BUDGET, top_preview=False)
    build_metro_police(model)
    ground = [bm.copy() for bm, roof in model._visual if not roof]
    obj = common._to_object(common.combine(ground), NAME + "_cut", painted=True)
    old = common.PREVIEW_SIZE
    common.PREVIEW_SIZE = size
    try:
        common.render_preview(NAME + "_cut", obj, top=True)
        common.render_preview(NAME + "_cut_iso", obj, view=(0.75, 0.85, 1.0))
    finally:
        common.PREVIEW_SIZE = old


def build_all():
    """Builds `metro_police` and `metro_police_lod1`. Returns [(name, ok)].

    `grip=(0, 0, 0)` keeps the origin at the bottom centre of the footprint: the apron, the steps
    and the lamp posts stick out past it, so the automatic bounding-box centring would shift it."""
    ok = common.run_model(NAME, CATEGORY, BUDGET, build_metro_police, grip=(0.0, 0.0, 0.0),
                          top_preview=True, view=FRONT_VIEW)
    _render_extra(NAME, BACK_VIEW, "_back")
    lod_ok = common.run_model(NAME + "_lod1", CATEGORY, LOD_BUDGET, build_metro_police_lod1,
                              grip=(0.0, 0.0, 0.0), view=FRONT_VIEW)
    _render_cutaway()
    return [(NAME, ok), (NAME + "_lod1", lod_ok)]


if __name__ == "__main__":
    failed = [name for name, ok in build_all() if not ok]
    if failed:
        print("METRO POLICE FAILED:", ", ".join(failed))
        sys.exit(1)
    print("METRO POLICE OK")
