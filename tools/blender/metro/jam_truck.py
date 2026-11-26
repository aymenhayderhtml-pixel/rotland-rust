"""Army truck for the jam: a canvas-covered cargo bed with ribs, tailgate down, torn canvas, a cab with glass,
headlights, grille and mirrors, six tyres (one flat), a spare tyre, jerrycans, an exhaust stack and generic white
markings (no insignia, no stars). Olive drab body. Budget 20000 tris.
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
import jam_car_shell as shell  # noqa: E402
from jam_car_interior import _seat, _steering_wheel  # noqa: E402
from jam_util import add_bbox_collider, box, pbox, wheel_parts  # noqa: E402

# new: DECISIONS 2026-11-26 (truck size in metres: 6.8 m long, 2.4 m wide, canvas 2.4 m high)
HALF_L = 3.4
HALF_W = 1.2
CARGO_FRONT = 1.4  # the cab stands behind this line (y)
CANVAS_BASE = 0.5
CANVAS_TOP = 2.4
WHEEL_R = 0.45
WHEEL_HALF = 0.2
BUDGET = 20000
ARMY = "cloth_olive"
CANVAS = "canvas_khaki"
RIBS = 9


def _cab():
    out = []
    bl = [(-HALF_W, 1.5, 0.5), (HALF_W, 1.5, 0.5), (HALF_W, 3.0, 0.5), (-HALF_W, 3.0, 0.5)]
    tp = [(-HALF_W + 0.1, 1.6, 2.0), (HALF_W - 0.1, 1.6, 2.0), (HALF_W - 0.1, 2.8, 2.0), (-HALF_W + 0.1, 2.8, 2.0)]
    out.append((car_parts._closed_loft(bl, tp, [ARMY] * 4, ARMY, "gun_black"), True))
    for face, rect, state in ((2, (0.12, 0.88, 0.2, 0.9), "glass"), (1, (0.2, 0.8, 0.25, 0.9), "crack"),
                              (3, (0.2, 0.8, 0.25, 0.9), "hole")):
        out += [(bm, True) for bm in shell.window_parts(bl, tp, face, rect, state, False)]
    out.append((pbox((-HALF_W + 0.05, 2.95, 0.5), (HALF_W - 0.05, HALF_L - 0.1, 0.95), ARMY, bevel=0.03), False))
    out.append((pbox((-HALF_W + 0.1, 2.95, 0.95), (HALF_W - 0.1, HALF_L - 0.1, 1.0), "gun_black", bevel=0.01), False))
    out.append((pbox((-HALF_W - 0.04, HALF_L - 0.3, 0.3), (HALF_W + 0.04, HALF_L + 0.02, 0.5), "gun_black",
                     bevel=0.03), False))  # front bumper
    for k in range(6):  # grille slats
        x = -0.6 + k * 0.24
        out.append((box((x - 0.04, HALF_L - 0.02, 0.55), (x + 0.04, HALF_L + 0.02, 0.9), "metal"), False))
    for sx in (-1, 1):
        out.append((pbox((sx * 0.8 - 0.14, HALF_L - 0.05, 0.6), (sx * 0.8 + 0.14, HALF_L + 0.02, 0.78), "white",
                         bevel=0.01), False))  # headlights
        out.append((car_parts._beam((sx * HALF_W, 2.9, 1.2), (sx * (HALF_W + 0.25), 2.9, 1.6), 0.03, color="metal"),
                    False))  # mirror arm
        out.append((pbox((sx * (HALF_W + 0.2) - 0.06, 2.85, 1.45), (sx * (HALF_W + 0.2) + 0.06, 2.95, 1.7),
                         "gun_black", bevel=0.01), False))  # mirror
        out.append((pbox((sx * (HALF_W - 0.01), 2.0, 0.9), (sx * (HALF_W + 0.02), 2.7, 1.9), ARMY, bevel=0.01), False))
        out.append((box((sx * (HALF_W + 0.01), 1.9, 1.3), (sx * (HALF_W + 0.02), 2.4, 1.7), "white"), False))  # marking
    return out


def _canvas(rng):
    out = []
    xb, xt = HALF_W - 0.2, HALF_W - 0.35
    top = common.new_bm()
    common.add_quad(top, [(-xt, -HALF_L, CANVAS_TOP), (xt, -HALF_L, CANVAS_TOP),
                          (xt, CARGO_FRONT, CANVAS_TOP), (-xt, CARGO_FRONT, CANVAS_TOP)], (0, 0, 1), CANVAS)
    out.append((top, True))
    for side in (-1, 1):
        bm = common.new_bm()
        common.add_quad(bm, [(side * xb, -HALF_L, CANVAS_BASE), (side * xb, CARGO_FRONT, CANVAS_BASE),
                             (side * xt, CARGO_FRONT, CANVAS_TOP), (side * xt, -HALF_L, CANVAS_TOP)],
                        (side, 0, 0), CANVAS)
        out.append((bm, False))
        for _ in range(3):
            y0 = rng.uniform(-HALF_L + 0.5, CARGO_FRONT - 1.2)
            z0 = rng.uniform(CANVAS_BASE + 0.2, CANVAS_TOP - 0.8)
            tear = common.new_bm()
            common.add_quad(tear, [(side * (xb + 0.01), y0, z0), (side * (xb + 0.01), y0 + 0.7, z0),
                                   (side * (xt + 0.01), y0 + 0.7, z0 + 0.6), (side * (xt + 0.01), y0, z0 + 0.6)],
                            (side, 0, 0), "gun_black")
            out.append((tear, False))
    for k in range(RIBS):  # arched ribs over the canvas
        y = -HALF_L + 0.2 + k * (CARGO_FRONT + HALF_L - 0.4) / (RIBS - 1)
        for side in (-1, 1):
            out.append((car_parts._beam((side * xb, y, CANVAS_BASE), (side * xt, y, CANVAS_TOP), 0.035,
                                        color="metal", segments=6), True))
        out.append((car_parts._beam((-xt, y, CANVAS_TOP), (xt, y, CANVAS_TOP), 0.035, color="metal", segments=6), True))
    for side in (-1, 1):
        out.append((car_parts._beam((side * xb, -HALF_L, CANVAS_BASE), (side * xb, CARGO_FRONT, CANVAS_BASE), 0.04,
                                    color="metal", segments=6), False))
    for side in (-1, 1):  # rope lashings along the canvas top
        out.append((car_parts._beam((side * xt, -HALF_L, CANVAS_TOP - 0.02), (side * xt, CARGO_FRONT,
                                     CANVAS_TOP - 0.02), 0.012, color="gun_black", segments=4), True))
    return out


def _tailgate_and_load(rng):
    out = []
    out.append((pbox((-HALF_W + 0.3, -HALF_L - 0.5, CANVAS_BASE - 0.06), (HALF_W - 0.3, -HALF_L, CANVAS_BASE + 0.02),
                     "gun_black", bevel=0.01), False))  # tailgate down
    out.append((pbox((-HALF_W + 0.25, -HALF_L + 0.05, 0.3), (HALF_W - 0.25, CARGO_FRONT, CANVAS_BASE - 0.06),
                     "gun_black", bevel=0.02), False))  # cargo floor
    for cx, cy in ((-0.5, -0.4), (0.4, -0.3), (0.0, -1.4)):
        out.append((pbox((cx - 0.35, cy - 0.3, CANVAS_BASE + 0.06), (cx + 0.35, cy + 0.3, CANVAS_BASE + 0.66),
                         "wood_crate", bevel=0.02), False))
        out.append((box((cx - 0.36, cy - 0.01, CANVAS_BASE + 0.4), (cx + 0.36, cy + 0.01, CANVAS_BASE + 0.44),
                        "metal"), False))  # strap band
    out.append((pbox((-0.9, -1.9, CANVAS_BASE + 0.7), (0.9, 0.2, CANVAS_BASE + 0.74), ARMY, bevel=0.01), False))
    for k in range(4):  # jerrycans on the outside of the cargo, right side
        y = -0.6 - k * 0.42
        out.append((pbox((HALF_W - 0.2, y - 0.15, 0.6), (HALF_W + 0.02, y + 0.15, 1.0), "cloth_olive", bevel=0.03), False))
        out.append((car_parts._beam((HALF_W - 0.1, y, 1.02), (HALF_W - 0.1, y, 1.08), 0.02, color="metal", segments=6),
                    False))
    for _ in range(14):  # dust on the canvas and cargo floor
        x = rng.uniform(-0.9, 0.9)
        y = rng.uniform(-3.0, 1.0)
        out.append((pbox((x, y, CANVAS_TOP - 0.01), (x + 0.3, y + 0.3, CANVAS_TOP + 0.01), "dirt", bevel=0.0), True))
    return out


def _chassis_and_extras():
    out = []
    for x in (-0.6, 0.6):
        out.append((pbox((x - 0.07, -HALF_L, 0.25), (x + 0.07, HALF_L - 0.1, 0.5), "gun_black", bevel=0.01), False))
    tank = common.make_frustum(0.2, 0.2, -0.9, 0.9, segments=12, color="metal")  # fuel tank
    common.transform(tank, rot=(math.pi / 2, 0.0, 0.0), loc=(-0.9, 0.4, 0.35))
    out.append((tank, False))
    stack = common.make_frustum(0.05, 0.05, 0.0, 2.2, segments=10, color="rust")
    common.transform(stack, loc=(HALF_W - 0.12, 3.1, 0.5))
    out.append((stack, False))  # exhaust stack
    out.append((pbox((-HALF_W - 0.05, -HALF_L - 0.02, 0.2), (HALF_W + 0.05, -HALF_L - 0.5, 0.45), "gun_black",
                     bevel=0.02), False))  # rear bumper, under the tailgate
    return out


def _wheels():
    spots = [(-(HALF_W - 0.1), 2.5, 1.0), ((HALF_W - 0.1), 2.5, 1.0), (-(HALF_W - 0.1), -1.8, 1.0),
             ((HALF_W - 0.1), -1.8, 0.72), (-(HALF_W - 0.1), -2.8, 1.0), ((HALF_W - 0.1), -2.8, 1.0)]
    out = []
    for x, y, flat in spots:
        out += [(bm, False) for bm in wheel_parts(x, y, WHEEL_R, WHEEL_HALF, flat=flat)]
    spare = wheel_parts(-(HALF_W + 0.22), 1.0, WHEEL_R, WHEEL_HALF)[0]
    common.transform(spare, rot=(0.0, 0.0, 0.0), loc=(0.0, 0.0, 0.7))
    out.append((spare, False))  # spare tyre on the left flank of the cab
    return out


def _details():
    """Cab seats and steering wheel (visible from the top preview), troop benches, mudguards, toolboxes, jerrycans."""
    out = []
    for x, y in ((-0.6, 2.25), (0.6, 2.25)):
        out += [(bm, True) for bm in _seat((x, y, 0.75), False)]
    out += [(bm, True) for bm in _steering_wheel((-0.6, 2.25, 0.75), 2.8, False)]
    for side in (-1, 1):
        out.append((pbox((side * (HALF_W - 0.45) - 0.12, -3.2, 0.8), (side * (HALF_W - 0.45) + 0.12, 0.8, 0.9),
                         "cloth_olive", bevel=0.02), False))  # troop bench
        for y in (-2.0, 1.0):
            out.append((pbox((side * (HALF_W - 0.02) - 0.03, y - 0.25, 0.55), (side * (HALF_W + 0.14), y + 0.25,
                              0.9), "gun_black", bevel=0.02), False))  # mudguard over the rear wheels
        out.append((pbox((side * (HALF_W + 0.02), -2.0, 0.5), (side * (HALF_W + 0.2), -1.2, 0.8), "metal",
                         bevel=0.02), False))  # toolbox
        for k in range(4):
            y = 0.2 - k * 0.42
            out.append((pbox((side * (HALF_W - 0.2), y - 0.15, 0.6), (side * (HALF_W + 0.02), y + 0.15, 1.0),
                             "cloth_olive", bevel=0.03), False))
    return out


def _hardware(rng):
    """Bolt heads on the chassis rails and cab, door frame bars, bench backrests, a second crate tier, grime."""
    out = []
    for _ in range(100):
        bolt = common.make_frustum(0.025, 0.025, 0.0, 0.02, segments=8, color="metal")
        common.transform(bolt, rot=(math.pi / 2, 0.0, 0.0), loc=(rng.choice((-0.67, 0.67)) if rng.random() < 0.5
                                                                 else rng.uniform(-HALF_W, HALF_W),
                                                                 rng.uniform(-HALF_L + 0.2, HALF_L - 0.2), 0.45))
        out.append((bolt, False))
    for side in (-1, 1):
        for y0, y1, z0, z1 in ((2.0, 2.1, 0.5, 0.95), (2.6, 2.7, 0.5, 0.95), (2.0, 2.7, 0.5, 0.55)):
            out.append((pbox((side * (HALF_W + 0.01), y0, z0), (side * (HALF_W + 0.03), y1, z1), "white",
                             bevel=0.0), False))  # door frame bars
        for y in (-2.9, -2.6):
            out.append((pbox((side * (HALF_W - 0.45) - 0.1, y, 0.9), (side * (HALF_W - 0.45) + 0.1, y + 0.1, 1.25),
                             "cloth_olive", bevel=0.02), False))  # bench backrest
    for cx, cy in ((-0.5, -0.4), (0.4, -0.3)):
        out.append((pbox((cx - 0.3, cy - 0.25, CANVAS_BASE + 0.67), (cx + 0.3, cy + 0.25, CANVAS_BASE + 1.0),
                         "wood_crate", bevel=0.02), False))
    for _ in range(40):
        x = rng.uniform(-HALF_W + 0.1, HALF_W - 0.1)
        y = rng.uniform(-HALF_L + 0.3, HALF_L - 0.3)
        out.append((pbox((x, y, 0.0), (x + 0.25, y + 0.25, 0.01), "dirt", bevel=0.0), False))
    return out


def _build(model):
    rng = random.Random(31)
    parts = (_chassis_and_extras() + _cab() + _canvas(rng) + _tailgate_and_load(rng) + _wheels()
             + _details() + _hardware(rng))
    add_bbox_collider(model, [bm for bm, _ in parts], inset=0.1)
    for bm, roof in parts:
        model.add(bm, roof=roof)


def build_all():
    """Builds army_truck_jam. Returns [(name, ok)]."""
    return [("army_truck_jam", common.run_model("army_truck_jam", "vehicles", BUDGET, _build, top_preview=True))]
