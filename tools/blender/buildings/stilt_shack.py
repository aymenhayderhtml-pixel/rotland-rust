"""stilt_shack: small wooden shack on 1.2 m stilts with front steps, tin roof, and porch (<= 3000 tris).

Walk-in via the front (+Y) steps onto the porch (z=1.2) and through the front door.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "stilt_shack", "buildings", 3000
STILT_H = 1.2
TOP = STILT_H + 2.3
WALL_COLORS = ("bark_light", "wood_crate", "bark_dark")


def build(model):
    k = kit.Kit(model)

    # 6 wooden stilts from ground (z=0) to floor underside (z=1.1)
    for sx in (-2.7, 0.0, 2.7):
        for sy in (-2.7, 2.7):
            k.solid((sx - 0.12, sy - 0.12, 0.0), (sx + 0.12, sy + 0.12, STILT_H - 0.1), "bark_dark", base="stilt")
    # Cross-braces under the shack
    k.box((-2.7, -2.75, 0.45), (2.7, -2.65, 0.55), "bark_dark")
    k.box((-2.7, 2.65, 0.45), (2.7, 2.75, 0.55), "bark_dark")

    # Raised floor deck (body: Y -2.9..1.5, porch: Y 1.5..2.9)
    k.solid((-2.9, -2.9, STILT_H - 0.1), (2.9, 2.9, STILT_H), "wood_crate", base="floor")

    # Front steps (+Y, X: -0.7..0.7, Y: 2.9..4.15, rising from z=0 to z=STILT_H)
    steps = 5
    for s in range(steps):
        sz = (s + 1) * (STILT_H / steps)
        sy1 = 4.15 - s * 0.25
        sy0 = 4.15 - (s + 1) * 0.25
        k.solid((-0.7, sy0, max(0.0, sz - 0.26)), (0.7, sy1, sz), "bark_light", base=f"step_{s}")

    # Porch posts and railings (leaving step entry open at X: -0.7..0.7)
    for px in (-2.8, 2.8):
        k.solid((px - 0.08, 2.74, STILT_H), (px + 0.08, 2.90, TOP), "bark_dark", base="porch_post")
    k.solid((-2.8, 2.78, STILT_H), (-0.7, 2.88, STILT_H + 0.85), "bark_dark", base="porch_rail_l")
    k.solid((0.7, 2.78, STILT_H), (2.8, 2.88, STILT_H + 0.85), "bark_dark", base="porch_rail_r")

    # Shack walls (X: -2.9..2.9, Y: -2.9..1.5)
    door = ((-0.55, 1.3, STILT_H), (0.55, 1.65, STILT_H + 2.0))
    win_l = ((-3.0, -1.4, STILT_H + 0.8), (-2.7, -0.1, STILT_H + 1.65))
    win_r = ((2.7, -1.4, STILT_H + 0.8), (3.0, -0.1, STILT_H + 1.65))
    k.wall((-2.9, 1.35, STILT_H), (2.9, 1.5, TOP), openings=[door], colors=WALL_COLORS, course=0.35, base="w_front")
    k.wall((-2.9, -2.9, STILT_H), (2.9, -2.75, TOP), colors=WALL_COLORS, course=0.35, base="w_back")
    k.wall((-2.9, -2.75, STILT_H), (-2.75, 1.35, TOP), openings=[win_l], colors=WALL_COLORS, course=0.35, base="w_left")
    k.wall((2.75, -2.75, STILT_H), (2.9, 1.35, TOP), openings=[win_r], colors=WALL_COLORS, course=0.35, base="w_right")

    k.frame(door, 1.52, 1, "bark_dark", sill=False)
    k.frame(win_l, -2.92, -1, "bark_dark")
    k.frame(win_r, 2.92, 1, "bark_dark")

    # Sloped tin roof (metal + rust panels) covering both shack and porch
    roof_panels = 6
    panel_w = 6.2 / roof_panels
    for i in range(roof_panels):
        x0 = -3.1 + i * panel_w
        x1 = x0 + panel_w
        col = "rust" if i % 2 == 0 else "metal"
        k.box((x0, -3.1, TOP), (x1, 3.05, TOP + 0.08), col, roof=True)
    k.collider((-3.1, -3.1, TOP), (3.1, 3.05, TOP + 0.08), "tin_roof")

    # Interior: bed, table, crate
    k.solid((-2.6, -2.6, STILT_H), (-1.4, -0.8, STILT_H + 0.45), "bark_dark", base="bed")
    k.box((-2.55, -2.55, STILT_H + 0.45), (-1.45, -0.85, STILT_H + 0.60), "canvas_khaki")
    k.solid((1.2, -2.4, STILT_H), (2.4, -1.4, STILT_H + 0.75), "wood_crate", base="table")
    k.crate(2.1, 0.5, 0.55, z=STILT_H)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(6.0, 7.5, 1.0))
    k.empty("door_1", (0.0, 1.425, STILT_H))
    k.empty("loot_1", (1.8, -1.9, STILT_H + 0.75))  # table
    k.empty("loot_2", (2.1, 0.5, STILT_H + 0.55))   # crate
    k.empty("spawn_1", (-0.8, 0.0, STILT_H))
    k.empty("spawn_2", (0.8, -0.8, STILT_H))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
