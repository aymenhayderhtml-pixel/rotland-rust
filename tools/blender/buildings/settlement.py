"""settlement: walled survivor camp, 40 x 40 m lot. Palisade wall with a 5 m gate on the +Y side
(Godot -Z), two guard platforms beside the gate, a trader stall, a bank hut, two tents, a
campfire ring, a water barrel, crates, sandbags and a flag pole.

Markers are empties (npc_*, player_respawn, gate, safe_zone, campfire, lot_size). Facing: a
marker's rot_z turns its +Y forward, so the NPC looks the way the marker points.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402
import settlement_parts as parts  # noqa: E402

NAME, CATEGORY, BUDGET = "settlement", "buildings", 7200
HALF = parts.HALF
SAFE_HALF = 17.5  # the safe zone sits inside the wall, 2.5 m clear of it
LOG = ("bark_light", "bark_dark")


def face(x, y, tx, ty):
    """Marker turn (radians) so the NPC's forward (+Y) points at (tx, ty)."""
    return math.atan2(-(tx - x), ty - y)


def trader_stall(k):
    """Counter on the lane side, crates behind, awning on four posts. The trader stands behind the counter."""
    k.solid((-7.6, 6.6, 0.0), (-6.8, 8.4, 1.0), "bark_light", base="counter")
    parts.crate(k, -10.4, 6.4)
    parts.crate(k, -10.4, 7.5)
    for px, py in ((-10.7, 5.9), (-6.3, 5.9), (-10.7, 8.8), (-6.3, 8.8)):
        k.box((px, py, 0.0), (px + 0.15, py + 0.15, 2.4), "bark_dark")
    k.box((-10.9, 5.6, 2.4), (-6.1, 9.4, 2.55), "red_paint", roof=True)


def bank_shack(k):
    """Solid hut with a counter window facing the lane, a sign board above it and a flat roof."""
    window = ((5.9, 7.0, 1.0), (6.3, 8.0, 1.9))
    k.wall((6.0, 6.0, 0.0), (6.2, 9.0, 2.6), openings=[window], colors=LOG, course=0.35, base="bank_wall")
    k.wall((6.0, 8.8, 0.0), (10.0, 9.0, 2.6), colors=LOG, course=0.35, base="bank_wall")
    k.wall((9.8, 6.0, 0.0), (10.0, 9.0, 2.6), colors=LOG, course=0.35, base="bank_wall")
    k.wall((6.0, 6.0, 0.0), (10.0, 6.2, 2.6), colors=LOG, course=0.35, base="bank_wall")
    k.box((5.8, 5.8, 2.6), (10.2, 9.2, 2.8), "rock_dark", roof=True)
    k.solid((6.3, 6.6, 0.0), (6.8, 8.4, 1.0), "bark_light", base="bank_counter")
    k.box((5.85, 6.6, 2.05), (6.0, 8.4, 2.45), "wood_crate")  # sign board
    k.box((5.8, 6.9, 2.15), (5.85, 8.1, 2.35), "white")  # sign plate


def guild_hall(k):
    """Timber guild hall on the east side: log walls with a walk-in door on the lane side (-Y),
    a gable roof, a sign board over the door, and a long table, two benches and a banner inside."""
    x0, x1, y0, y1, top = 12.5, 18.5, 4.0, 10.0, 2.4
    cx, ridge_z = 15.5, 3.6
    door = ((14.6, y0, 0.0), (16.4, y0 + 0.2, 2.2))
    k.wall((x0, y0, 0.0), (x1, y0 + 0.2, top), openings=[door], colors=LOG, course=0.8, base="guild_wall")
    k.wall((x0, y1 - 0.2, 0.0), (x1, y1, top), colors=LOG, course=0.8, base="guild_wall")
    k.wall((x0, y0 + 0.2, 0.0), (x0 + 0.2, y1 - 0.2, top), colors=LOG, course=0.8, base="guild_wall")
    k.wall((x1 - 0.2, y0 + 0.2, 0.0), (x1, y1 - 0.2, top), colors=LOG, course=0.8, base="guild_wall")

    # Gable roof, ridge along X (eaves overhang 0.3 m on the door and back sides)
    k.roof((cx, 7.0, ridge_z), (cx, 3.7, top), span=(1.0, 0.0, 0.0), width=6.4, color="rock_dark")
    k.roof((cx, 7.0, ridge_z), (cx, 10.3, top), span=(1.0, 0.0, 0.0), width=6.4, color="rock_dark")
    k.gable([(x0, y0, top), (x0, y1, top), (x0, 7.0, ridge_z)], (0.2, 0.0, 0.0), "bark_dark", roof=True)
    k.gable([(x1, y0, top), (x1, y1, top), (x1, 7.0, ridge_z)], (-0.2, 0.0, 0.0), "bark_dark", roof=True)

    # Door trim and the guild sign board over the doorway
    k.frame(door, y0, -1, "bark_dark")
    k.box((14.8, 3.92, 2.45), (16.2, 4.0, 2.8), "wood_crate")
    k.box((14.95, 3.88, 2.52), (16.05, 3.92, 2.73), "brass")

    # Inside: long table with two trestle legs, a bench either side, banner on the back wall
    k.box((13.9, 6.5, 0.78), (17.1, 7.3, 0.9), "bark_light")
    for lx in (14.1, 16.7):
        k.box((lx, 6.6, 0.0), (lx + 0.2, 7.2, 0.78), "bark_dark")
    k.box((13.9, 5.9, 0.0), (17.1, 6.25, 0.45), "bark_dark")
    k.box((13.9, 7.55, 0.0), (17.1, 7.9, 0.45), "bark_dark")
    k.box((15.0, 9.7, 1.2), (16.0, 9.8, 2.2), "red_paint")


