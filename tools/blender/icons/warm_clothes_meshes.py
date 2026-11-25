"""Icon meshes for the R32 warm and desert clothing items in data/items.json.

Each builder returns a list of bmeshes (built in build coordinates, 1 unit = 1 m, z up).
"""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402


def _cyl(r0, r1, z0, z1, color, seg=12):
    return common.make_frustum(r0, r1, z0, z1, segments=seg, color=color)


def _box(size, loc, color, rot=(0.0, 0.0, 0.0)):
    bm = common.make_box(size, color=color)
    common.transform(bm, rot=rot, loc=loc)
    return bm


def _blob(subdiv, jitter, seed, radius, loc, color):
    return common.make_icosphere(subdiv=subdiv, jitter=jitter, seed=seed, radius=radius, loc=loc, color=color)


def warm_jacket():
    # Puffy quilted winter jacket in pine_snow_dark with dark baffle seams, front zip and snow_white collar
    parts = [
        # Waist hem and three stacked puffy torso baffles with recessed dark quilt seams
        _box((0.23, 0.125, 0.030), (0.0, 0.0, 0.015), "pine_dark"),
        _box((0.26, 0.145, 0.075), (0.0, 0.0, 0.068), "pine_snow_dark"),
        _box((0.245, 0.132, 0.012), (0.0, 0.0, 0.112), "pine_dark"),
        _box((0.27, 0.150, 0.075), (0.0, 0.0, 0.156), "pine_snow_dark"),
        _box((0.255, 0.136, 0.012), (0.0, 0.0, 0.200), "pine_dark"),
        _box((0.28, 0.152, 0.075), (0.0, 0.0, 0.244), "pine_snow_dark"),
        _box((0.29, 0.142, 0.045), (0.0, 0.0, 0.304), "pine_snow_dark"),
        # Front zipper track and brass pull on the -Y front face
        _box((0.014, 0.008, 0.315), (0.0, -0.077, 0.165), "metal"),
        _box((0.018, 0.012, 0.024), (0.0, -0.080, 0.285), "brass"),
        # Thick lighter fleece collar and dark inner neck opening
        _box((0.105, 0.085, 0.020), (0.0, 0.0, 0.332), "pine_dark"),
        _box((0.165, 0.042, 0.062), (0.0, 0.048, 0.352), "snow_white", rot=(-0.18, 0.0, 0.0)),
        _box((0.044, 0.115, 0.058), (-0.068, -0.005, 0.350), "snow_white", rot=(0.15, -0.18, 0.0)),
        _box((0.044, 0.115, 0.058), (0.068, -0.005, 0.350), "snow_white", rot=(0.15, 0.18, 0.0)),
    ]
    # Left (-X) and right (+X) puffy quilted sleeves with dark wrist cuffs
    for side in (-1.0, 1.0):
        tilt = -side * 0.42
        parts.extend([
            _box((0.085, 0.095, 0.125), (side * 0.175, 0.0, 0.235), "pine_snow_dark", rot=(0.0, tilt, 0.0)),
            _box((0.078, 0.088, 0.012), (side * 0.205, 0.0, 0.172), "pine_dark", rot=(0.0, tilt, 0.0)),
            _box((0.080, 0.090, 0.105), (side * 0.230, 0.0, 0.118), "pine_snow_dark", rot=(0.0, tilt, 0.0)),
            _box((0.068, 0.076, 0.028), (side * 0.258, 0.0, 0.056), "pine_dark", rot=(0.0, tilt, 0.0)),
        ])
    return parts


