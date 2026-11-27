"""R41 loot container models: army_crate, metro_locker, med_cabinet, camp_stash.

Props: origin at bottom center, front +Y, with a box collider.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS_DIR = os.path.abspath(os.path.join(HERE, ".."))
sys.path.insert(0, TOOLS_DIR)

import common  # noqa: E402


def _rot_box(size, loc, color, rot=(0.0, 0.0, 0.0)):
    bm = common.make_box(size, color=color)
    common.transform(bm, rot=rot, loc=loc)
    return bm


def build_army_crate(model):
    """Long olive footlocker with stencils and latches."""
    # Main olive footlocker body + stepped lid
    model.add(common.make_box((1.08, 0.52, 0.34), loc=(0.0, 0.0, 0.17), color="cloth_olive"))
    model.add(common.make_box((1.12, 0.56, 0.08), loc=(0.0, 0.0, 0.38), color="cloth_olive"))
    model.add(common.make_box((1.10, 0.54, 0.016), loc=(0.0, 0.0, 0.34), color="gun_black"))
    # Reinforced black corner bumpers
    for sx in (-0.52, 0.52):
        for sy in (-0.24, 0.24):
            model.add(common.make_box((0.08, 0.08, 0.42), loc=(sx, sy, 0.21), color="gun_black"))
    # Front brass latches (+Y)
    for x in (-0.28, 0.28):
        model.add(common.make_box((0.06, 0.03, 0.11), loc=(x, 0.27, 0.32), color="brass"))
    # Side carrying handles
    for x in (-0.56, 0.56):
        model.add(common.make_box((0.03, 0.18, 0.03), loc=(x, 0.0, 0.22), color="metal"))
    # Stencil markings on front and top
    model.add(common.make_box((0.30, 0.01, 0.045), loc=(0.0, 0.262, 0.20), color="wheat_gold"))
    model.add(common.make_box((0.18, 0.01, 0.030), loc=(0.0, 0.262, 0.13), color="wheat_gold"))
    model.add(common.make_box((0.36, 0.08, 0.01), loc=(0.0, 0.0, 0.422), color="wheat_gold"))
    model.add_collider("box", common.make_box((1.12, 0.56, 0.42), loc=(0.0, 0.0, 0.21), color="cloth_olive"))


def build_metro_locker(model):
    """Dented grey metal locker, door ajar."""
    # Back, sides, top, bottom, and shelf (leaving front open)
    model.add(common.make_box((0.50, 0.03, 1.85), loc=(0.0, -0.235, 0.925), color="metal"))
    model.add(common.make_box((0.03, 0.47, 1.85), loc=(-0.235, 0.0, 0.925), color="metal"))
    model.add(common.make_box((0.03, 0.47, 1.85), loc=(0.235, 0.0, 0.925), color="metal"))
    model.add(common.make_box((0.50, 0.50, 0.04), loc=(0.0, 0.0, 1.83), color="rock_mid"))
    model.add(common.make_box((0.50, 0.50, 0.04), loc=(0.0, 0.0, 0.02), color="rock_dark"))
    model.add(common.make_box((0.44, 0.42, 0.025), loc=(0.0, -0.02, 1.38), color="rock_dark"))
    # Dark interior backing
    model.add(common.make_box((0.42, 0.40, 1.74), loc=(0.0, -0.02, 0.92), color="gun_black"))
    # Rust patches on side
    model.add(common.make_box((0.01, 0.16, 0.22), loc=(0.252, 0.04, 0.45), color="rust_dark"))
    # Door swung ajar ~38 deg from left hinge (-0.235, 0.235)
    hinge_x, hinge_y = -0.235, 0.235
    angle = -0.65
    door_w = 0.46
    cx = hinge_x + (door_w / 2) * math.cos(angle)
    cy = hinge_y + (door_w / 2) * math.sin(angle)
    model.add(_rot_box((door_w, 0.025, 1.76), (cx, cy, 0.925), "rock_light", rot=(0.0, 0.0, angle)))
    # Louvers & handle on door
    for z in (1.58, 1.52, 1.46, 0.30, 0.24):
        model.add(_rot_box((0.24, 0.032, 0.018), (cx, cy, z), "rock_dark", rot=(0.0, 0.0, angle)))
    hx = hinge_x + 0.39 * math.cos(angle)
    hy = hinge_y + 0.39 * math.sin(angle)
    model.add(_rot_box((0.025, 0.055, 0.14), (hx, hy, 0.95), "chrome", rot=(0.0, 0.0, angle)))
    model.add_collider("box", common.make_box((0.50, 0.50, 1.85), loc=(0.0, 0.0, 0.925), color="metal"))


def build_med_cabinet(model):
    """White wall cabinet with a red cross. Sits on the floor against a wall, 1.4 m tall."""
    # Plinth + white cabinet body (1.40 m tall)
    model.add(common.make_box((0.68, 0.28, 0.08), loc=(0.0, 0.0, 0.04), color="rock_mid"))
    model.add(common.make_box((0.70, 0.30, 1.32), loc=(0.0, 0.0, 0.74), color="car_white"))
    # Front door panel + slight gap seam
    model.add(common.make_box((0.64, 0.025, 1.24), loc=(0.0, 0.158, 0.74), color="white"))
    # Prominent red cross on upper front
    model.add(common.make_box((0.26, 0.012, 0.08), loc=(0.0, 0.172, 0.98), color="red_paint"))
    model.add(common.make_box((0.08, 0.012, 0.26), loc=(0.0, 0.172, 0.98), color="red_paint"))
    # Chrome handle & lock
    model.add(common.make_box((0.022, 0.035, 0.14), loc=(0.24, 0.180, 0.74), color="chrome"))
    model.add(common.make_box((0.025, 0.015, 0.035), loc=(0.24, 0.172, 0.58), color="metal"))
    model.add_collider("box", common.make_box((0.70, 0.32, 1.40), loc=(0.0, 0.0, 0.70), color="white"))


def build_camp_stash(model):
    """Tarp-covered pile of boxes and a cooler."""
    # Base wooden crate + cardboard box
    model.add(common.make_box((0.62, 0.48, 0.38), loc=(-0.16, -0.08, 0.19), color="wood_crate"))
    model.add(common.make_box((0.44, 0.38, 0.28), loc=(-0.12, -0.05, 0.52), color="canvas_khaki"))
    # Blue & white camping cooler beside it
    model.add(common.make_box((0.46, 0.34, 0.30), loc=(0.26, 0.14, 0.15), color="car_blue"))
    model.add(common.make_box((0.48, 0.36, 0.06), loc=(0.26, 0.14, 0.33), color="white"))
    # Draped blue tarp over the box stack
    model.add(_rot_box((0.72, 0.54, 0.04), (-0.14, -0.06, 0.67), "tarp_blue", rot=(0.08, -0.05, 0.1)))
    model.add(_rot_box((0.70, 0.04, 0.56), (-0.16, -0.32, 0.40), "tarp_blue", rot=(-0.12, 0.0, 0.0)))
    model.add(_rot_box((0.04, 0.52, 0.58), (-0.48, -0.08, 0.38), "tarp_blue", rot=(0.0, -0.15, 0.0)))
    # Rope tie-down
    model.add(_rot_box((0.74, 0.025, 0.025), (-0.14, -0.06, 0.68), "bark_light", rot=(0.08, -0.05, 0.1)))
    model.add_collider("box", common.make_box((1.05, 0.85, 0.68), loc=(0.0, 0.0, 0.34), color="tarp_blue"))
