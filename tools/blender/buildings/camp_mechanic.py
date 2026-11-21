"""camp_mechanic: Open garage 7x5 m with roof on posts, tool wall, tire stack, jack, oil barrels, npc_1 and car_spot empties.

Spacious vehicle repair garage with clear drive-in bay, oil-stained concrete floor,
corrugated roof supported on heavy perimeter posts, comprehensive mechanic tool wall,
roll-around tool chest, tire stack, hydraulic floor jack, 55-gal oil barrels, and creeper.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_mechanic", "buildings", 4000


def build(model):
    k = kit.Kit(model)

    # ================= GARAGE SLAB (7 x 5 m) =================
    k.solid((-3.5, -2.5, 0.0), (3.5, 2.5, 0.10), "rock_dark", base="garage_floor")

    # Yellow painted parking guide lines marking the car_spot service bay
    k.box((-0.60, -1.80, 0.102), (-0.48, 1.80, 0.106), "wheat_gold")
    k.box((1.68, -1.80, 0.102), (1.80, 1.80, 0.106), "wheat_gold")
    k.box((-0.60, -1.80, 0.102), (1.80, -1.68, 0.106), "wheat_gold")

    # Oil stain dark patches on floor
    k.box((0.0, -0.6, 0.101), (1.2, 0.6, 0.103), "gun_black")
    k.box((2.0, -2.1, 0.101), (3.0, -1.1, 0.103), "gun_black")

    # ================= STRUCTURAL POSTS & ROOF =================
    # Perimeter posts: 2 front posts at corners (x: -3.35, 3.20), wide open vehicle entrance!
    # Front posts (+Y: y = 2.25, height 3.3 m)
    for px in (-3.35, 3.20):
        k.solid((px, 2.25, 0.10), (px + 0.15, 2.40, 3.30), "metal", base="post_front")
    # Mid-depth posts (y = -0.10, height 3.05 m)
    for px in (-3.35, 3.20):
        k.solid((px, -0.18, 0.10), (px + 0.15, -0.02, 3.05), "metal", base="post_mid")
    # Rear posts (-Y: y = -2.40, height 2.80 m)
    for px in (-3.35, 0.0, 3.20):
        k.solid((px, -2.42, 0.10), (px + 0.15, -2.28, 2.80), "metal", base="post_rear")

    # Front header truss beam spanning across the wide entrance
    k.box((-3.45, 2.25, 3.25), (3.45, 2.40, 3.42), "metal")
    # Rear header beam
    k.box((-3.45, -2.42, 2.75), (3.45, -2.28, 2.90), "metal")
    # Side roof beams
    for px in (-3.35, 3.20):
        for s in range(5):
            f0 = s / 5
            f1 = (s + 1) / 5
            y0 = 2.35 - f0 * 4.7
            y1 = 2.35 - f1 * 4.7
            z0 = 3.35 - f0 * 0.55
            z1 = 3.35 - f1 * 0.55
            k.box((px, y1, min(z0, z1)), (px + 0.15, y0, max(z0, z1) + 0.08), "metal")

    # Overhead shop fluorescent light fixture suspended in service bay
    k.box((-0.10, 0.0, 2.95), (1.30, 0.08, 3.03), "metal")
    k.box((-0.05, 0.02, 2.91), (1.25, 0.06, 2.95), "white")

    # Corrugated metal roof (slopes down from front to back)
    k.roof((0.0, 2.55, 3.45), (0.0, -2.65, 2.85), span=(1.0, 0.0, 0.0), width=7.3, color="metal")

    # Solid back wall (x: -3.5 to 3.5, y: -2.5 to -2.38, z: 0.1 to 2.8 m)
    k.solid((-3.5, -2.50, 0.10), (3.5, -2.38, 2.80), "bark_dark", base="wall_rear")
    # Partial side wall on west side (x: -3.5 to -3.38, y: -2.5 to -0.2)
    k.solid((-3.5, -2.38, 0.10), (-3.38, -0.20, 2.95), "bark_dark", base="wall_side_w")

    # ================= TOOL WALL (Back wall x: -3.3 to -0.6, y = -2.38) =================
    k.solid((-3.25, -2.38, 1.05), (-0.65, -2.34, 2.35), "bark_light", base="tool_pegboard")
    k.box((-3.25, -2.36, 2.31), (-0.65, -2.32, 2.35), "bark_dark")
    k.box((-3.25, -2.36, 1.05), (-0.65, -2.32, 1.09), "bark_dark")

    # Hanging tools on pegboard:
    # Sledgehammer
    k.box((-3.15, -2.33, 1.35), (-3.11, -2.31, 2.15), "bark_light")
    k.box((-3.22, -2.33, 2.05), (-3.04, -2.31, 2.18), "gun_black")
    # Pipe wrench
    k.box((-2.92, -2.33, 1.45), (-2.86, -2.31, 2.10), "red_paint")
    k.box((-2.96, -2.33, 1.95), (-2.82, -2.31, 2.14), "metal")
    # 4-way cross rim wrench
    k.box((-2.65, -2.33, 1.70), (-2.35, -2.31, 1.74), "metal")
    k.box((-2.52, -2.33, 1.55), (-2.48, -2.31, 1.89), "metal")
    # Crowbar
    k.box((-2.22, -2.33, 1.35), (-2.18, -2.31, 2.05), "gun_black")
    k.box((-2.26, -2.33, 2.00), (-2.18, -2.31, 2.06), "gun_black")
    # Wrenches set
    for i, wx in enumerate((-1.95, -1.82, -1.70, -1.58, -1.46)):
        h = 0.28 - i * 0.03
        k.box((wx, -2.33, 1.65), (wx + 0.04, -2.31, 1.65 + h), "metal")
    # Hanging black rubber fan belts
    belt1 = common.make_frustum(0.12, 0.12, 0.0, 0.02, segments=10, center=(0.0, 0.0), color="tire_black")
    common.transform(belt1, rot=(math.pi / 2, 0.0, 0.0), loc=(-1.20, -2.32, 1.80))
    model.add(belt1)
    belt2 = common.make_frustum(0.16, 0.16, 0.0, 0.02, segments=10, center=(0.0, 0.0), color="tire_black")
    common.transform(belt2, rot=(math.pi / 2, 0.0, 0.0), loc=(-0.90, -2.32, 1.75))
    model.add(belt2)

    # ================= ROLL-CAB TOOL CHEST (x: -3.2 to -2.3, y: -2.3 to -1.6) =================
    k.solid((-3.15, -2.25, 0.18), (-2.35, -1.65, 1.05), "red_paint", base="tool_chest")
    for cx, cy in ((-3.05, -2.15), (-3.05, -1.75), (-2.45, -2.15), (-2.45, -1.75)):
        k.box((cx - 0.04, cy - 0.04, 0.10), (cx + 0.04, cy + 0.04, 0.18), "gun_black")
    k.box((-3.15, -2.25, 1.05), (-2.35, -1.65, 1.07), "tire_black")
    k.box((-2.35, -2.05, 0.95), (-2.28, -1.85, 0.99), "chrome")
    for dz in (0.32, 0.48, 0.64, 0.80, 0.94):
        k.box((-3.05, -1.64, dz), (-2.45, -1.62, dz + 0.04), "chrome")

    # ================= MECHANIC WORKBENCH (x: -2.1 to -0.7, y: -2.3 to -1.6) =================
    k.solid((-2.10, -2.25, 0.10), (-0.70, -1.65, 0.92), "wood_crate", base="mechanic_bench")
    # Vice
    k.box((-0.95, -1.78, 0.92), (-0.72, -1.55, 1.10), "gun_black")
    k.box((-0.90, -1.58, 1.02), (-0.76, -1.48, 1.08), "metal")
    k.box((-0.85, -1.52, 0.98), (-0.81, -1.40, 1.02), "metal")
    # Battery on bench
    k.box((-1.95, -2.15, 0.92), (-1.65, -1.85, 1.15), "gun_black")
    k.box((-1.92, -2.12, 1.15), (-1.86, -2.06, 1.19), "brass")
    k.box((-1.74, -2.12, 1.15), (-1.68, -2.06, 1.19), "metal")
    # Oil can & funnel
    k.box((-1.45, -2.05, 0.92), (-1.32, -1.92, 1.08), "red_paint")
    k.box((-1.20, -2.05, 0.92), (-1.08, -1.93, 1.12), "white")

    # ================= TIRE STACKS (x: -3.2 to -2.0, y: -0.6 to 0.7) =================
    # Primary stack of 3 tires
    for tz in (0.10, 0.35, 0.60):
        tire = common.make_frustum(0.44, 0.44, tz, tz + 0.24, segments=12, center=(-2.65, 0.0), color="tire_black")
        model.add(tire)
        rim = common.make_frustum(0.24, 0.24, tz + 0.02, tz + 0.22, segments=12, center=(-2.65, 0.0), color="metal")
        model.add(rim)
        hub = common.make_frustum(0.10, 0.10, tz + 0.04, tz + 0.20, segments=8, center=(-2.65, 0.0), color="rock_dark")
        model.add(hub)
    k.solid((-3.10, -0.45, 0.10), (-2.20, 0.45, 0.85), "tire_black", base="tire_stack")

    # Second tire stack (2 tires at x = -2.75, y = 0.95)
    for tz in (0.10, 0.35):
        tire = common.make_frustum(0.44, 0.44, tz, tz + 0.24, segments=12, center=(-2.70, 0.95), color="tire_black")
        model.add(tire)
        rim = common.make_frustum(0.24, 0.24, tz + 0.02, tz + 0.22, segments=12, center=(-2.70, 0.95), color="metal")
        model.add(rim)
    k.solid((-3.15, 0.50, 0.10), (-2.25, 1.40, 0.60), "tire_black", base="tire_stack_2")

    # Fourth tire leaning against the stack
    tire_lean = common.make_frustum(0.44, 0.44, 0.0, 0.24, segments=12, center=(0.0, 0.0), color="tire_black")
    common.transform(tire_lean, rot=(0.25, 0.0, 0.0), loc=(-2.05, 0.10, 0.10))
    model.add(tire_lean)

    # ================= HYDRAULIC FLOOR JACK (x: -0.7 to 0.0, y: 0.1 to 1.1) =================
    k.solid((-0.65, 0.20, 0.10), (-0.25, 0.95, 0.26), "red_paint", base="floor_jack")
    k.box((-0.68, 0.85, 0.10), (-0.64, 0.95, 0.22), "gun_black")
    k.box((-0.26, 0.85, 0.10), (-0.22, 0.95, 0.22), "gun_black")
    k.box((-0.68, 0.22, 0.10), (-0.64, 0.32, 0.20), "gun_black")
    k.box((-0.26, 0.22, 0.10), (-0.22, 0.32, 0.20), "gun_black")
    k.box((-0.55, 0.55, 0.24), (-0.35, 0.85, 0.34), "red_paint")
    saddle = common.make_frustum(0.09, 0.09, 0.34, 0.38, segments=10, center=(-0.45, 0.75), color="tire_black")
    model.add(saddle)
    k.box((-0.47, 0.05, 0.25), (-0.43, 0.35, 1.15), "chrome")
    k.box((-0.52, 0.03, 1.12), (-0.38, 0.07, 1.20), "tire_black")

    # ================= OIL BARRELS (x: 1.8 to 3.1, y: -2.2 to -1.0) =================
    d1 = common.make_frustum(0.28, 0.28, 0.10, 1.00, segments=12, center=(2.15, -1.80), color="shirt_blue")
    model.add(d1)
    k.box((2.12, -1.83, 1.00), (2.18, -1.77, 1.35), "metal")
    k.box((2.15, -1.80, 1.30), (2.35, -1.77, 1.34), "metal")
    k.solid((1.87, -2.08, 0.10), (2.43, -1.52, 1.00), "shirt_blue", base="barrel_1")

    d2 = common.make_frustum(0.28, 0.28, 0.10, 1.00, segments=12, center=(2.75, -1.85), color="red_paint")
    model.add(d2)
    k.solid((2.47, -2.13, 0.10), (3.03, -1.57, 1.00), "red_paint", base="barrel_2")

    d3 = common.make_frustum(0.28, 0.28, 0.10, 1.00, segments=12, center=(2.45, -1.25), color="rock_dark")
    model.add(d3)
    k.solid((2.17, -1.53, 0.10), (2.73, -0.97, 1.00), "rock_dark", base="barrel_3")

    # ================= MECHANIC CREEPER (x: -0.2 to 0.4, y: -1.2 to -0.2) =================
    k.box((-0.15, -1.15, 0.10), (0.45, -0.05, 0.16), "wood_crate")
    k.box((-0.08, -1.12, 0.16), (0.38, -0.88, 0.22), "shirt_blue")
    for cx, cy in ((-0.12, -1.08), (-0.12, -0.12), (0.42, -1.08), (0.42, -0.12)):
        k.box((cx - 0.03, cy - 0.03, 0.10), (cx + 0.03, cy + 0.03, 0.14), "metal")

    # Empties
    k.empty("car_spot", (0.60, 0.0, 0.10), rot_z=0.0)
    k.empty("npc_1", (-1.80, -1.10, 0.10), rot_z=math.pi / 4)
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(7.0, 5.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
