"""watchtower: 3 x 3 m wooden tower, 7 m to the platform, with a visual ladder on the +Y side.

Legs and platform have colliders (you can stand on the platform). The ladder is visual only: the
climber code moves the player along it using the ladder_bottom / ladder_top markers.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "watchtower", "buildings", 3000
PLATFORM_Z = 7.0


def build(model):
    k = kit.Kit(model)
    for x in (-1.35, 1.35):
        for y in (-1.35, 1.35):
            k.solid((x - 0.1, y - 0.1, 0.0), (x + 0.1, y + 0.1, PLATFORM_Z - 0.2), "bark_dark", base="leg")

    # Cross braces on all four sides, between the legs
    for z in (2.0, 4.0, 5.6):
        k.box((-1.35, 1.3, z), (1.35, 1.4, z + 0.12), "bark_light")
        k.box((-1.35, -1.4, z), (1.35, -1.3, z + 0.12), "bark_light")
        k.box((-1.4, -1.3, z), (-1.3, 1.3, z + 0.12), "bark_light")
        k.box((1.3, -1.3, z), (1.4, 1.3, z + 0.12), "bark_light")

    k.solid((-1.5, -1.5, PLATFORM_Z - 0.2), (1.5, 1.5, PLATFORM_Z), "wood_crate", base="platform")
    rail_top = PLATFORM_Z + 0.9
    k.box((-1.5, -1.5, PLATFORM_Z + 0.02), (1.5, -1.42, rail_top), "bark_dark")
    k.box((-1.5, 1.42, PLATFORM_Z + 0.02), (1.5, 1.5, rail_top), "bark_dark")
    k.box((-1.5, -1.42, PLATFORM_Z + 0.02), (-1.42, 1.42, rail_top), "bark_dark")
    k.box((1.42, -1.42, PLATFORM_Z + 0.02), (1.5, 1.42, rail_top), "bark_dark")

    # Ladder on the +Y side, between the front legs (visual only)
    k.box((-0.32, 1.2, 0.02), (-0.26, 1.3, PLATFORM_Z - 0.22), "bark_light")
    k.box((0.26, 1.2, 0.02), (0.32, 1.3, PLATFORM_Z - 0.22), "bark_light")
    z = 0.4
    while z < PLATFORM_Z - 0.4:
        k.box((-0.25, 1.22, z), (0.25, 1.28, z + 0.05), "bark_light")
        z += 0.4

    # Climb markers for the ladder above: foot 0.4 m in front of it (+Y), facing it (-Y);
    # step-off point on the deck 0.5 m from the ladder top.
    k.empty("ladder_bottom", (0.0, 1.7, 0.0), rot_z=math.pi)
    k.empty("ladder_top", (0.0, 0.8, PLATFORM_Z))
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(3.0, 3.0, 1.0))
    k.empty("loot_1", (0.5, -0.5, 0.0))  # on the ground, inside the four legs (the player can reach it)
    k.empty("spawn_1", (2.4, 0.0, 0.0))
    k.empty("spawn_2", (-2.4, 0.5, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
