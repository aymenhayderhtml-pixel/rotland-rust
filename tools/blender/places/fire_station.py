"""metro_fire_station (34 x 26 m, 2 floors + a 4-storey hose-drying tower) and its `_lod1`.

Build coordinates are metres: +X right, +Y front (the street side), +Z up, ground at Z = 0.
The origin is the bottom centre of the footprint (`grip=(0, 0, 0)`), so the JSON can place props by
building-local metres with front = game -Z.

Ground floor is walk-in (floor slab at z = 0.15, 0.3 m walls, door gaps with no collider):
  the bay hall across the whole front (three 5 m x 5 m garage openings),
  a service strip behind it (gear room, lockers, kitchen/day room, stair),
  and a hose-drying tower at the front-left corner with the brass pole dropped into the bay hall.
Upper floor is a closed brick facade with windows, grime and vines.
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
           os.path.join(BLENDER, "towers")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import bmesh  # noqa: E402
import bpy  # noqa: E402

import brick_parts  # noqa: E402
import common  # noqa: E402
import fire_props  # noqa: E402
import kit  # noqa: E402
import place_kit as pk  # noqa: E402
from jam_util import box, pbox  # noqa: E402

NAME = "metro_fire_station"
CATEGORY = "buildings"
BUDGET = 90000          # data: R42-03 "Rules for every R42 building" (main building 90k / 6k)
BUDGET_LOD1 = 6000

# new: DECISIONS 2026-11-27 (fire station dimensions, metres)
HX = 17.0               # half of the 34 m width
HY = 13.0               # half of the 26 m depth
GF_TOP = 6.2            # ground floor ceiling: tall enough for the 5 m garage bays plus the sign band
F1_TOP = 10.0           # 1st floor top (3.8 m storey)
ROOF_TOP = 12.1         # parapet top (11.2 m wall + a 0.9 m parapet)
F = pk.FLOOR_Z          # 0.15 raised walkable slab

# Hose tower: 5 x 5 m at the front-LEFT corner (x -17 .. -12, y 8 .. 13)
TX0, TX1 = -HX, -HX + 5.0
TY0, TY1 = HY - 5.0, HY
TOWER_H = 20.4          # 4 storeys of 5.05 m each, above the taller ground floor
TOWER_STOREY = (TOWER_H - 0.15) / 4.0

# The three garage bays. Openings are cut through the whole front wall (y 12.7 .. 13.0).
BAY_W = 5.0
BAY_H = 5.0
BAY_CENTRES = (-8.0, -1.5, 5.0)     # bay 1, bay 2, bay 3
BAY_Y0, BAY_Y1 = HY - pk.WALL_T, HY  # 12.7 .. 13.0
OFFICE_DOOR_X = (9.4, 11.2)          # side office door on the front wall, right of bay 3

# Interior: the bay hall runs y -1.0 .. 12.7; behind it four rooms across the full depth.
HALL_BACK_Y = -1.0
GEAR_X = (-HX + pk.WALL_T, -8.2)     # gear room (turnout coats, helmets, lockers)
STAIR_X = (-8.2, -4.6)               # stair up to the closed first floor, blocked by rubble
DAY_X = (-4.6, 7.0)                  # day room (table, chairs, bunk, TV)
KITCHEN_X = (7.0, HX - pk.WALL_T)    # kitchen run + toilet cubicle
ROOM_Y = (-HY + pk.WALL_T, HALL_BACK_Y - pk.WALL_T)   # -12.7 .. -1.3
DOORS = {                            # door gaps in the spine wall, per room
    "gear": (-15.2, -13.3),
    "stair": (-7.8, -6.0),
    "day": (-1.2, 0.6),
    "kitchen": (10.0, 11.8),
}

BRICK = "brick_red"
BRICK_DARK = "brick_dark"
TRIM = "limestone"
TRIM_DARK = "limestone_dark"
INTERIOR = "rock_light"
CEIL = "brick_dark"
PAINT_RED = "red_paint"


# ---------------------------------------------------------------- facade helpers


def _band(k, z0, z1, color, out=0.10):
    """A stone band running right round the block: four strips proud of the brick, never a slab
    across the footprint (a solid box would cover the whole top preview)."""
    k.box((-HX - out, HY - 0.02, z0), (HX + out, HY + out, z1), color, roof=True)
    k.box((-HX - out, -HY - out, z0), (HX + out, -HY + 0.02, z1), color, roof=True)
    k.box((-HX - out, -HY + 0.02, z0), (-HX + 0.02, HY - 0.02, z1), color, roof=True)
    k.box((HX - 0.02, -HY + 0.02, z0), (HX + out, HY - 0.02, z1), color, roof=True)


def face_u(axis, sign, coord):
    """place_kit's `u` for a point at world `coord` along a wall face.

    `place_kit._face_point` measures u left-to-right as seen from OUTSIDE, so a +Y wall maps
    u = -x, a -Y wall u = +x, a +X wall u = +y and a -X wall u = -y.
    """
    return -sign * coord if axis == 1 else sign * coord


def _pilaster(k, x, y, w, d, z0, z1, color=BRICK_DARK):
    k.box((x - w / 2, y - d / 2, z0), (x + w / 2, y + d / 2, z1), color)


def _stone_quoins(k, x, y, sx, sy, z0, z1, step=0.55, depth=0.13, width=0.55):
    """Alternating stone blocks up a corner (`sx`, `sy` are +-1: which way the corner faces)."""
    z = z0
    i = 0
    while z < z1 - 0.1:
        h = step if i % 2 == 0 else step * 0.7
        w = width if i % 2 == 0 else width * 0.68
        x0, x1 = sorted((x, x + sx * w))
        y0, y1 = sorted((y, y + sy * depth))
        k.box((x0, y0, z), (x1, y1, min(z1, z + h)), TRIM)
        i += 1
        z += h


# ---------------------------------------------------------------- outside


def build_outside(k):
    """Plinth, brick walls with the three bay openings and the office door, stone trim, storefront sign."""
    # Plinth / apron slab that the whole building stands on
    k.solid((-HX - 0.4, -HY - 0.4, 0.0), (HX + 0.4, HY + 0.9, 0.06), "concrete_stain", base="apron")

    bay_openings = []
    for cx in BAY_CENTRES:
        bay_openings.append(pk.opening(cx - BAY_W / 2, cx + BAY_W / 2, BAY_Y0, HY, F, BAY_H))
    office_door = pk.opening(OFFICE_DOOR_X[0], OFFICE_DOOR_X[1], BAY_Y0, HY, F, F + pk.DOOR_H)

    # Front wall (y 12.7 .. 13.0) with the three bays and the office door as gaps. It starts at the
    # tower's east face: the tower fills the front-left corner with its own walls.
    k.wall((TX1, BAY_Y0, F), (HX, HY, GF_TOP), openings=bay_openings + [office_door],
           color=BRICK, base="wall_front")
    # Back wall, and the left wall only up to the tower's back face
    k.wall((-HX, -HY, F), (HX, -HY + pk.WALL_T, GF_TOP), color=BRICK, base="wall_back")
    k.wall((-HX, -HY + pk.WALL_T, F), (-HX + pk.WALL_T, TY0, GF_TOP), color=BRICK, base="wall_left")
    k.wall((HX - pk.WALL_T, -HY + pk.WALL_T, F), (HX, BAY_Y0, GF_TOP), color=BRICK, base="wall_right")

    # Bay piers and lintels, so each opening reads as its own garage bay
    for i, cx in enumerate(BAY_CENTRES):
        _pilaster(k, cx - BAY_W / 2 - 0.36, BAY_Y1 - 0.16, 0.72, 0.62, F, GF_TOP)
        _pilaster(k, cx + BAY_W / 2 + 0.36, BAY_Y1 - 0.16, 0.72, 0.62, F, GF_TOP)
        # Steel lintel over the bay and a stone course above it
        k.box((cx - BAY_W / 2 - 0.72, BAY_Y1 - 0.34, BAY_H), (cx + BAY_W / 2 + 0.72, BAY_Y1 + 0.10, BAY_H + 0.26),
              "metal")
        k.box((cx - BAY_W / 2 - 0.72, BAY_Y1 - 0.10, BAY_H + 0.26),
              (cx + BAY_W / 2 + 0.72, BAY_Y1 + 0.16, BAY_H + 0.46), TRIM)
        # Roller-door rail stubs inside the reveal (the doors themselves are props)
        for side in (-1, 1):
            x = cx + side * (BAY_W / 2 - 0.09)
            k.box((x - 0.05, BAY_Y1 - 0.16, BAY_H - 0.1), (x + 0.05, BAY_Y1 - 0.04, BAY_H + 0.06), "rust")
        if i == 0:
            # Bay 1: the roller door half up, hanging just under the lintel
            k.model.add(_half_roller_door(cx, BAY_W))
        elif i == 2:
            # Bay 3: the door torn off its rail and hanging skewed across the opening
            torn = fire_props.torn_door_parts(width=BAY_W, height=BAY_H - 0.2)
            common.transform(torn, rot=(0.10, 0.0, 0.06), loc=(cx, BAY_Y1 - 0.10, BAY_H - 0.05))
            k.model.add(torn)
        # "FIRE DEPT" plate and the red light over each bay
        _bay_sign(k, cx)
        _bay_light(k, cx, BAY_H + 1.06)

    # Office door: stone surround, step, transom, and an open steel door leaf swung inward
    k.frame(office_door, HY, 1, color=TRIM, t=0.12, depth=0.10)
    k.solid((OFFICE_DOOR_X[0] - 0.5, HY, 0.0), (OFFICE_DOOR_X[1] + 0.5, HY + 0.6, F), "concrete_grey", base="step")
    k.box((OFFICE_DOOR_X[0] + 0.04, HY - 0.16, F + 2.05), (OFFICE_DOOR_X[1] - 0.04, HY - 0.02, F + pk.DOOR_H - 0.05),
          "glass_dark")
    leaf = pbox((0.0, -0.03, 0.0), (0.74, 0.03, pk.DOOR_H - 0.06), "metal", bevel=0.008)
    common.transform(leaf, rot=(0.0, 0.0, -1.15), loc=(OFFICE_DOOR_X[0] + 0.06, HY - 0.16, F))
    k.model.add(leaf)
    k.box((OFFICE_DOOR_X[0] - 0.7, HY - 0.02, F + 2.62), (OFFICE_DOOR_X[1] + 0.7, HY + 0.14, F + 3.05), "metal")

    # Stone bands: a belt course over the bays and the first-floor sill band
    _band(k, GF_TOP - 0.30, GF_TOP, TRIM, out=0.11)
    _band(k, F1_TOP - 0.34, F1_TOP - 0.06, TRIM, out=0.12)
    _band(k, 0.06, 0.52, TRIM_DARK, out=0.10)
    # Stone quoins on the three free corners
    _stone_quoins(k, -HX, HY, 1, 1, F, GF_TOP)
    _stone_quoins(k, HX, HY, -1, 1, F, GF_TOP)
    _stone_quoins(k, HX, -HY, -1, 1, F, GF_TOP)
    _stone_quoins(k, -HX, -HY, 1, 1, F, GF_TOP)
    # Wall-top coping
    _band(k, GF_TOP, GF_TOP + 0.16, TRIM, out=0.13)


def _half_roller_door(cx, width):
    """Bay 1's roller door, raised about halfway: slats from z 2.4 up to the lintel, plus the roll."""
    parts = []
    half = width / 2.0
    slat = 0.135
    z = 2.42
    i = 0
    while z < BAY_H - 0.10:
        col = "metal" if i % 2 == 0 else "rust"
        parts.append(box((cx - half + 0.06, -0.05, z), (cx + half - 0.06, 0.05, z + slat * 0.82), col))
        z += slat
        i += 1
    # The rolled-up drum just under the lintel + a hanging pull strap
    parts.append(common.make_frustum(0.16, 0.16, -half + 0.2, half - 0.2, segments=10, color="rust_dark"))
    common.transform(parts[-1], rot=(0.0, math.pi / 2.0, 0.0), loc=(cx, 0.0, BAY_H - 0.20))
    parts.append(box((cx - 0.03, -0.06, 2.06), (cx + 0.03, 0.06, 2.52), "leather"))
    # Slat guide shoes in the reveal
    for side in (-1, 1):
        parts.append(box((cx + side * (half - 0.12), -0.12, 2.30), (cx + side * (half - 0.02), 0.12, 2.52), "metal"))
    return common.combine(parts)


