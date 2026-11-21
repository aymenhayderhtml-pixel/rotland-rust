"""Shared helpers and setup for player-built base parts (R17-05).

All parts use wood plank styling, low-poly aesthetics, and strict 2m grid compatibility.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import bmesh
from mathutils import Matrix, Vector

import common
import kit

CATEGORY = "building_parts"
BUDGET_STRUCTURE = 300
BUDGET_DOOR = 400

# Color palette choices for wood plank base building
COLOR_FRAME = "bark_dark"
COLOR_PLANK_A = "wood_crate"
COLOR_PLANK_B = "bark_light"
COLOR_METAL = "metal"
COLOR_ROOF = "rock_dark"


def box_bm(lo, hi, color):
    """Builds a single colored box bmesh from min to max coordinates."""
    size = [hi[i] - lo[i] for i in range(3)]
    loc = [(lo[i] + hi[i]) / 2 for i in range(3)]
    return common.make_box(size, loc=loc, color=color)


def plane_bm(a, b, span, width, thick, color):
    """Flat slab whose top surface lies on line a-b. Thickness extends downward."""
    return kit._plane_bm(a, b, span, width, thick, color)
