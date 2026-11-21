"""office: 4-floor concrete office building with glass-band windows.

Lot 18 x 18 m, building footprint 16 x 16 m (1 m inside lot on every side).
Outer bounds: X in [-8.0, 8.0], Y in [-8.0, 8.0].
Top front-right corner (+X, +Y on Floor 4) collapsed with jagged broken edge and rubble pile at the foot.
Ground floor lobby open with reception desk, sofas, and barricaded elevators (upper floors closed).
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "office", "buildings", 6000
F = kit.FLOOR_TOP
WALL_TOP = 11.3  # top of 4th floor wall
PARAPET_TOP = 11.8  # roof edge parapet


def build(model):
    k = kit.Kit(model)

    # 1. Base floor slab (raised F=0.1 m above ground)
    # Footprint: x [-7.6, 7.6], y [-7.6, 7.6]
    k.solid((-7.6, -7.6, 0.0), (7.6, 7.6, F), "rock_mid", base="floor")

    # Front entrance stoop / apron (y 7.2 to 7.8)
    k.solid((-2.0, 7.2, 0.0), (-0.2, 7.8, F), "rock_light", base="stoop")

    # Corner base plinths (extending to +/-8.0 to define exact 16x16 footprint)
    k.box((-8.0, -8.0, 0.0), (-7.5, -7.5, 0.6), "rock_dark")
    k.box((-8.0, 7.2, 0.0), (-7.5, 7.7, 0.6), "rock_dark")
    k.box((7.2, -8.0, 0.0), (7.7, -7.5, 0.6), "rock_dark")

    # Plinth base trims along walls
    k.box((-7.6, 7.15, 0.0), (4.5, 7.25, 0.55), "rock_dark")
    k.box((-7.6, -7.6, 0.0), (7.6, -7.5, 0.55), "rock_dark")
    k.box((-7.6, -7.6, 0.0), (-7.5, 7.2, 0.55), "rock_dark")
    k.box((7.5, -7.6, 0.0), (7.6, 4.5, 0.55), "rock_dark")

    # 2. Ground Floor Exterior Walls
    # Front entrance door (1.2 m wide: x in [-1.7, -0.5])
    front_door = ((-1.7, 7.05, F), (-0.5, 7.25, kit.DOOR_H))

    # Ground floor front wall (z F to 3.2 m) with door opening
    k.wall((-7.6, 7.05, F), (4.5, 7.2, 3.2), openings=[front_door], color="rock_mid", base="wall_front_gf")

    # Ground floor storefront glass windows flanking door
    # Left storefront window
    k.box((-7.0, 7.1, 0.6), (-1.9, 7.18, 2.7), "glass_tint")
    k.box((-7.0, 7.08, 0.55), (-1.9, 7.16, 0.65), "metal")
    k.box((-7.0, 7.08, 2.65), (-1.9, 7.16, 2.75), "metal")
    for xm in (-5.3, -3.6):
        k.box((xm - 0.04, 7.08, 0.6), (xm + 0.04, 7.16, 2.7), "metal")

    # Right storefront window
    k.box((-0.3, 7.1, 0.6), (4.0, 7.18, 2.7), "glass_tint")
    k.box((-0.3, 7.08, 0.55), (4.0, 7.16, 0.65), "metal")
    k.box((-0.3, 7.08, 2.65), (4.0, 7.16, 2.75), "metal")
    for xm in (1.1, 2.5):
        k.box((xm - 0.04, 7.08, 0.6), (xm + 0.04, 7.16, 2.7), "metal")

    # Entrance door frame & canopy
    k.frame(front_door, 7.2, 1, color="metal", t=0.08, depth=0.06, sill=False)
    k.box((-2.1, 7.1, 2.45), (-0.1, 7.8, 2.6), "metal")  # entrance canopy

    # Back wall GF (z F to 3.2)
    k.solid((-7.6, -7.6, F), (7.6, -7.45, 3.2), "rock_mid", base="wall_back_gf")
    # West wall GF
    k.solid((-7.6, -7.6, F), (-7.45, 7.2, 3.2), "rock_mid", base="wall_west_gf")
    # East wall GF (intact to x=4.5; past 4.5 rubble pile connects)
    k.solid((7.45, -7.6, F), (7.6, 4.5, 3.2), "rock_mid", base="wall_east_gf")
    k.solid((4.5, 4.5, F), (7.6, 7.2, 3.2), "rock_dark", base="wall_corner_gf")

    # 3. Upper Floors (Floors 2, 3, 4: z 3.2 to 11.3)
    # Marked roof=True so top preview looks directly into GF lobby
    for flr_base, flr_top in ((3.2, 5.9), (5.9, 8.6)):
        k.box((-7.6, 7.05, flr_base), (7.6, 7.2, flr_top), "rock_mid", roof=True)
        k.collider((-7.6, 7.05, flr_base), (7.6, 7.2, flr_top), f"wall_front_{int(flr_base)}")
        k.box((-7.6, -7.6, flr_base), (7.6, -7.45, flr_top), "rock_mid", roof=True)
        k.collider((-7.6, -7.6, flr_base), (7.6, -7.45, flr_top), f"wall_back_{int(flr_base)}")
        k.box((-7.6, -7.6, flr_base), (-7.45, 7.2, flr_top), "rock_mid", roof=True)
        k.collider((-7.6, -7.6, flr_base), (-7.45, 7.2, flr_top), f"wall_west_{int(flr_base)}")
        k.box((7.45, -7.6, flr_base), (7.6, 7.2, flr_top), "rock_mid", roof=True)
        k.collider((7.45, -7.6, flr_base), (7.6, 7.2, flr_top), f"wall_east_{int(flr_base)}")

    # Floor 4 (z 8.6 to 11.3): Collapsed top front-right corner!
    k.box((-7.6, -7.6, 8.6), (7.6, -7.45, WALL_TOP), "rock_mid", roof=True)
    k.collider((-7.6, -7.6, 8.6), (7.6, -7.45, WALL_TOP), "wall_back_f4")

    k.box((-7.6, -7.6, 8.6), (-7.45, 7.2, WALL_TOP), "rock_mid", roof=True)
    k.collider((-7.6, -7.6, 8.6), (-7.45, 7.2, WALL_TOP), "wall_west_f4")

    # Front wall Floor 4: intact up to x=3.5; collapsed from 3.5 to 7.6
    k.box((-7.6, 7.05, 8.6), (3.5, 7.2, WALL_TOP), "rock_mid", roof=True)
    k.collider((-7.6, 7.05, 8.6), (3.5, 7.2, WALL_TOP), "wall_front_f4")

    # East wall Floor 4: intact up to y=3.5; collapsed from 3.5 to 7.2
    k.box((7.45, -7.6, 8.6), (7.6, 3.5, WALL_TOP), "rock_mid", roof=True)
    k.collider((7.45, -7.6, 8.6), (7.6, 3.5, WALL_TOP), "wall_east_f4")

    # Jagged fracture edges on Floor 4
    k.box((3.5, 7.05, 8.6), (4.6, 7.2, 9.9), "rock_light", roof=True)
    k.box((4.6, 7.05, 8.6), (5.6, 7.2, 9.3), "rock_mid", roof=True)
    k.box((5.6, 7.05, 8.6), (6.4, 7.2, 8.9), "rock_light", roof=True)

    k.box((7.45, 3.5, 8.6), (7.6, 4.6, 9.9), "rock_light", roof=True)
    k.box((7.45, 4.6, 8.6), (7.6, 5.6, 9.3), "rock_mid", roof=True)
    k.box((7.45, 5.6, 8.6), (7.6, 6.4, 8.9), "rock_light", roof=True)

    # Exposed bent rebar rods at fracture
    for rz in (9.1, 9.7, 10.4):
        k.box((3.3, 7.1, rz), (5.0, 7.15, rz + 0.04), "rust", roof=True)
        k.box((7.1, 3.3, rz), (7.15, 5.0, rz + 0.04), "rust", roof=True)
    # Angled broken slab chunk dangling inside
    k.box((4.5, 4.5, 8.8), (6.8, 6.8, 9.15), "rock_dark", roof=True)

    # 4. Horizontal Glass-Band Windows (dark tinted glass with concrete spandrels)
    glass_bands = [(2, 3.9, 5.2), (3, 6.6, 7.9), (4, 9.3, 10.6)]

    for flr, z0, z1 in glass_bands:
        # Precast light concrete spandrel trim
        k.box((-7.61, 7.16, z0 - 0.1), (7.61, 7.2, z0), "rock_light", roof=True)
        k.box((-7.61, 7.16, z1), (7.61, 7.2, z1 + 0.1), "rock_light", roof=True)
        k.box((-7.61, -7.6, z0 - 0.1), (7.61, -7.56, z0), "rock_light", roof=True)
        k.box((-7.61, -7.6, z1), (7.61, -7.56, z1 + 0.1), "rock_light", roof=True)
        k.box((-7.6, -7.61, z0 - 0.1), (-7.56, 7.21, z0), "rock_light", roof=True)
        k.box((-7.6, -7.61, z1), (-7.56, 7.21, z1 + 0.1), "rock_light", roof=True)
        k.box((7.56, -7.61, z0 - 0.1), (7.6, 7.21, z0), "rock_light", roof=True)
        k.box((7.56, -7.61, z1), (7.6, 7.21, z1 + 0.1), "rock_light", roof=True)

        # Front glass band
        x_hi = 3.2 if flr == 4 else 7.2
        k.box((-7.2, 7.17, z0), (x_hi, 7.21, z1), "glass_tint", roof=True)
        x_m = -7.2
        while x_m < x_hi - 0.4:
            k.box((x_m - 0.04, 7.16, z0), (x_m + 0.04, 7.22, z1), "metal", roof=True)
            x_m += 1.8

        # Back glass band
        k.box((-7.2, -7.61, z0), (7.2, -7.57, z1), "glass_tint", roof=True)
        x_m = -7.2
        while x_m < 6.8:
            k.box((x_m - 0.04, -7.62, z0), (x_m + 0.04, -7.56, z1), "metal", roof=True)
            x_m += 1.8

        # West glass band
        k.box((-7.61, -7.2, z0), (-7.57, 6.8, z1), "glass_tint", roof=True)
        y_m = -7.2
        while y_m < 6.4:
            k.box((-7.62, y_m - 0.04, z0), (-7.56, y_m + 0.04, z1), "metal", roof=True)
            y_m += 1.8

        # East glass band
        y_hi = 3.2 if flr == 4 else 6.8
        k.box((7.57, -7.2, z0), (7.61, y_hi, z1), "glass_tint", roof=True)
        y_m = -7.2
        while y_m < y_hi - 0.4:
            k.box((7.56, y_m - 0.04, z0), (7.62, y_m + 0.04, z1), "metal", roof=True)
            y_m += 1.8

    # 5. Rubble Pile at the foot of the collapsed corner (x in [4.5, 8.0], y in [4.5, 8.0])
    # Massive fallen concrete slabs and boulders piled up at the foot
    rubble_mounds = [
        ((4.5, 4.5, 0.0), (8.0, 8.0, 0.9), "rock_dark", "rubble_base"),
        ((5.0, 5.0, 0.9), (7.8, 7.8, 1.8), "rock_mid", "rubble_mid"),
        ((5.5, 5.5, 1.8), (7.4, 7.4, 2.6), "rock_light", "rubble_top"),
    ]
    for lo, hi, color, base in rubble_mounds:
        k.solid(lo, hi, color, base=base)

    # Scattered shattered blocks around the base
    debris = [
        ((4.0, 6.5, 0.0), (5.0, 7.8, 0.7), "rock_light"),
        ((6.5, 4.0, 0.0), (7.8, 5.0, 0.7), "rock_light"),
        ((4.2, 5.2, 0.0), (5.2, 6.2, 0.5), "rock_mid"),
        ((7.2, 6.8, 0.9), (7.9, 7.9, 2.1), "rock_dark"),
        ((5.2, 6.2, 1.7), (6.5, 7.2, 2.4), "rock_light"),
    ]
    for lo, hi, color in debris:
        k.solid(lo, hi, color, base="debris")

    # Bent rebar pieces sticking out of rubble pile
    k.box((5.6, 5.6, 2.2), (5.7, 5.7, 3.4), "rust")
    k.box((6.6, 6.2, 1.8), (6.7, 6.3, 3.1), "rust")
    k.box((6.0, 7.1, 1.5), (6.1, 7.2, 2.7), "rust")

    # 6. Roof: concrete slab, low parapet, and rooftop elevator/HVAC penthouse
    # Main roof body (x [-7.6, 3.5], y [-7.6, 7.2])
    k.box((-7.6, -7.6, WALL_TOP), (3.5, 7.2, WALL_TOP + 0.15), "rock_dark", roof=True)
    k.collider((-7.6, -7.6, WALL_TOP), (3.5, 7.2, WALL_TOP + 0.15), "roof_slab_w")

    # South part of roof (x [3.5, 7.6], y [-7.6, 3.5])
    k.box((3.5, -7.6, WALL_TOP), (7.6, 3.5, WALL_TOP + 0.15), "rock_dark", roof=True)
    k.collider((3.5, -7.6, WALL_TOP), (7.6, 3.5, WALL_TOP + 0.15), "roof_slab_e")

    # Parapet walls on roof
    k.box((-7.6, -7.6, WALL_TOP + 0.15), (7.6, -7.45, PARAPET_TOP), "rock_mid", roof=True)
    k.collider((-7.6, -7.6, WALL_TOP + 0.15), (7.6, -7.45, PARAPET_TOP), "parapet_back")

    k.box((-7.6, -7.6, WALL_TOP + 0.15), (-7.45, 7.2, PARAPET_TOP), "rock_mid", roof=True)
    k.collider((-7.6, -7.6, WALL_TOP + 0.15), (-7.45, 7.2, PARAPET_TOP), "parapet_west")

    k.box((-7.6, 7.05, WALL_TOP + 0.15), (3.5, 7.2, PARAPET_TOP), "rock_mid", roof=True)
    k.collider((-7.6, 7.05, WALL_TOP + 0.15), (3.5, 7.2, PARAPET_TOP), "parapet_front")

    k.box((7.45, -7.6, WALL_TOP + 0.15), (7.6, 3.5, PARAPET_TOP), "rock_mid", roof=True)
    k.collider((7.45, -7.6, WALL_TOP + 0.15), (7.6, 3.5, PARAPET_TOP), "parapet_east")

    # Coping stones
    k.box((-7.62, -7.62, PARAPET_TOP), (7.62, -7.43, PARAPET_TOP + 0.08), "rock_light", roof=True)
    k.box((-7.62, -7.62, PARAPET_TOP), (-7.43, 7.22, PARAPET_TOP + 0.08), "rock_light", roof=True)
    k.box((-7.62, 7.03, PARAPET_TOP), (3.55, 7.22, PARAPET_TOP + 0.08), "rock_light", roof=True)
    k.box((7.43, -7.62, PARAPET_TOP), (7.62, 3.55, PARAPET_TOP + 0.08), "rock_light", roof=True)

    # Rooftop HVAC Penthouse (x in [-2.5, 1.5], y in [-2.5, 1.5])
    k.box((-2.5, -2.5, WALL_TOP + 0.15), (1.5, 1.5, 13.5), "rock_mid", roof=True)
    k.collider((-2.5, -2.5, WALL_TOP + 0.15), (1.5, 1.5, 13.5), "hvac_penthouse")
    k.box((-2.55, -2.55, 13.5), (1.55, 1.55, 13.62), "rock_dark", roof=True)

    # Chiller AC units
    k.box((-5.5, -1.5, WALL_TOP + 0.15), (-3.5, 1.0, 12.6), "metal", roof=True)
    k.collider((-5.5, -1.5, WALL_TOP + 0.15), (-3.5, 1.0, 12.6), "ac_unit_1")
    k.box((-5.3, -1.3, 12.6), (-3.7, 0.8, 12.65), "rock_dark", roof=True)

    k.box((-1.5, -5.5, WALL_TOP + 0.15), (1.0, -3.5, 12.6), "metal", roof=True)
    k.collider((-1.5, -5.5, WALL_TOP + 0.15), (1.0, -3.5, 12.6), "ac_unit_2")

    # 7. Ground Floor Interior: Open Lobby (x in [-7.2, 4.0], y in [-1.5, 7.0])
    k.box((-7.2, -1.5, F), (4.0, 7.0, F + 0.02), "rock_light")
    k.box((-7.2, -1.5, 3.2), (4.0, 7.0, 3.32), "white", roof=True)

    # Interior partition walls enclosing lobby
    k.solid((-7.2, -1.65, F), (4.0, -1.5, 3.2), "white", base="wall_lobby_back")
    k.solid((3.85, -1.5, F), (4.0, 7.05, 3.2), "white", base="wall_lobby_east")

    # Reception Desk: L-shaped modern counter
    # Front counter (x in [-0.5, 2.0], y in [3.5, 4.3], z F to 1.15)
    k.solid((-0.5, 3.5, F), (2.0, 4.3, 1.15), "wood_crate", base="rec_front")
    k.box((-0.55, 3.45, 1.15), (2.05, 4.35, 1.2), "bark_dark")
    # Side return counter (x in [1.3, 2.0], y in [2.0, 3.5])
    k.solid((1.3, 2.0, F), (2.0, 3.5, 1.15), "wood_crate", base="rec_side")
    k.box((1.25, 1.95, 1.15), (2.05, 3.5, 1.2), "bark_dark")
    # Computer monitor on desk
    k.box((0.4, 3.8, 1.2), (0.9, 3.86, 1.55), "gun_black")
    k.box((0.6, 3.82, 1.2), (0.7, 3.95, 1.25), "metal")
    # Swivel chair behind desk
    k.solid((0.4, 2.6, F), (0.9, 3.1, 0.45), "shirt_blue", base="chair_seat")
    k.box((0.4, 2.6, 0.45), (0.9, 2.75, 0.95), "shirt_blue")
    k.box((0.62, 2.82, 0.0), (0.68, 2.88, 0.45), "metal")

    # Waiting Area:
    # Sofa 1 (at x in [-6.5, -4.5], y in [4.2, 5.2])
    k.solid((-6.5, 4.2, F), (-4.5, 5.2, 0.45), "shirt_blue", base="sofa_1")
    k.box((-6.5, 5.0, 0.45), (-4.5, 5.2, 0.85), "shirt_blue")
    # Sofa 2 (perpendicular, at x in [-6.5, -5.5], y in [2.2, 4.2])
    k.solid((-6.5, 2.2, F), (-5.5, 4.2, 0.45), "shirt_blue", base="sofa_2")
    k.box((-6.5, 2.2, 0.45), (-6.3, 4.2, 0.85), "shirt_blue")
    # Coffee table
    k.solid((-5.2, 2.5, F), (-4.2, 3.5, 0.45), "bark_dark", base="table")
    k.box((-4.8, 2.8, 0.45), (-4.4, 3.2, 0.48), "white")

    # Architectural planter box
    k.solid((-7.0, 5.8, F), (-6.0, 6.8, 0.65), "rock_mid", base="planter")
    k.box((-6.9, 5.9, 0.65), (-6.1, 6.7, 0.72), "soil_dark")
    bush = common.make_icosphere(subdiv=1, radius=(0.45, 0.45, 0.4), loc=(-6.5, 6.3, 1.05), color="leaf_dark")
    model.add(bush)

    # Back Wall: Elevators & Stairwell (Barricaded shut, upper floors closed)
    # Elevator doors
    k.box((-1.8, -1.52, F), (-0.4, -1.48, 2.3), "metal")
    k.box((0.2, -1.52, F), (1.6, -1.48, 2.3), "metal")
    k.box((-1.2, -1.49, 2.45), (-1.0, -1.47, 2.55), "brass")
    k.box((0.8, -1.49, 2.45), (1.0, -1.47, 2.55), "brass")

    # Heavy timber boards barricading elevators
    k.box((-2.0, -1.45, 0.5), (1.8, -1.35, 0.75), "wood_crate")
    k.box((-2.0, -1.45, 1.2), (1.8, -1.35, 1.45), "wood_crate")
    k.box((-1.7, -1.44, F), (1.5, -1.36, 2.2), "bark_dark")

    # Barricaded stairwell door
    k.box((-5.0, -1.52, F), (-3.8, -1.48, 2.2), "metal")
    k.solid((-4.8, -1.45, F), (-3.8, -0.8, 1.3), "metal", base="filing_cab_1")
    k.solid((-4.2, -1.3, 1.3), (-3.6, -0.7, 2.1), "metal", base="filing_cab_2")
    k.box((-4.6, -1.49, 2.35), (-4.2, -1.47, 2.55), "red_paint")

    # 8. Empties / Markers
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(18.0, 18.0, 1.0))
    k.empty("door_1", (-1.1, 7.125, F))

    # 3 loot empties
    # loot_1: behind reception desk on counter
    k.empty("loot_1", (0.8, 3.8, 1.2))
    # loot_2: on waiting area coffee table
    k.empty("loot_2", (-4.7, 3.0, 0.48))
    # loot_3: near barricaded filing cabinets
    k.empty("loot_3", (-3.2, -0.6, F))

    # 3 spawn empties
    # spawn_1: zombie behind reception desk
    k.empty("spawn_1", (0.7, 2.2, F))
    # spawn_2: zombie in waiting area
    k.empty("spawn_2", (-4.0, 4.8, F))
    # spawn_3: zombie outside by the rubble pile
    k.empty("spawn_3", (6.5, 6.5, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
