"""hospital: enterable 3-floor hospital with ambulance bay, 30 x 22 m lot.

Lot 30 x 22 m (x -15 to 15, y -11 to 11). Building footprint 21 x 18 m (x -7 to 14, y -9 to 9),
covered ambulance bay 7 x 17 m (x -14 to -7, y -8.5 to 8.5). Ground floor enterable, upper floors closed.
White walls with red stripe, red cross sign (boxes), ambulance bay with canopy roof on posts,
wide entrance doors with awning. Inside: reception waiting area, central corridor, 2 rooms with
beds (simple boxes), pharmacy/medical storage, blocked stairwell.
5 loot, 5 spawns.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "hospital", "buildings", 10000
F = kit.FLOOR_TOP  # 0.1 m
TOP1 = 3.2         # Floor 1 wall top
TOP2 = 6.2         # Floor 2 wall top
TOP3 = 9.2         # Floor 3 wall top
ROOF_TOP = 9.6     # Parapet top

WALL_COLOR = "white"
STRIPE_COLOR = "red_paint"
TRIM_COLOR = "rock_dark"


def build_ambulance_bay(k):
    """Covered ambulance bay on the west end (x -14 to -7, y -8.5 to 8.5)."""
    # Asphalt drive-through pad
    k.box((-14.0, -8.5, 0.0), (-7.0, 8.5, 0.04), "rock_dark")

    # Red emergency bay parking box
    k.box((-12.5, -6.5, 0.04), (-8.5, 6.5, 0.06), STRIPE_COLOR)
    # White border lines for parking box
    for y in (-6.5, 6.5):
        k.box((-12.44, y - 0.06, 0.06), (-8.56, y + 0.06, 0.08), "white")
    for x in (-12.5, -8.5):
        k.box((x - 0.06, -6.56, 0.06), (x + 0.06, 6.56, 0.08), "white")
    # White cross in center of bay (non-overlapping boxes)
    k.box((-11.0, -1.5, 0.06), (-10.0, 1.5, 0.08), "white")
    k.box((-11.5, -0.5, 0.06), (-11.0, 0.5, 0.08), "white")
    k.box((-10.0, -0.5, 0.06), (-9.5, 0.5, 0.08), "white")

    # 4 Support posts (z 0 to 3.8 m)
    posts = [(-13.5, -7.5), (-13.5, 7.5), (-7.5, -7.5), (-7.5, 7.5)]
    for px, py in posts:
        # Base concrete collar
        k.solid((px - 0.32, py - 0.32, 0.04), (px + 0.32, py + 0.32, 0.4), "rock_mid", base="bay_post_base")
        # Post shaft
        k.solid((px - 0.25, py - 0.25, 0.4), (px + 0.25, py + 0.25, 3.8), WALL_COLOR, base="bay_post")

    # Canopy roof slab
    k.box((-14.2, -8.7, 3.8), (-6.8, 8.7, 4.15), WALL_COLOR, roof=True)
    k.collider((-14.2, -8.7, 3.8), (-6.8, 8.7, 4.15), "canopy_roof")
    # Red fascia edge around canopy roof
    for lo, hi in [
        ((-14.18, -8.72, 3.78), (-6.78, -8.68, 4.17)),
        ((-14.18, 8.68, 3.78), (-6.78, 8.72, 4.17)),
        ((-14.22, -8.72, 3.78), (-14.18, 8.72, 4.17)),
    ]:
        k.box(lo, hi, STRIPE_COLOR, roof=True)

    # Red cross sign above emergency doors on the west hospital wall
    k.box((-7.08, 0.0, 4.2), (-7.05, 1.0, 5.2), WALL_COLOR, roof=True)
    k.box((-7.10, 0.38, 4.3), (-7.07, 0.62, 5.1), STRIPE_COLOR, roof=True)
    k.box((-7.12, 0.1, 4.58), (-7.07, 0.9, 4.82), STRIPE_COLOR, roof=True)


def build_exterior(k):
    """Main hospital building exterior, front entrance canopy, and facade red cross."""
    # Foundation floor slab (x -7 to 14, y -9 to 9)
    k.solid((-7.0, -9.0, 0.0), (14.0, 9.0, F), "rock_light", base="floor")

    # Openings for ground floor
    main_door = ((2.5, 8.8, 0.0), (5.0, 9.1, 2.5))           # Wide front double doors (2.5 m)
    emer_door = ((-7.1, -0.2, 0.0), (-6.8, 1.8, 2.5))         # Ambulance bay emergency entrance (2.0 m)

    # Ground floor windows
    win_f1 = ((-5.5, 8.8, 1.0), (-2.0, 9.1, 2.2))
    win_f2 = ((7.0, 8.8, 1.0), (10.0, 9.1, 2.2))
    win_f3 = ((11.0, 8.8, 1.0), (13.0, 9.1, 2.2))

    win_b1 = ((-5.5, -9.1, 1.0), (-2.0, -8.8, 2.2))
    win_b2 = ((3.0, -9.1, 1.0), (6.0, -8.8, 2.2))
    win_b3 = ((8.0, -9.1, 1.0), (12.0, -8.8, 2.2))

    win_r1 = ((13.8, -7.0, 1.0), (14.1, -3.0, 2.2))
    win_r2 = ((13.8, 3.0, 1.0), (14.1, 7.0, 2.2))

    win_l1 = ((-7.1, -7.0, 1.0), (-6.8, -4.0, 2.2))
    win_l2 = ((-7.1, 4.0, 1.0), (-6.8, 7.0, 2.2))

    # Ground floor exterior walls
    k.wall((-7.0, 8.85, F), (14.0, 9.0, TOP1), openings=[main_door, win_f1, win_f2, win_f3],
           color=WALL_COLOR, base="wall_front")
    k.wall((-7.0, -9.0, F), (14.0, -8.85, TOP1), openings=[win_b1, win_b2, win_b3],
           color=WALL_COLOR, base="wall_back")
    k.wall((-7.0, -8.85, F), (-6.85, 8.85, TOP1), openings=[emer_door, win_l1, win_l2],
           color=WALL_COLOR, base="wall_left")
    k.wall((13.85, -8.85, F), (14.0, 8.85, TOP1), openings=[win_r1, win_r2],
           color=WALL_COLOR, base="wall_right")

    # Red accent stripe between Floor 1 and Floor 2 (z 3.0 to 3.3)
    for lo, hi in [
        ((-7.02, 8.98, 3.0), (14.02, 9.02, 3.3)),
        ((-7.02, -9.02, 3.0), (14.02, -8.98, 3.3)),
        ((-7.02, -8.98, 3.0), (-6.98, 8.98, 3.3)),
        ((13.98, -8.98, 3.0), (14.02, 8.98, 3.3)),
    ]:
        k.box(lo, hi, STRIPE_COLOR)

    # Window frames and panes for ground floor windows
    for win, face, sign in [
        (win_f1, 9.0, 1), (win_f2, 9.0, 1), (win_f3, 9.0, 1),
        (win_b1, -9.0, -1), (win_b2, -9.0, -1), (win_b3, -9.0, -1),
        (win_r1, 14.0, 1), (win_r2, 14.0, 1),
        (win_l1, -7.0, -1), (win_l2, -7.0, -1),
    ]:
        k.frame(win, face, sign, color="metal", t=0.08, depth=0.06)
        (x0, y0, z0), (x1, y1, z1) = win
        # Horizontal and vertical mullions
        if abs(sign) == 1 and (face == 9.0 or face == -9.0):
            k.box(((x0 + x1) / 2 - 0.03, face + sign * 0.02, z0),
                  ((x0 + x1) / 2 + 0.03, face + sign * 0.05, z1), "metal")
            k.box((x0, face + sign * 0.02, (z0 + z1) / 2 - 0.03),
                  (x1, face + sign * 0.05, (z0 + z1) / 2 + 0.03), "metal")
            k.box((x0, face + sign * 0.01, z0), (x1, face + sign * 0.02, z1), "glass_tint")
        else:
            k.box((face + sign * 0.02, (y0 + y1) / 2 - 0.03, z0),
                  (face + sign * 0.05, (y0 + y1) / 2 + 0.03, z1), "metal")
            k.box((face + sign * 0.02, y0, (z0 + z1) / 2 - 0.03),
                  (face + sign * 0.05, y1, (z0 + z1) / 2 + 0.03), "metal")
            k.box((face + sign * 0.01, y0, z0), (face + sign * 0.02, y1, z1), "glass_tint")

    # Main entrance canopy awning over front doors
    k.box((1.5, 9.0, 3.2), (6.0, 10.2, 3.35), WALL_COLOR)
    k.collider((1.5, 9.0, 3.2), (6.0, 10.2, 3.35), "entrance_canopy")
    k.box((1.48, 10.18, 3.19), (6.02, 10.22, 3.36), STRIPE_COLOR)
    # Two support posts
    for hx in (2.0, 5.5):
        k.solid((hx - 0.08, 9.95, F), (hx + 0.08, 10.11, 3.2), "metal", base="canopy_post")
    # Low entrance ramp in front of main doors
    k.solid((1.5, 9.02, 0.01), (6.0, 10.0, F), "rock_mid", base="entrance_ramp")

    # Front wide entrance double doors
    k.frame(main_door, 9.0, 1, color="metal", t=0.1)
    # Sliding glass door panels (left closed, right open by 1.3 m)
    k.box((2.55, 8.95, F), (3.75, 9.02, 2.45), "metal")
    k.box((2.65, 8.94, F + 0.2), (3.65, 9.03, 2.35), "glass_tint")
    # Right panel slid back
    k.box((4.8, 8.92, F), (5.0, 9.02, 2.45), "metal")

    # Emergency entrance doors
    k.frame(emer_door, -7.0, -1, color="metal", t=0.1)
    k.box((-7.02, -0.15, F), (-6.95, 0.8, 2.45), "metal")
    k.box((-7.03, -0.05, F + 0.2), (-6.94, 0.7, 2.35), "glass_tint")
    k.box((-7.02, 1.6, F), (-6.95, 1.8, 2.45), "metal")

    # ---- BIG RED CROSS SIGN (BOXES) on front facade ----
    # White background panel
    k.box((2.2, 8.99, 6.8), (5.3, 9.06, 8.8), WALL_COLOR, roof=True)
    # Vertical red cross bar
    k.box((3.3, 9.07, 7.0), (4.2, 9.14, 8.6), STRIPE_COLOR, roof=True)
    # Horizontal red cross bar
    k.box((2.5, 9.06, 7.55), (5.0, 9.15, 8.05), STRIPE_COLOR, roof=True)


def build_interior(k):
    """Ground floor interior: reception, central corridor, 2 rooms with beds, pharmacy."""
    # Central corridor runs along X (x -7.0 to 7.5, y -1.4 to 1.4)
    # North corridor wall (y = 1.4, x -7.0 to 1.5)
    triage_door = ((-3.5, 1.25, 0.0), (-1.8, 1.55, kit.DOOR_H))
    reception_opening = ((1.2, 1.25, 0.0), (5.5, 1.55, TOP1))
    k.wall((-6.85, 1.35, F), (1.5, 1.5, TOP1), openings=[triage_door],
           color=WALL_COLOR, base="wall_corr_north")

    # South corridor wall (y = -1.4, x -7.0 to 13.85)
    ward_door = ((-3.5, -1.55, 0.0), (-1.8, -1.25, kit.DOOR_H))
    pharmacy_door = ((4.5, -1.55, 0.0), (6.2, -1.25, kit.DOOR_H))
    k.wall((-6.85, -1.5, F), (13.85, -1.35, TOP1), openings=[ward_door, pharmacy_door],
           color=WALL_COLOR, base="wall_corr_south")

    # Wall dividing Triage room from Reception (x = 1.5, y = 1.5 to 8.85)
    k.wall((1.35, 1.5, F), (1.5, 8.85, TOP1), color=WALL_COLOR, base="wall_triage_reception")

    # Wall dividing Ward from Pharmacy (x = 1.5, y = -8.85 to -1.5)
    k.wall((1.35, -8.85, F), (1.5, -1.5, TOP1), color=WALL_COLOR, base="wall_ward_pharmacy")

    # Wall enclosing closed stairwell / east wing (x = 7.5, y = 1.4 to 8.85)
    stairs_closed_door = ((7.35, 4.5, 0.0), (7.65, 6.0, kit.DOOR_H))
    k.wall((7.5, 1.4, F), (7.65, 8.85, TOP1), openings=[stairs_closed_door],
           color=WALL_COLOR, base="wall_east_stairs")

    # Blue guide navigation line on corridor floor
    k.box((-6.8, -0.1, F + 0.006), (7.5, 0.1, F + 0.016), "water_blue")

    # ---- RECEPTION & WAITING AREA (x 1.5 to 7.5, y 1.4 to 8.85) ----
    # L-shaped curved reception counter
    k.solid((4.5, 4.0, F), (7.2, 4.6, 1.1), WALL_COLOR, base="rec_counter_1")
    k.box((4.45, 3.95, 1.1), (7.25, 4.65, 1.16), "wood_crate")
    k.solid((4.5, 4.6, F), (5.1, 6.5, 1.1), WALL_COLOR, base="rec_counter_2")
    k.box((4.45, 4.65, 1.1), (5.15, 6.55, 1.16), "wood_crate")

    # Office computer monitor and chair behind counter
    k.box((5.8, 4.15, 1.16), (6.3, 4.4, 1.55), "gun_black")
    k.box((5.7, 4.41, 1.16), (6.4, 4.55, 1.18), "rock_light")
    k.solid((6.0, 5.2, F), (6.5, 5.7, 0.9), "shirt_blue", base="rec_chair")

    # Waiting room chairs (row of blue seats)
    k.solid((2.2, 2.5, F), (4.0, 3.2, 0.45), "shirt_blue", base="wait_chairs_1")
    k.box((2.2, 2.5, 0.45), (4.0, 2.65, 0.85), "shirt_blue")
    k.solid((2.2, 4.0, F), (4.0, 4.7, 0.45), "shirt_blue", base="wait_chairs_2")
    k.box((2.2, 4.0, 0.45), (4.0, 4.15, 0.85), "shirt_blue")

    # Water cooler in reception
    k.solid((2.0, 7.8, F), (2.5, 8.3, 0.9), WALL_COLOR, base="rec_water_cooler")
    k.box((2.1, 7.9, 0.9), (2.4, 8.2, 1.3), "water_blue")

    # ---- ROOM 1: TRIAGE / EMERGENCY ROOM (x -6.85 to 1.35, y 1.4 to 8.85) ----
    # Bed 1 (north wall)
    k.solid((-6.2, 6.2, F), (-4.0, 7.6, 0.45), "metal", base="triage_bed1_base")
    k.box((-6.15, 6.25, 0.45), (-4.05, 7.55, 0.62), WALL_COLOR)   # mattress
    k.box((-4.6, 6.35, 0.62), (-4.1, 7.45, 0.72), WALL_COLOR)      # pillow
    k.box((-6.1, 6.3, 0.62), (-4.7, 7.5, 0.64), "gown_blue")       # blanket
    # Bed wheels & headboard
    k.box((-4.0, 6.2, F), (-3.92, 7.6, 0.9), "metal")

    # Bed 2 (south wall of room)
    k.solid((-6.2, 2.5, F), (-4.0, 3.9, 0.45), "metal", base="triage_bed2_base")
    k.box((-6.15, 2.55, 0.45), (-4.05, 3.85, 0.62), WALL_COLOR)
    k.box((-4.6, 2.65, 0.62), (-4.1, 3.75, 0.72), WALL_COLOR)
    k.box((-6.1, 2.6, 0.62), (-4.7, 3.8, 0.64), "gown_blue")
    k.box((-4.0, 2.5, F), (-3.92, 3.9, 0.9), "metal")

    # Medical monitor carts beside beds
    k.solid((-3.5, 7.0, F), (-2.8, 7.6, 0.95), "metal", base="monitor_cart_1")
    k.box((-3.4, 7.1, 0.95), (-2.9, 7.5, 1.35), "gun_black")
    k.solid((-3.5, 2.5, F), (-2.8, 3.1, 0.95), "metal", base="monitor_cart_2")
    k.box((-3.4, 2.6, 0.95), (-2.9, 3.0, 1.35), "gun_black")

    # IV drip stand
    k.box((-3.8, 6.3, F), (-3.72, 6.38, 2.1), "metal")
    k.box((-3.9, 6.25, 2.05), (-3.62, 6.43, 2.1), "metal")
    k.box((-3.85, 6.3, 1.7), (-3.75, 6.38, 1.95), "water_blue")

    # Supply cabinet on east wall of triage
    k.solid((0.8, 5.0, F), (1.35, 7.5, 2.0), "metal", base="triage_cabinet")

    # ---- ROOM 2: PATIENT WARD (x -6.85 to 1.35, y -8.85 to -1.4) ----
    # Bed 3
    k.solid((-6.2, -4.6, F), (-4.0, -3.2, 0.45), "metal", base="ward_bed3_base")
    k.box((-6.15, -4.55, 0.45), (-4.05, -3.25, 0.62), WALL_COLOR)
    k.box((-4.6, -4.45, 0.62), (-4.1, -3.35, 0.72), WALL_COLOR)
    k.box((-6.1, -4.5, 0.62), (-4.7, -3.3, 0.64), "gown_blue")
    k.box((-4.0, -4.6, F), (-3.92, -3.2, 0.9), "metal")

    # Bed 4
    k.solid((-6.2, -8.2, F), (-4.0, -6.8, 0.45), "metal", base="ward_bed4_base")
    k.box((-6.15, -8.15, 0.45), (-4.05, -6.85, 0.62), WALL_COLOR)
    k.box((-4.6, -8.05, 0.62), (-4.1, -6.95, 0.72), WALL_COLOR)
    k.box((-6.1, -8.1, 0.62), (-4.7, -6.9, 0.64), "gown_blue")
    k.box((-4.0, -8.2, F), (-3.92, -6.8, 0.9), "metal")

    # Privacy curtain partition screens
    k.box((-3.8, -5.2, F), (-3.74, -2.6, 2.0), WALL_COLOR)
    k.box((-3.8, -5.2, 2.0), (-3.74, -2.6, 2.06), "metal")
    k.box((-3.8, -8.8, F), (-3.74, -6.2, 2.0), WALL_COLOR)
    k.box((-3.8, -8.8, 2.0), (-3.74, -6.2, 2.06), "metal")

    # Oxygen cylinder on cart beside Bed 3
    k.solid((-3.4, -4.2, F), (-3.0, -3.8, 0.9), STRIPE_COLOR, base="oxygen_tank")
    k.box((-3.3, -4.1, 0.9), (-3.1, -3.9, 1.05), "metal")

    # Nightstand tables
    k.solid((-6.0, -2.2, F), (-5.2, -1.5, 0.75), WALL_COLOR, base="ward_table")

    # ---- CORRIDOR GURNEY (CRASH CART) ----
    k.solid((-0.8, -0.4, F), (1.2, 0.4, 0.5), "metal", base="corridor_gurney")
    k.box((-0.75, -0.35, 0.5), (1.15, 0.35, 0.65), WALL_COLOR)
    k.box((0.8, -0.3, 0.65), (1.1, 0.3, 0.75), WALL_COLOR)

    # ---- PHARMACY / MEDICAL STORAGE (x 1.5 to 13.85, y -8.85 to -1.4) ----
    # 3 Large metal shelving units
    for sy in (-7.8, -5.2, -2.6):
        k.solid((8.0, sy - 0.45, F), (13.2, sy + 0.45, 2.2), "metal", base="pharm_shelf")
        # Medicine boxes and supplies on shelves
        for sx in (8.5, 9.8, 11.2, 12.4):
            # First aid box (white with red cross)
            k.box((sx, sy - 0.3, 0.8), (sx + 0.5, sy + 0.3, 1.15), WALL_COLOR)
            k.box((sx + 0.2, sy + 0.3, 0.9), (sx + 0.3, sy + 0.32, 1.05), STRIPE_COLOR)
            k.box((sx + 0.12, sy + 0.3, 0.95), (sx + 0.38, sy + 0.32, 1.0), STRIPE_COLOR)
            # Top tier bottles/cartons
            k.box((sx + 0.1, sy - 0.2, 1.5), (sx + 0.45, sy + 0.2, 1.85), "wood_crate")

    # Dispensing counter in pharmacy
    k.solid((2.5, -4.5, F), (4.2, -2.2, 1.0), WALL_COLOR, base="pharm_counter")
    k.box((2.45, -4.55, 1.0), (4.25, -2.15, 1.06), "wood_crate")

    # ---- CLOSED UPPER FLOOR STAIRS & ELEVATORS (x 7.5 to 13.85, y 1.4 to 8.85) ----
    # Elevator doors (metal sliding doors)
    k.box((8.0, 8.78, F), (10.5, 8.86, 2.5), "metal")
    k.box((9.2, 8.76, F), (9.3, 8.88, 2.5), "rock_dark")  # center split line
    k.box((9.0, 8.87, 2.6), (9.5, 8.9, 2.75), STRIPE_COLOR) # floor indicator

    # Boarded up stairwell doorway
    for bz in (0.3, 0.75, 1.2, 1.65, 2.1):
        k.solid((7.4, 4.3, bz), (7.6, 6.2, bz + 0.3), "wood_crate", base="stairs_plank")
    # Warning sign on planks
    k.box((7.38, 4.8, 1.0), (7.42, 5.7, 1.45), STRIPE_COLOR)
    k.box((7.36, 4.9, 1.05), (7.4, 5.6, 1.4), WALL_COLOR)
    # Concrete rubble block
    k.solid((7.8, 4.5, F), (8.8, 5.8, 0.8), "rock_mid", base="stairs_rubble")


def build_upper_and_roof(k):
    """Floors 2 and 3 (closed) and roof with elevator penthouse and AC chillers.
    All visual parts use roof=True so top preview displays the ground floor."""
    # Floor 1 ceiling / Floor 2 slab
    k.box((-7.0, -9.0, TOP1), (14.0, 9.0, TOP1 + 0.2), "rock_dark", roof=True)
    k.collider((-7.0, -9.0, TOP1), (14.0, 9.0, TOP1 + 0.2), "floor2_slab")

    # Floor 2 exterior walls
    k.box((-7.0, 8.85, TOP1 + 0.2), (14.0, 9.0, TOP2), WALL_COLOR, roof=True)
    k.box((-7.0, -9.0, TOP1 + 0.2), (14.0, -8.85, TOP2), WALL_COLOR, roof=True)
    k.box((-7.0, -8.85, TOP1 + 0.2), (-6.85, 8.85, TOP2), WALL_COLOR, roof=True)
    k.box((13.85, -8.85, TOP1 + 0.2), (14.0, 8.85, TOP2), WALL_COLOR, roof=True)

    # Floor 2 red stripe
    for lo, hi in [
        ((-7.02, 8.98, 6.0), (14.02, 9.02, 6.3)),
        ((-7.02, -9.02, 6.0), (14.02, -8.98, 6.3)),
        ((-7.02, -8.98, 6.0), (-6.98, 8.98, 6.3)),
        ((13.98, -8.98, 6.0), (14.02, 8.98, 6.3)),
    ]:
        k.box(lo, hi, STRIPE_COLOR, roof=True)

    # Floor 2 ceiling / Floor 3 slab
    k.box((-7.0, -9.0, TOP2), (14.0, 9.0, TOP2 + 0.2), "rock_dark", roof=True)
    k.collider((-7.0, -9.0, TOP2), (14.0, 9.0, TOP2 + 0.2), "floor3_slab")

    # Floor 3 exterior walls
    k.box((-7.0, 8.85, TOP2 + 0.2), (14.0, 9.0, TOP3), WALL_COLOR, roof=True)
    k.box((-7.0, -9.0, TOP2 + 0.2), (14.0, -8.85, TOP3), WALL_COLOR, roof=True)
    k.box((-7.0, -8.85, TOP2 + 0.2), (-6.85, 8.85, TOP3), WALL_COLOR, roof=True)
    k.box((13.85, -8.85, TOP2 + 0.2), (14.0, 8.85, TOP3), WALL_COLOR, roof=True)

    # Upper floor window panes (Floors 2 & 3)
    for z_base in (4.2, 7.2):
        # Front windows
        for wx0, wx1 in ((-5.5, -2.0), (-0.5, 2.0), (5.5, 8.0), (9.5, 12.5)):
            k.box((wx0, 8.98, z_base), (wx1, 9.04, z_base + 1.2), "glass_tint", roof=True)
            k.box((wx0 - 0.05, 8.96, z_base - 0.05), (wx1 + 0.05, 9.06, z_base + 1.25), "metal", roof=True)
        # Back windows
        for wx0, wx1 in ((-5.5, -2.0), (0.0, 3.5), (6.0, 9.5), (10.5, 13.0)):
            k.box((wx0, -9.04, z_base), (wx1, -8.98, z_base + 1.2), "glass_tint", roof=True)
        # Side windows
        for wy0, wy1 in ((-7.0, -4.0), (-2.0, 1.0), (3.0, 6.0)):
            k.box((13.98, wy0, z_base), (14.04, wy1, z_base + 1.2), "glass_tint", roof=True)
            k.box((-7.04, wy0, z_base), (-6.98, wy1, z_base + 1.2), "glass_tint", roof=True)

    # Roof slab
    k.box((-7.1, -9.1, TOP3), (14.1, 9.1, TOP3 + 0.2), "rock_dark", roof=True)
    k.collider((-7.1, -9.1, TOP3), (14.1, 9.1, TOP3 + 0.2), "roof_slab")

    # Parapet rim around roof
    k.box((-7.15, 8.95, TOP3 + 0.2), (14.15, 9.15, ROOF_TOP), WALL_COLOR, roof=True)
    k.box((-7.15, -9.15, TOP3 + 0.2), (14.15, -8.95, ROOF_TOP), WALL_COLOR, roof=True)
    k.box((-7.15, -8.95, TOP3 + 0.2), (-6.95, 8.95, ROOF_TOP), WALL_COLOR, roof=True)
    k.box((13.95, -8.95, TOP3 + 0.2), (14.15, 8.95, ROOF_TOP), WALL_COLOR, roof=True)

    # Parapet red fascia cap
    k.box((-7.2, 8.92, ROOF_TOP - 0.08), (14.2, 9.18, ROOF_TOP + 0.04), STRIPE_COLOR, roof=True)
    k.box((-7.2, -9.18, ROOF_TOP - 0.08), (14.2, -8.92, ROOF_TOP + 0.04), STRIPE_COLOR, roof=True)
    k.box((-7.2, -8.92, ROOF_TOP - 0.08), (-6.92, 8.92, ROOF_TOP + 0.04), STRIPE_COLOR, roof=True)
    k.box((13.92, -8.92, ROOF_TOP - 0.08), (14.2, 8.92, ROOF_TOP + 0.04), STRIPE_COLOR, roof=True)

    # Elevator penthouse / stair machinery room on roof
    k.box((7.5, -6.0, TOP3 + 0.2), (11.5, -2.0, TOP3 + 2.0), WALL_COLOR, roof=True)
    k.box((7.45, -6.05, TOP3 + 1.95), (11.55, -1.95, TOP3 + 2.1), STRIPE_COLOR, roof=True)

    # Rooftop HVAC chiller units
    k.box((-3.0, -4.0, TOP3 + 0.2), (1.0, -1.0, TOP3 + 1.4), "metal", roof=True)
    k.box((-2.8, -3.8, TOP3 + 1.4), (0.8, -1.2, TOP3 + 1.55), "rock_dark", roof=True)
    k.box((-4.5, 2.0, TOP3 + 0.2), (-1.5, 5.0, TOP3 + 1.3), "metal", roof=True)

    # Helipad marking on the flat roof
    k.box((2.0, -3.0, TOP3 + 0.21), (6.0, 1.0, TOP3 + 0.23), "rock_mid", roof=True)
    # White 'H' on helipad
    k.box((2.8, -2.5, TOP3 + 0.23), (3.2, 0.5, TOP3 + 0.25), "white", roof=True)
    k.box((4.8, -2.5, TOP3 + 0.23), (5.2, 0.5, TOP3 + 0.25), "white", roof=True)
    k.box((3.2, -1.2, TOP3 + 0.23), (4.8, -0.8, TOP3 + 0.25), "white", roof=True)


def build(model):
    k = kit.Kit(model)
    build_ambulance_bay(k)
    build_exterior(k)
    build_interior(k)
    build_upper_and_roof(k)

    # Empties
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(30.0, 22.0, 1.0))
    # Doors
    k.empty("door_1", (3.75, 9.0, F))     # Main front entrance
    k.empty("door_2", (-7.0, 0.5, F))     # Ambulance emergency entrance

    # 5 Loot spots
    k.empty("loot_1", (5.5, 4.8, 1.18))    # Reception desk
    k.empty("loot_2", (-5.1, 6.9, 0.65))   # Triage room Bed 1
    k.empty("loot_3", (-5.1, -3.9, 0.65))  # Patient ward Bed 3
    k.empty("loot_4", (10.0, -5.2, 1.18))  # Pharmacy medicine shelf
    k.empty("loot_5", (0.2, 0.0, 0.68))    # Corridor crash cart / gurney

    # 5 Zombie spawns
    k.empty("spawn_1", (3.5, 2.8, F))      # Reception waiting area
    k.empty("spawn_2", (-1.0, 0.0, F))     # Central corridor
    k.empty("spawn_3", (-3.5, 5.0, F))     # Triage room
    k.empty("spawn_4", (-3.5, -5.0, F))    # Patient ward
    k.empty("spawn_5", (-10.5, 0.0, 0.04)) # Outside under ambulance bay


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
