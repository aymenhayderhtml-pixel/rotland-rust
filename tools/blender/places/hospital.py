"""metro_hospital: the R42 New Halden hospital, 38 x 30 m, 5 floors, walk-in ground floor.

Coordinates: origin at the bottom centre of the footprint, front = +Y (the street side).
Footprint x -19..19, y -15..15. Ground floor 0.15..4.2 (walk-in, 0.3 m walls, doors >= 1.8 x 2.4),
then 4 closed floors of 3.8 m: 4.2..8.0, 8.0..11.8, 11.8..15.6, 15.6..19.4. Roof deck 19.4..19.6
with AC units, a water tank and a helipad "H".

Ground floor plan (it reads at a glance from the cut-away preview):
  front-left   waiting area, three banks of seats
  front-right  reception hall with the long L desk, queue posts, lift core
  middle       a straight corridor with a fallen ceiling panel
  back-left    ward of 6 beds, each with a curtain rail
  back-right   pharmacy behind a mesh cage (door open), the office beyond it, toilets in the far corner
Plus debris, an overturned bed, papers, dark stains.

`build_all()` returns [(name, ok)] and also writes `metro_hospital_lod1` (the outside only).
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
           os.path.join(BLENDER, "towers"),
           os.path.join(BLENDER, "overgrowth")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402
import kit  # noqa: E402
import place_kit as pk  # noqa: E402

NAME = "metro_hospital"
LOD1 = "metro_hospital_lod1"
CATEGORY = "buildings"
BUDGET = 90000
BUDGET_LOD1 = 6000

HX, HY = 19.0, 15.0             # half footprint: 38 x 30 m
G_TOP = 4.2                     # ground floor ceiling (walk-in height)
FH = 3.8                        # upper floor height
FLOORS = 4                      # closed floors above the ground floor
ROOF_Z = G_TOP + FH * FLOORS    # 19.4
PARAPET = 1.0

WALL = "white"
PLINTH = "concrete_grey"
RED = "red_paint"
BLUE = "gown_blue"
TRIM = "metal"
F = pk.FLOOR_Z                  # 0.15, the walkable floor top
T = pk.WALL_T                   # 0.3, wall thickness

# Interior faces of the shell and the corridor band
IX, IY = HX - T, HY - T         # 18.7, 14.7
CY0, CY1 = -1.4, 1.4            # corridor walls (inner faces)
BZ0, BZ1 = CY0 - T, CY1 + T     # -1.7, 1.7

# Vertical partitions, as (x0, x1)
W_WAIT_REC = (-2.75, -2.45)     # waiting room | reception hall
W_WARD_PHARM = (-0.15, 0.15)    # ward | pharmacy
W_PHARM_OFF = (9.35, 9.65)      # pharmacy | office + toilets
W_REC_OFF = (12.45, 12.75)      # reception hall | reception back office

CANOPY_Y1 = HY + 5.2            # the ER canopy sticks 5.2 m out over the ambulance bay


# ------------------------------------------------------------------ shell


def _front_openings():
    """Door and window gaps in the +Y wall (used by the wall, the glass and the LOD1)."""
    doors = [pk.opening(4.2, 7.8, HY - T, HY, F, F + pk.DOOR_H),        # main ER doors (3.6 m)
             pk.opening(-13.6, -11.8, HY - T, HY, F, F + pk.DOOR_H)]    # second, smaller side door
    wins = [pk.opening(-17.4, -14.6, HY - T, HY, 1.2, 3.2),
            pk.opening(-10.6, -8.2, HY - T, HY, 1.2, 3.2),
            pk.opening(-7.0, -4.2, HY - T, HY, 1.2, 3.2),
            pk.opening(-0.6, 3.4, HY - T, HY, 1.0, 3.2),
            pk.opening(9.4, 13.4, HY - T, HY, 1.0, 3.2),
            pk.opening(15.2, 17.6, HY - T, HY, 1.2, 3.2)]
    return doors, wins


def _back_openings():
    doors = [pk.opening(-6.0, -4.2, -HY, -HY + T, F, F + pk.DOOR_H)]     # back service door
    wins = [pk.opening(-17.0, -14.0, -HY, -HY + T, 1.3, 3.1),
            pk.opening(-11.0, -8.6, -HY, -HY + T, 1.3, 3.1),
            pk.opening(0.6, 3.6, -HY, -HY + T, 1.3, 3.1),
            pk.opening(7.0, 10.0, -HY, -HY + T, 1.3, 3.1),
            pk.opening(13.0, 16.4, -HY, -HY + T, 1.3, 3.1)]
    return doors, wins


def _left_openings():
    doors = [pk.opening(-HX, -HX + T, -9.0, -6.4, F, F + pk.DOOR_H)]     # ward side door
    wins = [pk.opening(-HX, -HX + T, 3.0, 6.0, 1.3, 3.1),
            pk.opening(-HX, -HX + T, -2.4, 0.6, 1.3, 3.1)]
    return doors, wins


def _right_openings():
    return [pk.opening(HX - T, HX, -10.0, -6.4, 1.3, 3.1),
            pk.opening(HX - T, HX, -3.4, 0.2, 1.3, 3.1),
            pk.opening(HX - T, HX, 5.0, 9.0, 1.3, 3.1)]


def _ground_shell(k):
    """Floor slab, four walls with every door and window gap, plinth and painted band."""
    f_doors, f_wins = _front_openings()
    b_doors, b_wins = _back_openings()
    l_doors, l_wins = _left_openings()
    r_wins = _right_openings()

    pk.wall(k, (-HX, HY - T, F), (HX, HY, G_TOP), openings=f_doors + f_wins, color=WALL, base="wall_front")
    pk.wall(k, (-HX, -HY, F), (HX, -HY + T, G_TOP), openings=b_doors + b_wins, color=WALL, base="wall_back")
    pk.wall(k, (-HX, -HY, F), (-HX + T, HY, G_TOP), openings=l_doors + l_wins, color=WALL, base="wall_left")
    pk.wall(k, (HX - T, -HY, F), (HX, HY, G_TOP), openings=r_wins, color=WALL, base="wall_right")

    # Raised walkable slab (with its own floor collider) and a two-tone tile pattern on top
    pk.floor_slab(k, -HX, -HY, HX, HY, z_top=F, color="concrete_stain")
    pk.floor_tiles(k, -IX, -IY, IX, IY, F + 0.008, tile=1.2,
                   color_a="bone_white", color_b="rock_light", variety=0.30, seed=771)
    # The corridor's painted guide line, the way real hospitals mark the route
    k.box((-IX, -0.09, F + 0.012), (IX, 0.09, F + 0.018), "water_blue")
    # Plinth band outside at the bottom
    for lo, hi in (((-HX - 0.06, HY - 0.06, 0.0), (HX + 0.06, HY + 0.06, 0.5)),
                   ((-HX - 0.06, -HY - 0.06, 0.0), (HX + 0.06, -HY + 0.06, 0.5)),
                   ((-HX - 0.06, -HY, 0.0), (-HX + 0.06, HY, 0.5)),
                   ((HX - 0.06, -HY, 0.0), (HX + 0.06, HY, 0.5))):
        k.box(lo, hi, PLINTH)
    # Painted red band under the ground floor ceiling
    for lo, hi in (((-HX - 0.04, HY - 0.02, 3.55), (HX + 0.04, HY + 0.03, 3.78)),
                   ((-HX - 0.04, -HY - 0.03, 3.55), (HX + 0.04, -HY + 0.02, 3.78)),
                   ((-HX - 0.03, -HY, 3.55), (-HX + 0.02, HY, 3.78)),
                   ((HX - 0.02, -HY, 3.55), (HX + 0.03, HY, 3.78))):
        k.box(lo, hi, RED)
    return f_doors, f_wins, b_wins, l_wins, r_wins


def _ground_windows(k, f_wins, b_wins, l_wins, r_wins):
    """Glass, frames and damage state on every ground floor opening."""
    states = ("dark", "boarded", "broken", "dark", "broken", "boarded")
    for i, op in enumerate(f_wins):
        (x0, _y0, z0), (x1, _y1, z1) = op
        pk.window(k, HY, 1, 1, (x0 + x1) / 2.0, z0, z1, state=states[i % len(states)], width=x1 - x0)
    for i, op in enumerate(b_wins):
        (x0, _y0, z0), (x1, _y1, z1) = op
        pk.window(k, -HY, 1, -1, (x0 + x1) / 2.0, z0, z1, state=states[(i + 2) % len(states)], width=x1 - x0)
    for i, op in enumerate(l_wins):
        (_x0, y0, z0), (_x1, y1, z1) = op
        pk.window(k, -HX, 0, -1, (y0 + y1) / 2.0, z0, z1, state=states[(i + 3) % len(states)], width=y1 - y0)
    for i, op in enumerate(r_wins):
        (_x0, y0, z0), (_x1, y1, z1) = op
        pk.window(k, HX, 0, 1, (y0 + y1) / 2.0, z0, z1, state=states[(i + 1) % len(states)], width=y1 - y0)

    # Door surrounds and the ER door leaves (one slid open, one lying on the hall floor)
    f_doors, _ = _front_openings()
    for op in f_doors:
        pk.frame(k, op, HY, 1, color=TRIM, t=0.10)
    (mx0, _a, _b), (mx1, _c, _d) = f_doors[0]
    k.box((mx0, HY - 0.26, F + pk.DOOR_H - 0.10), (mx1, HY - 0.18, F + pk.DOOR_H), TRIM)
    k.box((mx0 + 0.10, HY - 0.20, F), (mx0 + 1.45, HY - 0.14, F + pk.DOOR_H - 0.10), TRIM)
    k.box((mx0 + 0.22, HY - 0.21, F + 0.22), (mx0 + 1.33, HY - 0.13, F + pk.DOOR_H - 0.42), "glass_dark")
    k.box((mx1 - 1.15, HY - 0.20, F), (mx1 - 0.10, HY - 0.14, F + pk.DOOR_H - 0.10), TRIM)
    leaf = pk.box_bm((-0.62, -0.03, 0.0), (0.62, 0.03, pk.DOOR_H - 0.10), TRIM)
    common.transform(leaf, rot=(math.pi / 2.0, 0.0, 0.42), loc=(2.6, HY - 2.4, F + 0.035))
    k.model.add(leaf)
    (sx0, _e, _f), (sx1, _g, _h) = f_doors[1]
    k.box((sx0 + 0.04, HY - 0.22, F), (sx0 + 0.52, HY - 0.16, F + pk.DOOR_H - 0.05), TRIM)


# ------------------------------------------------------------------ ground floor plan


def _interior_walls(k):
    """Partitions that make the waiting room, hall, corridor, ward, pharmacy, office and toilets."""
    # Waiting room | reception hall (door at y 4.0..5.8)
    pk.wall(k, (W_WAIT_REC[0], BZ1, F), (W_WAIT_REC[1], IY, G_TOP),
            openings=[pk.opening(W_WAIT_REC[0], W_WAIT_REC[1], 4.0, 5.8, F, F + pk.DOOR_H)],
            color="bone_white", base="wall_wait_rec")
    # Reception hall | reception back office (door at y 8.0..9.8)
    pk.wall(k, (W_REC_OFF[0], BZ1, F), (W_REC_OFF[1], IY, G_TOP),
 openings=[pk.opening(W_REC_OFF[0], W_REC_OFF[1], 8.0, 9.8, F, F + pk.DOOR_H)],
            color="bone_white", base="wall_rec_office")

    # Corridor front wall (y 1.4..1.7)
    front = [pk.opening(2.0, 7.0, CY1, BZ1, F, F + 3.0),          # hall <-> corridor, wide
             pk.opening(-14.0, -12.2, CY1, BZ1, F, F + pk.DOOR_H),
             pk.opening(13.6, 15.4, CY1, BZ1, F, F + pk.DOOR_H)]
    pk.wall(k, (-IX, CY1, F), (IX, BZ1, G_TOP), openings=front, color="bone_white", base="wall_corr_front")
    # Corridor back wall (y -1.7..-1.4)
    back = [pk.opening(-16.0, -14.2, BZ0, CY0, F, F + pk.DOOR_H),
            pk.opening(-8.0, -6.2, BZ0, CY0, F, F + pk.DOOR_H),
            pk.opening(3.0, 4.8, BZ0, CY0, F, F + pk.DOOR_H),
            pk.opening(12.5, 14.3, BZ0, CY0, F, F + pk.DOOR_H)]
    pk.wall(k, (-IX, BZ0, F), (IX, CY0, G_TOP), openings=back, color="bone_white", base="wall_corr_back")

    # Ward | pharmacy (solid)
    pk.wall(k, (W_WARD_PHARM[0], -IY, F), (W_WARD_PHARM[1], BZ0, G_TOP), color="bone_white", base="wall_ward")
    # Pharmacy | office/toilets (door at y -5.6..-3.8)
    pk.wall(k, (W_PHARM_OFF[0], -IY, F), (W_PHARM_OFF[1], BZ0, G_TOP),
            openings=[pk.opening(W_PHARM_OFF[0], W_PHARM_OFF[1], -5.6, -3.8, F, F + pk.DOOR_H)],
            color="bone_white", base="wall_pharm_off")
    # Office | toilets (y -6.15..-5.85, door at x 15.5..17.3)
    pk.wall(k, (W_PHARM_OFF[1], -6.15, F), (IX, -5.85, G_TOP),
            openings=[pk.opening(15.5, 17.3, -6.15, -5.85, F, F + pk.DOOR_H)],
            color="bone_white", base="wall_office_toilet")

    # Toilet cubicles: three stalls along the back wall, doors facing the sinks
    for x in (11.25, 13.35, 15.45, 17.55):
        pk.wall(k, (x - 0.06, -IY, F), (x + 0.06, -10.7, 2.3), color="bone_white", base="stall")
    pk.wall(k, (W_PHARM_OFF[1], -10.70, F), (17.6, -10.58, 2.3),
            openings=[pk.opening(11.6, 12.8, -10.70, -10.58, F, 2.1),
                      pk.opening(13.7, 14.9, -10.70, -10.58, F, 2.1),
                      pk.opening(15.8, 17.0, -10.70, -10.58, F, 2.1)],
            color="bone_white", base="stall_front")

    # Ceiling over the ground floor (roof=True, so the top preview shows the plan)
    k.box((-HX, -HY, G_TOP), (HX, HY, G_TOP + 0.2), "rock_dark", roof=True)
    k.model.add_collider("ceil_1", common.make_box((HX * 2.0, HY * 2.0, 0.2),
                                                  loc=(0.0, 0.0, G_TOP + 0.1), color="white"))


def _ceiling_panel_fallen(k):
    """A fallen acoustic ceiling panel on the corridor floor and the hole it left above."""
    rng = random.Random(9153)
    for i in range(7):
        slab = common.make_box((rng.uniform(0.55, 0.85), rng.uniform(0.55, 0.85), 0.05),
                               color="rock_light" if i % 2 else "bone_white")
        common.transform(slab, rot=(rng.uniform(-0.12, 0.12), rng.uniform(-0.12, 0.12), rng.uniform(0.0, 3.14)),
                         loc=(8.6 + rng.uniform(-0.9, 0.9), 0.1 + rng.uniform(-0.8, 0.8), F + 0.02 + i * 0.035))
        k.model.add(slab)
    for _ in range(4):
        bm = common.make_box((rng.uniform(0.6, 1.2), 0.04, 0.035), color=TRIM)
        common.transform(bm, rot=(0.0, 0.0, rng.uniform(0.0, 3.14)),
                         loc=(8.6 + rng.uniform(-1.1, 1.1), 0.1 + rng.uniform(-0.8, 0.8), F + 0.32))
        k.model.add(bm)
    k.box((7.8, -0.9, G_TOP - 0.03), (9.8, 1.1, G_TOP + 0.02), "soot", roof=True)
    for x in (7.82, 9.78):
        k.box((x - 0.03, -0.9, G_TOP - 0.6), (x + 0.03, 0.85, G_TOP - 0.55), TRIM, roof=True)
    k.box((7.8, 1.05, G_TOP - 0.45), (8.6, 1.10, G_TOP - 0.03), TRIM, roof=True)


def _debris_and_stains(k):
    """Floor stains, scattered papers, rubble, an overturned bed and wall grime."""
    rng = random.Random(2244)
    for cx, cy, rx, ry, seed in ((-8.0, 6.5, 1.1, 0.8, 1), (-11.0, -8.5, 1.4, 1.0, 2),
                                 (5.5, 0.0, 0.9, 1.6, 3), (15.4, 8.6, 0.8, 0.7, 4),
                                 (2.0, -6.6, 1.6, 1.2, 5), (-16.0, -9.0, 0.7, 1.1, 6),
                                 (-12.5, 11.0, 0.9, 0.7, 7)):
        pk.patch(k, cx, cy, rx, ry, F + 0.013, "blood_dark", seed=seed)
        pk.patch(k, cx + 0.2, cy - 0.1, rx * 0.45, ry * 0.5, F + 0.015, "soot", seed=seed + 40)
    for i in range(58):
        x = rng.uniform(-IX + 0.5, IX - 0.5)
        y = rng.uniform(-IY + 0.5, IY - 0.5)
        if 4.2 < x < 8.0 and y > IY - 1.2:
            continue
        sheet = common.make_box((rng.uniform(0.16, 0.26), rng.uniform(0.20, 0.32), 0.004),
                                color=("white", "bone_white", "rock_light")[i % 3])
        common.transform(sheet, rot=(rng.uniform(-0.05, 0.05), rng.uniform(-0.05, 0.05), rng.uniform(0.0, 3.14)),
                         loc=(x, y, F + 0.015 + rng.uniform(0.0, 0.008)))
        k.model.add(sheet)
    for i in range(36):
        s = rng.uniform(0.06, 0.20)
        chunk = common.make_box((s, s * rng.uniform(0.6, 1.3), s * 0.4),
                                color=("rock_mid", "concrete_stain", "bone_white", "rock_light")[i % 4])
        common.transform(chunk, rot=(rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3), rng.uniform(0.0, 3.14)),
                         loc=(rng.uniform(-IX + 0.4, IX - 0.4), rng.uniform(-IY + 0.4, IY - 0.4), F + s * 0.2))
        k.model.add(chunk)

    # An overturned bed: the frame tipped on its side, the mattress slid off it, in the ward aisle
    frame = common.combine([
        pk.box_bm((-0.5, -1.02, 0.42), (0.5, 1.02, 0.48), "rock_light"),
        pk.box_bm((-0.5, 1.0, 0.42), (0.5, 1.06, 1.18), TRIM),
        pk.box_bm((-0.5, -1.06, 0.42), (0.5, -1.0, 1.02), TRIM),
        pk.box_bm((-0.5, -1.0, 0.42), (-0.44, 1.0, 0.48), TRIM),
        pk.box_bm((0.44, -1.0, 0.42), (0.5, 1.0, 0.48), TRIM),
        pk.box_bm((-0.5, -1.0, 0.66), (0.5, 1.0, 0.72), "white"),
        pk.box_bm((-0.42, -0.6, 0.72), (0.42, 0.4, 0.755), "gown_blue"),
    ])
    common.transform(frame, rot=(0.0, math.radians(96.0), math.radians(24.0)), loc=(-6.4, -8.6, F + 0.05))
    k.model.add(frame)
    mat = common.make_box((0.96, 1.9, 0.16), color="white")
    common.transform(mat, rot=(0.0, 0.0, math.radians(-18.0)), loc=(-4.4, -10.4, F + 0.09))
    k.model.add(mat)

    for face, axis, sign, u, seed in ((IY, 1, -1, -9.0, 11), (IY, 1, -1, 4.0, 12),
                                      (-IY, 1, 1, -9.0, 13), (-IX, 0, 1, -2.0, 14),
                                      (IX, 0, -1, -10.0, 15), (IX, 0, -1, 6.0, 16)):
        pk.grime(k, face, axis, sign, u, F, 2.6, seed=seed, width=3.2, streaks=6)


def _waiting_area(k):
    """Front-left waiting room: three banks of seats, a coffee table, a wall TV, a water cooler."""
    for row, y in enumerate((12.4, 11.0, 9.6)):
        k.box((-17.6, y - 0.26, F), (-4.6, y + 0.26, F + 0.06), "gun_black")
        k.box((-17.6, y - 0.03, F + 0.36), (-4.6, y + 0.03, F + 0.42), TRIM)
        for i in range(5):
            cx = -16.6 + i * 2.9
            k.box((cx - 0.05, y - 0.03, F + 0.06), (cx + 0.05, y + 0.03, F + 0.38), TRIM)
        x = -17.1
        while x < -4.7:
            seat_lo = x
            if row == 1 and abs(x + 11.0) < 0.4:
                # one seat knocked flat on the frame, its backrest folded forward
                k.box((x, y - 0.24, F + 0.06), (x + 0.94, y + 0.22, F + 0.16), "shirt_blue")
                k.box((x + 0.02, y - 0.30, F + 0.12), (x + 0.92, y + 0.10, F + 0.22), "shirt_blue")
                x += 1.0
                continue
            k.box((x, y - 0.26, F + 0.42), (x + 0.94, y + 0.22, F + 0.50), "shirt_blue")
            k.box((x + 0.02, y - 0.30, F + 0.50), (x + 0.92, y - 0.22, F + 0.96), "shirt_blue")
            k.box((seat_lo - 0.05, y - 0.30, F + 0.76), (seat_lo + 0.02, y + 0.24, F + 0.80), "shirt_blue")
            k.box((x + 0.92, y - 0.30, F + 0.76), (x + 0.99, y + 0.24, F + 0.80), "shirt_blue")
            x += 1.0
    # Two chairs tipped over in the aisle
    for cx, cy, rot in ((-8.4, 13.4, 0.6), (-13.6, 6.8, -1.2)):
        chair = common.combine([
            pk.box_bm((-0.24, -0.23, 0.42), (0.24, 0.23, 0.49), "shirt_blue"),
            pk.box_bm((-0.23, -0.30, 0.49), (0.23, -0.20, 0.92), "shirt_blue"),
            pk.box_bm((-0.21, -0.20, 0.0), (-0.17, 0.20, 0.42), TRIM),
            pk.box_bm((0.17, -0.20, 0.0), (0.21, 0.20, 0.42), TRIM),
        ])
        common.transform(chair, rot=(0.0, math.radians(88.0), rot), loc=(cx, cy, F + 0.26))
        k.model.add(chair)
    # Low coffee table with magazines, and a dead plant
    k.box((-13.2, 6.6, F + 0.36), (-8.8, 8.0, F + 0.42), "bark_light")
    for lx in (-13.0, -9.0):
        k.box((lx, 6.7, F), (lx + 0.08, 7.9, F + 0.36), "bark_dark")
    for i in range(5):
        sheet = common.make_box((0.22, 0.30, 0.008), color=("white", "shirt_red", "bone_white")[i % 3])
        common.transform(sheet, rot=(0.0, 0.0, 0.3 * i), loc=(-11.4 + 0.1 * i, 7.2 + 0.06 * i, F + 0.43))
        k.model.add(sheet)
    k.model.add(common.make_frustum(0.20, 0.16, F, F + 0.34, segments=8, center=(-15.8, 6.2), color="brick_red"))
    k.model.add(common.make_frustum(0.16, 0.16, F, F + 0.02, segments=8, center=(-16.1, 6.6), color="dirt"))
    # Wall TV (dead), a notice board and a water cooler on the divider wall
    k.box((-2.48, 9.4, 2.2), (-2.42, 11.4, 3.2), "gun_black", roof=True)
    k.box((-2.46, 9.5, 2.24), (-2.40, 11.3, 3.16), "soot", roof=True)
    k.box((-2.44, 9.9, 2.0), (-2.40, 10.9, 2.18), TRIM, roof=True)
    k.box((-2.44, 5.0, 2.0), (-2.40, 6.4, 2.9), "bark_light", roof=True)
    for i in range(4):
        k.box((-2.43, 5.1 + i * 0.3, 2.1), (-2.39, 5.4 + i * 0.3, 2.75), "white", roof=True)
    k.box((-3.3, 12.2, F), (-2.8, 12.7, F + 0.95), "white")
    k.box((-3.25, 12.25, F + 0.95), (-2.85, 12.65, F + 1.35), "water_blue")
    pk.sign_text(k, "WAITING", -IX, 0, 1, 10.6, 2.2, 0.26, 0.40, color=BLUE)


def _reception(k):
    """Reception hall: the L desk, queue posts, the directory board, the lift core, dead ceiling lights."""
    # The long counter (6.2 m) with a return leg across the hall
    k.solid((2.4, 6.2, F), (8.6, 7.1, F + 1.1), "bark_light", base="rec_desk")
    k.box((2.3, 6.15, F + 1.1), (8.7, 7.15, F + 1.18), "rock_light")
    k.solid((7.9, 3.4, F), (8.8, 6.2, F + 1.1), "bark_light", base="rec_desk_ret")
    k.box((7.85, 3.35, F + 1.1), (8.85, 6.2, F + 1.18), "rock_light")
    k.box((2.5, 7.05, F + 0.12), (8.5, 7.1, F + 1.0), BLUE)      # painted front panel
    k.box((2.45, 6.2, F + 0.52), (8.55, 6.26, F + 0.62), "white")  # white band
    for cx in (4.0, 6.6):
        k.box((cx - 0.34, 6.5, F + 1.18), (cx + 0.34, 6.8, F + 1.6), "gun_black")
        k.box((cx - 0.30, 6.49, F + 1.22), (cx + 0.30, 6.48, F + 1.56), "soot")
        k.box((cx - 0.2, 6.85, F + 1.18), (cx + 0.2, 6.95, F + 1.2), "gun_black")
    k.box((5.4, 6.6, F + 1.18), (5.8, 6.9, F + 1.26), "white")
    k.box((5.45, 6.65, F + 1.26), (5.6, 6.85, F + 1.34), "white")
    # Office chairs behind the counter, one tipped over
    for cx, tilt in ((4.4, 0.0), (6.2, 0.0), (7.4, 0.9)):
        chair = common.combine([
            pk.box_bm((-0.25, -0.24, 0.44), (0.25, 0.24, 0.52), "pants_dark"),
            pk.box_bm((-0.24, -0.32, 0.52), (0.24, -0.22, 1.0), "pants_dark"),
            pk.box_bm((-0.04, -0.04, 0.06), (0.04, 0.04, 0.44), TRIM),
            pk.box_bm((-0.3, -0.3, 0.02), (0.3, 0.3, 0.08), TRIM),
        ])
        common.transform(chair, rot=(0.0, math.radians(84.0) if tilt else 0.0, tilt),
                         loc=(cx, 5.4, F + (0.22 if tilt else 0.0)))
        k.model.add(chair)
    # Filing cabinets behind the counter (staff side, x 2.4..8.4, y 2.2..3.0)
    for cx in (2.9, 4.1, 5.3):
        k.solid((cx - 0.55, 2.3, F), (cx + 0.55, 2.9, F + 1.4), TRIM, base="cab")
        for r in range(4):
            k.box((cx - 0.5, 2.88, F + 0.2 + r * 0.3), (cx + 0.5, 2.92, F + 0.44 + r * 0.3), "chrome")
    # Queue posts and ropes in front of the desk
    for i in range(4):
        px = 3.2 + i * 1.9
        k.model.add(common.make_frustum(0.16, 0.16, F, F + 0.04, segments=8, center=(px, 9.0), color=TRIM))
        k.box((px - 0.03, 8.97, F), (px + 0.03, 9.03, F + 0.95), "chrome")
        k.model.add(common.make_frustum(0.05, 0.05, F + 0.95, F + 1.0, segments=8, center=(px, 9.0), color="chrome"))
        if i < 3:
            k.box((px + 0.03, 8.98, F + 0.84), (px + 1.87, 9.02, F + 0.88), "shirt_red")
    # Directory board on the waiting-room divider, clear of the hall
    k.box((-2.44, 2.2, 1.7), (-2.38, 4.2, 3.0), "bark_dark", roof=True)
    k.box((-2.42, 2.3, 1.8), (-2.36, 4.1, 2.9), "bone_white", roof=True)
    for i in range(6):
        k.box((-2.41, 2.4, 2.72 - i * 0.15), (-2.35, 3.1 + 0.3 * (i % 3), 2.78 - i * 0.15), "soot", roof=True)
    # Lift core on the corridor wall of the hall: one closed pair of doors and one jammed open
    for i, cx in enumerate((9.4, 11.8)):
        k.box((cx - 1.0, BZ1 - 0.02, F), (cx + 1.0, BZ1 + 0.14, F + 2.5), "chrome")
        if i == 0:
            k.box((cx - 0.03, BZ1 + 0.14, F), (cx + 0.03, BZ1 + 0.16, F + 2.5), "gun_black")
            k.box((cx - 0.14, BZ1 + 0.14, F + 1.0), (cx + 0.14, BZ1 + 0.20, F + 1.3), TRIM)
        else:
            # Jammed open: dark shaft, one leaf slid back
            k.box((cx - 0.92, BZ1 + 0.12, F), (cx + 0.92, BZ1 + 0.16, F + 2.5), "soot")
            k.box((cx - 0.92, BZ1 + 0.14, F), (cx - 0.2, BZ1 + 0.20, F + 2.5), "chrome")
        k.box((cx - 0.9, BZ1 + 0.14, F + 2.6), (cx + 0.9, BZ1 + 0.20, F + 2.78), "gun_black", roof=True)
        k.box((cx - 0.6, BZ1 + 0.16, F + 2.64), (cx + 0.6, BZ1 + 0.20, F + 2.74), RED, roof=True)
    # Emergency sign band high on the hall side of the front wall
    pk.sign_text(k, "EMERGENCY", HY - T, 1, -1, 6.0, 3.3, 0.30, 0.44, color=RED, d=0.03)
    # Dead ceiling strip lights, one hanging by its wire
    for i in range(5):
        cx = 0.4 + i * 2.6
        k.box((cx - 0.14, 1.9, G_TOP - 0.12), (cx + 0.14, 4.4, G_TOP - 0.05), "bone_white", roof=True)
    hang = common.combine([
        pk.box_bm((-0.7, -0.14, -0.03), (0.7, 0.14, 0.03), TRIM),
        pk.box_bm((-0.64, -0.10, -0.05), (0.64, 0.10, -0.03), "bone_white"),
    ])
    common.transform(hang, rot=(0.42, 0.22, 0.0), loc=(0.4, 3.0, 3.2))
    k.model.add(hang)
    k.box((0.38, 2.98, 3.24), (0.42, 3.02, G_TOP - 0.03), "gun_black", roof=True)


def _records_room(k):
    """The back office right of the hall (x 12.75..18.7, y 1.7..14.7): desks, filing, a copier, shelves."""
    for i, (dx, dy) in enumerate(((14.0, 11.6), (14.0, 8.4), (17.4, 11.6))):
        k.solid((dx - 0.9, dy - 0.45, F), (dx + 0.9, dy + 0.45, F + 0.75), "bark_light", base="rec_office_desk")
        k.box((dx - 0.95, dy - 0.5, F + 0.75), (dx + 0.95, dy + 0.5, F + 0.80), "rock_light")
        k.box((dx - 0.26, dy - 0.2, F + 0.80), (dx + 0.26, dy + 0.2, F + 1.20), "gun_black")   # monitor
        k.box((dx - 0.23, dy - 0.19, F + 0.84), (dx + 0.23, dy + 0.02, F + 1.16), "soot")
        k.box((dx - 0.24, dy + 0.24, F + 0.80), (dx + 0.24, dy + 0.4, F + 0.83), "gun_black")   # keyboard
        for j in range(3):
            sheet = common.make_box((0.2, 0.28, 0.006), color=("white", "bone_white")[j % 2])
            common.transform(sheet, rot=(0.0, 0.0, 0.5 * j), loc=(dx + 0.5 + 0.04 * j, dy - 0.1, F + 0.805))
            k.model.add(sheet)
        # Chair, tipped on the middle desk
        chair = common.combine([
            pk.box_bm((-0.24, -0.23, 0.44), (0.24, 0.23, 0.52), "pants_dark"),
            pk.box_bm((-0.23, -0.31, 0.52), (0.23, -0.21, 1.0), "pants_dark"),
            pk.box_bm((-0.04, -0.04, 0.06), (0.04, 0.04, 0.44), TRIM),
        ])
        common.transform(chair, rot=(0.0, math.radians(86.0) if i == 1 else 0.0, 0.3 * i),
                         loc=(dx, dy + 1.1, F + (0.24 if i == 1 else 0.0)))
        k.model.add(chair)
    # Photocopier and a shelf wall along the right wall
    k.solid((17.8, 4.6, F), (18.6, 5.6, F + 1.1), "rock_light", base="copier")
    k.box((17.85, 4.7, F + 1.1), (18.55, 5.5, F + 1.18), "gun_black")
    k.box((17.9, 5.52, F + 0.3), (18.5, 5.58, F + 0.9), "soot")
    for r in range(3):
        k.box((12.95, 13.4, F + 0.5 + r * 0.55), (15.1, 14.0, F + 0.54 + r * 0.55), "chrome")
        for bx in range(3):
            k.box((13.05 + bx * 0.7, 13.45, F + 0.55 + r * 0.55), (13.55 + bx * 0.7, 13.95, F + 0.94 + r * 0.55),
                  ("bark_light", "white", "bone_white", "wood_crate")[(bx + r) % 4])
    for x in (12.95, 15.1):
        k.box((x - 0.04, 13.4, F), (x + 0.04, 14.0, F + 1.7), TRIM)
    # A tipped filing cabinet and spilled folders on the floor
    tipped = pk.box_bm((-0.55, -0.5, 0.0), (0.55, 0.5, 1.4), TRIM)
    common.transform(tipped, rot=(0.0, math.radians(90.0), math.radians(24.0)), loc=(16.0, 8.0, F + 0.5))
    k.model.add(tipped)
    for i in range(10):
        f = pk.box_bm((-0.14, -0.14, 0.0), (0.14, 0.14, 0.06),
                      ("bark_light", "white", "bone_white")[i % 3])
        common.transform(f, rot=(0.0, 0.0, i * 0.7), loc=(16.4 + 0.16 * (i % 3), 8.6 + 0.12 * (i % 4), F))
        k.model.add(f)
    pk.sign_text(k, "RECORDS", W_REC_OFF[1], 0, 1, 7.0, 2.6, 0.22, 0.34, color=BLUE)


def _bed(k, bx, by, flip=False):
    """One baked ward bed (frame, mattress, blanket, boards, castors) centred on (bx, by)."""
    hw, hl = 0.49, 1.02
    sign = -1.0 if flip else 1.0

    def put(x0, x1, y0, y1, z0, z1, color):
        ax, bx2 = bx + x0, bx + x1
        ay, by2 = by + sign * y0, by + sign * y1
        k.box((min(ax, bx2), min(ay, by2), z0), (max(ax, bx2), max(ay, by2), z1), color)

    for sx in (-hw, hw):
        put(sx - 0.03, sx + 0.03, -hl, hl, F + 0.40, F + 0.46, TRIM)
    for sy in (-hl, hl):
        put(-hw, hw, sy - 0.03, sy + 0.03, F + 0.40, F + 0.46, TRIM)
    put(-hw, hw, -hl, hl, F + 0.46, F + 0.64, "white")                  # mattress
    put(-hw + 0.02, hw - 0.02, -hl, 0.26, F + 0.635, F + 0.68, "gown_blue")   # blanket
    put(-0.30, 0.30, 0.55, 0.92, F + 0.64, F + 0.74, "bone_white")      # pillow
    put(-hw, hw, hl - 0.02, hl + 0.02, F + 0.42, F + 1.16, TRIM)        # head board
    put(-hw, hw, -hl - 0.02, -hl + 0.02, F + 0.42, F + 1.00, TRIM)      # foot board
    for sx in (-hw, hw):
        for sy in (-0.8, 0.8):
            put(sx - 0.05, sx + 0.05, sy - 0.05, sy + 0.05, F, F + 0.1, "soot")


def _ward(k):
    """Back-left ward: 6 beds in two rows, a curtain rail and curtain on each, a nurses' station."""
    beds = [(-17.0, -4.6), (-13.4, -4.6), (-9.8, -4.6),
            (-17.0, -12.2), (-13.4, -12.2), (-9.8, -12.2)]
    for i, (bx, by) in enumerate(beds):
        flip = by < -8
        _bed(k, bx, by, flip=flip)
        rail_y = by + (-1.55 if flip else 1.55)
        # Ceiling rail over the foot of the bed, with two drop rods
        k.box((bx - 1.5, rail_y - 0.04, 2.06), (bx + 1.5, rail_y + 0.04, 2.12), TRIM, roof=True)
        for cx in (bx - 1.4, bx + 1.4):
            k.box((cx - 0.03, rail_y - 0.03, 2.12), (cx + 0.03, rail_y + 0.03, G_TOP - 0.02), "gun_black", roof=True)
        # The curtain: fabric folds hanging from the rail, torn short on the third bed
        folds = 8
        span = 2.9
        for f in range(folds):
            fx = bx - 1.45 + f * (span / folds)
            low = F + 0.06 + 0.03 * (f % 3)
            if i == 2 and f > 4:
                low = F + 0.85                     # torn away at one end
            k.box((fx, rail_y - 0.05, 2.0), (fx + span / folds * 0.90, rail_y + 0.05, 2.06), BLUE)
            k.box((fx, rail_y - 0.045, low), (fx + span / folds * 0.88, rail_y + 0.045, 2.0), "gown_blue")
    # Nurses' station near the ward door end (clear of the beds)
    k.solid((-5.4, -3.4, F), (-2.4, -2.4, F + 1.05), "bark_light", base="nurse_desk")
    k.box((-5.45, -3.45, F + 1.05), (-2.35, -2.35, F + 1.12), "rock_light")
    k.box((-4.6, -3.1, F + 1.12), (-4.2, -2.8, F + 1.5), "gun_black")      # dead monitor
    k.box((-3.6, -3.2, F + 1.12), (-3.2, -2.9, F + 1.16), "white")         # papers
    k.box((-5.0, -4.0, F), (-4.4, -3.5, F + 0.5), "pants_dark")            # stool
    # Wall cupboard and three oxygen cylinders by the side door
    k.box((-IX, -13.6, 1.4), (-IX + 0.45, -11.2, 2.4), "bone_white", roof=True)
    k.box((-IX + 0.02, -13.55, 1.45), (-IX + 0.47, -11.25, 2.35), TRIM, roof=True)
    for i in range(3):
        k.model.add(common.make_frustum(0.15, 0.15, F, F + 1.05, segments=8,
                                        center=(-18.3, -12.0 + i * 0.44), color="leaf_dark"))
        k.box((-18.45, -12.15 + i * 0.44, F + 1.05), (-18.15, -11.85 + i * 0.44, F + 1.12), TRIM)
    # A screen on wheels and a drip stand between the rows
    k.box((-7.4, -8.4, F), (-6.6, -8.3, F + 1.7), "bone_white")
    for lx in (-7.35, -6.65):
        k.box((lx, -8.55, F), (lx + 0.06, -8.35, F + 0.1), TRIM)
    k.box((-12.0, -8.3, F), (-11.94, -8.24, F + 1.9), "chrome")
    k.model.add(common.make_frustum(0.03, 0.03, F + 1.55, F + 1.85, segments=6, center=(-11.97, -8.27),
                                    color="water_blue"))
    pk.sign_text(k, "WARD", W_WARD_PHARM[0], 0, -1, -8.0, 2.4, 0.28, 0.42, color=BLUE)


