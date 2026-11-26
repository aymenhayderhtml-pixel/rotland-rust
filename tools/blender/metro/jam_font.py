"""Stroke lettering for signs (gantry, bus destination board). Letters are flat bars on a plane.

Each letter is a list of strokes (x0, y0, x1, y1) in a unit cell (y up). Text reads correctly from +Y
(the front of the sign), so x is mirrored when the bars are placed.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from jam_util import bar  # noqa: E402

FONT = {
    "A": [(0, 0, 0.5, 1), (0.5, 1, 1, 0), (0.25, 0.45, 0.75, 0.45)],
    "C": [(1, 1, 0, 1), (0, 1, 0, 0), (0, 0, 1, 0)],
    "D": [(0, 0, 0, 1), (0, 0, 0.6, 0), (0.6, 0, 1, 0.3), (1, 0.3, 1, 0.7), (1, 0.7, 0.6, 1), (0.6, 1, 0, 1)],
    "E": [(0, 0, 0, 1), (0, 0, 1, 0), (0, 0.5, 0.8, 0.5), (0, 1, 1, 1)],
    "H": [(0, 0, 0, 1), (1, 0, 1, 1), (0, 0.5, 1, 0.5)],
    "I": [(0.5, 0, 0.5, 1)],
    "L": [(0, 0, 0, 1), (0, 0, 1, 0)],
    "N": [(0, 0, 0, 1), (0, 1, 1, 0), (1, 0, 1, 1)],
    "O": [(0, 0, 1, 0), (1, 0, 1, 1), (1, 1, 0, 1), (0, 1, 0, 0)],
    "R": [(0, 0, 0, 1), (0, 1, 1, 1), (1, 1, 1, 0.5), (1, 0.5, 0, 0.5), (0.4, 0.5, 1, 0)],
    "S": [(1, 1, 0, 1), (0, 1, 0, 0.5), (0, 0.5, 1, 0.5), (1, 0.5, 1, 0), (1, 0, 0, 0)],
    "T": [(0, 1, 1, 1), (0.5, 1, 0.5, 0)],
    "V": [(0, 1, 0.5, 0), (0.5, 0, 1, 1)],
    "W": [(0, 1, 0.25, 0), (0.25, 0, 0.5, 0.6), (0.5, 0.6, 0.75, 0), (0.75, 0, 1, 1)],
}


def text_bms(text, centre_x, base_z, letter_w, letter_h, y, color, depth=0.04, bevel=0.0):
    """Bars spelling `text`, centred on centre_x, from base_z up, on the plane at height y (reads from +Y)."""
    gap = letter_w * 0.35
    space = letter_w * 0.6
    total = sum(space if ch == " " else letter_w for ch in text) + gap * (len(text) - 1)
    thick = letter_h * 0.11
    x = centre_x - total / 2
    out = []
    for ch in text:
        if ch == " ":
            x += space + gap
            continue
        for u0, v0, u1, v1 in FONT[ch]:
            out.append(bar(-(x + u0 * letter_w), base_z + v0 * letter_h, -(x + u1 * letter_w),
                           base_z + v1 * letter_h, y, thick, depth, color, bevel=bevel))
        x += letter_w + gap
    return out
