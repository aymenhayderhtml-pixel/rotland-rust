"""Highway props for the jam: highway_sign_gantry ("NEW HALDEN" with exit arrows, 9 m wide, 6 m high, with a
lattice truss, lamps, bolts) and highway_barrier (concrete jersey barrier, 3 m long, one dented side, chips).
No real brands or logos. Budgets: gantry 10000, barrier 10000.
"""
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
from jam_font import text_bms  # noqa: E402
from jam_util import bar, extrude_sections, pbox  # noqa: E402

# new: DECISIONS 2026-11-26 (gantry in metres)
POST_X = 4.1
POST_TOP = 5.6
BEAM_Z = (5.6, 6.0)
BEAM_HALF = 4.6
BOARD_X = 4.4
BOARD_Z = (2.9, 5.5)
BOARD_Y = (0.08, 0.26)
STROKE_Y = 0.27
LETTER_W = 0.55
LETTER_H = 0.9
TEXT_Z = 4.2
GANTRY_BUDGET = 10000
BARRIER_BUDGET = 10000
GANTRY_VIEW = (0.6, 1.0, 0.45)  # preview camera on the sign face (+Y is the front)
PROFILE = [(-0.42, 0.0), (0.42, 0.0), (0.30, 0.45), (0.22, 0.80), (-0.22, 0.80), (-0.30, 0.45)]
DENTED = [(-0.42, 0.0), (0.42, 0.0), (0.26, 0.42), (0.14, 0.70), (-0.20, 0.76), (-0.30, 0.45)]


def _arrows():
    """Two exit arrows under the name, tips on the outside."""
    out = []
    for sign in (-1, 1):
        tip_x, tail_x, z = sign * 3.9, sign * 2.6, 3.5
        out.append(bar(tail_x, z, tip_x, z, STROKE_Y, 0.14, 0.04, "white", bevel=0.01))
        out.append(bar(tip_x, z, tip_x - sign * 0.45, z + 0.42, STROKE_Y, 0.14, 0.04, "white", bevel=0.01))
        out.append(bar(tip_x, z, tip_x - sign * 0.45, z - 0.42, STROKE_Y, 0.14, 0.04, "white", bevel=0.01))
    return out


def _truss():
    """Lattice on the beam: vertical bars, diagonals, and the two side trusses between the posts."""
    out = []
    for i in range(-9, 10):
        x = i * 0.45
        out.append(pbox((x - 0.04, -0.27, BEAM_Z[0] + 0.1), (x + 0.04, 0.27, BEAM_Z[1] - 0.1), "metal", bevel=0.01))
    for i in range(-9, 9):
        x0, x1 = i * 0.45, (i + 1) * 0.45
        for sgn in (-1, 1):
            out.append(bar(x0, BEAM_Z[0] + 0.1, x1, BEAM_Z[1] - 0.1, sgn * 0.26, 0.05, 0.05, "metal"))
    for x in (-POST_X, POST_X):
        out.append(car_parts._beam((x, -0.2, 1.2), (x * 0.8, 0.0, BEAM_Z[0]), 0.06, color="metal", segments=8))
        out.append(car_parts._beam((x, 0.2, 1.2), (x * 0.8, 0.0, BEAM_Z[0]), 0.06, color="metal", segments=8))
    return out


def _bolts(rng):
    """Bolt heads on the board frame corners, on the beam top and along the posts."""
    out = []
    for x in (-BOARD_X + 0.3, BOARD_X - 0.3):
        for z in (BOARD_Z[0] + 0.3, BOARD_Z[1] - 0.3):
            bolt = common.make_frustum(0.05, 0.05, -0.02, 0.02, segments=8, color="metal")
            common.transform(bolt, rot=(math.pi / 2.0, 0.0, 0.0), loc=(x, BOARD_Y[1], z))
            out.append(bolt)
    for _ in range(24):
        bolt = common.make_frustum(0.03, 0.03, 0.0, 0.03, segments=8, color="metal")
        common.transform(bolt, loc=(rng.uniform(-BEAM_HALF, BEAM_HALF), 0.0, BEAM_Z[1]))
        out.append(bolt)
    for x in (-POST_X, POST_X):
        for z in (0.5, 2.0, 3.5, 5.0):
            bolt = common.make_frustum(0.04, 0.04, -0.02, 0.02, segments=8, color="metal")
            common.transform(bolt, rot=(0.0, 0.0, 0.0), loc=(x + 0.15, 0.2, z))
            out.append(bolt)
    return out