def _pharmacy(k):
    """Back-right pharmacy: the mesh cage (its door swung open), stock inside, a dispensing counter."""
    cx0, cx1, cy0, cy1 = 1.2, 8.6, -13.4, -8.0
    top = 2.7
    post = 0.06
    for sx in (cx0, cx1):
        for sy in (cy0, cy1):
            k.box((sx - post, sy - post, F), (sx + post, sy + post, F + top), TRIM)

    def mesh(axis, coord, u0, u1, z0, z1):
        u = u0
        while u <= u1 + 1e-6:
            if axis == 1:
                k.box((u - 0.012, coord - 0.012, z0), (u + 0.012, coord + 0.012, z1), "chrome")
            else:
                k.box((coord - 0.012, u - 0.012, z0), (coord + 0.012, u + 0.012, z1), "chrome")
            u += 0.24
        for zr in (z0 + 0.5, z0 + 1.5, z1 - 0.15):
            if axis == 1:
                k.box((u0, coord - 0.012, zr), (u1, coord + 0.012, zr + 0.024), "chrome")
            else:
                k.box((coord - 0.012, u0, zr), (coord + 0.012, u1, zr + 0.024), "chrome")

    mesh(1, cy0, cx0, cx1, F + 0.1, F + top)
    mesh(0, cx0, cy0, cy1, F + 0.1, F + top)
    mesh(0, cx1, cy0, cy1, F + 0.1, F + top)
    mesh(1, cy1, cx0, 5.4, F + 0.1, F + top)
    mesh(1, cy1, 7.2, cx1, F + 0.1, F + top)
    # Ceiling mesh, a few cross bars only (it is never seen from below in detail)
    for x in (2.4, 4.6, 6.8):
        k.box((x - 0.012, cy0, F + top - 0.05), (x + 0.012, cy1, F + top), "chrome", roof=True)
    # The cage door, swung 100 degrees open into the pharmacy
    door = common.combine([
        pk.box_bm((-0.9, -0.02, 0.0), (0.0, 0.02, 1.4), TRIM),
        pk.box_bm((-0.9, -0.02, 0.0), (0.0, 0.02, 0.05), TRIM),
        pk.box_bm((-0.9, -0.02, 1.34), (0.0, 0.02, 1.4), TRIM),
        pk.box_bm((-0.72, -0.05, 0.62), (-0.62, 0.05, 0.74), "brass"),
    ])
    for i in range(1, 5):
        door = common.combine([door, pk.box_bm((-0.18 * i - 0.012, -0.012, 0.05),
                                               (-0.18 * i + 0.012, 0.012, 1.34), "chrome")])
    common.transform(door, rot=(0.0, 0.0, math.radians(100.0)), loc=(7.2, cy1 + 0.12, F + 0.1))
    k.model.add(door)
    # Shelves inside, stocked, one shelf collapsed at the end
    rng = random.Random(321)
    for z in (0.55, 1.15, 1.75):
        k.box((cx0 + 0.15, cy0 + 0.12, F + z), (cx1 - 0.15, cy0 + 1.55, F + z + 0.05), "chrome")
        x = cx0 + 0.3
        while x < cx1 - 0.6:
            w = rng.uniform(0.2, 0.45)
            k.box((x, cy0 + 0.2, F + z + 0.05), (x + w, cy0 + 0.6, F + z + 0.38),
                  ("white", "bone_white", "wood_crate", BLUE)[rng.randrange(4)])
            if rng.random() < 0.5:
                k.box((x + 0.04, cy0 + 0.6, F + z + 0.2), (x + w - 0.04, cy0 + 0.62, F + z + 0.32), RED)
            x += w + rng.uniform(0.1, 0.25)
    crum = common.make_box((1.4, 0.05, 0.03), color="chrome")
    common.transform(crum, rot=(0.0, 0.55, 0.0), loc=(3.0, cy0 + 1.3, F + 1.0))
    k.model.add(crum)
    # Spilled boxes on the pharmacy floor
    for i in range(9):
        b = pk.box_bm((-0.16, -0.11, 0.0), (0.16, 0.11, 0.22),
                      ("white", "bone_white", "wood_crate")[i % 3])
        common.transform(b, rot=(0.0, 0.0, rng.uniform(0.0, 3.14)),
                         loc=(rng.uniform(2.2, 8.0), rng.uniform(-7.6, -4.0), F))
        k.model.add(b)
    # Dispensing counter just outside the cage, with a crate on it
    k.solid((5.0, -7.4, F), (7.6, -6.8, F + 1.05), "bark_light", base="disp")
    k.box((4.95, -7.45, F + 1.05), (7.65, -6.75, F + 1.12), "rock_light")
    k.box((5.2, -7.2, F + 1.12), (5.7, -7.0, F + 1.3), "white")
    k.box((0.4, -14.2, F), (1.1, -13.5, F + 0.14), "wood_crate")           # a pallet in the corner
    pk.sign_text(k, "PHARMACY", W_WARD_PHARM[1], 0, 1, -4.4, G_TOP - 0.95, 0.26, 0.40, color=BLUE)