def _bay_sign(k, cx):
    """The red "FIRE DEPT" plate over one bay door (z 5.30 .. 5.94, under the top band)."""
    z0 = BAY_H + 0.34
    k.box((cx - 1.30, BAY_Y1 - 0.02, z0), (cx + 1.30, BAY_Y1 + 0.08, z0 + 0.62), PAINT_RED)
    k.box((cx - 1.20, BAY_Y1 + 0.08, z0 + 0.06), (cx + 1.20, BAY_Y1 + 0.10, z0 + 0.56), "soot")
    letters = fire_props.letters_bm("FIRE DEPT", 0.135, 0.38, color="white", depth=0.015)
    fire_props.letters_on(letters, (cx, BAY_Y1 + 0.11, z0 + 0.10), "+Y")
    k.model.add(letters)


def _bay_light(k, cx, z):
    """A red light and its bracket over one bay door."""
    k.box((cx - 0.14, BAY_Y1 + 0.02, z - 0.02), (cx + 0.14, BAY_Y1 + 0.20, z + 0.06), "metal")
    dome = common.make_icosphere(subdiv=1, radius=(0.17, 0.15, 0.17), loc=(cx, BAY_Y1 + 0.20, z + 0.16),
                                 color=PAINT_RED)
    k.model.add(dome)
    k.box((cx - 0.19, BAY_Y1 + 0.08, z + 0.32), (cx + 0.19, BAY_Y1 + 0.30, z + 0.37), "metal")


def _slab_around_tower(k, z0, z1, color, roof=False, base=None):
    """A horizontal slab covering the footprint MINUS the tower square (an L), so the tower shaft
    stays open through it. `base` also adds a matching collider."""
    pieces = [
        (-HX, -HY, HX, TY0),        # full width behind the tower
        (TX1, TY0, HX, HY),         # the strip in front, right of the tower
    ]
    for x0, y0, x1, y1 in pieces:
        k.box((x0, y0, z0), (x1, y1, z1), color, roof=roof)
        if base:
            k.collider((x0, y0, z0), (x1, y1, z1), base)


def build_upper(k):
    """First floor slab, the closed brick first-floor facade with windows, the roof slab and parapet."""
    # Ground-floor ceiling / first-floor slab (visual only; a collider keeps the player under it)
    k.box((-HX + pk.WALL_T, -HY + pk.WALL_T, GF_TOP), (HX - pk.WALL_T, TY0, GF_TOP + 0.30), CEIL, roof=True)
    k.box((TX1, TY0, GF_TOP), (HX - pk.WALL_T, BAY_Y0, GF_TOP + 0.30), CEIL, roof=True)
    k.collider((-HX, -HY, GF_TOP), (HX, BAY_Y0, GF_TOP + 0.30), "ceil1")

    # First-floor walls (closed facade, brick). The front run starts at the tower and the left run
    # stops at it, so the tower keeps its own full-height shaft.
    k.box((TX1, -HY, GF_TOP + 0.16), (HX, HY, F1_TOP), BRICK, roof=True)
    k.box((-HX, -HY, GF_TOP + 0.16), (HX, -HY + pk.WALL_T, F1_TOP), BRICK, roof=True)
    k.box((-HX, -HY, GF_TOP + 0.16), (-HX + pk.WALL_T, TY0, F1_TOP), BRICK, roof=True)
    k.box((HX - pk.WALL_T, -HY, GF_TOP + 0.16), (HX, BAY_Y0, F1_TOP), BRICK, roof=True)
    k.box((-HX, TY0, GF_TOP + 0.16), (TX1, HY - pk.WALL_T, GF_TOP + 0.40), TRIM, roof=True)

    # First-floor windows on all four faces
    z0, z1 = GF_TOP + 1.15, GF_TOP + 3.05
    front_bays = (-8.0, -1.5, 5.0, 11.2)
    pk.window_band(k, HY, 1, 1, [face_u(1, 1, x) for x in front_bays], z0, z1, roof=True)
    pk.window_band(k, -HY, 1, -1, [face_u(1, -1, x) for x in (-13.0, -6.4, 0.2, 6.8, 13.2)], z0, z1, roof=True)
    pk.window_band(k, -HX, 0, -1, [face_u(0, -1, y) for y in (-9.0, -3.0, 3.0)], z0, z1, roof=True)
    pk.window_band(k, HX, 0, 1, [face_u(0, 1, y) for y in (-9.0, -3.0, 3.0)], z0, z1, roof=True)

    # Grime and vines on the brick (2-5 cm proud, never z-fighting)
    for x in (-12.0, 3.0, 9.0):
        pk.grime(k, HY, 1, 1, face_u(1, 1, x), GF_TOP + 0.4, F1_TOP - 0.2, seed=int(abs(x) * 3) + 1,
                 width=3.0, streaks=6, roof=True)
    for x in (-3.2, 9.8):
        pk.vines(k, HY, 1, 1, face_u(1, 1, x), 3.4, GF_TOP + 0.3, F1_TOP - 0.3, seed=int(abs(x) * 5) + 2,
                 leaves=62, roof=True)
    pk.grime(k, -HY, 1, -1, face_u(1, -1, 3.2), GF_TOP + 0.4, F1_TOP - 0.2, seed=77, width=5.0, streaks=7,
             roof=True)
    pk.vines(k, -HY, 1, -1, face_u(1, -1, -8.0), 3.0, GF_TOP + 0.3, F1_TOP - 0.3, seed=78, leaves=55, roof=True)
    pk.grime(k, HX, 0, 1, face_u(0, 1, -6.0), GF_TOP + 0.4, F1_TOP - 0.2, seed=79, width=4.0, streaks=5,
             roof=True)
    pk.vines(k, -HX, 0, -1, face_u(0, -1, -4.0), 2.6, 1.2, GF_TOP - 0.3, seed=80, leaves=48, roof=True)

    # Roof slab, parapet and coping (an L around the tower, which rises through the roof)
    _slab_around_tower(k, F1_TOP, F1_TOP + 0.26, "rock_dark", roof=True, base="roof_slab")
    t = 0.34
    k.box((TX1, HY - t, F1_TOP + 0.26), (HX, HY, ROOF_TOP), BRICK, roof=True)
    k.box((-HX, -HY, F1_TOP + 0.26), (HX, -HY + t, ROOF_TOP), BRICK, roof=True)
    k.box((-HX, -HY + t, F1_TOP + 0.26), (-HX + t, TY0, ROOF_TOP), BRICK, roof=True)
    k.box((HX - t, -HY + t, F1_TOP + 0.26), (HX, HY - t, ROOF_TOP), BRICK, roof=True)
    k.box((TX1, TY0, F1_TOP + 0.26), (TX1 + t, HY - t, ROOF_TOP), BRICK, roof=True)
    k.box((TX1, HY - t - 0.06, ROOF_TOP), (HX + 0.08, HY + 0.06, ROOF_TOP + 0.14), TRIM, roof=True)
    k.box((-HX - 0.08, -HY - 0.06, ROOF_TOP), (HX + 0.08, -HY + t + 0.06, ROOF_TOP + 0.14), TRIM, roof=True)
    k.box((-HX - 0.08, -HY + t, ROOF_TOP), (-HX + t + 0.06, TY0, ROOF_TOP + 0.14), TRIM, roof=True)
    k.box((HX - t - 0.06, -HY + t, ROOF_TOP), (HX + 0.08, HY - t, ROOF_TOP + 0.14), TRIM, roof=True)
    k.box((TX1 - 0.06, TY0, ROOF_TOP), (TX1 + t + 0.06, HY - t, ROOF_TOP + 0.14), TRIM, roof=True)

    # Roof equipment: two AC units, a vent stack, a leaning antenna, a hose-drying frame
    pk.ac_unit(k, 3.4, -6.4, F1_TOP + 0.26)
    pk.ac_unit(k, -8.6, -3.0, F1_TOP + 0.26)
    pk.roof_vent(k, 12.0, -10.0, F1_TOP + 0.26, h=1.5)
    pk.roof_antenna(k, 13.6, 10.2, F1_TOP + 0.26, h=3.6)
    for i, (x, y) in enumerate(((-2.0, 6.0), (2.0, 6.0))):
        k.box((x - 0.08, y - 0.08, F1_TOP + 0.26), (x + 0.08, y + 0.08, F1_TOP + 2.1), "rust", roof=True)
    k.box((-2.1, 5.9, F1_TOP + 2.0), (2.1, 6.1, F1_TOP + 2.2), "rust", roof=True)
    # Two mast lights on the roof corners and a water tank
    for mx, my in ((-14.0, -11.0), (14.0, 10.0)):
        k.box((mx - 0.10, my - 0.10, F1_TOP + 0.26), (mx + 0.10, my + 0.10, F1_TOP + 3.2), "metal", roof=True)
        k.box((mx - 0.55, my - 0.30, F1_TOP + 3.2), (mx + 0.20, my + 0.30, F1_TOP + 3.42), "metal", roof=True)
        k.box((mx - 0.50, my - 0.26, F1_TOP + 3.14), (mx - 0.10, my + 0.26, F1_TOP + 3.20), "white", roof=True)
    tank = common.make_frustum(1.10, 1.10, 0.0, 2.20, segments=12, color="rust")
    common.transform(tank, loc=(9.4, -11.4, F1_TOP + 1.40))
    k.model.add(tank, roof=True)
    for lx, ly in ((8.5, -12.3), (10.3, -12.3), (8.5, -10.5), (10.3, -10.5)):
        k.box((lx - 0.09, ly - 0.09, F1_TOP + 0.26), (lx + 0.09, ly + 0.09, F1_TOP + 1.40), "metal", roof=True)
    # Roof hatch with a raised kerb
    k.box((6.0, 3.4, F1_TOP + 0.26), (8.0, 5.4, F1_TOP + 0.50), "metal", roof=True)
    k.box((6.1, 3.5, F1_TOP + 0.50), (7.9, 5.3, F1_TOP + 0.56), "rust", roof=True)