def camp_clutter(k):
    """Visual plan step 7: life along the walls, clear of the walk points, beds and farm plots
    (data/camp_life.json, data/farming.json; their Godot z is minus Blender y)."""
    # Firewood pile behind the chopping spot (west wall)
    for i, (y, z) in enumerate(((-1.2, 0.0), (-2.0, 0.0), (-2.8, 0.0), (-1.6, 0.3), (-2.4, 0.3), (-2.0, 0.6))):
        log = common.make_frustum(0.16, 0.16, 0.0, 1.6, segments=8, color="bark_light" if i % 2 else "bark_dark")
        common.transform(log, rot=(0.0, math.pi / 2, 0.0), loc=(-18.6, y, 0.16 + z))
        k.model.add(log)
    k.collider((-18.6, -3.0, 0.0), (-17.0, -0.9, 0.95), "woodpile")
    # Crates and barrels along the back wall
    parts.crate(k, -7.0, -18.3)
    parts.crate(k, -5.9, -18.3, size=0.8)
    k.barrel(-9.0, -18.4, "red_paint")
    k.barrel(-9.7, -17.8, "shirt_blue")
    k.barrel(6.0, -18.4, "rock_mid")
    k.crate(7.2, -18.3, 0.7)
    # Goods on the trader's counter: cans and a bottle
    for i, (y, color) in enumerate(((6.8, "red_paint"), (7.1, "gun_black"), (7.4, "brass"), (7.8, "water_blue"),
                                    (8.1, "red_paint"))):
        k.box((-7.4, y - 0.07, 1.0), (-7.2, y + 0.07, 1.12 + 0.04 * (i % 2)), color)
    # Stripes on the red awning
    for x in (-10.2, -9.0, -7.8, -6.6):
        k.box((x, 5.6, 2.55), (x + 0.5, 9.4, 2.56), "white", roof=True)
    # Lanterns on posts inside the gate
    for x in (-2.6, 2.6):
        k.box((x - 0.05, 15.8, 0.0), (x + 0.05, 15.9, 2.3), "bark_dark")
        k.box((x - 0.12, 15.73, 2.3), (x + 0.12, 15.97, 2.55), "brass")
        k.box((x - 0.15, 15.7, 2.55), (x + 0.15, 16.0, 2.6), "gun_black")


def build(model):
    k = kit.Kit(model)
    parts.palisade(k)
    parts.guard_platform(k, -8.6, -6.0)
    parts.guard_platform(k, 6.0, 8.6)
    parts.sandbags(k, -5.6, -3.0, 17.0)
    parts.sandbags(k, 3.0, 5.6, 17.0)
    trader_stall(k)
    bank_shack(k)
    guild_hall(k)
    parts.tent(k, -13.0, -8.0, "pine_mid")
    parts.tent(k, 12.0, -8.0, "rock_mid")
    parts.campfire_ring(k)
    parts.water_barrel(k, -4.5, -2.5)
    parts.crate(k, 8.0, 2.5)
    parts.crate(k, 9.2, 2.5)
    parts.flagpole(k, 0.0, -15.0)
    camp_clutter(k)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(2 * HALF, 2 * HALF, 1.0))
    k.empty("safe_zone", (0.0, 0.0, 0.0), scale=(SAFE_HALF, SAFE_HALF, 1.0))
    k.empty("campfire", (0.0, 0.0, 0.0))
    k.empty("gate", (0.0, HALF, 0.0))
    k.empty("player_respawn", (0.0, 2.6, 0.0))
    k.empty("npc_trader", (-8.2, 7.7, 0.0), rot_z=face(-8.2, 7.7, -7.2, 7.7))  # faces the lane
    k.empty("npc_banker", (7.2, 7.5, 0.0), rot_z=face(7.2, 7.5, 6.2, 7.5))  # faces the window
    # Guild hall: the keeper stands behind the long table, looking at the door (-Y)
    k.empty("npc_guild", (15.5, 8.0, 0.0), rot_z=face(15.5, 8.0, 15.5, 4.0))
    k.empty("guild_door", (15.5, 3.8, 0.0))  # just outside the doorway, on the lane side
    k.empty("npc_guard_1", (-7.3, 17.9, 2.7))  # on the platforms, facing out (+Y)
    k.empty("npc_guard_2", (7.3, 17.9, 2.7))
    # Inside ladders up to each guard deck (ladder at x = platform centre, y = 16.25).
    # ladder_bottom: on the ground 0.4 m in front of the ladder, facing it (+Y).
    # ladder_top: on the deck 0.5 m past the top of the ladder, where the climber steps off.
    k.empty("ladder_bottom_1", (-7.3, 15.85, 0.0))
    k.empty("ladder_top_1", (-7.3, 16.75, 2.7))
    k.empty("ladder_bottom_2", (7.3, 15.85, 0.0))
    k.empty("ladder_top_2", (7.3, 16.75, 2.7))
    k.empty("npc_survivor_1", (-9.8, -7.6, 0.0), rot_z=face(-9.8, -7.6, 0.0, 0.0))  # by the campfire
    k.empty("npc_survivor_2", (-2.9, -2.2, 0.0), rot_z=face(-2.9, -2.2, 0.0, 0.0))
    k.empty("npc_survivor_3", (9.4, -7.2, 0.0), rot_z=face(9.4, -7.2, 0.0, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
