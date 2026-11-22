"""warehouse_lod1: low-detail exterior shell of warehouse (budget 500 tris).

Same outer footprint (23 x 11 m on 26 x 16 m lot), same roof shape, loading dock outline,
and main corrugated color bands. No interior geometry, pallets, or small details.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "warehouse_lod1", "buildings", 500
WALL_TOP = 4.6
RIDGE_Z = 5.8
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)

    # 1. Base floor / foundation
    k.box((-11.5, -6.5, 0.0), (11.5, 4.5, F), "rock_dark")

    # 2. Simplified exterior walls (broad alternating bands)
    # Back wall (Y = -6.5)
    bw = 23.0 / 6
    for i in range(6):
        x0 = -11.5 + i * bw
        x1 = -11.5 + (i + 1) * bw
        col = "metal" if i % 2 == 0 else "rock_dark"
        k.box((x0, -6.5, F), (x1, -6.35, WALL_TOP), col)

    # Front wall (Y = 4.5)
    # Roller door opening represented as dark indented panel (-7.0 to -3.0)
    # Left of door (-11.5 to -7.0)
    k.box((-11.5, 4.35, F), (-9.25, 4.5, WALL_TOP), "metal")
    k.box((-9.25, 4.35, F), (-7.0, 4.5, WALL_TOP), "rock_dark")
    # Roller door (half-open look: dark open bottom, corrugated top)
    k.box((-7.0, 4.38, F), (-3.0, 4.48, 2.2), "gun_black")
    k.box((-7.0, 4.38, 2.2), (-3.0, 4.48, 4.0), "rock_dark")
    k.box((-7.0, 4.35, 4.0), (-3.0, 4.5, WALL_TOP), "metal")
    # Wall between door and dock (-3.0 to 0.5)
    k.box((-3.0, 4.35, F), (-1.25, 4.5, WALL_TOP), "rock_dark")
    k.box((-1.25, 4.35, F), (0.5, 4.5, WALL_TOP), "metal")
    # Wall behind loading dock (0.5 to 11.5)
    for i in range(4):
        x0 = 0.5 + i * 2.75
        x1 = 0.5 + (i + 1) * 2.75
        col = "rock_dark" if i % 2 == 0 else "metal"
        k.box((x0, 4.35, F), (x1, 4.5, WALL_TOP), col)

    # Left wall (X = -11.5) - between back and front wall inner faces
    bw_left = (4.35 - (-6.35)) / 3
    for i in range(3):
        y0 = -6.35 + i * bw_left
        y1 = -6.35 + (i + 1) * bw_left
        col = "metal" if i % 2 == 0 else "rock_dark"
        k.box((-11.5, y0, F), (-11.35, y1, WALL_TOP), col)

    # Right wall (X = 11.5) with side door shape - between back and front wall inner faces
    k.box((11.35, -6.35, F), (11.5, -2.0, WALL_TOP), "rock_dark")
    # Side door
    k.box((11.38, -2.0, F), (11.48, -0.8, kit.DOOR_H), "gun_black")
    k.box((11.35, -2.0, kit.DOOR_H), (11.5, -0.8, WALL_TOP), "metal")
    k.box((11.35, -0.8, F), (11.5, 4.35, WALL_TOP), "metal")

    # 3. Loading Dock Platform (0.5 to 11.0, Y: 4.5 to 6.8, H: 0.7)
    dock_top = 0.7
    k.box((0.5, 4.5, 0.0), (11.0, 6.8, dock_top), "rock_mid")
    k.box((0.51, 6.74, dock_top - 0.02), (10.99, 6.82, dock_top + 0.02), "brass")
    # Steps
    k.box((-0.3, 5.2, 0.0), (0.5, 6.8, 0.35), "rock_mid")
    # Dock canopy
    k.box((0.3, 4.5, 3.8), (11.2, 7.0, 3.92), "metal")
    for px in (0.8, 10.6):
        k.box((px - 0.08, 6.6 - 0.08, dock_top), (px + 0.08, 6.6 + 0.08, 3.8), "gun_black")

    # 4. Shallow Gable Roof
    ridge = (0.0, -1.0, RIDGE_Z)
    k.roof(ridge, (0.0, -6.8, WALL_TOP), span=(1.0, 0.0, 0.0), width=23.6, color="metal", thick=0.14)
    k.roof(ridge, (0.0, 4.8, WALL_TOP), span=(1.0, 0.0, 0.0), width=23.58, color="metal", thick=0.14)
    k.ridge_cap((0.0, -1.0, RIDGE_Z), 0, 23.8, color="rock_dark")

    # Gable ends
    k.gable([(-11.5, -6.5, WALL_TOP), (-11.5, 4.5, WALL_TOP), (-11.5, -1.0, RIDGE_Z)], (0.15, 0.0, 0.0), "metal", roof=True)
    k.gable([(11.35, -6.5, WALL_TOP), (11.35, 4.5, WALL_TOP), (11.35, -1.0, RIDGE_Z)], (0.15, 0.0, 0.0), "metal", roof=True)

    # 5. Marker
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(26.0, 16.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