def build_tower(k):
    """The 4-storey hose-drying tower at the front-left corner: brick shaft, louvred top, hose frames."""
    x0, x1, y0, y1 = TX0, TX1, TY0, TY1
    # The shaft sits in front of the wall line, so it gets its own four walls. The ground floor is
    # part of the station: a door gap in its east wall lets the player walk in under the tower.
    door = pk.opening(x1 - pk.WALL_T, x1, y0 + 2.0, y0 + 3.8, F, F + pk.DOOR_H)
    k.wall((x0, y0, F), (x0 + pk.WALL_T, y1, TOWER_H), color=BRICK_DARK, base="tower_w")
    k.box((x0 + pk.WALL_T, y1 - pk.WALL_T, F), (x1, y1, TOWER_H), BRICK, roof=True)
    k.wall((x1 - pk.WALL_T, y0, F), (x1, y0 + pk.WALL_T, TOWER_H), openings=[door],
           color=BRICK_DARK, base="tower_e")
    k.wall((x0 + pk.WALL_T, y0, F), (x1 - pk.WALL_T, y0 + pk.WALL_T, TOWER_H), color=BRICK_DARK, base="tower_s")
    # Tank floor of the tower at ground level, and the vertical shaft of the drying well above it
    k.box((x0 + pk.WALL_T, y0 + pk.WALL_T, F), (x1 - pk.WALL_T, y1 - pk.WALL_T, F + 3.4), "soot", roof=True)
    for a in range(1, 4):
        k.box((x0 + pk.WALL_T, y0 + pk.WALL_T, F + a * TOWER_STOREY - 0.30),
              (x1 - pk.WALL_T, y1 - pk.WALL_T, F + a * TOWER_STOREY - 0.12), "rust_dark", roof=True)
    # A drying rack frame in the tower base with four hoses hanging from it
    k.box((x0 + 0.6, y1 - 2.6, F + 2.9), (x1 - 0.6, y1 - 2.4, F + 3.0), "metal", roof=True)
    for i in range(4):
        x = x0 + 1.0 + i * 0.75
        k.box((x - 0.04, y1 - 2.55, F + 1.1), (x + 0.04, y1 - 2.45, F + 2.9), "cloth_olive", roof=True)
        k.box((x - 0.06, y1 - 2.58, F + 1.05), (x + 0.06, y1 - 2.42, F + 1.12), "brass", roof=True)
    # Pole guide: the brass pole passes the first-floor hole down here too (visual, above the ceiling)
    k.box((x0 + 2.1, y0 + 2.1, GF_TOP), (x0 + 2.4, y0 + 2.4, GF_TOP + 0.3), "metal", roof=True)
    # Stone bands every storey (the tower reads as four stacked storeys)
    for a in range(1, 4):
        z = F + a * TOWER_STOREY
        k.box((x0 - 0.10, y0 - 0.10, z - 0.14), (x1 + 0.10, y1 + 0.10, z + 0.14), TRIM, roof=True)
    k.box((x0 - 0.12, y0 - 0.12, TOWER_H - 0.40), (x1 + 0.12, y1 + 0.12, TOWER_H), TRIM, roof=True)
    k.box((x0 - 0.20, y0 - 0.20, TOWER_H), (x1 + 0.20, y1 + 0.20, TOWER_H + 0.18), "rock_dark", roof=True)
    # Corner quoins up the two street corners
    _stone_quoins(k, x0, y1, 1, -1, F, TOWER_H - 0.5, step=0.6, width=0.5)
    _stone_quoins(k, x1, y1, -1, -1, F, TOWER_H - 0.5, step=0.6, width=0.5)
    # Tall louvred drying vents on three faces at every storey
    for a in range(4):
        z = F + a * TOWER_STOREY + 1.0
        for face_coord, axis, sign, u in ((y1, 1, 1, face_u(1, 1, (x0 + x1) / 2.0)),
                                          (x1, 0, 1, face_u(0, 1, (y0 + y1) / 2.0)),
                                          (x0, 0, -1, face_u(0, -1, (y0 + y1) / 2.0))):
            pk.face_box(k, face_coord, axis, sign, u - 0.85, u + 0.85, z, z + 2.2, 0.0, 0.09, BRICK_DARK, roof=True)
            for li in range(9):
                lz = z + 0.10 + li * 0.24
                pk.face_box(k, face_coord, axis, sign, u - 0.80, u + 0.80, lz, lz + 0.10, 0.06, 0.13,
                            "metal" if li % 2 == 0 else "rust", roof=True)
    # The tower's little pitched cap and a lightning rod
    ridge_z = TOWER_H + 1.5
    for side in (-1, 1):
        slope = common.make_box((x1 - x0 + 0.5, 2.7, 0.14), color="rust_dark")
        common.transform(slope, rot=(side * 0.55, 0.0, 0.0),
                         loc=((x0 + x1) / 2.0, (y0 + y1) / 2.0 + side * 1.22, TOWER_H + 0.75))
        k.model.add(slope, roof=True)
    k.model.add(common.make_frustum(0.05, 0.02, ridge_z, ridge_z + 2.2, segments=8,
                                    center=((x0 + x1) / 2.0, (y0 + y1) / 2.0), color="metal"))
    # Hose-drying frames poking out of the top vents
    for i, x in enumerate((x0 + 1.1, (x0 + x1) / 2.0, x1 - 1.1)):
        k.box((x - 0.05, y1 - 0.30, TOWER_H - 1.4), (x + 0.05, y1 + 0.60, TOWER_H - 1.3), "metal", roof=True)
    # Drying hoses visible through the top louvre
    for i in range(4):
        x = x0 + 0.9 + i * 1.05
        k.box((x - 0.04, y1 - 0.55, TOWER_H - 3.2), (x + 0.04, y1 - 0.48, TOWER_H - 1.45), "cloth_olive", roof=True)
    # Pipe: the tower connects to the roof of the main block
    k.box((x1 - 0.30, y0 + 0.30, F1_TOP), (x1 + 0.10, TY1 - 0.30, F1_TOP + 0.40), "rust", roof=True)


# ---------------------------------------------------------------- ground floor inside


