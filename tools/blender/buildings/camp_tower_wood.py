"""camp_tower_wood: 6 m wooden guard tower with ladder, platform, rails, canopy and guard_1 empty.

Platform sits at z=6.0 m. Legs and platform have colliders. Visual ladder on the +Y side.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_tower_wood", "buildings", 2500
PLATFORM_Z = 6.0


def build(model):
    k = kit.Kit(model)

    # 4 main corner legs
    for x in (-1.2, 1.2):
        for y in (-1.2, 1.2):
            k.solid((x - 0.12, y - 0.12, 0.0), (x + 0.12, y + 0.12, PLATFORM_Z - 0.2), "bark_dark", base="leg")

    # Diagonal and horizontal cross-braces on all 4 sides (side braces fit between front/back)
    for z in (1.6, 3.4, 4.8):
        k.box((-1.2, 1.15, z), (1.2, 1.25, z + 0.12), "bark_light")
        k.box((-1.2, -1.25, z), (1.2, -1.15, z + 0.12), "bark_light")
        k.box((-1.25, -1.15, z), (-1.15, 1.15, z + 0.12), "bark_light")
        k.box((1.15, -1.15, z), (1.25, 1.15, z + 0.12), "bark_light")

    # Platform frame and floor (with hatch opening for ladder on +Y side: x -0.35..0.35, y 0.65..1.35)
    # Support beams under deck
    k.box((-1.35, -1.35, PLATFORM_Z - 0.25), (1.35, 1.35, PLATFORM_Z - 0.15), "bark_dark")
    # Deck floorboards around hatch
    k.solid((-1.4, -1.4, PLATFORM_Z - 0.15), (1.4, 0.65, PLATFORM_Z), "wood_crate", base="deck_south")
    k.solid((-1.4, 0.65, PLATFORM_Z - 0.15), (-0.35, 1.4, PLATFORM_Z), "wood_crate", base="deck_west")
    k.solid((0.35, 0.65, PLATFORM_Z - 0.15), (1.4, 1.4, PLATFORM_Z), "wood_crate", base="deck_east")

    # Platform railings (z=6.0 to 6.9 m)
    rail_top = PLATFORM_Z + 0.9
    # South rail (full width)
    k.solid((-1.4, -1.4, PLATFORM_Z), (1.4, -1.32, rail_top), "bark_dark", base="rail_south")
    # West rail (fits between south and north rails)
    k.solid((-1.4, -1.32, PLATFORM_Z), (-1.32, 1.32, rail_top), "bark_dark", base="rail_west")
    # East rail (fits between south and north rails)
    k.solid((1.32, -1.32, PLATFORM_Z), (1.4, 1.32, rail_top), "bark_dark", base="rail_east")
    # North rail (+Y side, with gap for ladder hatch)
    k.solid((-1.4, 1.32, PLATFORM_Z), (-0.35, 1.4, rail_top), "bark_dark", base="rail_north_w")
    k.solid((0.35, 1.32, PLATFORM_Z), (1.4, 1.4, rail_top), "bark_dark", base="rail_north_e")

    # Intermediate horizontal rails for detail (proud of main rail by 2 cm)
    for rz in (PLATFORM_Z + 0.3, PLATFORM_Z + 0.6):
        k.box((-1.42, -1.42, rz), (1.42, -1.30, rz + 0.08), "bark_light")
        k.box((-1.42, -1.30, rz), (-1.30, 1.30, rz + 0.08), "bark_light")
        k.box((1.30, -1.30, rz), (1.42, 1.30, rz + 0.08), "bark_light")
        k.box((-1.42, 1.30, rz), (-0.37, 1.42, rz + 0.08), "bark_light")
        k.box((0.37, 1.30, rz), (1.42, 1.42, rz + 0.08), "bark_light")

    # Ladder on +Y side (visual only)
    k.box((-0.32, 1.28, 0.02), (-0.26, 1.36, PLATFORM_Z - 0.02), "bark_light")
    k.box((0.26, 1.28, 0.02), (0.32, 1.36, PLATFORM_Z - 0.02), "bark_light")
    z = 0.35
    while z < PLATFORM_Z - 0.1:
        k.box((-0.25, 1.29, z), (0.25, 1.35, z + 0.05), "bark_light")
        z += 0.35

    # 4 roof support posts rising above platform railing
    for x in (-1.3, 1.3):
        for y in (-1.3, 1.3):
            k.box((x - 0.08, y - 0.08, rail_top), (x + 0.08, y + 0.08, PLATFORM_Z + 1.68), "bark_dark")

    # Wooden canopy roof (gable roof along X, ridge at z=8.3 m, eaves at z=7.7 m)
    roof_ridge = (0.0, 0.0, PLATFORM_Z + 2.3)
    k.roof(roof_ridge, (0.0, 1.6, PLATFORM_Z + 1.7), span=(1.0, 0.0, 0.0), width=3.22, color="wood_crate")
    k.roof(roof_ridge, (0.0, -1.6, PLATFORM_Z + 1.7), span=(1.0, 0.0, 0.0), width=3.20, color="wood_crate")
    for rx in (-1.55, 1.55 - 0.1):
        k.gable([(rx, -1.55, PLATFORM_Z + 1.58), (rx, 1.55, PLATFORM_Z + 1.58), (rx, 0.0, PLATFORM_Z + 2.18)],
                (0.1, 0.0, 0.0), "bark_dark", roof=True)

    # Markers
    k.empty("guard_1", (0.0, 0.0, PLATFORM_Z), rot_z=0.0)
    k.empty("ladder_bottom", (0.0, 1.7, 0.0), rot_z=math.pi)
    k.empty("ladder_top", (0.0, 0.6, PLATFORM_Z))
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(3.0, 3.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
