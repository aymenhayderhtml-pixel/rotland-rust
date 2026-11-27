"""R41 rare medicine & parts models: antibiotics, surgery_kit, adrenaline_shot, gun_parts, car_parts.

Build coordinates: +Y is forward, +Z is up, 1 unit = 1 m.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS_DIR = os.path.abspath(os.path.join(HERE, ".."))
sys.path.insert(0, TOOLS_DIR)
sys.path.insert(0, os.path.join(TOOLS_DIR, "weapons"))

import common  # noqa: E402
from parts import box, cyl_y  # noqa: E402


def _rot_box(size, loc, color, rot=(0.0, 0.0, 0.0)):
    bm = common.make_box(size, color=color)
    common.transform(bm, rot=rot, loc=loc)
    return bm


def build_antibiotics(model):
    """Pill bottle + blister pack."""
    # Amber pill bottle + white cap + label with red stripe facing -Y
    model.add(common.make_frustum(0.038, 0.038, 0.0, 0.115, segments=10, center=(-0.045, 0.0), color="carrot_orange"))
    model.add(common.make_frustum(0.040, 0.040, 0.030, 0.090, segments=10, center=(-0.045, 0.0), color="white"))
    model.add(box((0.030, 0.010, 0.020), (-0.045, -0.038, 0.060), "red_paint"))
    model.add(common.make_frustum(0.042, 0.042, 0.115, 0.142, segments=10, center=(-0.045, 0.0), color="white"))
    # Blister pack leaning next to the bottle, pills facing -Y
    pack_rot = (0.22, 0.0, -0.25)
    model.add(_rot_box((0.085, 0.008, 0.115), (0.045, 0.015, 0.055), "chrome", rot=pack_rot))
    for row in (-0.025, 0.025):
        for col in (-0.020, 0.020):
            pill = common.make_icosphere(subdiv=1, radius=(0.012, 0.009, 0.012), color="white")
            common.transform(pill, loc=(col, -0.008, row))
            common.transform(pill, rot=pack_rot, loc=(0.045, 0.015, 0.055))
            model.add(pill)


def build_surgery_kit(model):
    """Orange hard case with a red cross, half open showing tools."""
    # Bottom half of case + dark foam insert
    model.add(box((0.26, 0.18, 0.055), (0.0, 0.0, 0.0275), "carrot_orange"))
    model.add(box((0.23, 0.15, 0.015), (0.0, 0.0, 0.052), "gun_black"))
    # Front white badge + red cross on the lower case lip (-Y)
    model.add(box((0.050, 0.006, 0.042), (0.0, -0.091, 0.028), "white"))
    model.add(box((0.034, 0.008, 0.012), (0.0, -0.092, 0.028), "red_paint"))
    model.add(box((0.012, 0.008, 0.034), (0.0, -0.092, 0.028), "red_paint"))
    # Surgical tools inside: scalpel, forceps, rolled gauze
    model.add(_rot_box((0.14, 0.014, 0.008), (-0.02, -0.03, 0.063), "chrome", rot=(0.0, 0.0, 0.15)))
    model.add(_rot_box((0.12, 0.016, 0.008), (0.01, 0.02, 0.063), "metal", rot=(0.0, 0.0, -0.1)))
    model.add(cyl_y(0.018, 0.018, -0.05, 0.04, "white", x=0.075, z=0.070, segments=8))
    # Hinged lid tilted open (~55 deg back around +Y edge so interior & cross face -Y)
    lid_rot = (0.95, 0.0, 0.0)
    lid_loc = (0.0, 0.12, 0.125)
    model.add(_rot_box((0.26, 0.18, 0.035), lid_loc, "carrot_orange", rot=lid_rot))
    # White badge + red cross on top/inside of lid
    badge = common.make_box((0.09, 0.09, 0.008), color="white")
    cross_h = common.make_box((0.065, 0.022, 0.012), color="red_paint")
    cross_v = common.make_box((0.022, 0.065, 0.012), color="red_paint")
    for part in (badge, cross_h, cross_v):
        common.transform(part, loc=(0.0, 0.0, -0.020))
        common.transform(part, rot=lid_rot, loc=lid_loc)
        model.add(part)


def build_adrenaline_shot(model):
    """Auto-injector pen, yellow and grey."""
    # Main yellow barrel + grey grip collar + white label
    parts = [
        common.make_frustum(0.016, 0.016, 0.0, 0.12, segments=10, color="brass"),
        common.make_frustum(0.018, 0.018, 0.03, 0.07, segments=10, color="rock_mid"),
        common.make_frustum(0.017, 0.017, 0.075, 0.11, segments=10, color="white"),
        # Grey/black needle shroud tip
        common.make_frustum(0.015, 0.011, 0.12, 0.155, segments=10, color="gun_black"),
        # Red safety cap at base
        common.make_frustum(0.018, 0.018, -0.028, 0.0, segments=10, color="red_paint"),
    ]
    for p in parts:
        # Tilt slightly so it reads dynamically in 3/4 icon and ground view
        common.transform(p, rot=(0.0, 0.45, 0.0), loc=(-0.03, 0.0, 0.03))
        model.add(p)


def build_gun_parts(model):
    """Small oily cloth bundle with springs, a bolt and a barrel piece."""
    # Oily dark cloth bundle
    model.add(box((0.22, 0.16, 0.028), (0.0, 0.0, 0.014), "dirt"))
    model.add(_rot_box((0.18, 0.12, 0.020), (-0.01, 0.01, 0.034), "bark_dark", rot=(0.0, 0.0, 0.2)))
    # Barrel piece
    model.add(cyl_y(0.012, 0.010, -0.09, 0.09, "gun_black", x=-0.04, z=0.056, segments=10))
    # Bolt carrier
    model.add(_rot_box((0.026, 0.095, 0.024), (0.035, 0.01, 0.056), "chrome", rot=(0.0, 0.0, -0.25)))
    # Recoil spring (segmented rings along X)
    for i in range(6):
        ring = common.make_frustum(0.013, 0.013, -0.004, 0.004, segments=8, color="metal")
        common.transform(ring, rot=(0.0, math.pi / 2, 0.3), loc=(-0.03 + i * 0.016, -0.045 + i * 0.005, 0.058))
        model.add(ring)


def build_car_parts(model):
    """Box with a spark plug set, a belt and a filter."""
    # Open cardboard box
    model.add(box((0.26, 0.18, 0.08), (0.0, 0.0, 0.04), "wood_crate"))
    model.add(box((0.23, 0.15, 0.015), (0.0, 0.0, 0.075), "bark_dark"))
    # Blue oil filter
    model.add(common.make_frustum(0.040, 0.040, 0.075, 0.165, segments=10, center=(-0.055, 0.01), color="car_blue"))
    model.add(common.make_frustum(0.036, 0.036, 0.165, 0.172, segments=10, center=(-0.055, 0.01), color="metal"))
    # Two spark plugs (ceramic white + metal hex)
    for x in (0.035, 0.075):
        model.add(common.make_frustum(0.012, 0.012, 0.075, 0.11, segments=6, center=(x, -0.02), color="metal"))
        model.add(common.make_frustum(0.009, 0.009, 0.11, 0.15, segments=8, center=(x, -0.02), color="white"))
        model.add(common.make_frustum(0.004, 0.004, 0.15, 0.162, segments=6, center=(x, -0.02), color="brass"))
    # Serpentine belt loop (8-segment octagonal ring)
    for i in range(8):
        a = i * math.pi / 4
        seg = common.make_box((0.045, 0.012, 0.016), color="tire_black")
        common.transform(seg, rot=(0.0, 0.0, a), loc=(0.03 + 0.055 * math.sin(a), 0.03 - 0.055 * math.cos(a), 0.095))
        model.add(seg)
