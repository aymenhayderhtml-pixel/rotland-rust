"""apartment: 3-floor brick apartment block with flat roof, water tank, and rusty fire escape.

Lot 20 x 14 m, building footprint 18 x 12 m (1 m inside lot on every side).
Outer bounds: X in [-9.0, 9.0], Y in [-6.0, 6.0].
Front door on +Y side (1.2 m wide), ground floor entrance hall open with mailboxes and a stair
that ends at a collapsed/boarded landing. Upper floors closed.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "apartment", "buildings", 6000
F = kit.FLOOR_TOP
WALL_TOP = 8.6  # top of 3rd floor wall
PARAPET_TOP = 9.2  # top of roof parapet
COPING_TOP = 9.28  # top of parapet stone coping


def build(model):
    k = kit.Kit(model)

    # 1. Base floor slab (raised F=0.1 m above ground)
    # Footprint: x in [-9.0, 8.2], y in [-6.0, 6.0]. Fire escape on +X extends to 9.0.
    k.solid((-9.0, -6.0, 0.0), (8.2, 6.0, F), "rock_mid", base="floor")

    # 2. Main exterior walls & openings
    # Front door (1.2 m wide, centered at x=0.0: x in [-0.6, 0.6])
    front_door = ((-0.6, 5.82, F), (0.6, 6.02, kit.DOOR_H))

    # Ground floor front wall with door opening (z F to 3.0)
    k.wall((-9.0, 5.85, F), (8.2, 6.0, 3.0), openings=[front_door], color="red_paint", base="wall_front_gf")

    # Upper floors front wall (z 3.0 to 8.6) - visual marked roof=True so top preview shows interior
    k.box((-9.0, 5.85, 3.0), (8.2, 6.0, WALL_TOP), "red_paint", roof=True)
    k.collider((-9.0, 5.85, 3.0), (8.2, 6.0, WALL_TOP), "wall_front_upper")

    # Back wall (y -6.0 to -5.85, z F to 8.6)
    k.solid((-9.0, -6.0, F), (8.2, -5.85, 3.0), "red_paint", base="wall_back_gf")
    k.box((-9.0, -6.0, 3.0), (8.2, -5.85, WALL_TOP), "red_paint", roof=True)
    k.collider((-9.0, -6.0, 3.0), (8.2, -5.85, WALL_TOP), "wall_back_upper")

    # West wall (x -9.0 to -8.85, y -5.85 to 5.85, z F to 8.6)
    k.solid((-9.0, -5.85, F), (-8.85, 5.85, 3.0), "red_paint", base="wall_west_gf")
    k.box((-9.0, -5.85, 3.0), (-8.85, 5.85, WALL_TOP), "red_paint", roof=True)
    k.collider((-9.0, -5.85, 3.0), (-8.85, 5.85, WALL_TOP), "wall_west_upper")

    # East wall (x 8.05 to 8.2, y -5.85 to 5.85, z F to 8.6)
    k.solid((8.05, -5.85, F), (8.2, 5.85, 3.0), "red_paint", base="wall_east_gf")
    k.box((8.05, -5.85, 3.0), (8.2, 5.85, WALL_TOP), "red_paint", roof=True)
    k.collider((8.05, -5.85, 3.0), (8.2, 5.85, WALL_TOP), "wall_east_upper")

    # Stone plinth / foundation trim along base (z 0.02 to 0.55, proud, butted, split for door)
    k.box((-8.96, 5.98, 0.02), (-0.6, 6.03, 0.55), "rock_mid")
    k.box((0.6, 5.98, 0.02), (8.16, 6.03, 0.55), "rock_mid")
    k.box((-8.96, -6.03, 0.02), (8.16, -5.98, 0.55), "rock_mid")
    k.box((-9.03, -6.03, 0.02), (-8.98, 6.03, 0.55), "rock_mid")
    k.box((8.18, -6.03, 0.02), (8.23, 6.03, 0.55), "rock_mid")

    # Architectural stone belt courses between floors (proud, butted at corners)
    for zb, is_rf in ((2.95, False), (5.75, True)):
        k.box((-8.96, 5.98, zb), (8.16, 6.025, zb + 0.15), "rock_light", roof=is_rf)
        k.box((-8.96, -6.025, zb), (8.16, -5.98, zb + 0.15), "rock_light", roof=is_rf)
        k.box((-9.025, -6.025, zb), (-8.98, 6.025, zb + 0.15), "rock_light", roof=is_rf)
        k.box((8.18, -6.025, zb), (8.225, 6.025, zb + 0.15), "rock_light", roof=is_rf)

    # Front door frame and stone portico trim
    k.frame(front_door, 6.0, 1, color="rock_light", t=0.08, depth=0.04, sill=False)
    k.box((-0.9, 5.92, 2.25), (0.9, 6.04, 2.38), "rock_light")  # portico lintel proud of wall

    # 3. Windows across all 3 floors
    window_floors = [(1, 1.1, 2.5), (2, 3.8, 5.2), (3, 6.5, 7.9)]

    # Front windows
    for flr, z0, z1 in window_floors:
        x_cols = (-6.5, -3.8, 3.5, 6.0) if flr == 1 else (-6.5, -3.8, 0.0, 3.5, 6.0)
        is_upper = flr > 1
        for xc in x_cols:
            w_box = ((xc - 0.5, 5.94, z0), (xc + 0.5, 6.0, z1))
            k.box((xc - 0.45, 5.96, z0 + 0.05), (xc + 0.45, 6.02, z1 - 0.05), "tire_black", roof=is_upper)
            # Stone lintel and sill
            k.box((xc - 0.55, 5.95, z1), (xc + 0.55, 6.02, z1 + 0.12), "rock_light", roof=is_upper)
            k.box((xc - 0.55, 5.95, z0 - 0.1), (xc + 0.55, 6.02, z0), "rock_light", roof=is_upper)
            # Window frame strips
            k.box((xc - 0.5, 5.97, z0), (xc - 0.44, 6.03, z1), "rock_light", roof=is_upper)
            k.box((xc + 0.44, 5.97, z0), (xc + 0.5, 6.03, z1), "rock_light", roof=is_upper)
            # Boarded or broken details
            if (flr == 1 and xc in (-6.5, 3.5)) or (flr == 2 and xc in (-3.8, 6.0)):
                k.boards(w_box, 6.0, 1, color="bark_light", rows=(0.25, 0.5, 0.75))
            elif flr == 3 and xc in (-6.5, 0.0):
                k.boards(w_box, 6.0, 1, color="bark_dark", rows=(0.3, 0.7))
            elif flr == 2 and xc == 0.0:
                k.box((xc - 0.35, 5.97, z0 + 0.4), (xc + 0.3, 6.03, z0 + 0.52), "bark_light", roof=is_upper)

    # Back windows
    for flr, z0, z1 in window_floors:
        is_upper = flr > 1
        for xc in (-6.5, -3.8, 0.0, 3.5, 6.0):
            w_box = ((xc - 0.5, -6.0, z0), (xc + 0.5, -5.94, z1))
            k.box((xc - 0.45, -6.02, z0 + 0.05), (xc + 0.45, -5.96, z1 - 0.05), "tire_black", roof=is_upper)
            k.box((xc - 0.55, -6.02, z1), (xc + 0.55, -5.95, z1 + 0.12), "rock_light", roof=is_upper)
            k.box((xc - 0.55, -6.02, z0 - 0.1), (xc + 0.55, -5.95, z0), "rock_light", roof=is_upper)
            if (flr == 1 and xc in (0.0, -6.5)) or (flr == 2 and xc == 3.5):
                k.boards(w_box, -6.0, -1, color="bark_light", rows=(0.3, 0.7))

    # West side windows (x = -9.0)
    for flr, z0, z1 in window_floors:
        is_upper = flr > 1
        for yc in (-3.5, 0.0, 3.5):
            w_box = ((-9.0, yc - 0.5, z0), (-8.94, yc + 0.5, z1))
            k.box((-9.02, yc - 0.45, z0 + 0.05), (-8.96, yc + 0.45, z1 - 0.05), "tire_black", roof=is_upper)
            k.box((-9.02, yc - 0.55, z1), (-8.95, yc + 0.55, z1 + 0.12), "rock_light", roof=is_upper)
            k.box((-9.02, yc - 0.55, z0 - 0.1), (-8.95, yc + 0.55, z0), "rock_light", roof=is_upper)
            if flr == 1 and yc == 0.0:
                k.boards(w_box, -9.0, -1, color="bark_light")

    # East side windows (fire escape side, x = 8.2)
    for flr, z0, z1 in window_floors:
        is_upper = flr > 1
        for yc in (-3.5, -1.0, 1.0, 3.5):
            w_box = ((8.14, yc - 0.5, z0), (8.2, yc + 0.5, z1))
            k.box((8.16, yc - 0.45, z0 + 0.05), (8.22, yc + 0.45, z1 - 0.05), "tire_black", roof=is_upper)
            k.box((8.15, yc - 0.55, z1), (8.22, yc + 0.55, z1 + 0.12), "rock_light", roof=is_upper)
            k.box((8.15, yc - 0.55, z0 - 0.1), (8.22, yc + 0.55, z0), "rock_light", roof=is_upper)
            if flr == 1 and yc == -1.0:
                k.boards(w_box, 8.2, 1, color="bark_light", rows=(0.2, 0.5, 0.8))

    # 4. Rusty Fire Escape on East side (x from 8.2 to 9.0, y in [-2.2, 2.2])
    # Floor 2 platform at z=3.0, Floor 3 platform at z=5.8
    for z_plat in (3.0, 5.8):
        # Platform grate
        k.solid((8.2, -2.2, z_plat), (9.0, 2.2, z_plat + 0.06), "rust", base="fe_plat")
        # Support brackets underneath into wall
        for y_brk in (-2.0, 0.0, 2.0):
            k.box((8.2, y_brk - 0.04, z_plat - 0.35), (9.0, y_brk + 0.04, z_plat), "rust")
        # Railing along outer edge (x = 9.0)
        k.box((8.95, -2.2, z_plat + 0.85), (9.0, 2.2, z_plat + 0.92), "rust")  # top rail
        k.box((8.96, -2.2, z_plat + 0.45), (9.0, 2.2, z_plat + 0.50), "rust")  # mid rail
        for y_post in (-2.2, -1.1, 0.0, 1.1, 2.2):
            k.box((8.95, y_post - 0.03, z_plat + 0.06), (9.0, y_post + 0.03, z_plat + 0.92), "rust")
        # Railing ends (north & south edges)
        for y_end in (-2.2, 2.2):
            k.box((8.2, y_end - 0.03, z_plat + 0.85), (9.0, y_end + 0.03, z_plat + 0.92), "rust")
            k.box((8.2, y_end - 0.02, z_plat + 0.45), (9.0, y_end + 0.02, z_plat + 0.50), "rust")

    # Angled stairs between Floor 2 and Floor 3 (y -1.8 to 1.8)
    stair_steps = 8
    for s in range(stair_steps):
        t = s / stair_steps
        zs = 3.06 + t * (5.8 - 3.06)
        ys = -1.8 + t * 3.6
        k.box((8.35, ys, zs), (8.75, ys + 0.35, zs + 0.05), "rust")
    # Diagonal handrail along stair
    k.box((8.75, -1.8, 3.9), (8.8, 1.8, 6.7), "rust")

    # Drop ladder hanging from Floor 2 down to z=1.4
    k.box((8.55, 1.8, 1.4), (8.6, 1.85, 3.0), "rust")
    k.box((8.95, 1.8, 1.4), (9.0, 1.85, 3.0), "rust")
    z_rung = 1.6
    while z_rung < 2.95:
        k.box((8.55, 1.8, z_rung), (9.0, 1.85, z_rung + 0.04), "rust")
        z_rung += 0.35

    # Roof ladder extending from Floor 3 platform over parapet to roof
    k.box((8.55, -1.8, 5.86), (8.6, -1.75, PARAPET_TOP + 0.4), "rust")
    k.box((8.95, -1.8, 5.86), (9.0, -1.75, PARAPET_TOP + 0.4), "rust")
    z_rung = 6.0
    while z_rung < PARAPET_TOP + 0.3:
        k.box((8.55, -1.8, z_rung), (9.0, -1.75, z_rung + 0.04), "rust")
        z_rung += 0.35
    # Gooseneck grab rails over parapet
    k.box((8.1, -1.8, PARAPET_TOP + 0.4), (8.95, -1.75, PARAPET_TOP + 0.45), "rust")

    # 5. Roof: slab, parapet walls, coping stone, and rooftop structures
    k.box((-9.0, -6.0, WALL_TOP), (8.2, 6.0, WALL_TOP + 0.15), "rock_dark", roof=True)
    k.collider((-9.0, -6.0, WALL_TOP), (8.2, 6.0, WALL_TOP + 0.15), "roof_slab")

    # Low parapet walls around roof (butted at corners)
    k.box((-8.85, 5.85, WALL_TOP + 0.15), (8.05, 6.0, PARAPET_TOP), "red_paint", roof=True)
    k.collider((-8.85, 5.85, WALL_TOP + 0.15), (8.05, 6.0, PARAPET_TOP), "parapet_front")

    k.box((-8.85, -6.0, WALL_TOP + 0.15), (8.05, -5.85, PARAPET_TOP), "red_paint", roof=True)
    k.collider((-8.85, -6.0, WALL_TOP + 0.15), (8.05, -5.85, PARAPET_TOP), "parapet_back")

    k.box((-9.0, -6.0, WALL_TOP + 0.15), (-8.85, 6.0, PARAPET_TOP), "red_paint", roof=True)
    k.collider((-9.0, -6.0, WALL_TOP + 0.15), (-8.85, 6.0, PARAPET_TOP), "parapet_west")

    k.box((8.05, -6.0, WALL_TOP + 0.15), (8.2, 6.0, PARAPET_TOP), "red_paint", roof=True)
    k.collider((8.05, -6.0, WALL_TOP + 0.15), (8.2, 6.0, PARAPET_TOP), "parapet_east")

    # Stone coping atop parapet (butted at corners)
    k.box((-8.83, 5.83, PARAPET_TOP), (8.03, 6.02, COPING_TOP), "rock_light", roof=True)
    k.box((-8.83, -6.02, PARAPET_TOP), (8.03, -5.83, COPING_TOP), "rock_light", roof=True)
    k.box((-9.02, -6.02, PARAPET_TOP), (-8.83, 6.02, COPING_TOP), "rock_light", roof=True)
    k.box((8.03, -6.02, PARAPET_TOP), (8.22, 6.02, COPING_TOP), "rock_light", roof=True)

    # Water tank on roof (at x=-4.0, y=-2.0)
    tx, ty = -4.0, -2.0
    tz_base = WALL_TOP + 0.15
    tz_tank = tz_base + 1.2
    tz_top = tz_tank + 2.0
    tz_roof = tz_top + 0.8
    for lx in (tx - 0.9, tx + 0.9):
        for ly in (ty - 0.9, ty + 0.9):
            k.box((lx - 0.08, ly - 0.08, tz_base), (lx + 0.08, ly + 0.08, tz_tank), "bark_dark", roof=True)
            k.collider((lx - 0.08, ly - 0.08, tz_base), (lx + 0.08, ly + 0.08, tz_tank), "tank_leg")
    # Cross braces
    k.box((tx - 0.9, ty - 0.04, tz_base + 0.5), (tx + 0.9, ty + 0.04, tz_base + 0.55), "bark_dark", roof=True)
    k.box((tx - 0.04, ty - 0.9, tz_base + 0.55), (tx + 0.04, ty + 0.9, tz_base + 0.60), "bark_dark", roof=True)
    # Cylindrical tank body
    tank_bm = common.make_frustum(1.15, 1.15, tz_tank, tz_top, segments=12, center=(tx, ty), color="bark_dark")
    model.add(tank_bm, roof=True)
    k.collider((tx - 1.2, ty - 1.2, tz_tank), (tx + 1.2, ty + 1.2, tz_roof), "water_tank")
    for zh in (tz_tank + 0.3, tz_tank + 1.0, tz_tank + 1.7):
        hoop = common.make_frustum(1.18, 1.18, zh, zh + 0.05, segments=12, center=(tx, ty), color="rock_dark")
        model.add(hoop, roof=True)
    cone_bm = common.make_frustum(1.25, 0.0, tz_top, tz_roof, segments=12, center=(tx, ty), color="rock_dark")
    model.add(cone_bm, roof=True)

    # Roof access bulkhead / stair doghouse (x: 0.5 to 3.0, y: 0.5 to 2.8)
    k.box((0.5, 0.5, tz_base), (3.0, 2.8, tz_base + 1.8), "red_paint", roof=True)
    k.collider((0.5, 0.5, tz_base), (3.0, 2.8, tz_base + 1.8), "bulkhead")
    k.box((0.45, 0.45, tz_base + 1.8), (3.05, 2.85, tz_base + 1.95), "rock_mid", roof=True)
    k.box((1.2, 2.81, tz_base), (2.3, 2.85, tz_base + 1.6), "metal", roof=True)

    # Chimney stack & vent pipe
    k.chimney(5.5, -3.0, tz_base, tz_base + 1.5, color="red_paint")
    k.box((-1.0 - 0.06, -4.0 - 0.06, tz_base), (-1.0 + 0.06, -4.0 + 0.06, tz_base + 1.1), "metal", roof=True)

    # 6. Ground Floor Interior: Entrance Hall (x -3.38 to 3.38, y 0.82 to 5.82)
    k.box((-3.38, 0.82, F + 0.008), (3.38, 5.82, F + 0.02), "rock_light")
    k.box((-3.5, 0.8, 3.0), (3.5, 5.85, 3.12), "white", roof=True)

    # Hallway partition walls separating lobby from closed units
    k.solid((3.4, 0.8, F), (3.55, 5.85, 3.0), "white", base="wall_hall_east")
    k.solid((-3.55, 0.8, F), (-3.4, 5.85, 3.0), "white", base="wall_hall_west")
    k.solid((-3.5, 0.65, F), (3.5, 0.8, 3.0), "white", base="wall_hall_back")

    # Closed apartment doors along lobby walls
    k.box((3.38, 3.5, F), (3.42, 4.5, 2.1), "bark_dark")
    k.box((3.36, 4.3, 0.95), (3.39, 4.38, 1.05), "brass")  # brass doorknob
    k.box((1.2, 0.8, F), (2.2, 0.84, 2.1), "bark_dark")
    k.box((1.35, 0.84, 0.95), (1.43, 0.87, 1.05), "brass")

    # Mailboxes on East wall (brass bank of boxes)
    k.solid((3.35, 1.5, 0.8), (3.42, 2.8, 1.8), "brass", base="mailboxes")
    for my in (1.6, 2.0, 2.4):
        for mz in (1.0, 1.3, 1.6):
            k.box((3.34, my, mz), (3.36, my + 0.3, mz + 0.2), "rock_dark")

    # Bulletin / notice board on West wall
    k.box((-3.42, 4.2, 1.1), (-3.38, 5.2, 1.8), "wood_crate")
    k.box((-3.38, 4.3, 1.2), (-3.37, 4.6, 1.45), "white")
    k.box((-3.38, 4.75, 1.3), (-3.37, 5.1, 1.65), "white")

    # Hallway radiator / bench
    k.solid((2.2, 5.4, F), (3.3, 5.75, 0.5), "metal", base="radiator")

    # Staircase that ends at a collapsed/boarded landing
    stair_w0, stair_w1 = -3.3, -1.8
    num_steps = 6
    for s in range(num_steps):
        step_z = F + (s + 1) * (1.3 / num_steps)
        step_y0 = 3.2 - (s + 1) * (1.8 / num_steps)
        step_y1 = 3.2 - s * (1.8 / num_steps)
        k.solid((stair_w0, step_y0, F), (stair_w1, step_y1, step_z), "rock_mid", base=f"step_{s}")

    landing_z = 1.4
    k.solid((stair_w0, 0.8, F), (stair_w1, 1.4, landing_z), "rock_mid", base="landing")

    # Boarded / collapsed landing barrier blocking access to upper floors
    k.solid((-3.35, 0.8, landing_z), (-1.75, 0.95, 2.9), "bark_dark", base="barricade_wall")
    k.box((-3.3, 0.95, landing_z + 0.2), (-1.8, 1.05, landing_z + 0.4), "wood_crate")
    k.box((-3.3, 0.95, landing_z + 0.6), (-1.8, 1.05, landing_z + 0.8), "bark_light")
    k.box((-3.3, 0.95, landing_z + 1.0), (-1.8, 1.05, landing_z + 1.2), "wood_crate")
    k.box((-3.1, 0.96, landing_z), (-2.0, 1.04, 2.8), "bark_dark")
    k.solid((-2.9, 1.3, landing_z), (-2.1, 1.8, landing_z + 0.35), "white", base="rubble_stair")

    # Stair handrail
    k.box((-1.82, 1.4, landing_z), (-1.76, 1.46, landing_z + 0.85), "metal")
    k.box((-1.82, 3.2, F), (-1.76, 3.26, F + 0.85), "metal")
    k.box((-1.81, 1.46, landing_z + 0.80), (-1.77, 3.2, F + 0.80), "metal")

    # 7. Empties / Markers
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(20.0, 14.0, 1.0))
    k.empty("door_1", (0.0, 5.925, F))

    # 3 loot empties
    k.empty("loot_1", (2.7, 5.5, 0.55))
    k.empty("loot_2", (-2.5, 2.5, F))
    k.empty("loot_3", (2.8, 1.2, F))

    # 3 spawn empties
    k.empty("spawn_1", (0.0, 3.5, F))
    k.empty("spawn_2", (-1.0, 2.8, F))
    k.empty("spawn_3", (-2.5, 6.5, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