def wool_hat():
    # Knit red beanie with a ribbed folded brim, white band, and fluffy white pom-pom
    ribs = [
        _box(
            (0.012, 0.016, 0.050),
            (0.094 * math.cos(2.0 * math.pi * i / 12.0), 0.094 * math.sin(2.0 * math.pi * i / 12.0), 0.026),
            "red_paint",
            rot=(0.0, 0.0, 2.0 * math.pi * i / 12.0),
        )
        for i in range(12)
    ]
    return [
        _cyl(0.096, 0.098, 0.0, 0.052, "shirt_red", seg=12),
        *ribs,
        _cyl(0.091, 0.089, 0.050, 0.078, "red_paint", seg=12),
        _cyl(0.090, 0.087, 0.078, 0.096, "white", seg=12),
        _cyl(0.087, 0.073, 0.096, 0.132, "red_paint", seg=12),
        _cyl(0.073, 0.044, 0.132, 0.162, "red_paint", seg=12),
        _cyl(0.044, 0.014, 0.162, 0.178, "red_paint", seg=12),
        _blob(2, 0.22, 42, (0.034, 0.034, 0.032), (0.0, 0.0, 0.198), "white"),
    ]


def desert_scarf():
    # Folded sand_dark desert scarf with wrapped neck coils, woven stripes and fringe ends
    parts = [
        # Wrapped neck coils
        _box((0.22, 0.16, 0.042), (0.0, 0.03, 0.135), "sand_dark", rot=(0.22, 0.0, 0.12)),
        _box((0.224, 0.164, 0.012), (0.0, 0.03, 0.145), "sandstone", rot=(0.22, 0.0, 0.12)),
        _box((0.19, 0.14, 0.038), (0.01, 0.035, 0.172), "sand_dark", rot=(0.26, -0.06, -0.15)),
        # Under-fold scarf tail draping forward-left with sandstone and sand_light stripes
        _box((0.082, 0.19, 0.018), (-0.045, -0.055, 0.075), "sand_dark", rot=(0.58, 0.0, -0.22)),
        _box((0.084, 0.022, 0.020), (-0.062, -0.115, 0.036), "sandstone", rot=(0.58, 0.0, -0.22)),
        _box((0.084, 0.016, 0.020), (-0.069, -0.138, 0.021), "sand_light", rot=(0.58, 0.0, -0.22)),
        # Top overlapping scarf tail draping forward-right with woven stripes
        _box((0.086, 0.20, 0.018), (0.042, -0.050, 0.098), "sand_dark", rot=(0.52, 0.05, 0.28)),
        _box((0.088, 0.022, 0.020), (0.066, -0.112, 0.060), "sandstone_dark", rot=(0.52, 0.05, 0.28)),
        _box((0.088, 0.016, 0.020), (0.075, -0.135, 0.047), "sand_light", rot=(0.52, 0.05, 0.28)),
    ]
    # Fringe tassels at the tips of both folded tails
    for k in range(4):
        lateral = -0.027 + k * 0.018
        parts.append(_box(
            (0.008, 0.026, 0.008),
            (-0.074 + lateral * math.cos(-0.22), -0.156 + lateral * math.sin(-0.22), 0.009),
            "sand_light",
            rot=(0.58, 0.0, -0.22),
        ))
        parts.append(_box(
            (0.008, 0.026, 0.008),
            (0.083 + lateral * math.cos(0.28), -0.154 + lateral * math.sin(0.28), 0.035),
            "sand_light",
            rot=(0.52, 0.05, 0.28),
        ))
    return parts


def hand_warmer():
    # Small rectangular car_red heat pouch with a crimped seam border and white wavy heat squiggles
    parts = [
        _box((0.112, 0.014, 0.148), (0.0, 0.0, 0.074), "red_paint"),
        _box((0.094, 0.024, 0.128), (0.0, 0.0, 0.074), "car_red"),
    ]
    for x_base in (-0.020, 0.020):
        for step, z_mid in enumerate((0.043, 0.063, 0.083, 0.103)):
            sign = -1.0 if step % 2 == 0 else 1.0
            parts.append(_box(
                (0.008, 0.005, 0.025),
                (x_base + sign * 0.004, -0.0135, z_mid),
                "white",
                rot=(0.0, sign * 0.38, 0.0),
            ))
    return parts
