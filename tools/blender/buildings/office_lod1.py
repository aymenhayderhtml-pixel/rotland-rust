"""office_lod1: low-poly distant LOD for office.

Same footprint (16 x 16 m in 18 x 18 m lot), same silhouette (4 floors, collapsed top front-right corner,
rubble pile at foot, roof penthouse), no interior, no window frames.
Budget: 800 triangles.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "office_lod1", "buildings", 800
WALL_TOP = 11.3
PARAPET_TOP = 11.8


def build(model):
    k = kit.Kit(model)

    # 1. Main building solid box: floors 1 to 3 (z 0.0 to 8.6)
    k.box((-7.6, -7.6, 0.0), (7.6, 7.2, 8.6), "rock_mid")
    k.collider((-7.6, -7.6, 0.0), (7.6, 7.2, 8.6), "body_lower")

    # Corner base plinths (defining exact 16x16 footprint bounds [-8.0, 8.0])
    k.box((-8.0, -8.0, 0.01), (-7.5, -7.5, 0.6), "rock_dark")
    k.box((-8.0, 7.2, 0.01), (-7.5, 7.7, 0.6), "rock_dark")
    k.box((7.2, -8.0, 0.01), (7.7, -7.5, 0.6), "rock_dark")

    # Plinth base trims (proud by 2cm, sitting between corner plinths and door)
    k.box((-7.5, 7.2, 0.04), (-1.7, 7.24, 0.55), "rock_dark")
    k.box((-0.5, 7.2, 0.04), (4.5, 7.24, 0.55), "rock_dark")
    k.box((-7.5, -7.62, 0.04), (7.2, -7.5, 0.55), "rock_dark")
    k.box((-7.62, -7.5, 0.04), (-7.5, 7.18, 0.55), "rock_dark")
    k.box((7.5, -7.5, 0.04), (7.62, 4.45, 0.55), "rock_dark")

    # Entrance door (flush dark box, bottom at 0.04 to avoid Z=0 plane)
    k.box((-1.7, 7.18, 0.04), (-0.5, 7.22, kit.DOOR_H), "rock_dark")
    k.box((-1.85, 7.15, 2.45), (-0.35, 7.6, 2.58), "metal")

    # Ground floor storefront glass strips
    k.box((-7.0, 7.18, 0.6), (-1.9, 7.22, 2.7), "glass_tint")
    k.box((-0.3, 7.18, 0.6), (4.0, 7.22, 2.7), "glass_tint")

    # 2. Floor 4: with collapsed top front-right corner!
    k.box((-7.6, -7.6, 8.6), (3.5, 7.2, WALL_TOP), "rock_mid")
    k.collider((-7.6, -7.6, 8.6), (3.5, 7.2, WALL_TOP), "body_f4_w")

    k.box((3.5, -7.6, 8.6), (7.6, 3.5, WALL_TOP), "rock_mid")
    k.collider((3.5, -7.6, 8.6), (7.6, 3.5, WALL_TOP), "body_f4_se")

    # Jagged fracture steps at collapsed corner
    k.box((3.5, 7.05, 8.6), (4.6, 7.2, 9.9), "rock_light")
    k.box((4.6, 7.05, 8.6), (5.6, 7.2, 9.3), "rock_mid")
    k.box((7.45, 3.5, 8.6), (7.6, 4.6, 9.9), "rock_light")
    k.box((7.45, 4.6, 8.6), (7.6, 5.6, 9.3), "rock_mid")

    # 3. Horizontal Glass-Band Windows on upper floors (Floors 2, 3, 4)
    # Front glass bands
    for flr, z0, z1 in ((2, 3.9, 5.2), (3, 6.6, 7.9), (4, 9.3, 10.6)):
        x_hi = 3.2 if flr == 4 else 7.2
        k.box((-7.2, 7.18, z0), (x_hi, 7.22, z1), "glass_tint")
        # Light spandrel band under window
        k.box((-7.6, 7.17, z0 - 0.1), (7.6, 7.21, z0), "rock_light")

    # Back glass bands
    for z0, z1 in ((3.9, 5.2), (6.6, 7.9), (9.3, 10.6)):
        k.box((-7.2, -7.62, z0), (7.2, -7.58, z1), "glass_tint")

    # West glass bands
    for z0, z1 in ((3.9, 5.2), (6.6, 7.9), (9.3, 10.6)):
        k.box((-7.62, -7.2, z0), (-7.58, 6.8, z1), "glass_tint")

    # East glass bands
    for flr, z0, z1 in ((2, 3.9, 5.2), (3, 6.6, 7.9), (4, 9.3, 10.6)):
        y_hi = 3.2 if flr == 4 else 6.8
        k.box((7.58, -7.2, z0), (7.62, y_hi, z1), "glass_tint")

    # 4. Rubble Pile at foot of collapsed corner (x [4.5, 8.0], y [4.5, 8.0])
    k.box((4.5, 4.5, 0.02), (8.0, 8.0, 0.9), "rock_dark")
    k.box((5.0, 5.0, 0.9), (7.8, 7.8, 1.8), "rock_mid")
    k.box((5.5, 5.5, 1.8), (7.4, 7.4, 2.6), "rock_light")
    k.box((4.0, 6.5, 0.06), (5.0, 7.8, 0.7), "rock_light")
    k.box((6.5, 4.0, 0.08), (7.8, 5.0, 0.7), "rock_light")
    # Rebar rods
    k.box((5.6, 5.6, 2.2), (5.7, 5.7, 3.4), "rust")
    k.box((6.6, 6.2, 1.8), (6.7, 6.3, 3.1), "rust")

    # 5. Roof: slabs, parapet, penthouse
    k.box((-7.6, -7.6, WALL_TOP), (3.5, 7.2, WALL_TOP + 0.15), "rock_dark", roof=True)
    k.box((3.5, -7.6, WALL_TOP), (7.6, 3.5, WALL_TOP + 0.15), "rock_dark", roof=True)

    # Parapet
    k.box((-7.6, -7.6, WALL_TOP + 0.15), (7.6, -7.45, PARAPET_TOP), "rock_mid", roof=True)
    k.box((-7.6, -7.45, WALL_TOP + 0.15), (-7.45, 7.05, PARAPET_TOP), "rock_mid", roof=True)
    k.box((-7.6, 7.05, WALL_TOP + 0.15), (3.5, 7.2, PARAPET_TOP), "rock_mid", roof=True)
    k.box((7.45, -7.45, WALL_TOP + 0.15), (7.6, 3.5, PARAPET_TOP), "rock_mid", roof=True)

    # Penthouse & AC
    k.box((-2.5, -2.5, WALL_TOP + 0.15), (1.5, 1.5, 13.5), "rock_mid", roof=True)
    k.box((-5.5, -1.5, WALL_TOP + 0.15), (-3.5, 1.0, 12.6), "metal", roof=True)

    # 6. Markers: lot_size only
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(18.0, 18.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