def _office_and_toilets(k):
    """The back office right of the pharmacy, and the toilets behind it."""
    # Office
    k.solid((10.2, -3.2, F), (13.4, -2.6, F + 0.78), "bark_light", base="office_desk")
    k.box((10.15, -3.25, F + 0.78), (13.45, -2.55, F + 0.84), "rock_light")
    k.box((10.8, -3.0, F + 0.84), (11.24, -2.7, F + 1.2), "gun_black")
    k.box((12.4, -3.0, F + 0.84), (12.9, -2.72, F + 0.9), "white")
    k.box((11.4, -4.2, F), (11.9, -3.7, F + 0.5), "pants_dark")
    k.box((11.45, -4.2, F + 0.5), (11.85, -4.05, F + 1.0), "pants_dark")
    k.solid((17.4, -3.2, F), (18.4, -2.5, F + 1.45), TRIM, base="office_cab")
    for r in range(4):
        k.box((17.45, -2.55, F + 0.2 + r * 0.31), (18.35, -2.5, F + 0.45 + r * 0.31), "chrome")
    k.solid((10.0, -5.85, F), (11.5, -5.45, F + 1.9), "bark_dark", base="office_shelf")
    for r in range(4):
        k.box((10.05, -5.85, F + 0.3 + r * 0.42), (11.45, -5.5, F + 0.34 + r * 0.42), "bark_light")
        for b in range(4):
            k.box((10.1 + b * 0.34, -5.8, F + 0.34 + r * 0.42), (10.38 + b * 0.34, -5.55, F + 0.66 + r * 0.42),
                  ("shirt_red", "pants_denim", "leaf_dark", "wood_crate")[(r + b) % 4])
    k.box((13.6, -5.2, F), (13.7, -5.1, F + 1.7), TRIM)
    k.box((13.4, -5.35, F + 1.55), (13.9, -4.95, F + 1.72), "pants_dark")
    pk.sign_text(k, "OFFICE", W_PHARM_OFF[1], 0, 1, -2.6, 2.6, 0.22, 0.34, color=BLUE)
    # Toilets: three cubicles on the back wall, a sink counter, a mirror strip
    for cx in (12.25, 14.35, 16.45):
        k.box((cx - 0.36, -13.9, F), (cx + 0.36, -13.3, F + 0.42), "bone_white")
        k.box((cx - 0.36, -14.2, F + 0.42), (cx + 0.36, -13.8, F + 0.80), "bone_white")
        k.box((cx - 0.30, -13.8, F + 0.42), (cx + 0.30, -13.4, F + 0.50), "white")
    k.box((10.0, -11.4, F + 0.82), (18.6, -10.9, F + 0.90), "bone_white")
    for i in range(4):
        cx = 10.6 + i * 2.0
        k.box((cx - 0.24, -11.3, F + 0.90), (cx + 0.24, -11.0, F + 0.95), "chrome")
        k.box((cx - 0.02, -11.25, F + 0.95), (cx + 0.02, -11.1, F + 1.12), "chrome")
    k.box((10.1, -IY, 1.6), (18.5, -IY + 0.1, 2.6), "chrome", roof=True)
    k.box((17.85, -10.58, F), (18.6, -10.52, F + 2.0), "bone_white")       # a disabled cubicle door


