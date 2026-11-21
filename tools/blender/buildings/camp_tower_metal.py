"""camp_tower_metal: 7 m metal scaffolding guard tower with sandbags, spotlight, ladder and light_1 empty.

Platform sits at z=7.0 m. Metal pipe scaffolding, sandbag parapet, spotlight mounted on front rail.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_tower_metal", "buildings", 3000
PLATFORM_Z = 7.0


def build(model):
    k = kit.Kit(model)

    # 4 corner steel scaffolding columns
    for x in (-1.25, 1.25):
        for y in (-1.25, 1.25):
            # Base plate on the ground
            k.solid((x - 0.2, y - 0.2, 0.0), (x + 0.2, y + 0.2, 0.05), "metal", base="baseplate")
            # Steel scaffolding leg
            k.solid((x - 0.08, y - 0.08, 0.05), (x + 0.08, y + 0.08, PLATFORM_Z - 0.2), "metal", base="leg")

    # Horizontal scaffolding crossbars and clean X-bracing between tiers
    cross_levels = (1.75, 3.5, 5.25)
    for z in cross_levels:
        k.box((-1.25, 1.2, z), (1.25, 1.28, z + 0.08), "metal")
        k.box((-1.25, -1.28, z), (1.25, -1.2, z + 0.08), "metal")
        k.box((-1.28, -1.25, z), (-1.2, 1.25, z + 0.08), "metal")
        k.box((1.2, -1.25, z), (1.28, 1.25, z + 0.08), "metal")

    # Clean diagonal X-braces for 3 structural tiers (z=0..1.75, 1.75..3.5, 3.5..5.25)
    tiers = ((0.1, 1.75), (1.83, 3.5), (3.58, 5.25))
    steps = 4
    for z0, z1 in tiers:
        dz = z1 - z0
        for s in range(steps):
            f0 = s / steps
            f1 = (s + 1) / steps
            sz0 = z0 + f0 * dz
            sz1 = z0 + f1 * dz
            y_a0 = -1.25 + f0 * 2.5
            y_a1 = -1.25 + f1 * 2.5
            y_b0 = 1.25 - f0 * 2.5
            y_b1 = 1.25 - f1 * 2.5
            # West side X
            k.box((-1.27, y_a0, sz0), (-1.21, y_a1, sz1), "rock_dark")
            k.box((-1.27, y_b1, sz0), (-1.21, y_b0, sz1), "rock_dark")
            # East side X
            k.box((1.21, y_a0, sz0), (1.27, y_a1, sz1), "rock_dark")
            k.box((1.21, y_b1, sz0), (1.27, y_b0, sz1), "rock_dark")
            # South side X
            x_a0 = -1.25 + f0 * 2.5
            x_a1 = -1.25 + f1 * 2.5
            x_b0 = 1.25 - f0 * 2.5
            x_b1 = 1.25 - f1 * 2.5
            k.box((x_a0, -1.27, sz0), (x_a1, -1.21, sz1), "rock_dark")
            k.box((x_b1, -1.27, sz0), (x_b0, -1.21, sz1), "rock_dark")

    # Sandbags around base for fortification (South, West, East)
    # South base sandbag wall (2 tiers)
    for tier, z0 in enumerate((0.0, 0.25)):
        shift = tier * 0.25
        for i in range(4):
            bx = -1.35 + shift + i * 0.65
            k.solid((bx, -1.75, z0), (bx + 0.6, -1.35, z0 + 0.25), "dirt", base="base_sandbag_s")
    # West base sandbag wall
    for tier, z0 in enumerate((0.0, 0.25)):
        shift = tier * 0.25
        for i in range(4):
            by = -1.35 + shift + i * 0.65
            k.solid((-1.75, by, z0), (-1.35, by + 0.6, z0 + 0.25), "dirt", base="base_sandbag_w")
    # East base sandbag wall
    for tier, z0 in enumerate((0.0, 0.25)):
        shift = tier * 0.25
        for i in range(4):
            by = -1.35 + shift + i * 0.65
            k.solid((1.35, by, z0), (1.75, by + 0.6, z0 + 0.25), "dirt", base="base_sandbag_e")

    # Platform deck at z=7.0 m (support frame from z=6.8 m)
    k.box((-1.45, -1.45, PLATFORM_Z - 0.2), (1.45, 1.45, PLATFORM_Z - 0.1), "metal")
    # Deck floor sections around ladder hatch (+Y side: x -0.35..0.35, y 0.65..1.45)
    k.solid((-1.45, -1.45, PLATFORM_Z - 0.1), (1.45, 0.65, PLATFORM_Z), "rock_dark", base="deck_south")
    k.solid((-1.45, 0.65, PLATFORM_Z - 0.1), (-0.35, 1.45, PLATFORM_Z), "rock_dark", base="deck_west")
    k.solid((0.35, 0.65, PLATFORM_Z - 0.1), (1.45, 1.45, PLATFORM_Z), "rock_dark", base="deck_east")

    # Metal railings (z=7.0 to 8.0 m)
    rail_top = PLATFORM_Z + 1.0
    for x in (-1.45, 1.45):
        for y in (-1.45, 1.45):
            k.box((x - 0.05, y - 0.05, PLATFORM_Z), (x + 0.05, y + 0.05, rail_top), "metal")
    # Intermediate railing stanchions
    k.box((-0.05, -1.45, PLATFORM_Z), (0.05, -1.4, rail_top), "metal")
    k.box((-1.45, -0.05, PLATFORM_Z), (-1.4, 0.05, rail_top), "metal")
    k.box((1.4, -0.05, PLATFORM_Z), (1.45, 0.05, rail_top), "metal")
    k.box((-0.35, 1.4, PLATFORM_Z), (-0.3, 1.45, rail_top), "metal")
    k.box((0.3, 1.4, PLATFORM_Z), (0.35, 1.45, rail_top), "metal")

    # Handrails (top rail and mid rail)
    for rz in (PLATFORM_Z + 0.5, rail_top):
        k.box((-1.45, -1.45, rz - 0.04), (1.45, -1.4, rz), "metal")
        k.box((-1.45, -1.45, rz - 0.04), (-1.4, 1.45, rz), "metal")
        k.box((1.4, -1.45, rz - 0.04), (1.45, 1.45, rz), "metal")
        k.box((-1.45, 1.4, rz - 0.04), (-0.35, 1.45, rz), "metal")
        k.box((0.35, 1.4, rz - 0.04), (1.45, 1.45, rz), "metal")

    # Sandbags on platform railing (front +Y side and sides for guard cover)
    for tier, z0 in enumerate((PLATFORM_Z, PLATFORM_Z + 0.22)):
        # West side of front rail
        k.solid((-1.35, 1.15, z0), (-0.45, 1.4, z0 + 0.22), "dirt", base="plat_sandbag_nw")
        # East side of front rail
        k.solid((0.45, 1.15, z0), (1.35, 1.4, z0 + 0.22), "dirt", base="plat_sandbag_ne")
        # Corner sandbags on south side
        k.solid((-1.35, -1.4, z0), (-0.75, -1.15, z0 + 0.22), "dirt", base="plat_sandbag_sw")
        k.solid((0.75, -1.4, z0), (1.35, -1.15, z0 + 0.22), "dirt", base="plat_sandbag_se")

    # Ladder on +Y side (visual only)
    k.box((-0.32, 1.28, 0.0), (-0.26, 1.36, PLATFORM_Z), "metal")
    k.box((0.26, 1.28, 0.0), (0.32, 1.36, PLATFORM_Z), "metal")
    z = 0.35
    while z < PLATFORM_Z - 0.1:
        k.box((-0.32, 1.29, z), (0.32, 1.35, z + 0.04), "metal")
        z += 0.35

    # Spotlight on the front rail (+Y, center at x=0.0, mounted on a bracket over the hatch)
    spot_z = PLATFORM_Z + 0.85
    # Mounting yoke/post from ladder rail
    k.box((-0.04, 1.38, PLATFORM_Z), (0.04, 1.46, spot_z - 0.05), "gun_black")
    k.box((-0.18, 1.40, spot_z - 0.06), (0.18, 1.46, spot_z + 0.06), "gun_black")

    # Spotlight cylinder housing (pointing +Y):
    # Built centered at (0, 0) in XY, from z=0 to z=0.30.
    # Rotated by -pi/2 around X so Z becomes +Y.
    spot = common.make_frustum(0.18, 0.18, 0.0, 0.30, segments=12, center=(0.0, 0.0), color="gun_black")
    common.transform(spot, rot=(-math.pi / 2, 0.0, 0.0), loc=(0.0, 1.42, spot_z))
    model.add(spot)
    # Spotlight lens at the front face
    lens = common.make_frustum(0.16, 0.16, 0.0, 0.02, segments=12, center=(0.0, 0.0), color="white")
    common.transform(lens, rot=(-math.pi / 2, 0.0, 0.0), loc=(0.0, 1.72, spot_z))
    model.add(lens)
    # Spotlight rim at the front face
    rim = common.make_frustum(0.19, 0.19, 0.0, 0.04, segments=12, center=(0.0, 0.0), color="metal")
    common.transform(rim, rot=(-math.pi / 2, 0.0, 0.0), loc=(0.0, 1.70, spot_z))
    model.add(rim)

    # Empties
    k.empty("light_1", (0.0, 1.76, spot_z), rot_z=0.0)
    k.empty("guard_1", (0.0, -0.3, PLATFORM_Z), rot_z=0.0)
    k.empty("ladder_bottom", (0.0, 1.7, 0.0), rot_z=math.pi)
    k.empty("ladder_top", (0.0, 0.6, PLATFORM_Z))
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(3.5, 3.5, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
