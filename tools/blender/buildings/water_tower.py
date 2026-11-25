"""water_tower: 10 m tall wooden tank on 4 legs with cross braces and ladder. Not enterable.

Legs and platform have colliders; ladder is visual only with ladder_bottom/ladder_top markers.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "water_tower", "buildings", 3500
PLAT_Z = 6.2


def build(model):
    k = kit.Kit(model)
    for x in (-1.35, 1.35):
        for y in (-1.35, 1.35):
            k.solid((x - 0.12, y - 0.12, 0.0), (x + 0.12, y + 0.12, PLAT_Z), "bark_dark", base="leg")

    # Horizontal cross braces on all four sides
    for z in (1.6, 3.2, 4.8):
        k.box((-1.35, 1.29, z), (1.35, 1.41, z + 0.12), "bark_light")
        k.box((-1.35, -1.41, z), (1.35, -1.29, z + 0.12), "bark_light")
        k.box((-1.41, -1.35, z), (-1.29, 1.35, z + 0.12), "bark_light")
        k.box((1.29, -1.35, z), (1.41, 1.35, z + 0.12), "bark_light")

    # Diagonal X-brace beams on front/back
    for z0, z1 in ((0.2, 1.6), (1.72, 3.2), (3.32, 4.8)):
        for y in (-1.35, 1.35):
            k.box((-1.3, y - 0.04, (z0 + z1) / 2 - 0.06), (1.3, y + 0.04, (z0 + z1) / 2 + 0.06), "bark_dark")

    # Support platform
    k.solid((-1.8, -1.8, PLAT_Z), (1.8, 1.8, PLAT_Z + 0.2), "bark_light", base="platform")

    # Wooden cylindrical tank + metal hoops + conical roof (total height ~10.2 m)
    model.add(common.make_frustum(1.65, 1.65, PLAT_Z + 0.2, 9.2, segments=12, color="wood_crate"))
    for hz in (6.8, 7.8, 8.8):
        model.add(common.make_frustum(1.69, 1.69, hz, hz + 0.08, segments=12, color="metal"))
    model.add(common.make_frustum(1.85, 0.0, 9.2, 10.2, segments=12, color="rock_dark"))
    k.collider((-1.65, -1.65, PLAT_Z + 0.2), (1.65, 1.65, 10.0), "tank")

    # Visual ladder on +Y side
    k.ladder(0.0, 1.45, PLAT_Z + 0.2, color="bark_light", half_w=0.28, rung_gap=0.35)

    k.empty("ladder_bottom", (0.0, 1.85, 0.0), rot_z=math.pi)
    k.empty("ladder_top", (0.0, 1.0, PLAT_Z + 0.2))
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(4.0, 4.0, 1.0))
    k.empty("loot_1", (0.4, -0.4, 0.0))
    k.empty("spawn_1", (2.4, 0.0, 0.0))
    k.empty("spawn_2", (-2.4, 0.5, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