def _exterior(k):
    """ER canopy on columns, the second side door plus its ramp, signs, and the overgrown front."""
    # --- the big ER canopy: slab, deep front fascia, four columns
    cx0, cx1 = 1.2, 10.8
    k.box((cx0, HY, 3.90), (cx1, CANOPY_Y1, 4.30), "white")
    k.box((cx0 - 0.12, HY, 4.30), (cx1 + 0.12, CANOPY_Y1, 4.42), "rock_mid")
    k.box((cx0 - 0.12, CANOPY_Y1 - 0.28, 3.62), (cx1 + 0.12, CANOPY_Y1, 4.50), "white")   # fascia
    k.box((cx0 - 0.14, CANOPY_Y1 - 0.30, 4.50), (cx1 + 0.14, CANOPY_Y1, 4.62), RED)       # red cap
    for cx in (cx0 - 0.12, cx1 + 0.12):
        k.box((cx - 0.07, HY, 3.62), (cx + 0.07, CANOPY_Y1, 4.62), RED)
    for cx in (cx0 + 0.55, cx1 - 0.55):
        for cy in (HY + 0.7, CANOPY_Y1 - 0.75):
            k.solid((cx - 0.20, cy - 0.20, 0.0), (cx + 0.20, cy + 0.20, 3.90), "rock_light", base="canopy_col")
            k.box((cx - 0.25, cy - 0.25, 0.0), (cx + 0.25, cy + 0.25, 0.14), PLINTH)
    # Painted marks on the asphalt under the canopy
    k.box((cx0 + 0.7, HY + 0.35, 0.0), (cx1 - 0.7, HY + 0.5, 0.03), "white")
    for i in range(6):
        k.box((2.2 + i * 1.35, CANOPY_Y1 - 1.3, 0.0), (2.9 + i * 1.35, CANOPY_Y1 - 0.95, 0.03), "white")
    # EMERGENCY on the canopy fascia, standing proud of the front face
    pk.sign_text(k, "EMERGENCY", CANOPY_Y1, 1, 1, (cx0 + cx1) / 2.0, 3.70, 0.60, 0.68,
                 color=RED, d=0.035)
    # Painted hospital sign band on the blank wall bay above the canopy: a white board with the
    # red cross and the name. The first upper floor leaves that bay windowless for exactly this.
    k.box((3.0, HY, 4.95), (9.0, HY + 0.06, 7.45), "white", roof=True)
    pk.cross_symbol(k, HY + 0.06, 1, 1, 6.0, 6.45, 0.62, color=RED, depth=0.05, roof=True)
    pk.sign_text(k, "HOSPITAL", HY + 0.06, 1, 1, 6.0, 5.12, 0.30, 0.42, color=BLUE, d=0.04, roof=True)
    # --- the second, smaller side door with its ramp and rails
    k.box((-14.2, HY, 3.00), (-11.2, HY + 1.5, 3.18), "white")
    k.box((-14.2, HY + 1.38, 2.92), (-11.2, HY + 1.5, 3.22), RED)
    for cx in (-14.1, -11.3):
        k.box((cx - 0.06, HY + 1.34, 0.0), (cx + 0.06, HY + 1.46, 2.92), "rock_light")
    k.box((-14.2, HY, 0.0), (-11.2, HY + 1.4, 0.16), "rock_mid")        # the ramp deck
    for rx in (-14.2, -11.26):
        k.box((rx - 0.04, HY, 0.16), (rx + 0.04, HY + 1.4, 1.05), TRIM)
        k.box((rx - 0.06, HY, 1.0), (rx + 0.06, HY + 1.4, 1.08), "chrome")
    k.box((-14.2, HY + 1.36, 0.16), (-11.2, HY + 1.44, 1.05), TRIM)
    pk.sign_text(k, "CLINIC", HY, 1, 1, -12.7, 3.30, 0.24, 0.36, color=BLUE, d=0.04)
    # --- a second ramp on the left wall to the ward side door, inside the lot
    ry0, ry1 = -10.2, -6.2
    k.box((-HX - 1.3, ry0, 0.0), (-HX, ry1, 0.16), "rock_mid")
    for ry in (ry0, ry1):
        k.box((-HX - 1.3, ry - 0.06, 0.16), (-HX, ry + 0.06, 1.05), TRIM)
        k.box((-HX - 1.32, ry - 0.08, 1.0), (-HX, ry + 0.08, 1.08), "chrome")
    k.box((-HX - 1.32, ry0, 0.16), (-HX - 1.24, ry1, 1.05), TRIM)
    pk.sign_text(k, "RAMP", -HX, 0, -1, -8.2, 1.5, 0.22, 0.34, color="white", d=0.03)
    # --- overgrowth: vines up the front and the sides, grime streaks
    for x in (-16.0, -9.0, -1.0, 4.0, 11.0, 16.0):
        pk.vines(k, HY, 1, 1, x, 3.4, 0.2, 4.0 if int(x) % 2 else 6.6, seed=int(x) + 90, leaves=95)
    for x in (-14.0, 0.0, 12.0):
        pk.vines(k, -HY, 1, -1, x, 3.0, 0.2, 4.6, seed=int(x) + 300, leaves=70)
    pk.vines(k, HX, 0, 1, 6.0, 3.6, 0.2, 5.6, seed=555, leaves=85)
    pk.vines(k, -HX, 0, -1, 2.0, 3.2, 0.2, 4.2, seed=556, leaves=70)
    for face, axis, sign, u, seed in ((HY, 1, 1, -6.0, 21), (HY, 1, 1, 14.0, 22),
                                      (-HY, 1, -1, 6.0, 23), (HX, 0, 1, -8.0, 24),
                                      (-HX, 0, -1, 8.0, 25)):
        pk.grime(k, face, axis, sign, u, F, 3.6, seed=seed, width=4.0, streaks=5)