def build_hall_floor(k):
    """The bay hall floor: concrete slab, painted bay outlines, drains, oil stains, rubble."""
    rng = random.Random(91)
    # Floor slab over the whole footprint (walkable, with a collider)
    pk.floor_slab(k, -HX, -HY, HX, HY, z_top=F, color="concrete_grey")
    # A darker, polished bay-hall floor in front of the doors
    pk.floor_tiles(k, -HX + pk.WALL_T, HALL_BACK_Y, HX - pk.WALL_T, BAY_Y0, F + 0.004,
                   tile=4.0, color_a="concrete_stain", color_b="concrete_grey", inset=0.02)
    # Painted bay markings in front of each door: two long side lines and a back stop line
    for cx in BAY_CENTRES:
        for side in (-1, 1):
            x = cx + side * (BAY_W / 2 - 0.18)
            pk.floor_decal(k, [(x - 0.06, 0.4), (x + 0.06, 0.4), (x + 0.06, 12.4), (x - 0.06, 12.4)],
                           F + 0.012, "wheat_gold")
        pk.floor_decal(k, [(cx - BAY_W / 2 + 0.2, 0.34), (cx + BAY_W / 2 - 0.2, 0.34),
                           (cx + BAY_W / 2 - 0.2, 0.5), (cx - BAY_W / 2 + 0.2, 0.5)], F + 0.012, "wheat_gold")
        # A painted number block between the bays
    # Cross-hatched keep-clear hatching in front of bay 3
    for i in range(7):
        x = 5.0 - 2.2 + i * 0.66
        pk.floor_decal(k, [(x, 11.2), (x + 0.30, 11.2), (x + 0.90, 12.5), (x + 0.60, 12.5)], F + 0.012,
                       "wheat_gold")
    # Floor drains (a dark recess plus a grid)
    for dx, dy in ((-8.0, 10.4), (-1.5, 10.4), (5.0, 10.4), (-1.5, 5.0), (-6.0, 0.4), (10.0, 0.4)):
        k.box((dx - 0.30, dy - 0.30, F - 0.02), (dx + 0.30, dy + 0.30, F + 0.01), "rock_dark")
        for gi in range(4):
            k.box((dx - 0.26, dy - 0.22 + gi * 0.15, F + 0.01), (dx + 0.26, dy - 0.12 + gi * 0.15, F + 0.03),
                  "metal")
    # Oil stains and tyre scuffs
    for i, (sx, sy, rx, ry) in enumerate(((-8.4, 7.0, 1.5, 1.0), (-2.6, 3.2, 1.1, 0.9), (4.2, 6.4, 1.4, 1.1),
                                          (10.6, 2.4, 1.2, 0.8), (-12.0, 9.0, 1.6, 1.0), (1.2, 8.6, 0.9, 1.3))):
        pk.patch(k, sx, sy, rx, ry, F + 0.010, "soot", seed=200 + i, jitter=0.4)
    for i, (sx, sy) in enumerate(((-8.0, 6.0), (-1.5, 4.0), (5.0, 7.0))):
        for side in (-1, 1):
            pk.floor_decal(k, [(sx + side * 0.55 - 0.10, sy), (sx + side * 0.55 + 0.10, sy),
                               (sx + side * 0.55 + 0.10, sy + 3.4), (sx + side * 0.55 - 0.10, sy + 3.4)],
                           F + 0.011, "concrete_stain")
    # Painted floor lines leading from each bay back into the hall
    for cx in BAY_CENTRES:
        pk.floor_decal(k, [(cx - 0.06, -0.6), (cx + 0.06, -0.6), (cx + 0.06, 1.2), (cx - 0.06, 1.2)],
                       F + 0.012, "white")


def build_hall_walls(k):
    """Interior partitions of the ground floor: the spine wall behind the bay hall, the three room
    dividers behind it and the toilet cubicle in the kitchen corner."""
    ry0, ry1 = ROOM_Y
    spine_openings = [pk.opening(a, b, HALL_BACK_Y - pk.WALL_T, HALL_BACK_Y, F, F + pk.DOOR_H)
                      for a, b in DOORS.values()]
    k.wall((-HX + pk.WALL_T, HALL_BACK_Y - pk.WALL_T, F), (HX - pk.WALL_T, HALL_BACK_Y, GF_TOP),
           openings=spine_openings, color=INTERIOR, base="wall_spine")

    # Room dividers (gear | stair | day | kitchen), each running the depth of the strip
    for x in (GEAR_X[1], STAIR_X[1], DAY_X[1]):
        k.wall((x, ry0, F), (x + pk.WALL_T, ry1, GF_TOP), color=INTERIOR, base="wall_room_div")

    # Toilet cubicle in the back-right corner of the kitchen strip
    k.wall((HX - 2.8, ry0, F), (HX - 2.5, -6.0, GF_TOP), color=INTERIOR, base="wall_toilet")
    k.wall((HX - 2.8, -6.3, F), (HX - pk.WALL_T, -6.0, GF_TOP),
           openings=[pk.opening(HX - 2.1, HX - 0.6, -6.3, -6.0, F, F + pk.DOOR_H)],
           color=INTERIOR, base="wall_toilet")


def build_hall_ceiling(k):
    """The bay hall ceiling: steel beams, pendant lamps, ducting and the pole hole surround."""
    # Steel beams across the hall
    for y in (2.0, 4.8, 7.6, 10.4):
        k.box((-HX + pk.WALL_T, y, GF_TOP - 0.34), (HX - pk.WALL_T, y + 0.26, GF_TOP - 0.02), "metal", roof=True)
        for side in (-1, 1):
            k.box((side * (HX - pk.WALL_T) - side * 0.02, y, GF_TOP - 0.40),
                  (side * (HX - pk.WALL_T) + side * 0.02, y + 0.26, GF_TOP - 0.02), "rust", roof=True)
    # Two runs of trunking along the ceiling with hanging lamp shades
    for x in (-12.0, 11.0):
        k.box((x - 0.16, HALL_BACK_Y, GF_TOP - 0.22), (x + 0.16, BAY_Y0, GF_TOP - 0.10), "metal", roof=True)
        for y in (1.6, 4.4, 7.2, 10.0):
            k.model.add(common.make_frustum(0.05, 0.05, GF_TOP - 0.60, GF_TOP - 0.20, segments=6,
                                            center=(x, y), color="metal"), roof=True)
            shade = common.make_frustum(0.30, 0.10, GF_TOP - 0.74, GF_TOP - 0.58, segments=10,
                                        center=(x, y), color="metal")
            k.model.add(shade, roof=True)
            k.model.add(common.make_frustum(0.13, 0.13, GF_TOP - 0.80, GF_TOP - 0.74, segments=8,
                                            center=(x, y), color="white"), roof=True)
    # A big extract duct running to the back wall, with two grilles
    for side in (-1, 1):
        k.box((side * 4.4 - 0.28, BAY_Y0 - 3.0, GF_TOP - 0.72), (side * 4.4 + 0.28, BAY_Y0 - 2.44, GF_TOP - 0.16),
              "metal", roof=True)
        k.box((side * 4.4 - 0.24, BAY_Y0 - 2.9, GF_TOP - 0.68), (side * 4.4 + 0.24, BAY_Y0 - 2.54, GF_TOP - 0.20),
              "rust_dark", roof=True)


def build_hall_dressing(k):
    """Hoses on the walls, a pole hole with the brass pole, hose racks, wall equipment, cones,
    debris and stains."""
    rng = random.Random(52)
    build_hall_wall_kit(k)
    # ---- Hoses hanging on the two side walls of the bay hall ----
    for i, (side, y, z) in enumerate(((-1, 3.2, 2.5), (-1, 3.2, 1.2), (-1, 8.4, 2.5),
                                      (1, 6.6, 2.5), (1, 6.6, 1.2), (1, 10.0, 2.5))):
        _wall_coil(k, side, y, z, "red_paint" if i % 2 == 0 else "cloth_olive")
    # A hose reel with its hose looping down the back wall
    reel = common.make_frustum(0.30, 0.30, -0.10, 0.10, segments=14, color="metal")
    common.transform(reel, rot=(math.pi / 2.0, 0.0, 0.0), loc=(-4.0, HALL_BACK_Y + 0.22, 1.5))
    k.model.add(reel)
    for i in range(9):
        a = 2.0 * math.pi * i / 9
        seg = common.make_box((0.36, 0.06, 0.06), color="red_paint")
        common.transform(seg, rot=(0.0, 0.0, a), loc=(-4.0 + math.cos(a) * 0.30, HALL_BACK_Y + 0.20,
                                                     1.5 + math.sin(a) * 0.30))
        k.model.add(seg)
    k.box((-4.10, HALL_BACK_Y + 0.06, 1.15), (-3.90, HALL_BACK_Y + 0.18, 1.22), "metal")

    # ---- The pole hole in the ceiling with the brass pole ----
    pole_x, pole_y = -9.4, 6.2
    k.box((pole_x - 0.9, pole_y - 0.9, GF_TOP - 0.18), (pole_x + 0.9, pole_y + 0.9, GF_TOP + 0.02), "gun_black",
          roof=True)
    for side in (-1, 1):
        k.box((pole_x + side * 0.9 - 0.06, pole_y - 0.9, GF_TOP - 0.20),
              (pole_x + side * 0.9 + 0.06, pole_y + 0.9, GF_TOP), "metal", roof=True)
        k.box((pole_x - 0.9, pole_y + side * 0.9 - 0.06, GF_TOP - 0.20),
              (pole_x + 0.9, pole_y + side * 0.9 + 0.06, GF_TOP), "metal", roof=True)
    # Brass pole from the first floor (above the ceiling) down to the floor, with a landing pad
    k.model.add(common.make_frustum(0.075, 0.075, F, GF_TOP + 1.4, segments=10, center=(pole_x, pole_y),
                                    color="brass"))
    k.model.add(common.make_frustum(0.20, 0.20, F, F + 0.10, segments=12, center=(pole_x, pole_y), color="brass"))
    k.collider((pole_x - 0.12, pole_y - 0.12, 0.0), (pole_x + 0.12, pole_y + 0.12, GF_TOP), "pole")
    # A chalked landing circle on the floor around the pole
    for i in range(12):
        a0 = 2.0 * math.pi * i / 12
        a1 = 2.0 * math.pi * (i + 1) / 12
        pk.floor_decal(k, [(pole_x + math.cos(a0) * 1.1, pole_y + math.sin(a0) * 1.1),
                           (pole_x + math.cos(a1) * 1.1, pole_y + math.sin(a1) * 1.1),
                           (pole_x + math.cos(a1) * 1.2, pole_y + math.sin(a1) * 1.2),
                           (pole_x + math.cos(a0) * 1.2, pole_y + math.sin(a0) * 1.2)],
                       F + 0.013, "white")

    # ---- Baked wall props: a hose rack each side, a gear rack by the gear door ----
    _place_prop(k, fire_props.build_hose_rack, loc=(-6.0, HALL_BACK_Y + 0.34, F), rot=(0.0, 0.0, math.pi))
    _place_prop(k, fire_props.build_hose_rack, loc=(7.4, HALL_BACK_Y + 0.34, F), rot=(0.0, 0.0, math.pi))
    _place_prop(k, fire_props.build_gear_rack, loc=(GEAR_X[0] + 3.4, -12.0, F), rot=(0.0, 0.0, 0.0))

    # ---- Cones and debris out on the hall floor ----
    for cx, cy in ((-8.6, 9.4), (-8.0, 9.9), (-1.6, 2.0), (5.4, 8.8)):
        _place_prop(k, fire_props.build_traffic_cone, loc=(cx, cy, F), rot=(0.0, 0.0, rng.uniform(0.0, 3.0)))
    # Bundled hose on the floor and a fallen ceiling tile
    for i in range(3):
        coil = common.combine(fire_props.ring_boxes(0.0, 0.0, 0.0, 0.36, seg=12, thick=0.08, depth=0.16,
                                                   color="red_paint", plane="xy"))
        common.transform(coil, loc=(-3.0 + i * 0.6, 0.9 + i * 0.2, F + 0.08 + i * 0.10))
        k.model.add(coil)
    fallen = box((-0.6, -0.6, -0.03), (0.6, 0.6, 0.03), "rock_light")
    common.transform(fallen, rot=(0.22, 0.0, 0.7), loc=(-5.2, 5.4, F + 0.04))
    k.model.add(fallen)
    # Rubble: a heap in the corner and loose pieces along the walls
    _rubble(k, -14.8, 1.2, F, 1.6, 1.2, seed=7)
    _rubble(k, 14.9, 8.6, F, 1.2, 1.4, seed=8)
    _rubble(k, -2.6, 12.2, F, 1.0, 0.8, seed=9)
    for i in range(14):
        x = rng.uniform(-15.6, 15.6)
        y = rng.uniform(-0.4, 12.2)
        s = rng.uniform(0.08, 0.22)
        chunk = common.make_icosphere(subdiv=1, jitter=0.4, seed=300 + i, radius=(s, s, s * 0.6),
                                      loc=(x, y, F + s * 0.4), color=rng.choice(("rock_mid", "rock_dark",
                                                                                "concrete_stain")))
        k.model.add(chunk)
    # Dark stains under the trucks' usual spots and along the wall
    for i, (sx, sy) in enumerate(((-8.0, 5.0), (-1.5, 4.4), (-11.0, 11.4))):
        pk.patch(k, sx + 0.4, sy - 0.4, 1.3, 0.9, F + 0.011, "blood_dark", seed=400 + i, jitter=0.45)


