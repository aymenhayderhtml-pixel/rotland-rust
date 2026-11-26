"""Stroke lettering for survivor graffiti and missing-person posters.

Copies jam_font.FONT and adds the missing letters (B, F, G, K, M, U) in the same unit cell.
Text reads left-to-right from +Y (the street side of a facade), so X is mirrored when placed.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))

import common  # noqa: E402
from jam_util import bar  # noqa: E402

# new: DECISIONS 2026-11-26 (unit-cell stroke font extending jam_font.FONT)
FONT = {
    "A": [(0, 0, 0.5, 1), (0.5, 1, 1, 0), (0.25, 0.45, 0.75, 0.45)],
    "B": [(0, 0, 0, 1), (0, 1, 0.72, 1), (0.72, 1, 1, 0.78), (1, 0.78, 0.72, 0.52),
          (0, 0.52, 0.72, 0.52), (0.72, 0.52, 1, 0.28), (1, 0.28, 0.72, 0), (0.72, 0, 0, 0)],
    "C": [(1, 1, 0, 1), (0, 1, 0, 0), (0, 0, 1, 0)],
    "D": [(0, 0, 0, 1), (0, 0, 0.6, 0), (0.6, 0, 1, 0.3), (1, 0.3, 1, 0.7), (1, 0.7, 0.6, 1), (0.6, 1, 0, 1)],
    "E": [(0, 0, 0, 1), (0, 0, 1, 0), (0, 0.5, 0.8, 0.5), (0, 1, 1, 1)],
    "F": [(0, 0, 0, 1), (0, 1, 1, 1), (0, 0.52, 0.8, 0.52)],
    "G": [(1, 1, 0, 1), (0, 1, 0, 0), (0, 0, 1, 0), (1, 0, 1, 0.48), (1, 0.48, 0.5, 0.48)],
    "H": [(0, 0, 0, 1), (1, 0, 1, 1), (0, 0.5, 1, 0.5)],
    "I": [(0.5, 0, 0.5, 1)],
    "K": [(0, 0, 0, 1), (1, 1, 0.08, 0.5), (0.08, 0.5, 1, 0)],
    "L": [(0, 0, 0, 1), (0, 0, 1, 0)],
    "M": [(0, 0, 0, 1), (0, 1, 0.5, 0.35), (0.5, 0.35, 1, 1), (1, 1, 1, 0)],
    "N": [(0, 0, 0, 1), (0, 1, 1, 0), (1, 0, 1, 1)],
    "O": [(0, 0, 1, 0), (1, 0, 1, 1), (1, 1, 0, 1), (0, 1, 0, 0)],
    "R": [(0, 0, 0, 1), (0, 1, 1, 1), (1, 1, 1, 0.5), (1, 0.5, 0, 0.5), (0.4, 0.5, 1, 0)],
    "S": [(1, 1, 0, 1), (0, 1, 0, 0.5), (0, 0.5, 1, 0.5), (1, 0.5, 1, 0), (1, 0, 0, 0)],
    "T": [(0, 1, 1, 1), (0.5, 1, 0.5, 0)],
    "U": [(0, 1, 0, 0), (0, 0, 1, 0), (1, 0, 1, 1)],
    "V": [(0, 1, 0.5, 0), (0.5, 0, 1, 1)],
    "W": [(0, 1, 0.25, 0), (0.25, 0, 0.5, 0.6), (0.5, 0.6, 0.75, 0), (0.75, 0, 1, 1)],
}


def quad_bar(x0, z0, x1, z1, y, thick, color):
    """Single +Y-facing quad stroke from (x0, z0) to (x1, z1) on plane y (2 tris)."""
    dx, dz = x1 - x0, z1 - z0
    length = math.hypot(dx, dz)
    if length < 1e-5:
        return common.new_bm()
    tx, tz = dx / length, dz / length
    nx, nz = -tz, tx
    ext = thick * 0.35  # new: DECISIONS 2026-11-26 (overlap corners cleanly)
    ht = thick * 0.5
    ax, az = x0 - tx * ext, z0 - tz * ext
    bx, bz = x1 + tx * ext, z1 + tz * ext
    corners = [
        (ax - nx * ht, y, az - nz * ht),
        (bx - nx * ht, y, bz - nz * ht),
        (bx + nx * ht, y, bz + nz * ht),
        (ax + nx * ht, y, az + nz * ht),
    ]
    bm = common.new_bm()
    common.add_quad(bm, corners, outward=(0.0, 1.0, 0.0), color=color)
    return bm


def text_bms(text, centre_x, base_z, letter_w, letter_h, y, color, depth=0.04, bevel=0.0,
             flat=False, thick_scale=1.0):
    """Bars spelling `text`, centred on centre_x, from base_z up, on plane y (reads from +Y)."""
    gap = letter_w * 0.35  # new: DECISIONS 2026-11-26
    space = letter_w * 0.6  # new: DECISIONS 2026-11-26
    total = sum(space if ch == " " else letter_w for ch in text) + gap * (len(text) - 1)
    thick = letter_h * 0.12 * thick_scale  # new: DECISIONS 2026-11-26
    x = centre_x - total / 2
    out = []
    for ch in text:
        if ch == " ":
            x += space + gap
            continue
        for u0, v0, u1, v1 in FONT[ch]:
            x0, z0 = -(x + u0 * letter_w), base_z + v0 * letter_h
            x1, z1 = -(x + u1 * letter_w), base_z + v1 * letter_h
            if flat:
                out.append(quad_bar(x0, z0, x1, z1, y, thick, color))
            else:
                out.append(bar(x0, z0, x1, z1, y, thick, depth, color, bevel=bevel))
        x += letter_w + gap
    return out
