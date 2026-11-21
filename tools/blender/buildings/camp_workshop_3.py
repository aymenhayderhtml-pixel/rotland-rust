"""camp_workshop_3: Tier 3 machine shop (6x4 m open shed with lathe, welding set, engine on stand).

Open industrial workshop with concrete slab floor, corrugated roof, metal lathe,
portable oxy-acetylene welding rig, V8 engine mounted on an engine stand, and interact_1 empty.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_workshop_3", "buildings", 4000


def build(model):
    k = kit.Kit(model)

    # ================= SHED STRUCTURE (6 x 4 m) =================
    # Concrete floor slab
    k.solid((-3.0, -2.0, 0.0), (3.0, 2.0, 0.12), "rock_mid", base="floor_slab")

    # Perimeter posts: 2 front posts at corners (x: -2.85, 2.70), open wide entrance!
    # Front corner posts (+Y: y = 1.85, height 3.15 m)
    for px in (-2.85, 2.70):
        k.solid((px, 1.75, 0.12), (px + 0.15, 1.90, 3.15), "metal", base="post_front")
    # Rear posts (-Y: y = -1.90, height 2.60 m)
    for px in (-2.85, 0.0, 2.70):
        k.solid((px, -1.90, 0.12), (px + 0.15, -1.75, 2.60), "metal", base="post_rear")

    # Front and rear top header beams
    k.box((-3.0, 1.75, 3.10), (3.0, 1.90, 3.25), "metal")
    k.box((-3.0, -1.90, 2.55), (3.0, -1.75, 2.70), "metal")
    # Cross rafters (connecting front to back)
    for px in (-2.85, 0.0, 2.70):
        for s in range(4):
            f0 = s / 4
            f1 = (s + 1) / 4
            y0 = 1.85 - f0 * 3.65
            y1 = 1.85 - f1 * 3.65
            z0 = 3.18 - f0 * 0.55
            z1 = 3.18 - f1 * 0.55
            k.box((px, y1, min(z0, z1)), (px + 0.15, y0, max(z0, z1) + 0.08), "metal")

    # Corrugated metal roof (sloped plane, ridge along X from -3.1 to 3.1)
    k.roof((0.0, 2.10, 3.30), (0.0, -2.15, 2.65), span=(1.0, 0.0, 0.0), width=6.3, color="rock_dark")

    # Solid back wall (corrugated metal panels along -Y)
    k.solid((-2.9, -1.95, 0.12), (2.9, -1.85, 2.55), "rock_dark", base="wall_rear")
    # Partial side walls
    k.solid((-2.95, -1.85, 0.12), (-2.85, 0.2, 2.80), "rock_dark", base="wall_side_w")
    k.solid((2.85, -1.85, 0.12), (2.95, -0.4, 2.80), "rock_dark", base="wall_side_e")

    # Overhead shop light fixture suspended from center rafter
    k.box((-0.60, 0.0, 2.80), (0.60, 0.08, 2.88), "metal")
    k.box((-0.55, 0.02, 2.76), (0.55, 0.06, 2.80), "white")           # fluorescent tube

    # ================= 1. INDUSTRIAL METAL LATHE (along back wall x: -1.0 to 1.4, y: -1.7 to -1.1) =================
    # Lathe cast iron pedestals
    k.solid((-1.0, -1.65, 0.12), (-0.4, -1.15, 0.85), "shirt_blue", base="lathe_head_base")
    k.solid((0.8, -1.65, 0.12), (1.3, -1.15, 0.85), "shirt_blue", base="lathe_tail_base")
    # Chip pan between pedestals
    k.box((-1.0, -1.68, 0.80), (1.3, -1.12, 0.85), "rock_dark")
    # Twin prism lathe bed rails
    k.box((-0.4, -1.58, 0.85), (0.8, -1.46, 0.98), "metal")
    k.box((-0.4, -1.34, 0.85), (0.8, -1.22, 0.98), "metal")
    # Headstock gearbox (left)
    k.solid((-1.0, -1.65, 0.85), (-0.4, -1.15, 1.35), "shirt_blue", base="lathe_headstock")
    k.box((-0.95, -1.14, 1.05), (-0.85, -1.11, 1.25), "red_paint")
    k.box((-0.75, -1.14, 1.05), (-0.65, -1.11, 1.25), "metal")
    # 3-jaw chuck
    chuck = common.make_frustum(0.18, 0.18, 0.0, 0.16, segments=12, center=(0.0, 0.0), color="metal")
    common.transform(chuck, rot=(0.0, math.pi / 2, 0.0), loc=(-0.40, -1.40, 1.10))
    model.add(chuck)
    k.box((-0.30, -1.48, 1.06), (-0.22, -1.32, 1.14), "gun_black")
    # Carriage / apron
    k.solid((-0.15, -1.65, 0.85), (0.25, -1.12, 1.05), "gun_black", base="lathe_carriage")
    k.box((-0.05, -1.50, 1.05), (0.15, -1.25, 1.18), "metal")
    k.box((0.02, -1.42, 1.18), (0.12, -1.32, 1.28), "gun_black")
    k.box((0.06, -1.34, 1.20), (0.10, -1.20, 1.24), "metal")
    k.box((-0.08, -1.11, 0.90), (0.06, -1.08, 1.02), "metal")         # handwheel
    # Tailstock (right)
    k.solid((0.55, -1.58, 0.85), (0.85, -1.22, 1.18), "shirt_blue", base="lathe_tailstock")
    k.box((0.45, -1.42, 1.08), (0.55, -1.38, 1.12), "metal")
    k.box((0.85, -1.42, 1.08), (0.92, -1.38, 1.12), "metal")
    # Threaded lead screw
    k.box((-0.4, -1.14, 0.90), (0.8, -1.10, 0.94), "metal")

    # ================= 2. WELDING SET ON ROLLING CART (x: -1.7 to -0.9, y: 0.1 to 0.7) =================
    k.solid((-1.65, 0.15, 0.12), (-1.05, 0.60, 0.18), "gun_black", base="weld_cart")
    k.box((-1.68, 0.18, 0.12), (-1.64, 0.28, 0.28), "rock_dark")
    k.box((-1.06, 0.18, 0.12), (-1.02, 0.28, 0.28), "rock_dark")
    k.box((-1.65, 0.15, 0.18), (-1.05, 0.20, 1.40), "gun_black")
    k.box((-1.55, 0.15, 1.38), (-1.15, 0.25, 1.44), "gun_black")
    # Oxygen cylinder (tall blue/green)
    ox = common.make_frustum(0.11, 0.11, 0.18, 1.45, segments=10, center=(-1.48, 0.40), color="shirt_blue")
    model.add(ox)
    ox_cap = common.make_frustum(0.11, 0.05, 1.45, 1.55, segments=10, center=(-1.48, 0.40), color="shirt_blue")
    model.add(ox_cap)
    k.box((-1.52, 0.36, 1.55), (-1.44, 0.44, 1.65), "brass")
    # Acetylene cylinder (shorter red)
    ac = common.make_frustum(0.13, 0.13, 0.18, 1.15, segments=10, center=(-1.22, 0.40), color="red_paint")
    model.add(ac)
    ac_cap = common.make_frustum(0.13, 0.06, 1.15, 1.25, segments=10, center=(-1.22, 0.40), color="red_paint")
    model.add(ac_cap)
    k.box((-1.26, 0.36, 1.25), (-1.18, 0.44, 1.35), "brass")
    # Coiled twin hose on cart side
    k.box((-1.04, 0.28, 0.65), (-0.96, 0.48, 1.05), "red_paint")
    k.box((-1.02, 0.30, 0.67), (-0.94, 0.50, 1.07), "pine_dark")
    k.box((-1.08, 0.55, 0.95), (-1.00, 0.65, 1.25), "brass")         # torch
    # Welding helmet
    k.box((-1.45, 0.12, 0.95), (-1.25, 0.25, 1.20), "gun_black")
    k.box((-1.40, 0.10, 1.05), (-1.30, 0.12, 1.12), "glass_tint")

    # ================= 3. ENGINE ON ENGINE STAND (x: 0.1 to 1.3, y: 0.1 to 1.3) =================
    k.solid((0.15, 0.25, 0.12), (0.30, 1.25, 0.18), "red_paint", base="stand_base_front")
    k.solid((0.25, 0.70, 0.12), (1.10, 0.80, 0.18), "red_paint", base="stand_spine")
    k.solid((1.05, 0.35, 0.12), (1.20, 1.15, 0.18), "red_paint", base="stand_base_rear")
    for cx, cy in ((0.22, 0.30), (0.22, 1.20), (1.12, 0.40), (1.12, 1.10)):
        k.box((cx - 0.04, cy - 0.04, 0.12), (cx + 0.04, cy + 0.04, 0.20), "gun_black")
    k.solid((1.03, 0.69, 0.18), (1.17, 0.81, 0.95), "red_paint", base="stand_column")
    k.box((0.65, 0.71, 0.18), (1.07, 0.79, 0.85), "red_paint")
    k.box((0.90, 0.65, 0.88), (1.10, 0.85, 1.02), "gun_black")
    k.box((0.75, 0.55, 0.80), (0.93, 0.95, 1.10), "metal")

    # V8 Engine Block
    k.solid((0.25, 0.59, 0.72), (0.80, 0.91, 0.86), "gun_black", base="engine_oil_pan")
    k.solid((0.20, 0.49, 0.86), (0.83, 1.01, 1.15), "gun_black", base="engine_block")
    k.box((0.19, 0.45, 1.12), (0.81, 0.63, 1.25), "chrome")           # valve cover left
    k.box((0.19, 0.87, 1.12), (0.81, 1.05, 1.25), "chrome")           # valve cover right
    k.box((0.30, 0.61, 1.14), (0.73, 0.89, 1.22), "metal")
    air_cleaner = common.make_frustum(0.14, 0.14, 1.22, 1.32, segments=12, center=(0.50, 0.75), color="chrome")
    model.add(air_cleaner)
    k.box((0.13, 0.70, 0.82), (0.19, 0.80, 0.94), "metal")           # crankshaft pulley
    k.box((0.13, 0.87, 1.02), (0.20, 0.99, 1.14), "metal")           # alternator
    k.box((0.15, 0.69, 0.85), (0.19, 0.97, 1.12), "gun_black")       # belt
    k.box((0.25, 0.41, 0.95), (0.77, 0.49, 1.05), "rust")            # exhaust manifold
    k.box((0.25, 1.01, 0.95), (0.77, 1.09, 1.05), "rust")

    # ================= 4. SHOP WORKBENCH & CLUTTER (West side x: -2.8 to -2.0, y: -1.0 to 0.4) =================
    k.solid((-2.80, -0.95, 0.12), (-2.05, 0.35, 0.90), "wood_crate", base="shop_bench")
    k.box((-2.15, 0.15, 0.90), (-2.02, 0.35, 1.06), "gun_black")      # vice
    k.box((-2.75, -0.85, 0.90), (-2.35, -0.45, 1.15), "red_paint")     # red toolbox
    # 55-gallon oil drum near back corner (x = -2.3, y = -1.45)
    drum = common.make_frustum(0.28, 0.28, 0.12, 1.02, segments=12, center=(-2.30, -1.45), color="red_paint")
    model.add(drum)
    drum_rim1 = common.make_frustum(0.29, 0.29, 0.35, 0.40, segments=12, center=(-2.30, -1.45), color="rock_dark")
    model.add(drum_rim1)
    drum_rim2 = common.make_frustum(0.29, 0.29, 0.75, 0.80, segments=12, center=(-2.30, -1.45), color="rock_dark")
    model.add(drum_rim2)
    k.box((-2.33, -1.48, 1.02), (-2.27, -1.42, 1.35), "metal")
    k.box((-2.30, -1.45, 1.30), (-2.10, -1.42, 1.34), "metal")

    # Empties
    k.empty("interact_1", (0.0, 1.75, 0.0), rot_z=math.pi)
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(6.0, 4.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
