"""police: enterable 2-floor police station with fenced parking corner, 24 x 18 m lot.

Lot 24 x 18 m (x -12 to 12, y -9 to 9). Building footprint 15 x 15 m (x -4 to 11, y -8 to 7),
fenced parking corner (x -11 to -4, y -8 to 7). Ground floor enterable, upper floor closed.
Blue-gray walls, blue stripe, steps up to double doors, flagpole without a flag, barred windows,
fenced parking with barrier arm. Inside: front desk lobby, hallway, cell with bars, locker room,
armory/evidence room, blocked stairs.
5 loot, 4 spawns.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "police", "buildings", 8000
F = kit.FLOOR_TOP  # 0.1 m
TOP1 = 3.0         # Floor 1 wall top
TOP2 = 5.8         # Floor 2 wall top
ROOF_TOP = 6.2     # Parapet top

WALL_COLOR = "gown_blue"
STRIPE_COLOR = "shirt_blue"
TRIM_COLOR = "rock_dark"


def build_parking(k):
    """Fenced parking corner on the left (x -11 to -4, y -8 to 7)."""
    # Asphalt surface - sunken base so bottom does not share Z=0 plane with fixtures
    k.box((-11.0, -8.0, -0.05), (-4.0, 7.0, 0.04), "rock_dark")
    # White parking bay lines
    for y in (-5.0, -2.0, 1.0, 4.0):
        k.box((-10.5, y - 0.06, 0.04), (-7.0, y + 0.06, 0.05), "white")
    k.box((-10.5, -5.0, 0.04), (-10.38, 4.0, 0.05), "white")

    # Perimeter fence posts along west edge (x = -11.0)
    for y in (-8.0, -5.5, -3.0, -0.5, 2.0, 4.5, 7.0):
        k.solid((-11.06, y - 0.06, 0.0), (-10.94, y + 0.06, 2.2), "metal", base="fence_post")
    # Horizontal fence rails (west edge)
    k.solid((-11.04, -8.0, 2.14), (-10.96, 7.0, 2.2), "metal", base="fence_rail_top")
    k.box((-11.04, -8.0, 0.2), (-10.96, 7.0, 0.26), "metal")
    k.box((-11.04, -8.0, 1.15), (-10.96, 7.0, 1.21), "metal")

    # South edge fence (y = -8.0, x -11 to -4) - butts against west rails
    for x in (-8.7, -6.3):
        k.solid((x - 0.06, -8.06, 0.0), (x + 0.06, -7.94, 2.2), "metal", base="fence_post")
    k.solid((-10.96, -8.04, 2.14), (-4.0, -7.96, 2.2), "metal", base="fence_rail_top")
    k.box((-10.96, -8.04, 0.2), (-4.0, -7.96, 0.26), "metal")
    k.box((-10.96, -8.04, 1.15), (-4.0, -7.96, 1.21), "metal")

    # North front fence (y = 7.0, x -11 to -8.5) - butts against west rails
    for x in (-9.7, -8.5):
        k.solid((x - 0.06, 6.94, 0.0), (x + 0.06, 7.06, 2.2), "metal", base="fence_post")
    k.solid((-10.96, 6.96, 2.14), (-8.5, 7.04, 2.2), "metal", base="fence_rail_top")
    k.box((-10.96, 6.96, 0.2), (-8.5, 7.04, 0.26), "metal")
    k.box((-10.96, 6.96, 1.15), (-8.5, 7.04, 1.21), "metal")

    # Parking barrier at driveway entrance (x -8.5 to -4.0, y = 7.0)
    # Barrier housing cabinet at x = -4.4
    k.solid((-4.6, 6.75, 0.0), (-4.1, 7.25, 1.1), "shirt_blue", base="barrier_housing")
    k.box((-4.55, 6.78, 0.8), (-4.15, 7.22, 1.05), "rock_dark")
    # Pivot joint and barrier arm
    k.solid((-8.3, 6.94, 0.82), (-4.1, 7.06, 0.94), "white", base="barrier_arm")
    for rx in (-8.0, -7.0, -6.0, -5.0):
        k.box((rx, 6.93, 0.81), (rx + 0.35, 7.07, 0.95), "red_paint")

    # Dumpster in parking corner
    k.solid((-10.5, -7.2, 0.0), (-8.8, -5.8, 1.3), "shirt_blue", base="dumpster")
    k.box((-10.55, -7.25, 1.25), (-8.75, -5.75, 1.35), "metal")
    # Trash bag clutter beside dumpster
    k.box((-8.6, -7.0, 0.0), (-8.1, -6.4, 0.45), "pants_dark")
    k.box((-8.5, -6.3, 0.0), (-8.0, -5.8, 0.5), "pants_dark")


def build_exterior(k):
    """Building foundation, walls, steps, and flagpole."""
    # Foundation slab
    k.solid((-4.0, -8.0, 0.0), (11.0, 7.0, F), "rock_mid", base="floor")

    # Steps up to front double doors (x 2.0 to 4.0, y 7.0)
    k.solid((1.5, 7.0, 0.0), (4.5, 7.5, F), "rock_dark", base="steps_top")
    k.solid((1.5, 7.5, 0.0), (4.5, 8.0, 0.05), "rock_dark", base="steps_bottom")
    # Handrails for steps
    for hx in (1.55, 4.45):
        k.box((hx - 0.03, 7.0, F), (hx + 0.03, 7.05, F + 0.85), "metal")
        k.box((hx - 0.03, 7.95, 0.05), (hx + 0.03, 8.0, 0.05 + 0.85), "metal")
        k.box((hx - 0.03, 7.0, F + 0.8), (hx + 0.03, 8.0, F + 0.85), "metal")

    # Flagpole without a flag (in front of police station at x = 7.5, y = 7.6)
    k.solid((7.2, 7.3, 0.0), (7.8, 7.9, 0.25), "rock_dark", base="flagpole_base")
    k.model.add(common.make_frustum(0.08, 0.05, 0.25, 7.2, segments=8, center=(7.5, 7.6), color="metal"))
    k.collider((7.42, 7.52, 0.25), (7.58, 7.68, 7.2), "flagpole_col")
    # Finial ball at top
    k.box((7.44, 7.54, 7.2), (7.56, 7.66, 7.35), "brass")

    # Front double door opening & barred windows
    main_door = ((2.0, 6.8, 0.0), (4.0, 7.1, 2.4))
    side_door = ((-4.1, 1.0, 0.0), (-3.7, 2.2, kit.DOOR_H))
    win_f1 = ((-2.8, 6.8, 1.0), (-0.8, 7.1, 2.2))
    win_f2 = ((5.5, 6.8, 1.0), (7.5, 7.1, 2.2))
    win_f3 = ((8.5, 6.8, 1.0), (10.0, 7.1, 2.2))

    # Side and back windows
    win_left1 = ((-4.1, -6.5, 1.0), (-3.7, -4.5, 2.2))
    win_left2 = ((-4.1, -3.0, 1.0), (-3.7, -1.0, 2.2))
    win_right1 = ((10.8, -6.0, 1.0), (11.1, -4.0, 2.2))
    win_right2 = ((10.8, -1.5, 1.0), (11.1, 0.5, 2.2))
    win_right3 = ((10.8, 3.0, 1.0), (11.1, 5.0, 2.2))
    win_back1 = ((5.5, -8.1, 1.0), (8.5, -7.8, 2.2))

    # Ground floor exterior walls
    k.wall((-4.0, 6.85, F), (11.0, 7.0, TOP1), openings=[main_door, win_f1, win_f2, win_f3],
           color=WALL_COLOR, base="wall_front")
    k.wall((-4.0, -8.0, F), (11.0, -7.85, TOP1), openings=[win_back1],
           color=WALL_COLOR, base="wall_back")
    k.wall((-4.0, -7.85, F), (-3.85, 6.85, TOP1), openings=[side_door, win_left1, win_left2],
           color=WALL_COLOR, base="wall_left")
    k.wall((10.85, -7.85, F), (11.0, 6.85, TOP1), openings=[win_right1, win_right2, win_right3],
           color=WALL_COLOR, base="wall_right")

    # Blue accent stripe below floor 1 ceiling (z 2.65 to 2.95)
    for lo, hi in [
        ((-4.02, 6.98, 2.65), (11.02, 7.02, 2.95)),
        ((-4.02, -8.02, 2.65), (11.02, -7.98, 2.95)),
        ((-4.02, -7.98, 2.65), (-3.98, 6.98, 2.95)),
        ((10.98, -7.98, 2.65), (11.02, 6.98, 2.95)),
    ]:
        k.box(lo, hi, STRIPE_COLOR)

    # Window frames and metal security bars for all ground floor windows
    for win, face, sign in [
        (win_f1, 7.0, 1), (win_f2, 7.0, 1), (win_f3, 7.0, 1),
        (win_left1, -4.0, -1), (win_left2, -4.0, -1),
        (win_right1, 11.0, 1), (win_right2, 11.0, 1), (win_right3, 11.0, 1),
        (win_back1, -8.0, -1),
    ]:
        k.frame(win, face, sign, color="metal", t=0.08, depth=0.06)
        # Add security bars
        (x0, y0, z0), (x1, y1, z1) = win
        if abs(sign) == 1 and (face == 7.0 or face == -8.0):
            # Window along X
            step = 0.22
            x = x0 + 0.15
            while x < x1 - 0.05:
                k.box((x - 0.02, face + sign * 0.02, z0), (x + 0.02, face + sign * 0.05, z1), "gun_black")
                x += step
            k.box((x0, face + sign * 0.02, (z0 + z1) / 2 - 0.02), (x1, face + sign * 0.05, (z0 + z1) / 2 + 0.02), "gun_black")
        else:
            # Window along Y
            step = 0.22
            y = y0 + 0.15
            while y < y1 - 0.05:
                k.box((face + sign * 0.02, y - 0.02, z0), (face + sign * 0.05, y + 0.02, z1), "gun_black")
                y += step
            k.box((face + sign * 0.02, y0, (z0 + z1) / 2 - 0.02), (face + sign * 0.05, y1, (z0 + z1) / 2 + 0.02), "gun_black")

    # Front double door frame and visual door leaves
    k.frame(main_door, 7.0, 1, color="metal", t=0.1)
    # One leaf partly open, left leaf closed
    k.box((2.05, 6.95, F), (3.0, 7.02, 2.35), "metal")
    k.box((2.2, 6.94, 1.1), (2.85, 7.03, 2.1), "glass_tint")
    # Door handle
    k.box((2.85, 6.93, 1.05), (2.95, 7.04, 1.25), "brass")
    # Right leaf ajar (angled into lobby)
    k.box((3.0, 6.5, F), (3.1, 7.0, 2.35), "metal")

    # Side door frame
    k.frame(side_door, -4.0, -1, color="metal", t=0.08)
    k.box((-3.98, 1.05, F), (-3.92, 2.15, kit.DOOR_H), "metal")


def build_interior(k):
    """Ground floor interior: lobby, front desk, hallway, cell, locker room, armory."""
    # Hallway walls
    # Hallway runs from y = 3.0 down to y = -7.85 at x = 0.85 to 2.85
    # Wall dividing west rooms (stairs) from hallway (at x = 0.85); jail cell (y -2 to 3) has its own barred wall
    stairs_opening = ((0.7, -7.5, 0.0), (1.0, -4.5, TOP1))
    k.wall((0.85, -7.85, F), (1.0, -2.0, TOP1), openings=[stairs_opening],
           color="white", base="wall_hall_west")

    # Wall dividing east rooms (lobby, locker room, armory) from hallway (at x = 2.85)
    door_lockers = ((2.7, -0.2, 0.0), (3.0, 1.0, kit.DOOR_H))
    door_armory = ((2.7, -5.5, 0.0), (3.0, -4.3, kit.DOOR_H))
    k.wall((2.85, -7.85, F), (3.0, 3.0, TOP1), openings=[door_lockers, door_armory],
           color="white", base="wall_hall_east")

    # Dividing wall between Cell (y -2.0 to 3.0) and Stairwell (y -8.0 to -2.0)
    k.wall((-3.85, -2.15, F), (0.85, -2.0, TOP1), color="white", base="wall_cell_stairs")

    # Dividing wall between Lockers (y -2.5 to 3.0) and Armory (y -8.0 to -2.5)
    k.wall((3.0, -2.65, F), (10.85, -2.5, TOP1), color="white", base="wall_lockers_armory")

    # Wall separating Lobby from Locker room (y = 3.0, x = 3.0 to 10.85) - butts against hallway wall at x=3.0
    lobby_door = ((4.2, 2.85, 0.0), (5.4, 3.15, kit.DOOR_H))
    k.wall((3.0, 2.85, F), (10.85, 3.0, TOP1), openings=[lobby_door], color="white", base="wall_lobby_east")

    # ---- LOBBY & FRONT DESK ----
    # L-shaped front desk counter
    # Front section facing entrance
    k.solid((4.0, 3.8, F), (4.5, 6.2, 1.1), "gown_blue", base="counter_side")
    k.box((3.95, 3.75, 1.1), (4.55, 6.25, 1.16), "wood_crate")
    # Return section
    k.solid((1.5, 3.8, F), (4.0, 4.3, 1.1), "gown_blue", base="counter_front")
    k.box((1.45, 3.75, 1.1), (3.95, 4.35, 1.16), "wood_crate")

    # Office computer monitor, keyboard, chair behind desk
    k.box((2.8, 3.9, 1.16), (3.2, 4.15, 1.55), "gun_black")
    k.box((2.7, 4.16, 1.16), (3.3, 4.28, 1.18), "rock_light")
    # Swivel chair
    k.solid((3.0, 4.8, F), (3.4, 5.2, 0.9), "shirt_blue", base="desk_chair")

    # Filing cabinets behind desk against wall
    k.solid((5.0, 6.2, F), (6.2, 6.8, 1.4), "metal", base="file_cabinet_1")
    k.solid((6.3, 6.2, F), (7.5, 6.8, 1.4), "metal", base="file_cabinet_2")

    # Waiting benches in lobby
    k.solid((-2.5, 4.8, F), (-0.5, 5.4, 0.45), "wood_crate", base="lobby_bench")
    k.box((-2.5, 4.8, 0.45), (-0.5, 4.95, 0.85), "wood_crate")

    # Water cooler in lobby
    k.solid((-0.2, 6.2, F), (0.2, 6.6, 0.9), "white", base="water_cooler")
    k.box((-0.12, 6.28, 0.9), (0.12, 6.52, 1.3), "water_blue")

    # ---- JAIL CELL ----
    # Front barred wall facing hallway: at x = 0.85, y = -2.0 to 3.0
    # Curb and header
    k.solid((0.8, -2.0, F), (0.9, 3.0, F + 0.15), "gun_black", base="cell_curb")
    k.solid((0.8, -2.0, 2.85), (0.9, 3.0, TOP1), "gun_black", base="cell_header")
    # Vertical steel bars
    y = -1.85
    while y <= 2.85:
        # Leave door opening at y 0.6 to 1.6 clear
        if not (0.6 <= y <= 1.6):
            k.solid((0.82, y - 0.025, F + 0.15), (0.88, y + 0.025, 2.85), "gun_black", base="cell_bar")
        y += 0.22
    # Sliding barred door (parked open against the bars at y 1.6 to 2.5)
    k.box((0.75, 1.6, F + 0.15), (0.8, 2.5, 2.75), "gun_black")

    # Inside cell: Bunk bed against west wall
    k.solid((-3.8, -1.8, F), (-1.8, -0.8, 0.45), "metal", base="bunk_frame")
    k.box((-3.75, -1.75, 0.45), (-1.85, -0.85, 0.6), "rock_light")  # mattress
    k.box((-3.7, -1.7, 0.6), (-3.2, -0.9, 0.7), "white")             # pillow
    # Upper bunk
    k.box((-3.8, -1.8, 1.4), (-1.8, -0.8, 1.5), "metal")
    k.box((-3.75, -1.75, 1.5), (-1.85, -0.85, 1.65), "rock_light")
    # Bed corner posts
    for bx in (-3.8, -1.85):
        for by in (-1.8, -0.85):
            k.box((bx, by, F), (bx + 0.06, by + 0.06, 1.8), "metal")

    # Stainless steel prison toilet
    k.solid((-3.8, 1.8, F), (-3.2, 2.4, 0.75), "metal", base="cell_toilet")
    k.box((-3.8, 2.3, 0.75), (-3.2, 2.45, 1.1), "metal")

    # Small cell bench
    k.solid((-3.8, 0.3, F), (-2.6, 0.8, 0.45), "metal", base="cell_bench")

    # ---- LOCKER ROOM ----
    # Bank of lockers along east wall (x = 10.2 to 10.85, y = -2.0 to 2.5)
    k.solid((10.15, -2.0, F), (10.85, 2.5, 2.1), "shirt_blue", base="lockers_bank")
    # Locker doors lines and vents
    ly = -1.9
    while ly < 2.4:
        k.box((10.13, ly - 0.02, F), (10.16, ly + 0.02, 2.1), "rock_dark")
        k.box((10.12, ly + 0.1, 1.6), (10.14, ly + 0.35, 1.8), "gun_black")  # vent
        ly += 0.45
    # Locker room bench in center
    k.solid((5.5, -0.3, F), (8.5, 0.3, 0.45), "wood_crate", base="locker_bench")
    for bx in (5.6, 8.3):
        k.box((bx, -0.28, F), (bx + 0.08, 0.28, 0.45), "metal")
    # Duffel bag on bench
    k.box((6.2, -0.18, 0.45), (6.9, 0.18, 0.7), "pants_dark")

    # ---- ARMORY / EVIDENCE ROOM ----
    # Heavy storage shelves along east wall (x = 9.8 to 10.85, y = -7.5 to -3.2)
    k.solid((9.8, -7.5, F), (10.85, -3.2, 2.2), "metal", base="armory_shelves")
    # Ammo cans and weapon crates on shelves
    k.box((10.0, -7.0, 0.9), (10.6, -6.2, 1.25), "leaf_dark")
    k.box((10.0, -5.8, 0.9), (10.6, -5.0, 1.25), "leaf_dark")
    k.box((9.9, -4.5, 1.4), (10.7, -3.5, 1.85), "wood_crate")
    # Weapon rack along south wall
    k.solid((4.5, -7.85, F), (8.0, -7.5, 1.6), "bark_dark", base="gun_rack")
    for gx in (4.8, 5.5, 6.2, 6.9, 7.6):
        k.box((gx, -7.65, 0.3), (gx + 0.06, -7.55, 1.5), "gun_black")

    # Evidence desk
    k.solid((5.0, -4.5, F), (6.8, -3.5, 0.8), "metal", base="evidence_desk")

    # ---- BLOCKED STAIRS TO UPPER FLOOR ----
    # Staircase at south end of hallway (x = -3.85 to 0.85, y = -7.85 to -4.0)
    # Concrete steps rising toward south
    step_y = -4.2
    step_z = F
    while step_y > -7.5 and step_z < 2.0:
        k.solid((-3.85, step_y - 0.35, F), (0.85, step_y, step_z + 0.2), "rock_mid", base="stair_step")
        step_y -= 0.35
        step_z += 0.2

    # Boarded barricade blocking the stairs
    for bz in (0.3, 0.7, 1.1, 1.5, 1.9):
        k.solid((-3.85, -4.3, bz), (0.85, -4.18, bz + 0.25), "wood_crate", base="barricade_board")
    # Diagonal braces & hazard warning sign
    k.box((-2.5, -4.4, 0.8), (-0.5, -4.3, 1.4), "red_paint")
    k.box((-2.3, -4.42, 0.9), (-0.7, -4.28, 1.3), "white")
    # Concrete rubble pile in front of stairs
    k.solid((-2.0, -4.0, F), (-0.5, -3.3, 0.6), "rock_dark", base="rubble_1")
    k.box((-1.5, -3.6, 0.6), (-0.8, -3.2, 0.9), "rock_mid")


def build_upper_and_roof(k):
    """Upper floor (closed) and roof with AC units. All visual parts use roof=True for top view."""
    # Floor slab dividing ground and second floor
    k.box((-4.0, -8.0, TOP1), (11.0, 7.0, TOP1 + 0.2), "rock_dark", roof=True)
    k.collider((-4.0, -8.0, TOP1), (11.0, 7.0, TOP1 + 0.2), "floor2_slab")

    # Second floor exterior walls
    k.box((-4.0, 6.85, TOP1 + 0.2), (11.0, 7.0, TOP2), WALL_COLOR, roof=True)
    k.box((-4.0, -8.0, TOP1 + 0.2), (11.0, -7.85, TOP2), WALL_COLOR, roof=True)
    k.box((-4.0, -7.85, TOP1 + 0.2), (-3.85, 6.85, TOP2), WALL_COLOR, roof=True)
    k.box((10.85, -7.85, TOP1 + 0.2), (11.0, 6.85, TOP2), WALL_COLOR, roof=True)

    # Upper floor window frames and dark panes (decorative)
    for wx0, wx1 in ((-2.5, -0.5), (1.5, 3.5), (5.5, 7.5), (8.5, 10.0)):
        k.box((wx0, 6.98, 4.0), (wx1, 7.04, 5.2), "glass_tint", roof=True)
        k.box((wx0 - 0.06, 6.96, 3.94), (wx1 + 0.06, 7.06, 4.02), "metal", roof=True)
        k.box((wx0 - 0.06, 6.96, 5.18), (wx1 + 0.06, 7.06, 5.26), "metal", roof=True)
    for wy0, wy1 in ((-6.5, -4.5), (-3.0, -1.0), (1.0, 3.0), (4.5, 6.5)):
        k.box((10.98, wy0, 4.0), (11.04, wy1, 5.2), "glass_tint", roof=True)
        k.box((-4.04, wy0, 4.0), (-3.98, wy1, 5.2), "glass_tint", roof=True)

    # Upper floor blue stripe
    for lo, hi in [
        ((-4.02, 6.98, 5.4), (11.02, 7.02, 5.65)),
        ((-4.02, -8.02, 5.4), (11.02, -7.98, 5.65)),
        ((-4.02, -7.98, 5.4), (-3.98, 6.98, 5.65)),
        ((10.98, -7.98, 5.4), (11.02, 6.98, 5.65)),
    ]:
        k.box(lo, hi, STRIPE_COLOR, roof=True)

    # Roof slab
    k.box((-4.1, -8.1, TOP2), (11.1, 7.1, TOP2 + 0.15), "rock_dark", roof=True)
    k.collider((-4.1, -8.1, TOP2), (11.1, 7.1, TOP2 + 0.15), "roof_slab")

    # Parapet rim around roof
    k.box((-4.15, 6.95, TOP2 + 0.15), (11.15, 7.15, ROOF_TOP), WALL_COLOR, roof=True)
    k.box((-4.15, -8.15, TOP2 + 0.15), (11.15, -7.95, ROOF_TOP), WALL_COLOR, roof=True)
    k.box((-4.15, -7.95, TOP2 + 0.15), (-3.95, 6.95, ROOF_TOP), WALL_COLOR, roof=True)
    k.box((10.95, -7.95, TOP2 + 0.15), (11.15, 6.95, ROOF_TOP), WALL_COLOR, roof=True)
    # Parapet dark coping stone cap
    k.box((-4.2, 6.92, ROOF_TOP), (11.2, 7.18, ROOF_TOP + 0.05), TRIM_COLOR, roof=True)
    k.box((-4.2, -8.18, ROOF_TOP), (11.2, -7.92, ROOF_TOP + 0.05), TRIM_COLOR, roof=True)
    k.box((-4.2, -7.92, ROOF_TOP), (-3.92, 6.92, ROOF_TOP + 0.05), TRIM_COLOR, roof=True)
    k.box((10.92, -7.92, ROOF_TOP), (11.2, 6.92, ROOF_TOP + 0.05), TRIM_COLOR, roof=True)

    # Rooftop HVAC units and vents
    k.box((1.0, -1.0, TOP2 + 0.15), (3.5, 1.5, TOP2 + 1.1), "metal", roof=True)
    k.box((1.1, -0.9, TOP2 + 1.1), (3.4, 1.4, TOP2 + 1.25), "rock_dark", roof=True)
    k.box((-2.0, -5.0, TOP2 + 0.15), (-0.5, -3.5, TOP2 + 0.9), "metal", roof=True)
    k.box((6.0, -4.5, TOP2 + 0.15), (7.2, -3.3, TOP2 + 0.8), "metal", roof=True)


def build(model):
    k = kit.Kit(model)
    build_parking(k)
    build_exterior(k)
    build_interior(k)
    build_upper_and_roof(k)

    # Empties
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(24.0, 18.0, 1.0))
    # Doors
    k.empty("door_1", (3.0, 7.0, F))     # Front entrance
    k.empty("door_2", (-4.0, 1.6, F))    # Side parking door

    # 5 Loot spots
    k.empty("loot_1", (3.0, 4.0, 1.18))   # Front desk counter
    k.empty("loot_2", (-2.8, -1.3, 0.6))  # Jail cell bunk bed
    k.empty("loot_3", (7.0, 0.0, 0.48))   # Locker room bench
    k.empty("loot_4", (10.3, -5.5, 1.0))  # Armory shelf
    k.empty("loot_5", (-8.5, -5.5, 0.04)) # Fenced parking lot near dumpster

    # 4 Zombie spawns
    k.empty("spawn_1", (-2.2, 0.5, F))    # Inside jail cell
    k.empty("spawn_2", (2.0, 5.0, F))     # In lobby
    k.empty("spawn_3", (6.0, 1.2, F))     # In locker room
    k.empty("spawn_4", (-7.5, 1.5, 0.04)) # Outside in parking area


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