def _upper_floors(k):
    """Four closed facade floors (windows, bands, grime, vines) and the roof deck."""
    rng = random.Random(451)
    z = G_TOP + 0.2
    for f in range(FLOORS):
        z0, z1 = z, z + FH - 0.2
        k.box((-HX - 0.04, -HY - 0.04, z1), (HX + 0.04, HY + 0.04, z1 + 0.2), "rock_dark", roof=True)
        k.box((-HX, HY - T, z0), (HX, HY, z1), WALL, roof=True)
        k.box((-HX, -HY, z0), (HX, -HY + T, z1), WALL, roof=True)
        k.box((-HX, -HY + T, z0), (-HX + T, HY - T, z1), WALL, roof=True)
        k.box((HX - T, -HY + T, z0), (HX, HY - T, z1), WALL, roof=True)
        win_z0, win_z1 = z0 + 0.85, z0 + 2.75
        # On the first upper floor the two middle bays are left out for the hospital sign board
        front_u = [-16.4, -11.6, -6.8, -2.0, 2.8, 7.6, 12.4, 16.4]
        if f == 0:
            front_u = [-16.4, -11.6, -6.8, 12.4, 16.4]
        side_u = [-12.4, -7.2, -2.0, 3.2, 8.4, 12.6]
        pk.window_band(k, HY, 1, 1, front_u, win_z0, win_z1, width=2.6, roof=True)
        pk.window_band(k, -HY, 1, -1, front_u, win_z0, win_z1, width=2.6, roof=True)
        pk.window_band(k, -HX, 0, -1, side_u, win_z0, win_z1, width=2.4, roof=True)
        pk.window_band(k, HX, 0, 1, side_u, win_z0, win_z1, width=2.4, roof=True)
        # Painted floor band and grime under the sills
        k.box((-HX - 0.03, HY - 0.02, z1 - 0.42), (HX + 0.03, HY + 0.03, z1 - 0.24), RED, roof=True)
        k.box((-HX - 0.03, -HY - 0.03, z1 - 0.42), (HX + 0.03, -HY + 0.02, z1 - 0.24), RED, roof=True)
        k.box((-HX - 0.03, -HY, z1 - 0.42), (-HX + 0.02, HY, z1 - 0.24), RED, roof=True)
        k.box((HX - 0.02, -HY, z1 - 0.42), (HX + 0.03, HY, z1 - 0.24), RED, roof=True)
        for u in front_u:
            pk.grime(k, HY, 1, 1, u, win_z1 + 0.05, win_z1 + 0.9, seed=int(u * 7) + f, width=1.4,
                     streaks=2, roof=True)
        for u in side_u:
            pk.grime(k, HX, 0, 1, u, win_z1 + 0.05, win_z1 + 0.7, seed=int(u * 3) + f + 50,
                     width=1.2, streaks=2, roof=True)
        if f < 3:
            for x in (-15.0, -4.0, 6.0, 15.5):
                pk.vines(k, HY, 1, 1, x, 3.0, z0, min(z1, z0 + FH * 0.9), seed=int(x) + f * 31,
                         leaves=50, roof=True)
            for y in (-11.0, -1.0, 9.0):
                pk.vines(k, -HX, 0, -1, y, 2.8, z0, min(z1, z0 + FH * 0.8), seed=int(y) + f * 17,
                         leaves=42, roof=True)
        z = z1 + 0.2

    # Roof deck, parapet, head house, water tank, AC units and the helipad
    k.box((-HX - 0.08, -HY - 0.08, ROOF_Z), (HX + 0.08, HY + 0.08, ROOF_Z + 0.22), "concrete_grey", roof=True)
    k.model.add_collider("roof_slab", common.make_box((HX * 2.0 + 0.2, HY * 2.0 + 0.2, 0.2),
                                                    loc=(0.0, 0.0, ROOF_Z + 0.1), color="white"))
    pt = 0.32
    for lo, hi in (((-HX, HY - pt, ROOF_Z + 0.22), (HX, HY, ROOF_Z + PARAPET)),
                   ((-HX, -HY, ROOF_Z + 0.22), (HX, -HY + pt, ROOF_Z + PARAPET)),
                   ((-HX, -HY + pt, ROOF_Z + 0.22), (-HX + pt, HY - pt, ROOF_Z + PARAPET)),
                   ((HX - pt, -HY + pt, ROOF_Z + 0.22), (HX, HY - pt, ROOF_Z + PARAPET))):
        k.box(lo, hi, WALL, roof=True)
    k.box((-HX - 0.05, HY - pt - 0.05, ROOF_Z + PARAPET - 0.08), (HX + 0.05, HY + 0.05, ROOF_Z + PARAPET + 0.04),
          RED, roof=True)
    k.box((-HX - 0.05, -HY - 0.05, ROOF_Z + PARAPET - 0.08), (HX + 0.05, -HY + pt + 0.05, ROOF_Z + PARAPET + 0.04),
          RED, roof=True)
    k.box((-6.0, 4.0, ROOF_Z + 0.22), (0.0, 10.0, ROOF_Z + 3.2), "concrete_grey", roof=True)
    k.box((-6.2, 3.8, ROOF_Z + 3.2), (0.2, 10.2, ROOF_Z + 3.5), "rock_dark", roof=True)
    k.box((-1.4, 3.92, ROOF_Z + 0.22), (1.0, 4.02, ROOF_Z + 2.6), TRIM, roof=True)
    # Wooden water tank on legs
    for sx in (-1, 1):
        for sy in (-1, 1):
            k.box((12.0 + sx * 1.1 - 0.08, -9.0 + sy * 1.1 - 0.08, ROOF_Z + 0.22),
                  (12.0 + sx * 1.1 + 0.08, -9.0 + sy * 1.1 + 0.08, ROOF_Z + 1.4), TRIM, roof=True)
    k.model.add(common.make_frustum(1.5, 1.5, ROOF_Z + 1.4, ROOF_Z + 3.6, segments=10, center=(12.0, -9.0),
                                    color="bark_light"))
    k.model.add(common.make_frustum(1.7, 0.1, ROOF_Z + 3.6, ROOF_Z + 4.2, segments=10, center=(12.0, -9.0),
                                    color="bark_dark"))
    for x, y in ((-12.0, 8.0), (-9.0, 8.0), (-12.0, 5.0), (6.0, 8.5), (9.0, 8.5),
                 (15.0, -13.0), (15.0, -10.0)):
        pk.ac_unit(k, x, y, ROOF_Z + 0.22)
    pk.roof_vent(k, -14.0, -12.0, ROOF_Z + 0.22, 1.6)
    pk.roof_vent(k, 3.0, -11.0, ROOF_Z + 0.22, 1.2)
    pk.roof_antenna(k, 17.0, 12.6, ROOF_Z + 0.22, 3.2)
    pk.roof_antenna(k, -17.0, -12.6, ROOF_Z + 0.22, 2.6)
    pk.helipad(k, 8.0, -1.0, ROOF_Z + 0.235, radius=4.6, color="bone_white")
    for i in range(6):
        k.box((-HX + 1.0, -13.6 + i * 4.0, ROOF_Z + 0.55), (-HX + 1.2, -10.0 + i * 4.0, ROOF_Z + 0.68),
              TRIM, roof=True)
    del rng


