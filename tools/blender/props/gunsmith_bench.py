"""gunsmith_bench: camp gunsmith workbench (~2.2 x 0.9 m) with vise, tools, 3-rifle wall rack, lamp, ammo boxes."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "gunsmith_bench", "props", 1500

BENCH_W = 2.20
BENCH_D = 0.90
BENCH_H = 0.88


def _rifle_silhouette(k, z_mid, y_mid, style):
    """Adds a low-poly horizontal rifle silhouette resting on the rear rack."""
    if style == "hunting":
        k.box((-0.58, y_mid - 0.02, z_mid - 0.055), (-0.18, y_mid + 0.02, z_mid + 0.03), "wood_crate")
        k.box((-0.18, y_mid - 0.025, z_mid - 0.03), (0.24, y_mid + 0.025, z_mid + 0.025), "wood_crate")
        k.box((-0.12, y_mid - 0.022, z_mid + 0.015), (0.14, y_mid + 0.022, z_mid + 0.055), "metal")
        k.box((0.14, y_mid - 0.012, z_mid + 0.022), (0.62, y_mid + 0.012, z_mid + 0.045), "gun_black")
        k.box((-0.09, y_mid - 0.016, z_mid + 0.06), (0.11, y_mid + 0.016, z_mid + 0.095), "gun_black")
    elif style == "assault":
        k.box((-0.54, y_mid - 0.02, z_mid - 0.05), (-0.18, y_mid + 0.02, z_mid + 0.025), "bark_light")
        k.box((-0.18, y_mid - 0.025, z_mid - 0.01), (0.20, y_mid + 0.025, z_mid + 0.05), "gun_black")
        k.box((-0.16, y_mid - 0.018, z_mid - 0.09), (-0.09, y_mid + 0.018, z_mid - 0.01), "bark_dark")
        k.box((0.02, y_mid - 0.018, z_mid - 0.13), (0.10, y_mid + 0.018, z_mid - 0.01), "gun_black")
        k.box((0.20, y_mid - 0.014, z_mid + 0.018), (0.56, y_mid + 0.014, z_mid + 0.042), "metal")
        k.box((0.46, y_mid - 0.01, z_mid + 0.042), (0.50, y_mid + 0.01, z_mid + 0.075), "gun_black")
    else:
        k.box((-0.56, y_mid - 0.02, z_mid - 0.05), (-0.18, y_mid + 0.02, z_mid + 0.025), "gun_black")
        k.box((-0.18, y_mid - 0.024, z_mid - 0.015), (0.12, y_mid + 0.024, z_mid + 0.045), "metal")
        k.box((0.12, y_mid - 0.02, z_mid - 0.025), (0.36, y_mid + 0.02, z_mid + 0.015), "bark_dark")
        k.box((0.12, y_mid - 0.013, z_mid + 0.022), (0.60, y_mid + 0.013, z_mid + 0.046), "gun_black")
        k.box((0.12, y_mid - 0.012, z_mid - 0.005), (0.48, y_mid + 0.012, z_mid + 0.016), "metal")


def build(model):
    k = kit.Kit(model)

    # 1. Sturdy timber frame: 4 corner legs, lower shelf rails/slats, upper aprons
    for lx in (-1.04, 0.92):
        for ly in (-0.40, 0.28):
            k.box((lx, ly, 0.0), (lx + 0.12, ly + 0.12, 0.80), "bark_dark")

    for sy in (-0.38, 0.30):
        k.box((-0.92, sy, 0.16), (0.92, sy + 0.06, 0.22), "bark_dark")
    for sx in (-1.02, 0.96):
        k.box((sx, -0.32, 0.16), (sx + 0.06, 0.30, 0.22), "bark_dark")
    for i, sy in enumerate((-0.24, 0.0, 0.24)):
        k.box((-0.92, sy - 0.09, 0.22), (0.92, sy + 0.09, 0.25), "wood_crate" if i != 1 else "bark_light")

    for sy in (-0.38, 0.32):
        k.box((-0.92, sy, 0.72), (0.92, sy + 0.06, 0.80), "bark_dark")
    for sx in (-1.02, 0.96):
        k.box((sx, -0.32, 0.72), (sx + 0.06, 0.32, 0.80), "bark_dark")

    # Plank tabletop (2.20 x 0.90 m, z = 0.80..0.88) with steel front lip
    k.box((-1.10, -0.45, 0.80), (1.10, -0.005, BENCH_H), "wood_crate")
    k.box((-1.10, 0.005, 0.80), (1.10, 0.43, BENCH_H), "bark_light")
    k.box((-1.10, 0.43, 0.80), (1.10, 0.455, BENCH_H + 0.005), "metal")

    # 2. Rear wall gun rack (uprights, backboard, notched rails, 3 rifles)
    k.box((-1.10, -0.45, BENCH_H), (-1.00, -0.37, 2.12), "bark_dark")
    k.box((1.00, -0.45, BENCH_H), (1.10, -0.37, 2.12), "bark_dark")
    k.box((-1.10, -0.46, 2.12), (1.10, -0.34, 2.22), "bark_dark")
    for idx, z0 in enumerate((0.92, 1.32, 1.72)):
        col = "bark_light" if idx % 2 == 0 else "wood_crate"
        k.box((-1.00, -0.44, z0), (1.00, -0.40, z0 + 0.38), col)

    for rx in (-0.42, 0.36):
        k.box((rx - 0.03, -0.39, 0.96), (rx + 0.03, -0.34, 2.06), "bark_dark")
        for pz in (1.16, 1.48, 1.80):
            k.box((rx - 0.025, -0.34, pz), (rx + 0.025, -0.24, pz + 0.035), "metal")

    _rifle_silhouette(k, 1.24, -0.30, "hunting")
    _rifle_silhouette(k, 1.56, -0.30, "assault")
    _rifle_silhouette(k, 1.88, -0.30, "shotgun")

    # 3. Hanging lamp centered over the bench
    k.box((-0.03, -0.34, 2.15), (0.03, 0.08, 2.20), "metal")
    brace = common.make_box((0.04, 0.34, 0.04), color="metal")
    common.transform(brace, loc=(0.0, -0.22, 2.02), rot=(0.65, 0.0, 0.0))
    model.add(brace)
    k.box((-0.012, 0.038, 1.82), (0.012, 0.062, 2.15), "gun_black")
    model.add(common.make_frustum(0.20, 0.05, 1.68, 1.82, segments=8, center=(0.0, 0.05), color="pine_dark"))
    model.add(common.make_frustum(0.04, 0.03, 1.64, 1.71, segments=6, center=(0.0, 0.05), color="brass"))

    # 4. Heavy gunsmith vise on front-left corner
    k.box((-0.90, 0.28, BENCH_H), (-0.64, 0.45, 0.94), "gun_black")
    k.box((-0.86, 0.31, 0.94), (-0.68, 0.40, 1.07), "gun_black")
    k.box((-0.88, 0.37, 1.03), (-0.66, 0.40, 1.09), "metal")
    k.box((-0.86, 0.44, 0.94), (-0.68, 0.51, 1.07), "gun_black")
    k.box((-0.88, 0.44, 1.03), (-0.66, 0.47, 1.09), "metal")
    k.box((-0.79, 0.30, 0.98), (-0.75, 0.54, 1.02), "metal")
    k.box((-0.88, 0.53, 0.91), (-0.66, 0.55, 1.08), "metal")

    # 5. Scattered tools and gun work on the tabletop
    k.box((-0.34, -0.14, BENCH_H), (0.34, 0.28, BENCH_H + 0.015), "shirt_blue")
    k.box((-0.24, 0.04, BENCH_H + 0.015), (0.12, 0.09, BENCH_H + 0.055), "metal")
    k.box((0.12, 0.055, BENCH_H + 0.025), (0.30, 0.075, BENCH_H + 0.045), "gun_black")

    # Oil can + brass nozzle
    model.add(common.make_frustum(0.045, 0.045, BENCH_H, BENCH_H + 0.12, segments=6, center=(0.44, 0.18), color="red_paint"))
    model.add(common.make_frustum(0.02, 0.008, BENCH_H + 0.12, BENCH_H + 0.17, segments=6, center=(0.44, 0.18), color="brass"))

    # Ball-peen hammer
    ham_h = common.make_box((0.024, 0.28, 0.024), color="bark_light")
    common.transform(ham_h, loc=(-0.48, 0.04, BENCH_H + 0.012), rot=(0.0, 0.0, 0.25))
    model.add(ham_h)
    ham_head = common.make_box((0.09, 0.036, 0.036), color="metal")
    common.transform(ham_head, loc=(-0.51, 0.17, BENCH_H + 0.018), rot=(0.0, 0.0, 0.25))
    model.add(ham_head)

    # Two screwdrivers & two wrenches
    k.box((0.36, -0.24, BENCH_H), (0.39, -0.14, BENCH_H + 0.028), "red_paint")
    k.box((0.37, -0.14, BENCH_H + 0.008), (0.38, -0.02, BENCH_H + 0.02), "metal")
    k.box((0.44, -0.22, BENCH_H), (0.47, -0.12, BENCH_H + 0.028), "shirt_blue")
    k.box((0.45, -0.12, BENCH_H + 0.008), (0.46, 0.0, BENCH_H + 0.02), "metal")
    for wx, yaw in ((0.56, -0.2), (0.64, 0.15)):
        wr = common.make_box((0.025, 0.22, 0.012), color="metal")
        common.transform(wr, loc=(wx, 0.08, BENCH_H + 0.006), rot=(0.0, 0.0, yaw))
        model.add(wr)

    # Small brass parts tray
    k.box((-0.18, -0.30, BENCH_H), (0.14, -0.18, BENCH_H + 0.035), "wood_crate")
    k.box((-0.16, -0.28, BENCH_H + 0.035), (-0.03, -0.20, BENCH_H + 0.05), "brass")
    k.box((0.01, -0.28, BENCH_H + 0.035), (0.12, -0.20, BENCH_H + 0.05), "metal")

    # 6. Ammo boxes on tabletop right and lower shelf
    k.box((0.72, -0.14, BENCH_H), (1.02, 0.04, BENCH_H + 0.17), "pine_dark")
    k.box((0.71, -0.15, BENCH_H + 0.15), (1.03, 0.05, BENCH_H + 0.19), "metal")
    k.box((0.84, 0.04, BENCH_H + 0.08), (0.90, 0.06, BENCH_H + 0.15), "brass")
    k.box((0.74, 0.12, BENCH_H), (0.96, 0.28, BENCH_H + 0.10), "red_paint")
    k.box((0.735, 0.115, BENCH_H + 0.08), (0.965, 0.285, BENCH_H + 0.105), "brass")

    # Lower shelf ammo crates
    k.box((-0.78, -0.22, 0.25), (-0.32, 0.16, 0.53), "wood_crate")
    k.box((-0.79, -0.23, 0.49), (-0.31, 0.17, 0.55), "bark_dark")
    k.box((0.28, -0.20, 0.25), (0.74, 0.16, 0.49), "leaf_dark")
    k.box((0.27, -0.21, 0.45), (0.75, 0.17, 0.51), "brass")

    # Colliders (bench body + rear rack)
    k.collider((-1.12, -0.47, 0.0), (1.12, 0.47, BENCH_H + 0.02), base="bench")
    k.collider((-1.12, -0.47, BENCH_H + 0.02), (1.12, -0.24, 2.22), base="rack")

    # Empties
    k.empty("interact_1", (0.0, 0.90, 0.0), rot_z=math.pi)
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(2.6, 1.8, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
