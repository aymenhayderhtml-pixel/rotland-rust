"""base_workbench: rough plank workbench with saw, hammer, and nails box (<= 600 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "base_workbench", "building_parts", 600

# Dimensions fit 2m cell: bench is 1.50m wide, 0.76m deep, 0.88m table height
BENCH_W = 1.50
BENCH_D = 0.76
BENCH_H = 0.88
TOP_THICK = 0.08
LEG_W = 0.10
LEG_H = BENCH_H - TOP_THICK  # 0.80m


def build(model):
    parts = []

    # 1. Four heavy wooden 4x4 legs
    leg_x = BENCH_W / 2 - LEG_W / 2 - 0.05  # 0.65m
    leg_y = BENCH_D / 2 - LEG_W / 2 - 0.05  # 0.28m
    for sx in (-1, 1):
        for sy in (-1, 1):
            parts.append(common.make_box((LEG_W, LEG_W, LEG_H),
                                         loc=(sx * leg_x, sy * leg_y, LEG_H / 2),
                                         color="bark_dark"))

    # 2. Upper aprons / stretchers directly under the tabletop (Z = 0.74)
    apron_z = LEG_H - 0.05
    apron_h = 0.08
    # Front and back stretchers (along X)
    for sy in (-1, 1):
        parts.append(common.make_box((BENCH_W - 0.20, 0.06, apron_h),
                                     loc=(0.0, sy * leg_y, apron_z),
                                     color="bark_dark"))
    # Left and right stretchers (along Y)
    for sx in (-1, 1):
        parts.append(common.make_box((0.06, BENCH_D - 0.20, apron_h),
                                     loc=(sx * leg_x, 0.0, apron_z),
                                     color="bark_dark"))

    # 3. Lower tool storage shelf at Z = 0.16
    shelf_z = 0.18
    # 2 lower shelf support rails (along X)
    for sy in (-1, 1):
        parts.append(common.make_box((BENCH_W - 0.20, 0.05, 0.04),
                                     loc=(0.0, sy * (leg_y - 0.05), shelf_z - 0.03),
                                     color="bark_dark"))
    # 3 shelf deck planks (along X)
    for sy in (-1, 0, 1):
        parts.append(common.make_box((BENCH_W - 0.22, 0.14, 0.025),
                                     loc=(0.0, sy * 0.16, shelf_z),
                                     color="wood_crate"))

    # 4. Tabletop: 3 rough heavy planks running along X
    plank_d = BENCH_D / 3  # ~0.253m
    top_z = LEG_H + TOP_THICK / 2
    colors = ["wood_crate", "bark_light", "wood_crate"]
    for i, sy in enumerate((-1, 0, 1)):
        parts.append(common.make_box((BENCH_W, plank_d - 0.01, TOP_THICK),
                                     loc=(0.0, sy * plank_d, top_z),
                                     color=colors[i]))

    # Rear backstop rail along +Y edge of tabletop
    parts.append(common.make_box((BENCH_W, 0.04, 0.08),
                                 loc=(0.0, BENCH_D / 2 - 0.02, BENCH_H + 0.04),
                                 color="bark_dark"))

    # 5. Woodworking vise mounted on the front-left corner
    vise_x = -0.56
    vise_y = -(BENCH_D / 2 + 0.035)
    # Movable wooden front jaw block
    parts.append(common.make_box((0.22, 0.05, 0.14),
                                 loc=(vise_x, vise_y, BENCH_H - 0.07),
                                 color="wood_crate"))
    # Metal vise screw spindle
    parts.append(common.make_box((0.04, 0.12, 0.04),
                                 loc=(vise_x, vise_y + 0.04, BENCH_H - 0.08),
                                 color="metal"))
    # Vise T-bar handle
    parts.append(common.make_box((0.02, 0.02, 0.18),
                                 loc=(vise_x, vise_y - 0.03, BENCH_H - 0.08),
                                 color="metal"))

    # 6. Tools on top of the workbench (table surface at Z = BENCH_H = 0.88m)
    # A. Handsaw (on right side)
    saw_x = 0.35
    saw_y = -0.08
    saw_z = BENCH_H + 0.035
    # Wooden D-handle
    parts.append(common.make_box((0.09, 0.03, 0.07),
                                 loc=(saw_x + 0.18, saw_y, saw_z + 0.01),
                                 color="bark_dark"))
    # Metal saw blade body (tapered look with main blade + teeth strip)
    parts.append(common.make_box((0.32, 0.008, 0.05),
                                 loc=(saw_x, saw_y, saw_z),
                                 color="metal"))
    parts.append(common.make_box((0.32, 0.008, 0.015),
                                 loc=(saw_x, saw_y, BENCH_H + 0.008),
                                 color="gun_black"))

    # B. Claw hammer (on left side)
    ham_x = -0.22
    ham_y = 0.10
    ham_z = BENCH_H + 0.015
    # Wooden hammer handle (along Y)
    parts.append(common.make_box((0.025, 0.28, 0.025),
                                 loc=(ham_x, ham_y, ham_z),
                                 color="bark_light"))
    # Steel hammer head (at end of handle)
    parts.append(common.make_box((0.045, 0.07, 0.04),
                                 loc=(ham_x, ham_y + 0.14, ham_z + 0.01),
                                 color="metal"))
    # Hammer claw
    parts.append(common.make_box((0.03, 0.05, 0.025),
                                 loc=(ham_x, ham_y + 0.19, ham_z + 0.015),
                                 color="gun_black"))

    # C. Nails box (open wooden box with nails inside, in center-back)
    box_x = 0.06
    box_y = 0.16
    box_w = 0.18
    box_d = 0.14
    box_h = 0.07
    box_wall = 0.018
    # Bottom plate
    parts.append(common.make_box((box_w, box_d, 0.015),
                                 loc=(box_x, box_y, BENCH_H + 0.008),
                                 color="wood_crate"))
    # Front and back walls
    for sy in (-1, 1):
        parts.append(common.make_box((box_w, box_wall, box_h),
                                     loc=(box_x, box_y + sy * (box_d / 2 - box_wall / 2), BENCH_H + box_h / 2),
                                     color="wood_crate"))
    # Left and right walls
    for sx in (-1, 1):
        parts.append(common.make_box((box_wall, box_d - 2 * box_wall, box_h),
                                     loc=(box_x + sx * (box_w / 2 - box_wall / 2), box_y, BENCH_H + box_h / 2),
                                     color="wood_crate"))
    # Pile of metallic nails / hardware inside
    parts.append(common.make_box((box_w - 0.04, box_d - 0.04, 0.035),
                                 loc=(box_x, box_y, BENCH_H + 0.025),
                                 color="metal"))
    # A few shiny nail spikes sticking up
    parts.append(common.make_box((0.04, 0.04, 0.02),
                                 loc=(box_x + 0.02, box_y - 0.01, BENCH_H + 0.045),
                                 color="chrome"))

    model.add(common.combine(parts))

    # Convex collision box for the workbench
    model.add_collider("bench", common.make_box((BENCH_W + 0.04, BENCH_D + 0.08, BENCH_H + 0.08),
                                                loc=(0.0, -0.02, (BENCH_H + 0.08) / 2),
                                                color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