def _markers(k):
    """14 spawn empties: most inside the rooms, a few at the doors."""
    spawns = [
        (1, -13.0, 10.6, F),     # waiting room
        (2, -6.0, 5.0, F),       # waiting room, by the coffee table
        (3, 5.6, 10.4, F),       # reception hall in front of the desk
        (4, 15.0, 6.4, F),       # reception hall, lift side
        (5, 0.0, 0.0, F),        # corridor centre
        (6, -9.0, 0.0, F),       # corridor, ward end
        (7, -13.4, -8.4, F),     # ward aisle
        (8, -17.0, -6.6, F),     # ward, by the beds
        (9, 4.4, -10.6, F),      # pharmacy, inside the cage
        (10, 12.0, -4.2, F),     # office
        (11, 14.0, -12.4, F),    # toilets
        (12, 6.0, HY + 1.2, 0.0),    # outside, under the ER canopy at the main doors
        (13, -12.7, HY + 0.8, 0.0),  # outside, at the second side door
        (14, -17.0, -8.2, 0.0),      # outside, on the left ramp
    ]
    for i, x, y, z in spawns:
        pk.spawn(k, i, x, y, z)
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(38.0, 30.0, 1.0))
    k.empty("door_main", (6.0, HY, F))
    k.empty("door_side", (-12.7, HY, F))
    k.empty("door_back", (-5.1, -HY, F))


