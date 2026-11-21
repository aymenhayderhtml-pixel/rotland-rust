"""warehouse: large industrial metal warehouse, 23 x 11 m footprint on a 26 x 16 m lot.

Corrugated metal walls (alternating gray bands), shallow gable roof, half-open roller door on the
front, small side door on the right, and a raised loading dock platform on the front right.
Inside: one big open room with pallet stacks, crates, and storage racks.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "warehouse", "buildings", 4000
WALL_TOP = 4.6
RIDGE_Z = 5.8
F = kit.FLOOR_TOP

# Body footprint: X in [-11.5, 11.5], Y in [-6.5, 4.5] (within lot 26 x 16 m, with >=1m margin)
# Loading dock extends from Y 4.5 to 6.8 on the front right (X in [0.5, 11.0])
BAND_W = 1.0


def _build_corrugated_wall(k, p0, p1, y_face=True, openings=(), base="wall"):
    """Builds a wall along X or Y with alternating vertical bands of metal and rock_dark."""
    x0, y0 = p0
    x1, y1 = p1
    colors = ("metal", "rock_dark")

    if y_face:
        # Wall spans along X, fixed Y
        y_min, y_max = sorted((y0, y1))
        x_min, x_max = sorted((x0, x1))
        cuts = [x_min]
        for o_lo, o_hi in openings:
            for vx in (o_lo[0], o_hi[0]):
                if x_min < vx < x_max:
                    cuts.append(vx)
        cuts.append(x_max)
        cuts = sorted(set(cuts))

        for i in range(len(cuts) - 1):
            cx0, cx1 = cuts[i], cuts[i + 1]
            cmid_x = (cx0 + cx1) / 2
            matching = [o for o in openings if o[0][0] <= cmid_x <= o[1][0]]
            if not matching:
                k.collider((cx0, y_min, F), (cx1, y_max, WALL_TOP), base=base)
            else:
                o_lo, o_hi = matching[0]
                if o_lo[2] > F:
                    k.collider((cx0, y_min, F), (cx1, y_max, o_lo[2]), base=base)
                if o_hi[2] < WALL_TOP:
                    k.collider((cx0, y_min, o_hi[2]), (cx1, y_max, WALL_TOP), base=base)

        x = x_min
        b_idx = 0
        while x < x_max - 0.01:
            nxt = min(x + BAND_W, x_max)
            col = colors[b_idx % 2]
            mid = (x + nxt) / 2
            matching = [o for o in openings if o[0][0] <= mid <= o[1][0]]
            if not matching:
                k.box((x, y_min, F), (nxt, y_max, WALL_TOP), col)
            else:
                o_lo, o_hi = matching[0]
                if o_lo[2] > F:
                    k.box((x, y_min, F), (nxt, y_max, o_lo[2]), col)
                if o_hi[2] < WALL_TOP:
                    k.box((x, y_min, o_hi[2]), (nxt, y_max, WALL_TOP), col)
            x = nxt
            b_idx += 1
    else:
        # Wall spans along Y, fixed X
        x_min, x_max = sorted((x0, x1))
        y_min, y_max = sorted((y0, y1))
        cuts = [y_min]
        for o_lo, o_hi in openings:
            for vy in (o_lo[1], o_hi[1]):
                if y_min < vy < y_max:
                    cuts.append(vy)
        cuts.append(y_max)
        cuts = sorted(set(cuts))

        for i in range(len(cuts) - 1):
            cy0, cy1 = cuts[i], cuts[i + 1]
            cmid_y = (cy0 + cy1) / 2
            matching = [o for o in openings if o[0][1] <= cmid_y <= o[1][1]]
            if not matching:
                k.collider((x_min, cy0, F), (x_max, cy1, WALL_TOP), base=base)
            else:
                o_lo, o_hi = matching[0]
                if o_lo[2] > F:
                    k.collider((x_min, cy0, F), (x_max, cy1, o_lo[2]), base=base)
                if o_hi[2] < WALL_TOP:
                    k.collider((x_min, cy0, o_hi[2]), (x_max, cy1, WALL_TOP), base=base)

        y = y_min
        b_idx = 0
        while y < y_max - 0.01:
            nxt = min(y + BAND_W, y_max)
            col = colors[b_idx % 2]
            mid = (y + nxt) / 2
            matching = [o for o in openings if o[0][1] <= mid <= o[1][1]]
            if not matching:
                k.box((x_min, y, F), (x_max, nxt, WALL_TOP), col)
            else:
                o_lo, o_hi = matching[0]
                if o_lo[2] > F:
                    k.box((x_min, y, F), (x_max, nxt, o_lo[2]), col)
                if o_hi[2] < WALL_TOP:
                    k.box((x_min, y, o_hi[2]), (x_max, nxt, WALL_TOP), col)
            y = nxt
            b_idx += 1


def _add_pallet(k, cx, cy, cz):
    """Adds a wooden pallet (1.2 x 1.0 m, 0.15 m tall)."""
    for dy in (-0.42, 0.0, 0.42):
        k.box((cx - 0.6, cy + dy - 0.04, cz), (cx + 0.6, cy + dy + 0.04, cz + 0.10), "bark_dark")
    for dx in (-0.50, -0.25, 0.0, 0.25, 0.50):
        k.box((cx + dx - 0.08, cy - 0.48, cz + 0.10), (cx + dx + 0.08, cy + 0.48, cz + 0.15), "wood_crate")
    k.collider((cx - 0.6, cy - 0.48, cz), (cx + 0.6, cy + 0.48, cz + 0.15), base="pallet")


def build(model):
    k = kit.Kit(model)

    # 1. Concrete warehouse floor
    k.solid((-11.5, -6.5, 0.0), (11.5, 4.5, F), "rock_mid", base="floor")

    # 2. Doors definitions
    # Big roller door on front (+Y): half open (Z from F to 2.2 m is clear to walk under)
    roller_door = ((-7.0, 4.3, F), (-3.0, 4.5, 2.2))
    # Small side door on right wall (+X) so it is visible in front-right preview
    side_door = ((11.3, -1.8, F), (11.5, -0.6, kit.DOOR_H))

    # 3. Corrugated Walls with alternating bands
    # Back wall
    _build_corrugated_wall(k, (-11.5, -6.5), (11.5, -6.3), y_face=True, base="wall_back")
    # Front wall (has roller door opening)
    _build_corrugated_wall(k, (-11.5, 4.3), (11.5, 4.5), y_face=True, openings=[roller_door], base="wall_front")
    # Left wall (solid)
    _build_corrugated_wall(k, (-11.5, -6.3), (-11.3, 4.3), y_face=False, base="wall_left")
    # Right wall (has side door opening)
    _build_corrugated_wall(k, (11.3, -6.3), (11.5, 4.3), y_face=False, openings=[side_door], base="wall_right")

    # Foundation trim (concrete base trim around the building exterior)
    k.box((-11.55, -6.55, 0.0), (11.55, -6.45, 0.3), "rock_dark")
    k.box((-11.55, 4.45, 0.0), (-7.0, 4.55, 0.3), "rock_dark")
    k.box((-3.0, 4.45, 0.0), (0.5, 4.55, 0.3), "rock_dark")
    k.box((-11.55, -6.55, 0.0), (-11.45, 4.45, 0.3), "rock_dark")
    k.box((11.45, -6.55, 0.0), (11.55, -1.8, 0.3), "rock_dark")
    k.box((11.45, -0.6, 0.0), (11.55, 4.45, 0.3), "rock_dark")

    # 4. Big roller door visual details (half open)
    # The rolled up metal door hanging down from 2.2 to 3.8 m
    k.box((-6.95, 4.36, 2.2), (-3.05, 4.44, 3.8), "metal")
    # Yellow hazard bar along the bottom of the half-open door
    k.box((-6.95, 4.35, 2.2), (-3.05, 4.45, 2.3), "brass")
    # Horizontal slats on the roller door
    z = 2.45
    while z < 3.8:
        k.box((-6.95, 4.45, z - 0.02), (-3.05, 4.47, z + 0.02), "rock_dark")
        z += 0.25
    # Rolled canister drum at top of door
    k.box((-7.1, 4.25, 3.8), (-2.9, 4.55, 4.3), "rock_dark")
    # Side guide rails
    k.box((-7.08, 4.34, F), (-6.95, 4.52, 3.8), "gun_black")
    k.box((-3.05, 4.34, F), (-2.92, 4.52, 3.8), "gun_black")

    # Side door frame and canopy on right wall
    k.frame(side_door, 11.5, 1, color="metal")
    k.box((11.4, -2.0, 2.3), (12.2, -0.4, 2.36), "metal")

    # 5. Loading Dock Platform on front right (X: 0.5 to 11.0, Y: 4.5 to 6.8, height 0.7m)
    dock_top = 0.7
    k.solid((0.5, 4.5, 0.0), (11.0, 6.8, dock_top), "rock_mid", base="dock")
    # Yellow safety caution stripe along the edge
    k.box((0.5, 6.74, dock_top - 0.02), (11.0, 6.82, dock_top + 0.02), "brass")
    k.box((10.94, 4.5, dock_top - 0.02), (11.02, 6.82, dock_top + 0.02), "brass")
    # Rubber dock bumpers on front edge
    for bx in (2.5, 5.5, 8.5):
        k.solid((bx - 0.25, 6.8, 0.2), (bx + 0.25, 6.95, dock_top - 0.1), "gun_black", base="bumper")

    # Steps up to dock at left side of dock (X: -0.3 to 0.5, Y: 5.2 to 6.8)
    k.solid((-0.3, 5.2, 0.0), (0.5, 6.0, 0.24), "rock_mid", base="dock_step1")
    k.solid((-0.3, 6.0, 0.0), (0.5, 6.8, 0.48), "rock_mid", base="dock_step2")

    # Closed loading bay doors on warehouse wall behind dock
    for bay_x0, bay_x1 in ((1.8, 5.2), (6.2, 9.6)):
        k.box((bay_x0, 4.48, dock_top), (bay_x1, 4.52, dock_top + 2.8), "rock_dark")
        k.box((bay_x0 - 0.06, 4.47, dock_top), (bay_x0, 4.53, dock_top + 2.86), "brass")
        k.box((bay_x1, 4.47, dock_top), (bay_x1 + 0.06, 4.53, dock_top + 2.86), "brass")
        k.box((bay_x0 - 0.06, 4.47, dock_top + 2.8), (bay_x1 + 0.06, 4.53, dock_top + 2.86), "brass")
        bz = dock_top + 0.3
        while bz < dock_top + 2.8:
            k.box((bay_x0, 4.52, bz - 0.02), (bay_x1, 4.54, bz + 0.02), "metal")
            bz += 0.35

    # Dock metal canopy awning
    k.box((0.3, 4.5, 3.8), (11.2, 7.0, 3.92), "metal", roof=True)
    k.collider((0.3, 4.5, 3.8), (11.2, 7.0, 3.92), "canopy_roof")
    for px in (0.8, 10.6):
        k.solid((px - 0.08, 6.6 - 0.08, dock_top), (px + 0.08, 6.6 + 0.08, 3.8), "gun_black", base="canopy_post")

    # 6. Shallow Gable Roof
    ridge = (0.0, -1.0, RIDGE_Z)
    k.roof(ridge, (0.0, -6.8, WALL_TOP), span=(1.0, 0.0, 0.0), width=23.6, color="metal", thick=0.14)
    k.roof(ridge, (0.0, 4.8, WALL_TOP), span=(1.0, 0.0, 0.0), width=23.6, color="metal", thick=0.14)

    # Ridge cap
    k.ridge_cap((0.0, -1.0, RIDGE_Z), 0, 23.8, color="rock_dark")

    # Eave fascia trim (gives thick industrial edge)
    k.box((-11.8, -6.88, WALL_TOP - 0.08), (11.8, -6.80, WALL_TOP + 0.08), "rock_dark", roof=True)
    k.box((-11.8, 4.80, WALL_TOP - 0.08), (11.8, 4.88, WALL_TOP + 0.08), "rock_dark", roof=True)

    # Gable triangular ends (at X = -11.5 and 11.35)
    k.gable([(-11.5, -6.5, WALL_TOP), (-11.5, 4.5, WALL_TOP), (-11.5, -1.0, RIDGE_Z)], (0.15, 0.0, 0.0), "metal", roof=True)
    k.gable([(11.35, -6.5, WALL_TOP), (11.35, 4.5, WALL_TOP), (11.35, -1.0, RIDGE_Z)], (0.15, 0.0, 0.0), "metal", roof=True)

    # Roof rotary turbine ventilators along the ridge
    for rx in (-7.0, 0.0, 7.0):
        k.box((rx - 0.35, -1.35, RIDGE_Z + 0.05), (rx + 0.35, -0.65, RIDGE_Z + 0.50), "metal", roof=True)
        k.box((rx - 0.40, -1.40, RIDGE_Z + 0.50), (rx + 0.40, -0.60, RIDGE_Z + 0.58), "rock_dark", roof=True)

    # 7. Interior: Steel portal frame trusses
    for ty in (-4.0, -1.0, 2.0):
        k.box((-11.3, ty - 0.1, F), (-11.1, ty + 0.1, WALL_TOP), "gun_black")
        k.box((11.1, ty - 0.1, F), (11.3, ty + 0.1, WALL_TOP), "gun_black")
        k.box((-11.2, ty - 0.08, WALL_TOP - 0.15), (11.2, ty + 0.08, WALL_TOP), "gun_black")
        k.box((-0.1, ty - 0.08, WALL_TOP), (0.1, ty + 0.08, RIDGE_Z - 0.15), "gun_black")

    # 8. Interior: Pallet stacks, crates, and storage racks
    # Stack 1: Back-left corner (X = -8.5, Y = -4.5): 3 pallets with crates
    _add_pallet(k, -8.5, -4.5, F)
    _add_pallet(k, -8.5, -4.5, F + 0.15)
    _add_pallet(k, -8.5, -4.5, F + 0.30)
    k.solid((-9.0, -4.9, F + 0.45), (-8.3, -4.2, F + 1.05), "wood_crate", base="crate_s1_a")
    k.solid((-8.2, -4.8, F + 0.45), (-7.6, -4.3, F + 0.95), "wood_crate", base="crate_s1_b")

    # Stack 1b: Second pallet stack beside Stack 1
    _add_pallet(k, -10.0, -4.5, F)
    _add_pallet(k, -10.0, -4.5, F + 0.15)
    k.solid((-10.5, -4.9, F + 0.30), (-9.6, -4.1, F + 0.90), "wood_crate", base="crate_s1_c")

    # Stack 2: Center-left (X = -4.5, Y = -2.5): 2 pallets with crate and barrel
    _add_pallet(k, -4.5, -2.5, F)
    _add_pallet(k, -4.5, -2.5, F + 0.15)
    k.solid((-4.9, -2.9, F + 0.30), (-4.2, -2.2, F + 0.95), "wood_crate", base="crate_s2_a")
    k.barrel(-4.0, -2.3, color="red_paint")

    # Hand pallet jack near Stack 2 (red frame, black wheels, handle)
    k.solid((-3.2, -2.7, F), (-2.8, -1.7, F + 0.15), "red_paint", base="jack_fork")
    k.box((-3.05, -2.85, F), (-2.95, -2.7, F + 0.85), "gun_black")  # steering handle

    # Stack 3: Large industrial storage rack along back wall (X: 1.5 to 7.5, Y: -6.2 to -5.2)
    rack_x0, rack_x1 = 1.5, 7.5
    rack_y0, rack_y1 = -6.0, -5.2
    for rx in (rack_x0, (rack_x0 + rack_x1) / 2, rack_x1):
        for ry in (rack_y0, rack_y1):
            k.solid((rx - 0.06, ry - 0.06, F), (rx + 0.06, ry + 0.06, 3.2), "gun_black", base="rack_post")
    for sz in (1.0, 2.0, 3.0):
        k.box((rack_x0, rack_y0 - 0.05, sz), (rack_x1, rack_y0 + 0.05, sz + 0.08), "brass")
        k.box((rack_x0, rack_y1 - 0.05, sz), (rack_x1, rack_y1 + 0.05, sz + 0.08), "brass")
        k.solid((rack_x0, rack_y0, sz), (rack_x1, rack_y1, sz + 0.08), "wood_crate", base="rack_shelf")
    # Crates and boxes on rack tiers
    k.solid((2.0, -5.9, 1.08), (2.8, -5.3, 1.8), "wood_crate", base="rack_c1")
    k.solid((3.2, -5.9, 1.08), (4.2, -5.3, 1.7), "wood_crate", base="rack_c2")
    k.solid((5.0, -5.9, 1.08), (5.8, -5.3, 1.9), "wood_crate", base="rack_c3")
    k.solid((2.2, -5.9, 2.08), (3.0, -5.3, 2.7), "wood_crate", base="rack_c4")
    k.solid((4.5, -5.9, 2.08), (5.5, -5.3, 2.8), "wood_crate", base="rack_c5")

    # Stack 4: Near loading bay inside (X = 7.5, Y = 1.5): pallets and crates
    _add_pallet(k, 7.5, 1.5, F)
    _add_pallet(k, 7.5, 1.5, F + 0.15)
    k.solid((7.1, 1.1, F + 0.30), (7.9, 1.9, F + 0.90), "wood_crate", base="crate_s4")
    k.barrel(6.3, 1.5, color="red_paint")
    k.barrel(6.3, 2.2, color="metal")

    # Shipping desk / workbench in corner near side door (X = 10.0, Y = -3.5)
    k.solid((9.5, -4.0, F), (11.0, -3.0, 0.85), "bark_dark", base="desk")
    k.box((9.7, -3.8, 0.85), (10.2, -3.3, 1.2), "gun_black")  # terminal / equipment

    # Stack 5: Exterior loading dock cargo (X = 7.0, Y = 5.6): pallet on the dock
    _add_pallet(k, 7.0, 5.6, dock_top)
    k.solid((6.6, 5.2, dock_top + 0.15), (7.4, 6.0, dock_top + 0.75), "wood_crate", base="dock_crate")

    # 9. Empties / Markers
    # Lot size
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(26.0, 16.0, 1.0))
    # Doors
    k.empty("door_1", (-5.0, 4.5, F))
    k.empty("door_2", (11.5, -1.2, F))
    # 4 Loot markers
    k.empty("loot_1", (5.2, -5.6, 2.15))  # on industrial shelf tier 2
    k.empty("loot_2", (-8.5, -4.5, F + 1.05))  # on top of pallet stack 1
    k.empty("loot_3", (7.0, 5.6, dock_top + 0.75))  # on cargo on the loading dock
    k.empty("loot_4", (-4.9, -2.9, F + 0.95))  # on crate in stack 2
    # 3 Zombie spawn markers
    k.empty("spawn_1", (-8.0, -1.0, F))  # inside, back-left
    k.empty("spawn_2", (0.0, -2.0, F))  # inside, center
    k.empty("spawn_3", (8.5, -2.0, F))  # inside, right


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
