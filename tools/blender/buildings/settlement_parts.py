"""Parts of the survivor settlement (V1-15): palisade, guard platforms, stall, bank, tents, campfire.

Build coordinates: the lot is centred on the origin, +Y is the gate side (Godot -Z). The 5 m
clear lane from the gate (x -2..2) is kept free, so the player can walk straight in.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
import common  # noqa: E402

HALF = 20.0  # palisade line: a 40 m lot
GATE_CLEAR = 2.5  # the gate is 5 m wide
JAMB_X = 2.75  # gate jamb posts; their inner edge sits at 2.55 m, so the gap is 5.1 m
JAMB_R = 0.2
POST_R = 0.17  # ordinary posts: a 0.34 m log
WALL_COLLIDER_H = 2.8
LOG = ("bark_light", "bark_dark")


def palisade(k):
    """Sharpened log posts (cones, 6 sides) every ~0.6 m around the lot, a gap at the gate, and
    two lashing rails on the inside. Colliders are four solid wall pieces with the gate cut out."""
    model = k.model
    rng = random.Random(15)
    step = 2 * HALF / 80  # a post every 0.5 m, so the wall reads as solid
    xs = [-HALF + i * step for i in range(81)]
    spots = [(x, -HALF) for x in xs]  # south
    spots += [(x, HALF) for x in xs if abs(x) >= JAMB_X + JAMB_R + POST_R]  # north, gate left open
    spots += [(-HALF, y) for y in xs[1:-1]]  # west
    spots += [(HALF, y) for y in xs[1:-1]]  # east
    for x, y in spots:
        height = rng.uniform(2.85, 3.25)
        model.add(common.make_frustum(POST_R, 0.0, 0.0, height, segments=6, center=(x, y),
                                      rot_z=rng.uniform(0.0, 1.0), color="bark_light"))
    for jx in (-JAMB_X, JAMB_X):
        model.add(common.make_frustum(JAMB_R, 0.0, 0.0, 3.2, segments=6, center=(jx, HALF - 0.07),
                                      color="bark_dark"))

    for z0 in (0.9, 2.0):  # lashing rails, inside face
        k.box((-HALF, -HALF + 0.2, z0), (HALF, -HALF + 0.3, z0 + 0.1), "bark_dark")  # south
        k.box((-HALF + 0.2, -HALF, z0), (-HALF + 0.3, HALF, z0 + 0.1), "bark_dark")  # west
        k.box((HALF - 0.3, -HALF, z0), (HALF - 0.2, HALF, z0 + 0.1), "bark_dark")  # east
        k.box((-HALF, HALF - 0.3, z0), (-JAMB_X, HALF - 0.2, z0 + 0.1), "bark_dark")  # north, left
        k.box((JAMB_X, HALF - 0.3, z0), (HALF, HALF - 0.2, z0 + 0.1), "bark_dark")  # north, right

    top = WALL_COLLIDER_H
    k.collider((-HALF - 0.2, -HALF - 0.2, 0.0), (HALF + 0.2, -HALF + 0.2, top), base="palisade")
    k.collider((-HALF - 0.2, -HALF - 0.2, 0.0), (-HALF + 0.2, HALF + 0.2, top), base="palisade")
    k.collider((HALF - 0.2, -HALF - 0.2, 0.0), (HALF + 0.2, HALF + 0.2, top), base="palisade")
    k.collider((-HALF - 0.2, HALF - 0.2, 0.0), (-GATE_CLEAR - 0.05, HALF + 0.2, top), base="palisade")
    k.collider((GATE_CLEAR + 0.05, HALF - 0.2, 0.0), (HALF + 0.2, HALF + 0.2, top), base="palisade")


def guard_platform(k, x0, x1):
    """Wooden lookout on four legs, inside the wall: deck at 2.7 m, rails on three sides (open to the wall)."""
    y0, y1, top = 16.4, 19.1, 2.7
    k.solid((x0, y0, 2.5), (x1, y1, top), "bark_light", base="platform")
    for lx in (x0, x1 - 0.2):
        for ly in (y0, y1 - 0.2):
            k.box((lx, ly, 0.0), (lx + 0.2, ly + 0.2, 2.5), "bark_dark")
    k.box((x0, y0, top), (x0 + 0.08, y1, top + 0.9), "bark_light")  # west rail
    k.box((x1 - 0.08, y0, top), (x1, y1, top + 0.9), "bark_light")  # east rail
    # Back rail (toward the camp) has a gap where the inside ladder comes up onto the deck
    mid = (x0 + x1) / 2
    k.box((x0, y0, top), (mid - 0.5, y0 + 0.08, top + 0.9), "bark_light")
    k.box((mid + 0.5, y0, top), (x1, y0 + 0.08, top + 0.9), "bark_light")
    k.ladder(mid, y0 - 0.15, top)  # on the camp side, 0.15 m in front of the rail


def sandbags(k, x0, x1, y):
    """Two staggered rows of sandbags (0.6 x 0.45 x 0.3 m each), a low barrier."""
    count = int((x1 - x0) // 0.65)
    for row, z0 in enumerate((0.0, 0.3)):
        for i in range(count):
            x = x0 + row * 0.3 + i * 0.65
            k.solid((x, y, z0), (x + 0.6, y + 0.45, z0 + 0.3), "dirt", base="sandbag")


def crate(k, x, y, size=1.0):
    k.solid((x, y, 0.0), (x + size, y + size, size), "wood_crate", base="crate")


def tent(k, cx, cy, color):
    """A-frame tent: ridge along Y, 2.4 m wide, 3 m long, 1.6 m high. Door on the -Y end."""
    y0 = cy - 1.5
    tri = [(cx - 1.2, y0, 0.0), (cx + 1.2, y0, 0.0), (cx, y0, 1.6)]
    k.gable(tri, (0.0, 3.0, 0.0), color)
    k.box((cx - 0.45, y0 - 0.03, 0.0), (cx + 0.45, y0 + 0.01, 1.1), "bark_dark")


def campfire_ring(k):
    """Eight stones around the origin. No fire mesh: the game code adds the light."""
    for i in range(8):
        a = 2 * math.pi * i / 8
        stone = common.make_icosphere(subdiv=1, jitter=0.12, seed=i, radius=(0.2, 0.2, 0.15),
                                      loc=(1.1 * math.cos(a), 1.1 * math.sin(a), 0.15),
                                      rot=(0.0, 0.0, a), color="rock_mid")
        k.model.add(stone)


def water_barrel(k, x, y):
    k.model.add(common.make_frustum(0.42, 0.42, 0.0, 0.95, segments=8, center=(x, y), color="water_blue"))
    for z0 in (0.15, 0.7):  # two metal hoops
        k.model.add(common.make_frustum(0.43, 0.43, z0, z0 + 0.06, segments=8, center=(x, y), color="metal"))


def flagpole(k, x, y):
    k.model.add(common.make_frustum(0.06, 0.06, 0.0, 6.0, segments=6, center=(x, y), color="bark_dark"))
    k.box((x + 0.05, y - 0.03, 4.9), (x + 1.25, y + 0.03, 5.8), "red_paint")