def build(model):
    """Builds the whole hospital: ground floor, exterior, upper floors, roof, markers."""
    k = kit.Kit(model)
    f_doors, f_wins, b_wins, l_wins, r_wins = _ground_shell(k)
    _ground_windows(k, f_wins, b_wins, l_wins, r_wins)
    _interior_walls(k)
    _ceiling_panel_fallen(k)
    _waiting_area(k)
    _reception(k)
    _records_room(k)
    _ward(k)
    _pharmacy(k)
    _office_and_toilets(k)
    _debris_and_stains(k)
    _exterior(k)
    _upper_floors(k)
    _markers(k)


def build_lod1(model):
    """The outside only: mass, window strips, canopy, roof deck and helipad."""
    k = kit.Kit(model)
    k.box((-HX, -HY, 0.0), (HX, HY, ROOF_Z), WALL)
    f_doors, f_wins = _front_openings()
    for op in f_wins:
        (x0, _y0, z0), (x1, _y1, z1) = op
        k.box((x0 + 0.1, HY - 0.02, z0), (x1 - 0.1, HY + 0.04, z1), "soot")
    for op in f_doors:
        (x0, _y0, z0), (x1, _y1, z1) = op
        k.box((x0, HY - 0.02, z0), (x1, HY + 0.04, z1), "soot")
    z = G_TOP + 0.2
    for _f in range(FLOORS):
        k.box((-HX - 0.02, HY - 0.02, z + 0.85), (HX + 0.02, HY + 0.03, z + 2.75), "glass_dark")
        k.box((-HX - 0.02, -HY - 0.03, z + 0.85), (HX + 0.02, -HY + 0.02, z + 2.75), "glass_dark")
        k.box((-HX - 0.03, -HY, z + 0.85), (-HX + 0.02, HY, z + 2.75), "glass_dark")
        k.box((HX - 0.02, -HY, z + 0.85), (HX + 0.03, HY, z + 2.75), "glass_dark")
        k.box((-HX - 0.03, HY - 0.02, z + 3.3), (HX + 0.03, HY + 0.03, z + 3.5), RED)
        z += FH
    k.box((-HX - 0.1, -HY - 0.1, ROOF_Z), (HX + 0.1, HY + 0.1, ROOF_Z + PARAPET), "rock_light")
    k.box((1.2, HY, 3.90), (10.8, CANOPY_Y1, 4.42), "white")
    k.box((1.08, CANOPY_Y1 - 0.28, 3.62), (10.92, CANOPY_Y1, 4.62), RED)
    for cx in (1.75, 10.25):
        for cy in (HY + 0.7, CANOPY_Y1 - 0.75):
            k.box((cx - 0.2, cy - 0.2, 0.0), (cx + 0.2, cy + 0.2, 3.9), "rock_light")
    k.box((-6.0, 4.0, ROOF_Z + 0.22), (0.0, 10.0, ROOF_Z + 3.2), "concrete_grey")
    k.box((-13.0, 7.0, ROOF_Z + 0.22), (-8.0, 9.0, ROOF_Z + 1.1), "metal")
    k.model.add(common.make_frustum(1.5, 1.5, ROOF_Z + 1.4, ROOF_Z + 3.4, segments=8, center=(12.0, -9.0),
                                    color="bark_light"))
    pk.helipad(k, 8.0, -1.0, ROOF_Z + 0.24, radius=4.6, color="bone_white")


