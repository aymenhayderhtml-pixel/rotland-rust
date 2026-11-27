"""R41 army gear & rifle models: army_rifle, body_armor, army_helmet, ammo_556, army_ration, night_vision, battery.

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
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402

ARMY_RIFLE_GRIP = (0.0, 0.0, 0.0)


def _rot_box(size, loc, color, rot=(0.0, 0.0, 0.0)):
    bm = common.make_box(size, color=color)
    common.transform(bm, rot=rot, loc=loc)
    return bm


def build_army_rifle(model):
    """Modern service rifle: flat-top rail, adjustable stock, 30-round curved mag, olive & black."""
    # Lower & upper receiver
    model.add(rbox((0.046, 0.21, 0.056), (0.0, 0.08, 0.052), "cloth_olive", 0.2))
    model.add(rbox((0.044, 0.21, 0.042), (0.0, 0.08, 0.088), "gun_black", 0.2))
    # Ejection port, brass deflector, charging handle, forward assist
    model.add(box((0.004, 0.065, 0.018), (0.023, 0.10, 0.090), "metal"))
    model.add(box((0.010, 0.014, 0.016), (0.025, 0.055, 0.090), "gun_black"))
    model.add(rbox((0.026, 0.016, 0.010), (0.0, -0.030, 0.102), "metal", 0.25))
    # Full-length flat-top Picatinny rail + notches
    model.add(rbox((0.024, 0.36, 0.012), (0.0, 0.15, 0.114), "gun_black", 0.2))
    for i in range(10):
        y = -0.01 + i * 0.034
        model.add(box((0.026, 0.010, 0.004), (0.0, y, 0.120), "rock_dark"))
    # Flip-up rear & front iron sights
    model.add(rbox((0.022, 0.024, 0.022), (0.0, -0.015, 0.130), "gun_black", 0.2))
    model.add(rbox((0.020, 0.020, 0.024), (0.0, 0.32, 0.130), "gun_black", 0.2))
    model.add(box((0.004, 0.004, 0.014), (0.0, 0.32, 0.145), "metal"))
    # Handguard (olive M-LOK / quad-rail forend) + short foregrip area
    model.add(rbox((0.050, 0.20, 0.054), (0.0, 0.28, 0.072), "cloth_olive", 0.25))
    model.add(rbox((0.026, 0.16, 0.012), (0.0, 0.28, 0.042), "gun_black", 0.2))
    for x in (-0.026, 0.026):
        model.add(box((0.004, 0.16, 0.016), (x, 0.28, 0.072), "gun_black"))
    # Barrel, gas block, and flash hider
    model.add(cyl_y(0.010, 0.009, 0.38, 0.52, "gun_black", z=0.074, segments=10))
    model.add(rbox((0.022, 0.026, 0.032), (0.0, 0.40, 0.080), "gun_black", 0.2))
    model.add(loft_y([(0.52, 0.012), (0.555, 0.012), (0.560, 0.009)], "metal", z=0.074, segments=10))
    # Buffer tube & adjustable stock
    model.add(cyl_y(0.016, 0.016, -0.22, -0.02, "gun_black", z=0.076, segments=10))
    model.add(hull_side([(-0.08, 0.092), (-0.34, 0.095), (-0.34, -0.010), (-0.22, 0.010), (-0.08, 0.056)],
                        0.036, "cloth_olive"))
    model.add(rbox((0.040, 0.016, 0.106), (0.0, -0.348, 0.042), "tire_black", 0.25))
    # Pistol grip (origin at GRIP), trigger guard & trigger
    model.add(hull_side([(-0.022, 0.032), (0.022, 0.032), (0.005, -0.062), (-0.042, -0.062)],
                        0.032, "gun_black"))
    model.add(box((0.010, 0.055, 0.007), (0.0, 0.052, -0.012), "gun_black"))
    model.add(box((0.010, 0.007, 0.036), (0.0, 0.080, 0.008), "gun_black"))
    model.add(box((0.006, 0.006, 0.020), (0.0, 0.048, 0.012), "metal"))
    # Magazine well & 30-round curved magazine (own node `mag`)
    model.add(rbox((0.044, 0.068, 0.032), (0.0, 0.12, 0.022), "cloth_olive", 0.15))
    mag_outline = [(0.085, 0.02), (0.145, 0.02), (0.155, -0.07), (0.135, -0.155),
                   (0.095, -0.165), (0.090, -0.09), (0.085, -0.02)]
    model.add_part("mag", prism_yz(mag_outline, 0.030, "gun_black"))
    # Empties (grip_L + attachment mounts matching weapon_mounts.json)
    model.add_empty("grip_L", (0.0, 0.27, 0.015))
    model.add_empty("mount_optic", (0.0, 0.09, 0.120))
    model.add_empty("mount_muzzle", (0.0, 0.560, 0.074))
    model.add_empty("mount_magazine", (0.0, 0.115, -0.100))
    model.add_empty("mount_grip", (0.0, 0.275, 0.036))
    model.add_empty("mount_stock", (0.0, -0.356, 0.042))


def build_body_armor(model):
    """Plate carrier vest with pouches, olive, straps."""
    # Front (-Y for icon/preview) and back (+Y) ballistic plates
    model.add(_rot_box((0.30, 0.040, 0.34), (0.0, -0.085, 0.18), "cloth_olive", rot=(-0.06, 0.0, 0.0)))
    model.add(_rot_box((0.30, 0.040, 0.34), (0.0, 0.085, 0.18), "cloth_olive", rot=(0.06, 0.0, 0.0)))
    # Top beveled plate shoulders
    model.add(box((0.20, 0.038, 0.06), (0.0, -0.075, 0.36), "cloth_olive"))
    model.add(box((0.20, 0.038, 0.06), (0.0, 0.075, 0.36), "cloth_olive"))
    # Padded shoulder straps
    for x in (-0.11, 0.11):
        model.add(box((0.065, 0.16, 0.024), (x, 0.0, 0.39), "canvas_khaki"))
        model.add(box((0.055, 0.035, 0.030), (x, -0.06, 0.392), "tire_black"))
    # Side cummerbund straps
    for x in (-0.155, 0.155):
        model.add(box((0.022, 0.16, 0.11), (x, 0.0, 0.12), "cloth_olive"))
        model.add(box((0.026, 0.165, 0.020), (x, 0.0, 0.12), "tire_black"))
    # Three front mag pouches + flaps (facing -Y)
    for x in (-0.09, 0.0, 0.09):
        model.add(box((0.075, 0.045, 0.13), (x, -0.122, 0.11), "canvas_khaki"))
        model.add(box((0.078, 0.048, 0.030), (x, -0.124, 0.165), "cloth_olive"))
        model.add(box((0.020, 0.010, 0.025), (x, -0.148, 0.155), "tire_black"))
    # Upper chest admin pouch + brass patch
    model.add(box((0.16, 0.032, 0.085), (0.0, -0.116, 0.265), "pine_dark"))
    model.add(box((0.07, 0.008, 0.035), (0.0, -0.133, 0.270), "brass"))


def build_army_helmet(model):
    """Rounded combat helmet with a strap and a goggle band."""
    # Domed shell built from stacked frustums
    model.add(common.make_frustum(0.140, 0.135, 0.08, 0.14, segments=12, color="cloth_olive"))
    model.add(common.make_frustum(0.135, 0.110, 0.14, 0.20, segments=12, color="cloth_olive"))
    model.add(common.make_frustum(0.110, 0.055, 0.20, 0.235, segments=12, color="cloth_olive"))
    model.add(common.make_frustum(0.055, 0.0, 0.235, 0.245, segments=12, color="cloth_olive"))
    # Rim trim
    model.add(common.make_frustum(0.144, 0.144, 0.072, 0.086, segments=12, color="pine_dark"))
    # Front NVG shroud (-Y) + side accessory rails
    model.add(box((0.045, 0.022, 0.050), (0.0, -0.138, 0.155), "gun_black"))
    for x in (-0.138, 0.138):
        model.add(box((0.014, 0.095, 0.022), (x, 0.0, 0.115), "rock_dark"))
    # Goggle band around helmet + tactical goggles on front (-Y)
    model.add(common.make_frustum(0.146, 0.146, 0.105, 0.125, segments=12, color="tire_black"))
    model.add(box((0.14, 0.036, 0.054), (0.0, -0.145, 0.115), "gun_black"))
    model.add(box((0.125, 0.010, 0.040), (0.0, -0.162, 0.115), "glass_tint"))
    model.add(box((0.016, 0.014, 0.044), (0.0, -0.162, 0.115), "gun_black"))
    # Chin strap hanging down
    for x in (-0.115, 0.115):
        model.add(_rot_box((0.016, 0.022, 0.10), (x, 0.02, 0.04), "leather", rot=(0.0, -x * 1.4, 0.0)))
    model.add(box((0.16, 0.022, 0.016), (0.0, 0.02, 0.008), "leather"))
    model.add(box((0.024, 0.026, 0.020), (-0.04, 0.02, 0.008), "metal"))


def build_ammo_556(model):
    """Small cardboard box with a few loose 5.56 rounds."""
    # Cardboard box + olive band + white stencil strip
    model.add(box((0.14, 0.09, 0.065), (-0.02, 0.01, 0.0325), "wood_crate"))
    model.add(box((0.144, 0.094, 0.022), (-0.02, 0.01, 0.035), "cloth_olive"))
    model.add(box((0.08, 0.004, 0.016), (-0.02, 0.058, 0.050), "white"))
    # 4 loose 5.56mm rounds lying in front and standing
    for i, x in enumerate((-0.05, -0.01, 0.03)):
        y = -0.075 - (i % 2) * 0.012
        casing = common.make_frustum(0.009, 0.009, -0.025, 0.020, segments=8, color="brass")
        bullet = common.make_frustum(0.008, 0.002, 0.020, 0.042, segments=8, color="rust")
        for p in (casing, bullet):
            common.transform(p, rot=(math.pi / 2, 0.0, 0.25 - i * 0.2), loc=(x, y, 0.010))
            model.add(p)
    # 2 upright rounds next to the box
    for x, y in ((0.085, 0.02), (0.095, -0.015)):
        model.add(common.make_frustum(0.009, 0.009, 0.0, 0.045, segments=8, center=(x, y), color="brass"))
        model.add(common.make_frustum(0.008, 0.002, 0.045, 0.068, segments=8, center=(x, y), color="rust"))


def build_army_ration(model):
    """Brown MRE-style pouch with crimped seals and a printed label block."""
    # Main slightly puffed pouch
    model.add(box((0.18, 0.048, 0.22), (0.0, 0.0, 0.13), "potato_brown"))
    model.add(box((0.16, 0.056, 0.17), (0.0, 0.0, 0.13), "potato_brown"))
    # Top and bottom flat heat-crimped seals
    model.add(box((0.19, 0.012, 0.024), (0.0, 0.0, 0.248), "bark_light"))
    model.add(box((0.19, 0.012, 0.020), (0.0, 0.0, 0.010), "bark_light"))
    # Printed label block on front (-Y) and back (+Y)
    for sign in (-1.0, 1.0):
        model.add(box((0.12, 0.006, 0.10), (0.0, sign * 0.029, 0.145), "canvas_khaki"))
        model.add(box((0.09, 0.004, 0.018), (0.0, sign * 0.032, 0.172), "bark_dark"))
        model.add(box((0.09, 0.004, 0.012), (0.0, sign * 0.032, 0.140), "bark_dark"))
        model.add(box((0.06, 0.004, 0.012), (-0.015, sign * 0.032, 0.115), "cloth_olive"))


def build_night_vision(model):
    """Head mount with twin tubes and a battery box."""
    # Head harness ring & top strap
    model.add(common.make_frustum(0.115, 0.115, 0.05, 0.08, segments=12, color="tire_black"))
    model.add(box((0.035, 0.22, 0.018), (0.0, 0.0, 0.115), "cloth_olive"))
    # Front flip-down mount bracket
    model.add(box((0.06, 0.045, 0.055), (0.0, 0.125, 0.075), "gun_black"))
    model.add(box((0.11, 0.025, 0.020), (0.0, 0.155, 0.068), "metal"))
    # Twin NVG optic tubes + glowing green-blue objective lenses
    for x in (-0.042, 0.042):
        model.add(cyl_y(0.024, 0.028, 0.14, 0.23, "gun_black", x=x, z=0.068, segments=10))
        model.add(cyl_y(0.020, 0.020, 0.228, 0.233, "water_blue", x=x, z=0.068, segments=8))
        model.add(cyl_y(0.018, 0.018, 0.136, 0.141, "leaf_light", x=x, z=0.068, segments=8))
    # Rear battery box + connecting cable
    model.add(box((0.09, 0.045, 0.065), (0.0, -0.125, 0.065), "gun_black"))
    model.add(box((0.05, 0.010, 0.025), (0.0, -0.148, 0.065), "brass"))
    model.add(box((0.012, 0.16, 0.012), (0.115, 0.0, 0.085), "tire_black"))


def build_battery(model):
    """Chunky square battery with metal terminals."""
    # Main dark casing
    model.add(box((0.095, 0.095, 0.115), (0.0, 0.0, 0.0575), "gun_black"))
    # Yellow/brass top band & red stripe
    model.add(box((0.098, 0.098, 0.032), (0.0, 0.0, 0.096), "brass"))
    model.add(box((0.098, 0.098, 0.014), (0.0, 0.0, 0.068), "red_paint"))
    # Positive & negative terminals on top
    model.add(common.make_frustum(0.014, 0.014, 0.115, 0.138, segments=8, center=(-0.022, 0.0), color="chrome"))
    model.add(common.make_frustum(0.018, 0.014, 0.115, 0.134, segments=6, center=(0.022, 0.0), color="metal"))
    model.add(box((0.018, 0.005, 0.004), (-0.022, 0.049, 0.102), "red_paint"))