class _PropSink:
    """Collects the parts of a `fire_props` builder so they can be baked into the building."""

    def __init__(self):
        self.visual = []
        self.colliders = []

    def add(self, bm, roof=False):
        self.visual.append(bm)

    def add_collider(self, label, bm):
        self.colliders.append((label, bm))


def build_hall_wall_kit(k):
    """Wall-mounted firehouse equipment in the bay hall: a breathing-apparatus rack, extinguisher
    cupboards, a hose-drying rack, a white board, a wheel-chock shelf, a workbench and shelving."""
    zb = HALL_BACK_Y  # the spine wall the equipment hangs on
    # Breathing-apparatus rack: two steel frames with four cylinders and their harnesses
    for side in (-1, 1):
        x = side * 1.4
        k.box((x - 0.55, zb + 0.02, F), (x + 0.55, zb + 0.42, F + 1.85), "metal")
        for i in range(2):
            cx = x - 0.26 + i * 0.52
            cyl = common.make_frustum(0.11, 0.11, F + 0.10, F + 0.86, segments=10, color="gun_black")
            common.transform(cyl, loc=(cx, zb + 0.24, 0.0))
            k.model.add(cyl)
            k.model.add(common.make_frustum(0.035, 0.035, F + 0.86, F + 0.98, segments=8,
                                            center=(cx, zb + 0.24), color="brass"))
            k.box((cx - 0.13, zb + 0.10, F + 0.70), (cx + 0.13, zb + 0.38, F + 1.50), "cloth_olive")
            k.box((cx - 0.13, zb + 0.36, F + 1.10), (cx + 0.13, zb + 0.40, F + 1.18), "leather")
    # Two red extinguisher cupboards with a glass front and a hanging extinguisher beside each
    for x in (-13.0, 6.4):
        k.box((x - 0.45, zb + 0.02, F + 0.20), (x + 0.45, zb + 0.34, F + 1.20), PAINT_RED)
        k.box((x - 0.38, zb + 0.34, F + 0.28), (x + 0.38, zb + 0.36, F + 1.12), "glass_dark")
        k.box((x + 0.55, zb + 0.10, F + 0.55), (x + 0.75, zb + 0.34, F + 0.62), "metal")
        ext = common.make_frustum(0.09, 0.09, F + 0.06, F + 0.55, segments=10, center=(x + 0.65, zb + 0.24),
                                  color=PAINT_RED)
        k.model.add(ext)
        k.box((x + 0.60, zb + 0.16, F + 0.55), (x + 0.70, zb + 0.32, F + 0.65), "gun_black")
    # Hose-drying rack on the back wall: a frame with four long hoses hanging in loops
    k.box((-3.0, zb + 0.04, F + 2.9), (0.6, zb + 0.24, F + 3.0), "metal")
    for i in range(4):
        x = -2.7 + i * 0.85
        for side in (-1, 1):
            k.box((x - 0.05, zb + 0.10, F + 1.3), (x + 0.05, zb + 0.30, F + 2.9), "cloth_olive" if i % 2 else
                  "red_paint")
        k.box((x - 0.30, zb + 0.10, F + 1.20), (x + 0.30, zb + 0.30, F + 1.42), "cloth_olive" if i % 2 else
              "red_paint")
    # White board with a painted shift list and a big hose-coupling chart
    k.box((-6.6, zb + 0.02, F + 1.60), (-4.4, zb + 0.08, F + 2.60), "white")
    k.box((-6.5, zb + 0.08, F + 1.70), (-4.5, zb + 0.10, F + 2.50), "bone_white")
    for i in range(4):
        k.box((-6.34, zb + 0.10, F + 2.26 - i * 0.16), (-4.66, zb + 0.11, F + 2.31 - i * 0.16), "metal")
    # Wheel chocks on a shelf and a stack of them on the floor
    k.box((7.6, zb + 0.02, F + 0.90), (9.4, zb + 0.40, F + 0.96), "wood_crate")
    for i in range(3):
        k.box((7.8 + i * 0.45, zb + 0.06, F + 0.96), (8.1 + i * 0.45, zb + 0.34, F + 1.20), "gun_black")
    for i in range(4):
        chock = common.make_box((0.30, 0.24, 0.20), color="gun_black")
        common.transform(chock, rot=(0.0, 0.25, 0.2 * i), loc=(10.6 + (i % 2) * 0.36, zb + 0.55 + (i // 2) * 0.34,
                                                              F + 0.10))
        k.model.add(chock)
    # Workbench against the right wall with tools on a pegboard
    k.solid((HX - pk.WALL_T - 0.75, 1.4, F), (HX - pk.WALL_T - 0.05, 5.2, F + 0.92), "wood_crate",
            base="workbench")
    k.box((HX - pk.WALL_T - 0.78, 1.4, F + 0.92), (HX - pk.WALL_T - 0.02, 5.2, F + 0.98), "bark_dark")
    k.box((HX - pk.WALL_T - 0.06, 1.6, F + 1.20), (HX - pk.WALL_T - 0.02, 5.0, F + 2.30), "wood_crate")
    for i in range(9):
        y = 1.8 + (i % 5) * 0.60
        z = F + 1.35 + (i // 5) * 0.55
        k.box((HX - pk.WALL_T - 0.10, y, z), (HX - pk.WALL_T - 0.06, y + 0.06, z + 0.34), "metal")
    for i in range(5):  # tools lying on the bench
        k.box((HX - pk.WALL_T - 0.62, 1.7 + i * 0.6, F + 0.98), (HX - pk.WALL_T - 0.20, 2.0 + i * 0.6, F + 1.06),
              "rust")
    # Shelving unit with boxes and folded hose by the left wall
    for i in range(4):
        z = F + 0.30 + i * 0.55
        k.box((-HX + pk.WALL_T + 0.02, 9.4, z), (-HX + pk.WALL_T + 0.60, 12.4, z + 0.06), "rust")
        for j in range(4):
            k.box((-HX + pk.WALL_T + 0.06, 9.6 + j * 0.68, z + 0.06),
                  (-HX + pk.WALL_T + 0.56, 10.1 + j * 0.68, z + 0.06 + (0.30 if j % 2 else 0.22)),
                  "cloth_olive" if j % 2 else "wood_crate")


def _wall_coil(k, side, y, z, color, radius=0.34):
    """A coiled hose hanging flat on a side wall (the ring lies in the YZ plane, facing into the hall)."""
    parts = []
    seg = 12
    for i in range(seg):
        a = 2.0 * math.pi * i / seg
        length = 2.0 * radius * math.tan(math.pi / seg) * 1.3
        bm = common.make_box((0.14, length, 0.075), color=color)
        common.transform(bm, rot=(a + math.pi / 2.0, 0.0, 0.0),
                         loc=(0.0, math.cos(a) * radius, math.sin(a) * radius))
        parts.append(bm)
    for i in range(10):
        a = 2.0 * math.pi * i / 10
        length = 2.0 * radius * 0.62 * math.tan(math.pi / 10) * 1.3
        bm = common.make_box((0.13, length, 0.07), color=color)
        common.transform(bm, rot=(a + math.pi / 2.0, 0.0, 0.0),
                         loc=(0.0, math.cos(a) * radius * 0.62, math.sin(a) * radius * 0.62))
        parts.append(bm)
    # Two pegs and a hanging tail
    parts.append(common.make_box((0.22, 0.05, 0.05), color="metal"))
    common.transform(parts[-1], loc=(0.02, 0.0, radius + 0.06))
    bm = common.combine(parts)
    common.transform(bm, loc=(side * (HX - pk.WALL_T + 0.05), y, z))
    k.model.add(bm)


def _place_prop(k, build_fn, loc, rot):
    """Adds a prop built by `fire_props` straight into the building, moved and turned into place."""
    sink = _PropSink()
    build_fn(sink)
    if not sink.visual:
        return
    bm = common.combine(sink.visual)
    common.transform(bm, rot=rot, loc=loc)
    k.model.add(bm)
    for label, cbm in sink.colliders:
        moved = common.combine([cbm])
        common.transform(moved, rot=rot, loc=loc)
        k.model.add_collider("prop_" + label, moved)


def _rubble(k, cx, cy, z, rx, ry, seed=1):
    """A heap of broken brick and concrete with a few rebar sticks."""
    rng = random.Random(seed)
    for i in range(16):
        s = rng.uniform(0.10, 0.30)
        px = cx + rng.uniform(-rx, rx)
        py = cy + rng.uniform(-ry, ry)
        pz = z + rng.uniform(0.0, 0.34)
        chunk = common.make_icosphere(subdiv=1, jitter=0.45, seed=seed * 100 + i,
                                      radius=(s, s * rng.uniform(0.7, 1.2), s * rng.uniform(0.5, 0.9)),
                                      loc=(px, py, pz), rot=(rng.uniform(0, 3), rng.uniform(0, 3),
                                                             rng.uniform(0, 3)),
                                      color=rng.choice(("brick_dark", "concrete_stain", "rock_mid", "brick_red")))
        k.model.add(chunk)
    for i in range(3):
        bar = box((-0.02, -0.02, -0.35), (0.02, 0.02, 0.35), "rust")
        common.transform(bar, rot=(rng.uniform(-0.6, 0.6), rng.uniform(-0.6, 0.6), rng.uniform(0, 3)),
                         loc=(cx + rng.uniform(-rx * 0.7, rx * 0.7), cy + rng.uniform(-ry * 0.7, ry * 0.7),
                              z + 0.30))
        k.model.add(bar)


def build_gear_room(k):
    """Gear room: a row of turnout coats on wall hooks, helmet shelves, boot racks, benches, lockers
    and the gear-washing sink."""
    ry0, ry1 = ROOM_Y
    gx0, gx1 = GEAR_X[0], GEAR_X[1]
    # Rail of hooks down the left wall with six turnout coats hanging on it
    hook_x = gx0 + 0.30
    k.box((hook_x, ry0 + 1.0, F + 1.94), (hook_x + 0.10, ry0 + 5.4, F + 1.98), "metal")
    for i in range(6):
        y = ry0 + 1.3 + i * 0.70
        k.box((hook_x, y - 0.02, F + 1.90), (hook_x + 0.18, y + 0.02, F + 1.94), "metal")
        coat = common.combine(fire_props.coat_parts(0.0, 0.0, 0.0, burnt=(i == 4)))
        common.transform(coat, rot=(0.0, 0.0, -math.pi / 2.0), loc=(hook_x + 0.26, y, F + 2.02))
        k.model.add(coat)
    # Boot rack under the coats
    k.solid((gx0 + 0.35, ry0 + 1.1, F), (gx0 + 0.95, ry0 + 5.2, F + 0.26), "metal", base="boot_rack")
    for i in range(6):
        y = ry0 + 1.35 + i * 0.64
        k.box((gx0 + 0.40, y, F + 0.26), (gx0 + 0.45, y + 0.30, F + 0.60), "leather")
        k.box((gx0 + 0.52, y, F + 0.26), (gx0 + 0.57, y + 0.30, F + 0.58), "leather")
    # Bench in the middle of the room
    k.solid((gx0 + 2.2, ry0 + 2.0, F), (gx0 + 4.6, ry0 + 2.6, F + 0.44), "wood_crate", base="gear_bench")
    k.box((gx0 + 2.2, ry0 + 2.0, F + 0.44), (gx0 + 4.6, ry0 + 2.15, F + 0.52), "bark_dark")
    for bx in (gx0 + 2.3, gx0 + 4.5):
        k.box((bx - 0.05, ry0 + 2.05, F), (bx + 0.05, ry0 + 2.55, F + 0.44), "metal")
    # Open locker bank against the divider wall
    for by in (ry0 + 1.6, ry0 + 2.6, ry0 + 3.6):
        _locker(k, gx1 - 0.45, by)
    # Helmet shelf along the back wall with three helmets and two spare air cylinders
    k.box((gx0 + 1.6, ry0 + 0.02, F + 1.5), (gx0 + 5.4, ry0 + 0.35, F + 1.55), "wood_crate")
    for i in range(3):
        placed = common.combine(fire_props.helmet_parts(0.0, 0.0, 0.0, burnt=(i == 2), yaw=0.4 * i))
        common.transform(placed, rot=(0.0, 0.0, math.pi), loc=(gx0 + 2.1 + i * 1.4, ry0 + 0.20, F + 1.55))
        k.model.add(placed)
    for i in range(2):
        cyl = common.make_frustum(0.10, 0.10, 0.0, 0.72, segments=10, color="gun_black")
        common.transform(cyl, loc=(gx0 + 6.0 + i * 0.30, ry0 + 0.55, F))
        k.model.add(cyl)
        k.model.add(common.make_frustum(0.035, 0.035, 0.72, 0.86, segments=8,
                                        center=(gx0 + 6.0 + i * 0.30, ry0 + 0.55), color="brass"))
    # Gear-washing sink in the far corner
    k.solid((gx0 + 0.30, ry1 - 1.5, F), (gx0 + 1.7, ry1 - 0.3, F + 0.90), "concrete_grey", base="gear_sink")
    k.box((gx0 + 0.40, ry1 - 1.4, F + 0.90), (gx0 + 1.6, ry1 - 0.4, F + 0.94), "rock_dark")
    k.box((gx0 + 0.45, ry1 - 0.55, F + 0.94), (gx0 + 1.55, ry1 - 0.45, F + 0.96), "metal")
    # Turnout trousers on a second rail near the door
    k.box((gx0 + 5.6, ry1 - 1.0, F + 1.62), (gx0 + 5.65, ry1 - 0.2, F + 1.66), "metal")
    for i in range(2):
        y = ry1 - 0.55 - i * 0.45
        k.box((gx0 + 5.4, y - 0.18, F + 0.55), (gx0 + 5.6, y + 0.18, F + 1.62), "cloth_olive")
        k.box((gx0 + 5.38, y - 0.20, F + 1.30), (gx0 + 5.62, y + 0.20, F + 1.36), "wheat_gold")
    _rubble(k, gx0 + 3.0, ry1 - 2.4, F, 1.1, 0.9, seed=21)


def _locker(k, x, y, yaw=0.0):
    """A tall steel locker with its door swung open, vents, and a shelf inside. `yaw` turns the whole
    thing so a bank can stand against a wall running either way."""
    parts = []
    parts.append(box((-0.28, -0.30, 0.0), (0.28, 0.30, 1.95), "metal"))
    parts.append(box((-0.30, 0.28, 0.04), (0.30, 0.32, 1.92), "shirt_blue"))
    door = pbox((-0.28, -0.02, 0.0), (0.28, 0.02, 1.86), "shirt_blue", bevel=0.008)
    common.transform(door, rot=(0.0, 0.0, -1.05), loc=(-0.28, 0.30, 0.06))
    parts.append(door)
    for i in range(3):
        parts.append(box((-0.16, 0.30, 1.55 + i * 0.10), (0.16, 0.315, 1.60 + i * 0.10), "gun_black"))
    parts.append(box((-0.24, -0.26, 0.06), (0.24, 0.24, 0.10), "rock_dark"))
    parts.append(box((-0.24, -0.24, 0.70), (0.24, 0.24, 0.74), "rock_dark"))  # middle shelf
    bm = common.combine(parts)
    common.transform(bm, rot=(0.0, 0.0, yaw), loc=(x, y, F))
    k.model.add(bm)
    k.collider((x - 0.32, y - 0.34, F), (x + 0.32, y + 0.34, F + 1.95), "locker")


def build_day_room(k):
    """Day room: the table with four chairs, a bunk against the wall, a dead TV, a sofa, a notice board
    and a wall clock."""
    ry0, ry1 = ROOM_Y
    dx0, dx1 = DAY_X[0], DAY_X[1]
    cx, cy = (dx0 + dx1) / 2.0, ry0 + 4.2
    # Table with four chairs around it
    k.solid((cx - 1.5, cy - 0.95, F), (cx + 1.5, cy + 0.95, F + 0.74), "wood_crate", base="day_table")
    k.box((cx - 1.55, cy - 1.0, F + 0.74), (cx + 1.55, cy + 1.0, F + 0.80), "bark_dark")
    for lx in (cx - 1.35, cx + 1.25):
        for ly in (cy - 0.75, cy + 0.65):
            k.box((lx, ly, F), (lx + 0.10, ly + 0.10, F + 0.74), "metal")
    for cx2, cy2, yaw in ((cx - 0.8, cy - 1.45, 0.0), (cx + 0.8, cy - 1.45, 0.0),
                          (cx - 0.8, cy + 1.45, math.pi), (cx + 0.8, cy + 1.45, math.pi),
                          (cx - 2.15, cy, math.pi / 2.0), (cx + 2.15, cy, -math.pi / 2.0)):
        _chair(k, cx2, cy2, yaw)
    # Mugs and a plate left on the table
    for i in range(5):
        mug = common.make_frustum(0.05, 0.045, 0.0, 0.11, segments=8, color="bone_white")
        common.transform(mug, loc=(cx - 1.0 + i * 0.55, cy - 0.35 + (i % 2) * 0.6, F + 0.80))
        k.model.add(mug)
    plate = common.make_frustum(0.14, 0.14, 0.0, 0.02, segments=12, color="bone_white")
    common.transform(plate, loc=(cx + 0.7, cy + 0.35, F + 0.80))
    k.model.add(plate)
    # Bunk bed along the right wall
    k.solid((dx1 - 2.3, ry0 + 0.3, F), (dx1 - 0.4, ry0 + 2.9, F + 0.44), "metal", base="day_bunk")
    k.box((dx1 - 2.25, ry0 + 0.35, F + 0.44), (dx1 - 0.45, ry0 + 2.85, F + 0.60), "rock_light")
    k.box((dx1 - 1.10, ry0 + 0.40, F + 0.60), (dx1 - 0.50, ry0 + 1.20, F + 0.70), "white")
    k.box((dx1 - 2.20, ry0 + 1.4, F + 0.58), (dx1 - 0.50, ry0 + 2.80, F + 0.66), "shirt_blue")
    # Dead TV on a stand and a sagging sofa
    k.solid((dx1 - 1.6, ry1 - 1.1, F), (dx1 - 0.3, ry1 - 0.2, F + 0.55), "bark_dark", base="tv_stand")
    k.box((dx1 - 1.45, ry1 - 0.9, F + 0.55), (dx1 - 0.45, ry1 - 0.65, F + 1.05), "gun_black")
    k.box((dx1 - 1.37, ry1 - 0.66, F + 0.62), (dx1 - 0.53, ry1 - 0.64, F + 1.00), "soot")
    k.solid((dx0 + 0.4, ry0 + 0.3, F), (dx0 + 3.0, ry0 + 1.3, F + 0.42), "gown_blue", base="day_sofa")
    k.box((dx0 + 0.4, ry0 + 0.3, F + 0.42), (dx0 + 3.0, ry0 + 0.55, F + 0.92), "gown_blue")
    for i in range(3):
        k.box((dx0 + 0.55 + i * 0.85, ry0 + 0.60, F + 0.42), (dx0 + 1.25 + i * 0.85, ry0 + 1.25, F + 0.58),
              "shirt_blue")
    # Wall clock, stopped, and a notice board on the spine wall
    for ring, radius, colour in ((0, 0.24, "white"), (1, 0.19, "soot")):
        face = common.make_frustum(radius, radius, 0.0, 0.05, segments=14, color=colour)
        common.transform(face, rot=(math.pi / 2.0, 0.0, 0.0), loc=(cx + 1.4, ry1 - 0.10 - ring * 0.02, F + 2.5))
        k.model.add(face)
    k.box((cx + 1.4, ry1 - 0.16, F + 2.5), (cx + 1.42, ry1 - 0.13, F + 2.66), "gun_black")
    k.box((cx - 2.6, ry1 - 0.10, F + 1.35), (cx - 0.6, ry1 - 0.05, F + 2.40), "wood_crate")
    k.box((cx - 2.5, ry1 - 0.05, F + 1.55), (cx - 1.7, ry1 - 0.02, F + 2.20), "bone_white")
    k.box((cx - 1.6, ry1 - 0.05, F + 1.6), (cx - 0.9, ry1 - 0.02, F + 2.05), "white")
    for i in range(6):  # scattered papers on the floor
        x = dx0 + 1.0 + i * 0.55
        k.box((x, ry1 - 2.2, F), (x + 0.30, ry1 - 1.9, F + 0.04), "bone_white")
    _rubble(k, cx + 2.4, ry0 + 3.4, F, 1.3, 1.0, seed=31)


def build_kitchen(k):
    """Kitchen strip: the counter run with sink and stove, a fridge, wall cupboards, a bain-marie,
    a mop bucket, and the toilet cubicle in the corner."""
    ry0, ry1 = ROOM_Y
    kx0, kx1 = KITCHEN_X[0], KITCHEN_X[1]
    # Counter run along the divider wall
    _place_prop(k, fire_props.build_kitchen_counter, loc=(kx0 + 1.6, ry0 + 1.9, F),
                rot=(0.0, 0.0, math.pi / 2.0))
    k.box((kx0 + 0.10, ry0 + 1.1, F), (kx0 + 0.60, ry0 + 4.7, F + 0.05), "soot")
    # Wall cupboards over the counter
    k.box((kx0 + 0.02, ry0 + 1.2, F + 1.75), (kx0 + 0.42, ry0 + 4.6, F + 2.55), "wood_crate")
    for i in range(4):
        y = ry0 + 1.35 + i * 0.85
        k.box((kx0 + 0.42, y, F + 1.82), (kx0 + 0.45, y + 0.72, F + 2.48), "bark_dark")
        k.box((kx0 + 0.45, y + 0.30, F + 2.05), (kx0 + 0.48, y + 0.40, F + 2.20), "metal")
    # Fridge and a standing freezer on the back wall
    k.solid((kx0 + 2.6, ry0 + 0.32, F), (kx0 + 3.5, ry0 + 1.05, F + 1.85), "bone_white", base="kitchen_fridge")
    k.box((kx0 + 2.62, ry0 + 1.05, F + 0.95), (kx0 + 3.48, ry0 + 1.10, F + 1.02), "metal")
    k.solid((kx0 + 3.7, ry0 + 0.32, F), (kx0 + 4.6, ry0 + 1.05, F + 1.20), "white", base="kitchen_freezer")
    k.box((kx0 + 3.72, ry0 + 1.05, F + 0.75), (kx0 + 4.58, ry0 + 1.10, F + 0.82), "metal")
    # A prep table with pots
    k.solid((kx0 + 2.4, ry1 - 2.4, F), (kx0 + 4.4, ry1 - 1.5, F + 0.88), "metal", base="prep_table")
    k.box((kx0 + 2.35, ry1 - 2.45, F + 0.88), (kx0 + 4.45, ry1 - 1.45, F + 0.93), "rock_light")
    for i in range(3):
        pot = common.make_frustum(0.16, 0.15, 0.0, 0.22, segments=10, color="metal")
        common.transform(pot, loc=(kx0 + 2.7 + i * 0.7, ry1 - 2.0, F + 0.93))
        k.model.add(pot)
    # Mop bucket and a bundle of rags by the door
    k.solid((kx0 + 1.0, ry1 - 1.5, F), (kx0 + 1.4, ry1 - 1.1, F + 0.40), "water_blue", base="mop_bucket")
    k.box((kx0 + 1.1, ry1 - 1.4, F + 0.40), (kx0 + 1.15, ry1 - 1.35, F + 1.35), "bark_light")
    k.box((kx0 + 0.8, ry1 - 0.9, F), (kx0 + 1.5, ry1 - 0.5, F + 0.12), "cloth_olive")
    # Toilet cubicle: a pan, a cistern, a pipe, and a mirror on the wall
    k.solid((kx1 - 2.2, ry0 + 0.7, F), (kx1 - 1.5, ry0 + 1.5, F + 0.42), "white", base="toilet_pan")
    k.box((kx1 - 2.2, ry0 + 0.3, F + 0.42), (kx1 - 1.5, ry0 + 0.7, F + 1.05), "white")
    k.box((kx1 - 0.6, ry0 + 0.7, F + 1.35), (kx1 - 0.3, ry1 - 1.2, F + 1.42), "metal")
    k.box((kx1 - 2.9, ry0 + 0.35, F + 0.9), (kx1 - 1.9, ry0 + 0.40, F + 1.7), "glass_dark")
    _rubble(k, kx0 + 5.6, ry1 - 3.6, F, 1.0, 0.8, seed=33)


def _chair(k, cx, cy, yaw):
    """A simple dining chair, turned by `yaw`."""
    parts = [box((-0.22, -0.22, 0.0), (0.22, 0.22, 0.05), "wood_crate"),
             box((-0.22, -0.22, 0.42), (0.22, 0.22, 0.48), "wood_crate"),
             box((-0.22, -0.25, 0.05), (-0.16, -0.19, 0.42), "wood_crate"),
             box((0.16, -0.25, 0.05), (0.22, -0.19, 0.42), "wood_crate"),
             box((-0.22, 0.19, 0.05), (-0.16, 0.25, 0.42), "wood_crate"),
             box((0.16, 0.19, 0.05), (0.22, 0.25, 0.42), "wood_crate"),
             box((-0.22, -0.25, 0.48), (0.22, -0.19, 0.92), "wood_crate")]
    bm = common.combine(parts)
    common.transform(bm, rot=(0.0, 0.0, yaw), loc=(cx, cy, F))
    k.model.add(bm)
    k.collider((cx - 0.24, cy - 0.24, F), (cx + 0.24, cy + 0.24, F + 0.9), "chair")


def build_stairs(k):
    """The stair up to the closed first floor, blocked by a rubble heap."""
    x0, x1 = STAIR_X[0] + pk.WALL_T, STAIR_X[1] - pk.WALL_T
    y0, y1 = ROOM_Y[0], ROOM_Y[1]
    steps = 10
    for i in range(steps):
        z = F + i * 0.30
        yy = y1 - (i + 1) * ((y1 - y0) / steps)
        k.solid((x0, yy, F), (x1, yy + (y1 - y0) / steps, z + 0.30), "concrete_grey", base="stair_step")
    # The flight reaches a closed landing (the first floor is sealed)
    k.box((x0, y0, F + 3.0), (x1, y0 + 1.2, GF_TOP - 0.10), "concrete_grey")
    # Rubble heap burying the bottom three steps
    _rubble(k, (x0 + x1) / 2.0, y1 - 1.1, F + 0.3, 1.3, 0.8, seed=41)
    _rubble(k, (x0 + x1) / 2.0 + 0.3, y1 - 2.1, F + 0.8, 1.1, 0.7, seed=42)
    nosing = box((-0.55, -0.9, -0.3), (0.55, 0.9, 0.3), "rock_dark")
    common.transform(nosing, rot=(0.0, 0.4, 0.2), loc=((x0 + x1) / 2.0, y1 - 2.3, F + 0.8))
    k.model.add(nosing)
    # A handrail pulled off the wall and leaning across the flight
    rail = box((-0.05, -1.6, -0.03), (0.05, 1.6, 0.03), "metal")
    common.transform(rail, rot=(0.0, 0.5, 0.3), loc=((x0 + x1) / 2.0, y1 - 2.6, F + 1.2))
    k.model.add(rail)
    # A yellow hazard bar and a "closed" board at the bottom of the flight
    k.box((x0 + 0.1, y1 - 0.55, F + 0.9), (x1 - 0.1, y1 - 0.45, F + 1.05), "wheat_gold")
    k.box((x0 + 0.4, y1 - 0.52, F + 0.5), (x1 - 0.4, y1 - 0.48, F + 0.9), "white")


# ---------------------------------------------------------------- extra previews


def render_extra_view(name, obj, suffix, view):
    """One extra 384x384 preview (a front, a side, or a cut-away) next to the standard ones."""
    path = os.path.join(common.PREVIEW_DIR, name + suffix + ".png")
    os.makedirs(common.PREVIEW_DIR, exist_ok=True)
    scene = bpy.context.scene
    common._add_camera_and_sun(scene, obj, top=False, view=view)
    scene.render.resolution_x = common.PREVIEW_SIZE
    scene.render.resolution_y = common.PREVIEW_SIZE
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.film_transparent = False
    scene.view_settings.view_transform = "Standard"
    scene.render.filepath = path
    if os.path.exists(path):
        os.remove(path)
    common._setup_workbench(scene)
    bpy.ops.render.render(write_still=True)
    print(f"preview {name}{suffix}: -> {path}")


def render_front_and_cutaway():
    """Renders the straight-on front view and the ground-floor cut-away of the main building."""
    front = os.path.join(common.PREVIEW_DIR, NAME + "_front.png")
    # Straight-on front view: rebuild the visual only, never exported
    common.reset_scene()
    model = common.Model(NAME + "_front", CATEGORY, 10 ** 9, grip=(0.0, 0.0, 0.0))
    build(model)
    visual = common.combine([bm for bm, _ in model._visual])
    obj = common._to_object(visual, NAME + "_front", painted=True)
    obj["_is_weapon"] = False
    render_extra_view(NAME, obj, "_front", (0.02, 1.0, 0.16))
    render_extra_view(NAME, obj, "_front_low", (0.6, 1.0, 0.30))
    del front

    # Cut-away: the ground floor only, upper floors and roof left out, seen from above at an angle
    common.reset_scene()
    cut = common.Model(NAME + "_cutaway", CATEGORY, 10 ** 9, grip=(0.0, 0.0, 0.0))
    k = kit.Kit(cut)
    build_outside(k)
    build_hall_floor(k)
    build_hall_walls(k)
    build_tower(k)
    build_hall_dressing(k)
    build_gear_room(k)
    build_stairs(k)
    build_day_room(k)
    build_kitchen(k)
    visual = common.combine([bm for bm, _ in cut._visual])
    obj = common._to_object(visual, NAME + "_cutaway", painted=True)
    render_extra_view(NAME, obj, "_cutaway", (0.85, 0.55, 0.85))
    render_extra_view(NAME, obj, "_cutaway_top", (0.0, 0.02, 1.0))


# ---------------------------------------------------------------- lod1


def build_lod1(model):
    """Outside only: the block, the bays and doors as flat planes, the tower, the sign blocks, the roof."""
    k = kit.Kit(model)
    # Main block
    k.box((-HX, -HY, 0.0), (HX, HY, GF_TOP), BRICK)
    k.box((-HX - 0.10, -HY - 0.10, 0.0), (HX + 0.10, HY + 0.10, 0.52), TRIM_DARK)
    k.box((-HX, -HY, GF_TOP), (HX, HY, F1_TOP), BRICK)
    k.box((-HX - 0.10, -HY - 0.10, GF_TOP + 0.20), (HX + 0.10, HY + 0.10, GF_TOP + 0.56), TRIM)
    k.box((-HX - 0.10, -HY - 0.10, F1_TOP - 0.30), (HX + 0.10, HY + 0.10, F1_TOP), TRIM)
    k.box((-HX - 0.10, -HY - 0.10, F1_TOP), (HX + 0.10, HY + 0.10, ROOF_TOP), BRICK)
    k.box((-HX - 0.16, -HY - 0.16, ROOF_TOP), (HX + 0.16, HY + 0.16, ROOF_TOP + 0.16), TRIM)
    # Bay openings: dark recesses plus the roller-door panels
    for i, cx in enumerate(BAY_CENTRES):
        k.box((cx - BAY_W / 2, BAY_Y1 - 0.12, 0.0), (cx + BAY_W / 2, BAY_Y1 + 0.02, BAY_H), "soot")
        if i == 0:
            k.box((cx - BAY_W / 2 + 0.05, BAY_Y1 - 0.10, 2.4), (cx + BAY_W / 2 - 0.05, BAY_Y1 - 0.04, BAY_H),
                  "rust")
    # Office door and the red sign plates over the bays
    k.box((OFFICE_DOOR_X[0], BAY_Y1 - 0.10, 0.0), (OFFICE_DOOR_X[1], BAY_Y1 + 0.02, pk.DOOR_H), "soot")
    for cx in BAY_CENTRES:
        k.box((cx - 1.10, BAY_Y1, BAY_H + 0.92), (cx + 1.10, BAY_Y1 + 0.10, BAY_H + 1.62), PAINT_RED)
        k.box((cx - 0.20, BAY_Y1 + 0.04, BAY_H + 0.52), (cx + 0.20, BAY_Y1 + 0.14, BAY_H + 0.82), PAINT_RED)
    # First-floor windows as flat panes
    z0, z1 = GF_TOP + 1.15, GF_TOP + 3.05
    for u in (-14.2, -8.0, -1.5, 5.0, 11.2):
        k.box((u - 1.2, HY - 0.02, z0), (u + 1.2, HY + 0.03, z1), "glass_dark")
    for u in (-13.0, -6.4, 0.2, 6.8, 13.2):
        k.box((u - 1.2, -HY - 0.03, z0), (u + 1.2, -HY + 0.02, z1), "glass_dark")
    for u in (-9.0, -3.0, 3.0):
        k.box((-HX - 0.03, u - 1.2, z0), (-HX + 0.02, u + 1.2, z1), "glass_dark")
        k.box((HX - 0.02, u - 1.2, z0), (HX + 0.03, u + 1.2, z1), "glass_dark")
    # Hose tower
    k.box((TX0, TY0, 0.0), (TX1, TY1, TOWER_H), BRICK_DARK)
    for a in range(1, 4):
        z = F + a * TOWER_STOREY
        k.box((TX0 - 0.12, TY0 - 0.12, z - 0.14), (TX1 + 0.12, TY1 + 0.12, z + 0.14), TRIM)
    k.box((TX0 - 0.12, TY0 - 0.12, TOWER_H - 0.40), (TX1 + 0.12, TY1 + 0.12, TOWER_H + 0.18), TRIM)
    k.box((TX0 - 0.05, TY0 - 0.05, TOWER_H + 0.18), (TX1 + 0.05, TY1 + 0.05, TOWER_H + 1.6), "rust_dark")
    for a in range(4):
        z = F + a * TOWER_STOREY + 1.0
        k.box(((TX0 + TX1) / 2.0 - 0.85, TY1 - 0.02, z), ((TX0 + TX1) / 2.0 + 0.85, TY1 + 0.09, z + 2.2),
              BRICK_DARK)
        k.box((TX1 - 0.02, (TY0 + TY1) / 2.0 - 0.85, z), (TX1 + 0.09, (TY0 + TY1) / 2.0 + 0.85, z + 2.2),
              BRICK_DARK)
    # Roof equipment blocks
    k.box((1.4, -8.0, ROOF_TOP), (5.4, -4.0, ROOF_TOP + 1.0), "metal")
    k.box((-10.6, -4.6, ROOF_TOP), (-6.6, -1.4, ROOF_TOP + 1.0), "metal")
    k.box((13.5, 9.9, ROOF_TOP), (13.7, 10.5, ROOF_TOP + 3.6), "metal")
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(34.0, 26.0, 1.0))


# ---------------------------------------------------------------- markers


def build_markers(k):
    """13 `spawn_N` empties: the bay hall, each back room, and a few at the doors."""
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(34.0, 26.0, 1.0))
    k.empty("door_1", (-8.0, BAY_Y1 - 0.2, F))
    k.empty("door_2", (-1.5, BAY_Y1 - 0.2, F))
    k.empty("door_3", (5.0, BAY_Y1 - 0.2, F))
    k.empty("door_4", (10.3, HY - 0.2, F))
    pk.spawn(k, 1, -8.0, 9.4)
    pk.spawn(k, 2, -1.5, 6.0)
    pk.spawn(k, 3, 5.0, 9.2)
    pk.spawn(k, 4, -9.0, 2.2)
    pk.spawn(k, 5, 2.0, 2.6)
    pk.spawn(k, 6, 11.0, 4.0)
    pk.spawn(k, 7, -12.0, -6.4)   # gear room
    pk.spawn(k, 8, 0.4, -7.0)     # day room
    pk.spawn(k, 9, 12.0, -8.6)    # kitchen
    pk.spawn(k, 13, -6.4, -8.0)   # foot of the blocked stair
    pk.spawn(k, 10, -1.5, BAY_Y1 + 1.2, 0.0)   # on the forecourt, in front of bay 2
    pk.spawn(k, 11, 10.3, HY + 1.0, 0.0)        # outside the office door
    pk.spawn(k, 12, -1.5, -11.4)   # back of the day room


def build(model):
    k = kit.Kit(model)
    build_outside(k)
    build_hall_floor(k)
    build_hall_walls(k)
    build_hall_ceiling(k)
    build_upper(k)
    build_tower(k)
    build_hall_dressing(k)
    build_gear_room(k)
    build_stairs(k)
    build_day_room(k)
    build_kitchen(k)
    build_markers(k)


def build_main(model):
    build(model)


def build_all():
    """Builds metro_fire_station, its _lod1, the four props and the three trucks. Returns [(name, ok)]."""
    front = (1.0, 1.0, 0.6)
    out = [
        (NAME, common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0), top_preview=True,
                                view=front)),
        (NAME + "_lod1", common.run_model(NAME + "_lod1", CATEGORY, BUDGET_LOD1, build_lod1,
                                          grip=(0.0, 0.0, 0.0), view=front)),
    ]
    out += fire_props.build_all()
    render_front_and_cutaway()
    return out


if __name__ == "__main__":
    failed = [n for n, ok in build_all() if not ok]
    if failed:
        print("FIRE STATION FAILED:", ", ".join(failed))
        sys.exit(1)
    print("FIRE STATION OK")