def cutaway_preview():
    """Renders the walk-in ground floor alone, twice: a 3/4 view and a straight-down plan.

    `metro_hospital_cut.png` (from the front above) and `metro_hospital_cut_top.png` (a plan).
    Everything above the ground floor ceiling is dropped (upper floors, the roof deck and the
    ground floor ceiling slab), so the rooms read at a glance.
    """
    common.reset_scene()
    model = common.Model(NAME + "_cut", CATEGORY, BUDGET)
    build(model)
    limit = G_TOP - 0.4
    for top, view, name in ((False, (0.35, 1.0, 1.0), NAME + "_cut"),
                            (True, None, NAME + "_cut_plan")):
        parts = [bm.copy() for bm, _roof in model._visual
                 if not (bm.verts and min(v.co.z for v in bm.verts) >= limit)]
        obj = common.object_from_bm(common.combine(parts), name + "_view")
        common.render_preview(name, obj, top=top, view=view)
        common._remove_objects([obj])
    return os.path.join(common.PREVIEW_DIR, NAME + "_cut.png")


def _extra_preview(name, view, top=False):
    """Renders one more view of the whole model (the back, the front, the roof plan)."""
    common.reset_scene()
    model = common.Model(NAME, CATEGORY, BUDGET)
    build(model)
    obj = common.object_from_bm(common.combine([bm.copy() for bm, _r in model._visual]), name + "_view")
    common.render_preview(name, obj, top=top, view=view)
    common._remove_objects([obj])
    return os.path.join(common.PREVIEW_DIR, name + ".png")


def back_preview():
    """`metro_hospital_back.png`: the outside from behind (the -Y service side)."""
    return _extra_preview(NAME + "_back", (-0.8, -1.0, 0.75))


def front_preview():
    """`metro_hospital_front.png`: straight at the street front, where the ER canopy is."""
    return _extra_preview(NAME + "_front", (-0.15, 1.0, 0.42))


def build_all():
    """Builds metro_hospital + metro_hospital_lod1 + the cut-away and back previews. Returns [(name, ok)]."""
    ok = common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0),
                          top_preview=True, view=kit.FRONT_VIEW)
    ok1 = common.run_model(LOD1, CATEGORY, BUDGET_LOD1, build_lod1, grip=(0.0, 0.0, 0.0),
                           view=kit.FRONT_VIEW)
    back_preview()
    front_preview()
    cutaway_preview()
    return [(NAME, ok), (LOD1, ok1)]


if __name__ == "__main__":
    out = build_all()
    failed = [n for n, ok in out if not ok]
    for n, ok in out:
        print(f"hospital: {n} {'OK' if ok else 'FAILED'}")
    sys.exit(1 if failed else 0)
