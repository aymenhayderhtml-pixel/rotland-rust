"""Luggage props for the jam: luggage_pile, suitcase_open, clothes_spill (budgets 6000 each)."""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import car_parts  # noqa: E402
import common  # noqa: E402
from jam_util import bevel_bm, box  # noqa: E402

LUGGAGE = ["leather", "soda_purple", "wood_crate", "dog_tan", "canvas_khaki", "car_blue", "car_red"]
CLOTH = ["shirt_blue", "shirt_red", "gown_blue", "pants_denim", "white", "shirt_zombie"]
BUDGET = 10000  # new: DECISIONS 2026-11-26 (props budget, the luggage and cloth props are detailed up to it)


def _place(bms, cx, cy, z0, rot):
    """Turns and moves each bmesh of a prop part; the parts are built around the origin."""
    for bm in bms:
        common.transform(bm, rot=(0.0, 0.0, rot), loc=(cx, cy, z0))
    return bms


def _case(cx, cy, z0, size, rot, color, rng):
    """A hard suitcase: shell, corner caps, four wheels, zip, carry handle. Returns bmeshes."""
    sx, sy, sz = size
    h = sz / 2
    pieces = [bevel_bm(common.make_box(size), 0.012, color)]  # rounded shell
    for fx in (-1, 1):
        for fy in (-1, 1):
            pieces.append(common.make_box((0.05, 0.05, 0.04), loc=(fx * sx / 2, fy * sy / 2, h), color="metal"))
            pieces.append(common.make_frustum(0.03, 0.03, -h, -h + 0.03, segments=8,
                                              center=(fx * (sx / 2 - 0.05), fy * (sy / 2 - 0.05)),
                                              color="gun_black"))
    pieces.append(common.make_box((sx * 0.9, 0.012, 0.012), loc=(0.0, sy / 2 + 0.003, h - 0.02), color="gun_black"))
    pieces.append(car_parts._beam((-0.12, 0.0, h + 0.02), (0.12, 0.0, h + 0.02), 0.014, color="metal", segments=8))
    if rng.random() < 0.5:
        pieces.append(common.make_box((0.04, sy + 0.02, sz + 0.02), loc=(sx * 0.2, 0.0, 0.0), color="rust"))
    return _place(pieces, cx, cy, z0 + h, rot)


def _luggage_pile(model):
    rng = random.Random(41)
    for cx, cy in [(-0.45, -0.25), (0.25, -0.25), (-0.05, 0.3), (0.7, -0.35)]:
        size = (rng.uniform(0.45, 0.7), rng.uniform(0.3, 0.45), rng.uniform(0.2, 0.3))
        for bm in _case(cx, cy, 0.0, size, rng.uniform(-0.25, 0.25), rng.choice(LUGGAGE), rng):
            model.add(bm)
    for cx, cy in [(-0.1, -0.12), (0.45, 0.25), (-0.5, 0.1)]:
        size = (rng.uniform(0.4, 0.6), rng.uniform(0.3, 0.4), rng.uniform(0.2, 0.28))
        for bm in _case(cx, cy, 0.3, size, rng.uniform(-0.3, 0.3), rng.choice(LUGGAGE), rng):
            model.add(bm)
    for k, (cx, cy, color) in enumerate(((-0.6, 0.45, "canvas_khaki"), (0.75, 0.6, "soil_dark"),
                                         (0.2, -0.7, "canvas_khaki"))):  # duffel bags, slumped
        bag = common.make_icosphere(subdiv=3, jitter=0.04, seed=5 + k, radius=(0.25, 0.18, 0.2), color=color)
        model.add(_place([bag], cx, cy, 0.2, 0.2 * k)[0])
    model.add(box((-0.2, -0.3, 0.6), (0.1, -0.22, 0.62), "metal"))


def _suitcase_open(model):
    rng = random.Random(42)
    for bm in _case(0.0, 0.0, 0.0, (0.7, 0.45, 0.2), 0.0, "leather", rng):
        model.add(bm)
    model.add(box((-0.33, -0.205, 0.2), (0.33, 0.205, 0.22), "gun_black"))  # the open tray inside
    for cy, color in ((-0.12, "shirt_blue"), (0.02, "white"), (0.1, "pants_denim")):
        blob = common.make_icosphere(subdiv=4, jitter=0.25, seed=int(cy * 100) + 50,
                                     radius=(0.22, 0.13, 0.03), color=color)
        model.add(_place([blob], rng.uniform(-0.05, 0.05), cy, 0.22, rng.uniform(-0.2, 0.2))[0])
    sleeve = common.make_icosphere(subdiv=4, jitter=0.25, seed=9, radius=(0.12, 0.08, 0.03), color="shirt_red")
    model.add(_place([sleeve], 0.3, -0.05, 0.22, 0.3)[0])  # a shirt hanging over the edge
    lid = box((-0.35, -0.45, -0.015), (0.35, 0.0, 0.015), "leather")
    common.transform(lid, rot=(-math.radians(100.0), 0.0, 0.0), loc=(0.0, 0.225, 0.2))
    model.add(lid)  # lid opened up and back


def _clothes_spill(model):
    rng = random.Random(43)
    for i in range(12):
        sx, sy = rng.uniform(0.35, 0.8), rng.uniform(0.25, 0.6)
        cloth = common.make_icosphere(subdiv=3, jitter=0.3, seed=100 + i, radius=(sx / 2, sy / 2, 0.02),
                                      color=rng.choice(CLOTH))
        model.add(_place([cloth], rng.uniform(-1.1, 1.1), rng.uniform(-0.8, 0.8), rng.uniform(0.0, 0.01),
                         rng.uniform(0, math.pi))[0])
    for cx, cy, rot in ((0.7, -0.6, 0.4), (-0.9, 0.5, 2.1)):
        for bm in _case_shoe(cx, cy, rot):
            model.add(bm)


def _case_shoe(cx, cy, rot):
    shoe = [common.make_box((0.1, 0.24, 0.06), color="shoe_dark"),
            common.make_box((0.1, 0.07, 0.04), loc=(0.0, 0.1, 0.05), color="gun_black")]
    return _place(shoe, cx, cy, 0.0, rot)


def build_all():
    """Builds luggage_pile, suitcase_open and clothes_spill. Returns [(name, ok)]."""
    builds = [("luggage_pile", _luggage_pile), ("suitcase_open", _suitcase_open), ("clothes_spill", _clothes_spill)]
    return [(name, common.run_model(name, "props", BUDGET, fn)) for name, fn in builds]
