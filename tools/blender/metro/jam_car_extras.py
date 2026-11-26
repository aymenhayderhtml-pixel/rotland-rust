"""Car extras for the R38 jam cars: wheels (or bare rims when burnt), flat tyres, raised rust and dust patches,
roof rack luggage and the pickup's bed load.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

from jam_util import ground_rim, pbox, wheel_parts  # noqa: E402
import car_parts  # noqa: E402

CAR_WHEEL_R = 0.34  # new: DECISIONS 2026-11-26 (car wheel radius, m)
CAR_TYRE_HALF = 0.12  # new: DECISIONS 2026-11-26 (tyre half width, m)
FLAT_SQUASH = 0.72  # new: DECISIONS 2026-11-26 (a flat tyre keeps 72% of its height)
LUGGAGE = ["leather", "soda_purple", "wood_crate", "dog_tan", "canvas_khaki", "car_blue"]
BURNT_LUGGAGE = ["gun_black", "soil_dark"]


def wheels(spec, burnt):
    """Four wheels (tyre and rim). Burnt cars: no tyres, the rims lie on the ground beside the car.
    spec['flat'] lists flat tyres (0 front-left, 1 front-right, 2 rear-left, 3 rear-right)."""
    x, y = spec["W"] - 0.04, spec["wheel_y"]
    spots = [(-x, y), (x, y), (-x, -y), (x, -y)]
    out = []
    if burnt:
        for i, (sx, sy) in enumerate(spots):
            side = -1 if sx < 0 else 1
            out.append(ground_rim(side * (spec["W"] + 0.55 + 0.1 * i), sy * 0.6 + 0.2 * i, CAR_WHEEL_R,
                                  CAR_TYRE_HALF, yaw=0.3 * i))
        return out
    for i, (sx, sy) in enumerate(spots):
        flat = FLAT_SQUASH if i in spec["flat"] else 1.0
        out += wheel_parts(sx, sy, CAR_WHEEL_R, CAR_TYRE_HALF, flat=flat)
    return out


def _raised(lo, hi, color):
    return pbox(lo, hi, color, bevel=0.008)


def grime(spec, rng, burnt):
    """Raised rust and dust patches on the sills, doors, roof and bonnet. Burnt cars: many rust-orange patches.
    Returns (bm, roof) pairs."""
    w, half, sill, roof = spec["W"], spec["half_l"], spec["sill"], spec["roof"]
    cab, belt = spec["cab"], spec["belt"]
    dust = "soil_dark" if burnt else "dirt"
    out = []
    side_count = 9 if burnt else 4
    for side in (-1, 1):
        for _ in range(side_count):
            y = rng.uniform(-half + 0.4, half - 0.6)
            z = rng.uniform(sill - 0.05, belt - 0.3)
            length, height = rng.uniform(0.2, 0.6), rng.uniform(0.1, 0.3)
            x0, x1 = sorted((side * (w + 0.002), side * (w + 0.016)))
            out.append((_raised((x0, y, z), (x1, y + length, z + height), "rust"), False))
        for _ in range(2 if burnt else 1):
            y = rng.uniform(-half + 0.5, half - 0.8)
            z = rng.uniform(sill, belt - 0.2)
            x0, x1 = sorted((side * (w + 0.002), side * (w + 0.012)))
            out.append((_raised((x0, y, z), (x1, y + 0.3, z + 0.14), dust), False))
    roof_count = 7 if burnt else 3
    for _ in range(roof_count):
        x = rng.uniform(-cab["xt"] + 0.15, cab["xt"] - 0.3)
        y = rng.uniform(cab["tb"] + 0.2, cab["tf"] - 0.5)
        out.append((_raised((x, y, roof - 0.004), (x + rng.uniform(0.25, 0.6), y + rng.uniform(0.2, 0.5), roof + 0.012),
                            "rust"), True))
    for _ in range(4):
        x = rng.uniform(-cab["xt"] + 0.1, cab["xt"] - 0.2)
        y = rng.uniform(cab["tb"] + 0.1, cab["tf"] - 0.2)
        out.append((_raised((x, y, roof - 0.004), (x + rng.uniform(0.2, 0.5), y + rng.uniform(0.2, 0.4), roof + 0.01),
                            dust), True))
    hood_y0, hood_y1 = cab["bf"] + 0.1, half - 0.3
    for _ in range(3 if hood_y1 - 0.3 > hood_y0 else 0):
        y = rng.uniform(hood_y0, hood_y1 - 0.3)
        x = rng.uniform(-w + 0.2, w - 0.5)
        out.append((_raised((x, y, belt - 0.01), (x + rng.uniform(0.2, 0.45), y + 0.25, belt + 0.008), dust), False))
    return out


def rack_and_luggage(spec, rng, burnt):
    """Roof rack with luggage (2-4 boxes and a bag) when spec['rack'] is set. Returns bmeshes."""
    if not spec.get("rack"):
        return []
    cab = spec["cab"]
    xr = cab["xt"] - 0.18
    y0, y1 = cab["tb"] + 0.25, cab["tf"] - 0.25
    z = spec["roof"] + 0.10
    parts = [car_parts._beam((-xr, y0, z), (-xr, y1, z), 0.025, color="metal"),
             car_parts._beam((xr, y0, z), (xr, y1, z), 0.025, color="metal")]
    for y in (y0, y1):
        parts.append(car_parts._beam((-xr, y, z), (xr, y, z), 0.02, color="metal"))
    palette = BURNT_LUGGAGE if burnt else LUGGAGE
    for _ in range(rng.randint(2, 4)):
        sx, sy, sz = rng.uniform(0.45, 0.8), rng.uniform(0.3, 0.45), rng.uniform(0.22, 0.36)
        cx, cy = rng.uniform(-xr + 0.2, xr - 0.2 - sx / 2), rng.uniform(y0 + 0.2, y1 - 0.2 - sy)
        parts.append(pbox((cx - sx / 2, cy - sy / 2, z + 0.02), (cx + sx / 2, cy + sy / 2, z + 0.02 + sz),
                          rng.choice(palette), bevel=0.02))
    parts.append(pbox((-0.2, y1 - 0.5, z + 0.02), (0.2, y1 - 0.25, z + 0.22),
                      "soil_dark" if burnt else "canvas_khaki", bevel=0.04))
    return parts


def bed_load(spec, rng, burnt):
    """Pickup bed: crates and luggage under a tarp (none on burnt cars)."""
    bed = spec.get("bed")
    if not bed or burnt:
        return []
    top, half = bed["top"], spec["half_l"]
    y0, y1 = -half + 0.3, bed["front"] - 0.25
    parts = []
    for _ in range(3):
        sx, sy, sz = rng.uniform(0.4, 0.6), rng.uniform(0.4, 0.6), rng.uniform(0.25, 0.4)
        cx, cy = rng.uniform(-0.5, 0.5), rng.uniform(y0 + 0.3, y1 - 0.3)
        parts.append(pbox((cx - sx / 2, cy - sy / 2, top - sz), (cx + sx / 2, cy + sy / 2, top),
                          rng.choice(LUGGAGE), bevel=0.02))
    parts.append(pbox((-0.8, y0, top + 0.02), (0.8, y1, top + 0.06), "cloth_olive", bevel=0.01))
    return parts
