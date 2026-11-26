"""City buses for the jam: bus_city (upright, 12 m) and bus_city_side (the same bus rotated 90 degrees onto its
side, so the windowed side faces up and the underside, axles and exhaust face sideways).

Budget 20000 each. Side windows: glass, some cracked (dark) or missing (holes). The front door hangs open with a
full door panel. Destination board "NOT IN SERVICE". One tyre is flat. The roof is flagged for the top preview.
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
from jam_font import text_bms  # noqa: E402
from jam_util import add_bbox_collider, box, pbox, wheel_parts  # noqa: E402

# new: DECISIONS 2026-11-26 (bus size in metres: 12 m long, 2.5 m wide, 3.2 m high)
HALF_L = 6.0
HALF_W = 1.25
SILL = 0.70
ROOF = 3.20
NOSE = 0.5  # front top sits this far behind the front bottom (sloped windscreen)
WHEEL_R = 0.49
WHEEL_HALF = 0.2
BUDGET = 20000
BODY = "car_white"
STRIPE = "car_blue"
WINDOWS_PER_SIDE = 8
PATTERN = ["glass", "glass", "crack", "glass", "hole", "glass", "glass", "crack"]  # new: DECISIONS 2026-11-26
DOOR_OPEN_DEG = 55.0  # new: DECISIONS 2026-11-26
DOOR_LEN = 1.9


def _corners():
    bl = [(-HALF_W, -HALF_L, SILL), (HALF_W, -HALF_L, SILL), (HALF_W, HALF_L, SILL), (-HALF_W, HALF_L, SILL)]
    tp = [(-HALF_W + 0.08, -HALF_L, ROOF), (HALF_W - 0.08, -HALF_L, ROOF),
          (HALF_W - 0.08, HALF_L - NOSE, ROOF), (-HALF_W + 0.08, HALF_L - NOSE, ROOF)]
    return bl, tp


def _body(rng):
    """Closed body loft, glass with frames, pillars between windows, windscreens. Returns (bm, roof) pairs."""
    bl, tp = _corners()
    out = [(car_parts._closed_loft(bl, tp, [BODY] * 4, BODY, "metal"), True)]
    for face in (1, 3):
        for k in range(WINDOWS_PER_SIDE):
            u0 = 0.03 + k * 0.12
            state = PATTERN[k] if face == 1 else rng.choice(PATTERN)
            out += [(bm, True) for bm in shell.window_parts(bl, tp, face, (u0, u0 + 0.09, 0.32, 0.86), state, False)]
            if k < WINDOWS_PER_SIDE - 1:
                for v0, v1 in ((0.0, 0.3), (0.86, 1.0)):
                    a = shell.face_point(bl, tp, face, u0 + 0.105, v0) + shell._outward(face, 0.02)
                    b = shell.face_point(bl, tp, face, u0 + 0.105, v1) + shell._outward(face, 0.02)
                    out.append((car_parts._beam(tuple(a), tuple(b), 0.035, color=BODY, segments=6), True))
    out += [(bm, True) for bm in shell.window_parts(bl, tp, 2, (0.12, 0.88, 0.35, 0.92), "glass", False)]
    out += [(bm, True) for bm in shell.window_parts(bl, tp, 0, (0.12, 0.88, 0.35, 0.90), "glass", False)]
    return out


def _front_and_rear(lying=False):
    out = []
    out.append((pbox((-HALF_W - 0.03, HALF_L - 0.2, SILL - 0.2), (HALF_W + 0.03, HALF_L + 0.06, SILL + 0.1),
                     "gun_black", bevel=0.04), False))
    out.append((pbox((-HALF_W - 0.03, -HALF_L - 0.06, SILL - 0.2), (HALF_W + 0.03, -HALF_L + 0.2, SILL + 0.1),
                     "gun_black", bevel=0.04), False))
    out.append((pbox((-0.75, HALF_L - NOSE - 0.55, ROOF - 0.42), (0.75, HALF_L - NOSE - 0.4, ROOF - 0.1),
                     "gun_black", bevel=0.02), False))  # destination board
    out += [(bm, False) for bm in text_bms("NOT IN SERVICE", 0.0, ROOF - 0.36, 0.08, 0.18, HALF_L - NOSE - 0.36,
                                           "red_paint", depth=0.02)]
    for sx in (-1, 1):
        out.append((pbox((sx * 0.65 - 0.18, HALF_L + 0.0, SILL + 0.15), (sx * 0.65 + 0.18, HALF_L + 0.05,
                                                                       SILL + 0.35), "white", bevel=0.02), False))
        out.append((box((sx * 0.9 - 0.05, HALF_L + 0.03, SILL + 0.05), (sx * 0.9 + 0.05, HALF_L + 0.06,
                                                                        SILL + 0.12), "carrot_orange"), False))
        out.append((box((sx * 0.55 - 0.2, HALF_L - 0.05, SILL + 0.1), (sx * 0.55 + 0.2, HALF_L + 0.05,
                                                                       SILL + 0.14), "chrome"), False))
        if lying and sx < 0:
            continue  # omit ground-side mirror/marker so the lowest wheel sits on z=0
        out.append((box((sx * (HALF_W + 0.2) - 0.1, -HALF_L - 0.02, SILL + 0.1),
                        (sx * (HALF_W + 0.2) + 0.1, -HALF_L, SILL + 0.22), "red_paint"), False))
        out.append((car_parts._beam((sx * (HALF_W + 0.02), HALF_L - 1.2, ROOF - 0.7),
                                    (sx * (HALF_W + 0.25), HALF_L - 1.0, ROOF - 0.7), 0.03, color="metal"), False))
        out.append((pbox((sx * (HALF_W + 0.2) - 0.07, HALF_L - 1.25, ROOF - 0.95),
                         (sx * (HALF_W + 0.2) + 0.07, HALF_L - 0.95, ROOF - 0.7), "gun_black", bevel=0.01), False))
    return out


def _roof_and_skirt(rng, lying=False):
    out = []
    out.append((pbox((-0.9, -1.5, ROOF), (0.9, 1.5, ROOF + 0.4), "metal", bevel=0.04), True))  # AC unit
    for y in (-3.0, 2.0):
        out.append((pbox((-0.5, y, ROOF), (0.5, y + 1.0, ROOF + 0.04), "gun_black", bevel=0.02), True))  # hatches
    if lying:
        # Cut-outs around the axles and mid-chassis so axle and rail stubs show from the road side.
        skirt_spans = [(-HALF_L + 0.2, -HALF_L + 1.2), (-HALF_L + 2.8, -1.2), (1.0, HALF_L - 2.8),
                       (HALF_L - 1.2, HALF_L - 0.2)]
        for y0, y1 in skirt_spans:
            out.append((pbox((-HALF_W - 0.02, y0, SILL - 0.22), (-HALF_W + 0.005, y1, SILL + 0.02),
                             BODY, bevel=0.02), False))
            out.append((pbox((HALF_W - 0.005, y0, SILL - 0.22), (HALF_W + 0.02, y1, SILL + 0.02),
                             BODY, bevel=0.02), False))
    else:
        out.append((pbox((-HALF_W - 0.02, -HALF_L + 0.2, SILL - 0.22), (-HALF_W + 0.005, HALF_L - 0.2, SILL + 0.02),
                         BODY, bevel=0.02), False))  # skirt panels, bus colour
        out.append((pbox((HALF_W - 0.005, -HALF_L + 0.2, SILL - 0.22), (HALF_W + 0.02, HALF_L - 0.2, SILL + 0.02),
                         BODY, bevel=0.02), False))
    for k in range(8):  # rear engine grille slats
        z = SILL + 0.3 + k * 0.2
        out.append((pbox((-0.8, -HALF_L - 0.02, z), (0.8, -HALF_L + 0.02, z + 0.1), "rock_dark", bevel=0.01), False))
    for side in (-1, 1):
        for k in range(6):  # engine vent slats on the rear flank
            x0, x1 = sorted((side * (HALF_W + 0.01), side * (HALF_W + 0.03)))
            z = SILL - 0.05 + k * 0.1
            out.append((pbox((x0, -HALF_L + 1.6, z), (x1, -HALF_L + 2.8, z + 0.05), "rock_dark", bevel=0.005), False))
        x0, x1 = sorted((side * (HALF_W + 0.004), side * (HALF_W - 0.02)))
        out.append((pbox((x0, -HALF_L + 0.2, SILL + 0.55), (x1, HALF_L - 0.6, SILL + 0.75), STRIPE, bevel=0.01), False))
        for _ in range(4):
            y = rng.uniform(-HALF_L + 1.0, HALF_L - 1.5)
            z = rng.uniform(SILL - 0.1, SILL + 0.3)
            out.append((pbox((x0, y, z), (x1, y + rng.uniform(0.3, 0.7), z + 0.12), "rust", bevel=0.005), False))
    return out


def _arches():
    out = []
    for side in (-1, 1):
        for y in (HALF_L - 2.0, -HALF_L + 2.0):
            out.append((pbox((side * (HALF_W - 0.04), y - 0.75, SILL - 0.2), (side * (HALF_W + 0.1), y + 0.75,
                              SILL + 0.45), "gun_black", bevel=0.03), False))
    return out


def _wheels(side_only):
    out = []
    for y in (HALF_L - 2.0, -HALF_L + 2.0):
        for x in (-(HALF_W - 0.1), HALF_W - 0.1):
            if side_only and x < 0:
                continue
            flat = 0.72 if (y < 0 and x > 0) else 1.0  # new: DECISIONS 2026-11-26 (one flat tyre)
            out += [(bm, False) for bm in wheel_parts(x, y, WHEEL_R, WHEEL_HALF, flat=flat)]
    return out


def _door():
    """Front right door, open: dark hole in the side, a full door panel hinged at its front edge."""
    door_y0, door_y1 = HALF_L - 2.4, HALF_L - 0.4
    parts = [box((HALF_W - 0.01, door_y0, SILL), (HALF_W + 0.004, door_y1, SILL + 2.2), "gun_black")]
    panel = [pbox((-0.025, -DOOR_LEN, 0.0), (0.025, 0.0, 2.2), BODY, bevel=0.01),
             pbox((-0.035, -DOOR_LEN + 0.1, 0.15), (-0.025, -0.1, 0.9), "leather", bevel=0.005),
             pbox((0.025, -DOOR_LEN + 0.2, 1.1), (0.04, -0.2, 1.9), "glass_tint", bevel=0.0),
             box((0.02, -0.2, 2.0), (0.035, -0.12, 2.05), "chrome")]
    for part in panel:
        common.transform(part, rot=(0.0, 0.0, math.radians(DOOR_OPEN_DEG)), loc=(HALF_W + 0.02, door_y1, SILL))
    return [(bm, False) for bm in parts + panel]


def _underside():
    """Chassis rails, axles, fuel tank and exhaust under the floor (visible when the bus lies on its side)."""
    out = []
    for x in (-0.55, 0.55):
        out.append((box((x - 0.07, -HALF_L + 0.3, SILL - 0.45), (x + 0.07, HALF_L - 0.3, SILL - 0.2), "metal"), False))
    for y in (HALF_L - 2.0, -HALF_L + 2.0):
        out.append((car_parts._beam((-HALF_W + 0.1, y, WHEEL_R), (HALF_W - 0.1, y, WHEEL_R), 0.09, color="metal"),
                    False))
    tank = common.make_frustum(0.28, 0.28, -1.2, 1.2, segments=12, color="metal")
    common.transform(tank, rot=(math.pi / 2, 0.0, 0.0), loc=(0.95, -0.4, SILL - 0.35))
    out.append((tank, False))
    pipe = common.make_frustum(0.07, 0.07, -HALF_L + 0.5, HALF_L - 0.8, segments=8, color="metal")
    common.transform(pipe, rot=(math.pi / 2, 0.0, 0.0), loc=(-0.85, 0.0, SILL - 0.35))
    out.append((pipe, False))
    muffler = common.make_frustum(0.14, 0.14, -0.6, 0.6, segments=10, color="rock_dark")
    common.transform(muffler, rot=(math.pi / 2, 0.0, 0.0), loc=(-0.85, -HALF_L + 0.9, SILL - 0.35))
    out.append((muffler, False))
    return out


def _interior():
    """Eight rows of seats, poles to the roof, and the floor edge (visible from the top preview)."""
    out = []
    for k in range(8):
        y = -HALF_L + 1.0 + k * 1.2
        for x in (-0.85, 0.85):
            out.append((pbox((x - 0.25, y - 0.25, SILL + 0.35), (x + 0.25, y + 0.25, SILL + 0.5), "leather",
                             bevel=0.03), False))
            out.append((pbox((x - 0.25, y - 0.3, SILL + 0.5), (x + 0.25, y - 0.22, SILL + 0.98), "leather",
                             bevel=0.03), False))
        out.append((car_parts._beam((-0.25, y + 0.5, SILL), (-0.25, y + 0.5, ROOF - 0.1), 0.02, color="metal"),
                    False))
        for x in (-0.85, 0.85):  # headrests
            out.append((pbox((x - 0.14, y - 0.32, SILL + 0.98), (x + 0.14, y - 0.24, SILL + 1.14), "gun_black",
                             bevel=0.03), False))
        out.append((car_parts._beam((-HALF_W + 0.1, y, ROOF - 0.15), (HALF_W - 0.1, y, ROOF - 0.15), 0.02,
                                    color="metal", segments=6), False))  # ceiling handrail
    return out


def _parts(lying):
    rng = random.Random(21)
    parts = _body(rng) + _front_and_rear(lying) + _roof_and_skirt(rng, lying) + _arches() + _underside() + _interior()
    parts += _door() + _wheels(side_only=False)
    return parts


def _build(lying):
    def build(model):
        parts = _parts(lying)
        if lying:
            for bm, _ in parts:
                common.transform(bm, rot=(0.0, -math.pi / 2.0, 0.0))
        add_bbox_collider(model, [bm for bm, _ in parts], inset=0.1)
        for bm, roof in parts:
            model.add(bm, roof=roof)

    return build


def build_all():
    """Builds bus_city and bus_city_side. Returns [(name, ok)]."""
    upright = common.run_model("bus_city", "vehicles", BUDGET, _build(False), top_preview=True)
    lying = common.run_model("bus_city_side", "vehicles", BUDGET, _build(True), top_preview=True)
    return [("bus_city", upright), ("bus_city_side", lying)]