def _lamps():
    """Two lamp housings with lenses under the beam."""
    out = []
    for x in (-2.0, 2.0):
        out.append(pbox((x - 0.3, -0.45, BEAM_Z[0] - 0.22), (x + 0.3, 0.0, BEAM_Z[0]), "gun_black", bevel=0.03))
        out.append(pbox((x - 0.24, -0.46, BEAM_Z[0] - 0.18), (x + 0.24, -0.44, BEAM_Z[0] - 0.06), "white", bevel=0.01))
    return out


def _gantry(model):
    rng = random.Random(51)
    for x in (-POST_X, POST_X):
        model.add(pbox((x - 0.15, -0.2, 0.0), (x + 0.15, 0.2, POST_TOP), "metal", bevel=0.03))
        model.add(pbox((x - 0.3, -0.3, 0.0), (x + 0.3, 0.3, 0.12), "rock_mid", bevel=0.03))  # base plate
    model.add(pbox((-BEAM_HALF, -0.25, BEAM_Z[0]), (BEAM_HALF, 0.25, BEAM_Z[1]), "metal", bevel=0.03))
    model.add(pbox((-BOARD_X, BOARD_Y[0], BOARD_Z[0]), (BOARD_X, BOARD_Y[1], BOARD_Z[1]), "pine_dark", bevel=0.03))
    frame, y0, y1 = 0.12, BOARD_Y[1], BOARD_Y[1] + 0.02
    model.add(pbox((-BOARD_X, y0, BOARD_Z[1] - frame), (BOARD_X, y1, BOARD_Z[1]), "white", bevel=0.01))
    model.add(pbox((-BOARD_X, y0, BOARD_Z[0]), (BOARD_X, y1, BOARD_Z[0] + frame), "white", bevel=0.01))
    model.add(pbox((-BOARD_X, y0, BOARD_Z[0]), (-BOARD_X + frame, y1, BOARD_Z[1]), "white", bevel=0.01))
    model.add(pbox((BOARD_X - frame, y0, BOARD_Z[0]), (BOARD_X, y1, BOARD_Z[1]), "white", bevel=0.01))
    for bm in text_bms("NEW HALDEN", 0.0, TEXT_Z, LETTER_W, LETTER_H, STROKE_Y, "white", depth=0.04, bevel=0.012):
        model.add(bm)
    for bm in _arrows() + _truss() + _bolts(rng) + _lamps():
        model.add(bm)


def _barrier(model):
    """Jersey barrier: 12 sections with a little noise, a dent on one side, chips and lifting loops."""
    rng = random.Random(61)
    sections = []
    for i in range(12):
        x = -1.5 + i * 3.0 / 11
        base = DENTED if -0.25 < x < 0.25 else PROFILE
        sections.append((x, [(y + rng.uniform(-0.01, 0.01), z + rng.uniform(-0.01, 0.01)) for y, z in base]))
    body = extrude_sections(sections, "rock_light")
    model.add(body)
    model.add_collider("block", common.make_box((3.0, 0.6, 0.8), loc=(0.0, 0.0, 0.4), color="white"))
    for _ in range(10):
        chip = common.make_icosphere(subdiv=3, jitter=0.2, seed=rng.randint(0, 999),
                                     radius=(0.06, 0.05, 0.04),
                                     loc=(rng.uniform(-1.4, 1.4), rng.choice((-0.3, 0.3)) * rng.uniform(0.5, 1.0),
                                          rng.uniform(0.1, 0.75)), color="rock_mid")
        model.add(chip)
    for x in (-0.9, 0.9):
        model.add(car_parts._beam((x, -0.1, 0.8), (x, 0.1, 0.95), 0.02, color="metal", segments=6))


def build_all():
    """Builds highway_sign_gantry and highway_barrier. Returns [(name, ok)]."""
    gantry_ok = common.run_model("highway_sign_gantry", "props", GANTRY_BUDGET, _gantry, view=GANTRY_VIEW)
    return [("highway_sign_gantry", gantry_ok),
            ("highway_barrier", common.run_model("highway_barrier", "props", BARRIER_BUDGET, _barrier))]
