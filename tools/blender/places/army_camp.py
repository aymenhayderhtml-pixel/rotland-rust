"""R42-06 the army line camp: `evac_tent`, `field_kitchen`, `loudspeaker_pole`, `checkpoint_booth`,
`army_gear_scatter`, `flood_light` (+ `_lod1` where they are seen from far).

Everything is olive/khaki army kit with the stencilled word "EVACUATION" on the tent and plain painted
marks elsewhere: no insignia, no stars.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import bmesh  # noqa: E402

import common  # noqa: E402
from army_line import (DARK, KHAKI, METAL, OLIVE, beam_at, box, pbox, pbox_at,  # noqa: E402
                       sandbag, stencil_on_side, stencil_text)
from car_parts import _beam  # noqa: E402
from jam_util import ground_rim  # noqa: E402

RUST = "rust"
SOOT = "soot"
WHITE = "white"

# new: DECISIONS 2026-11-26 (R42 camp props: 4000 tris each, _lod1 1200)
PROP_BUDGET = 4000
PROP_LOD1_BUDGET = 1200

TENT_HALF_X, TENT_HALF_Y = 4.0, 2.5  # 8 x 5 m frame tent
TENT_EAVE, TENT_RIDGE = 2.05, 3.0


def _panel_loft(rings, color):
    """Open-ended loft (no caps) for a tube of fabric or sheet metal."""
    bm = common.new_bm()
    rv = [[bm.verts.new(p) for p in ring] for ring in rings]
    n = len(rings[0])
    for a, b in zip(rv, rv[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new([a[i], a[j], b[j], b[i]])
    common.paint(bm, color)
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


# --- evac_tent ---------------------------------------------------------------------------------
def _build_evac_tent(model, lod=False):
    """8 x 5 m white/olive frame tent: gable roof, open flap at the front, "EVACUATION" on the long side."""
    hx, hy = TENT_HALF_X, TENT_HALF_Y
    parts = []
    if lod:
        parts.append(box((-hx, -hy, 0.0), (hx, hy, 0.18), KHAKI))
        parts.append(box((-hx, -hy, TENT_EAVE - 0.3), (hx, hy, TENT_EAVE), WHITE))
        roof = common.make_box((hx * 2.0, hy * 2.0, 0.12), color=WHITE)
        common.transform(roof, rot=(0.0, 0.0, 0.0), loc=(0.0, 0.0, (TENT_EAVE + TENT_RIDGE) / 2))
        parts.append(roof)
        for bm in parts:
            model.add(bm)
        model.add_collider("tent", box((-hx, -hy, 0.0), (hx, hy, TENT_RIDGE), "white"))
        return
    rng = random.Random(8201)
    # ground sheet and interior floor
    parts.append(pbox((-hx, -hy, 0.0), (hx, hy, 0.12), "limestone_dark", bevel=0.02))
    parts.append(pbox((-hx + 0.2, -hy + 0.2, 0.12), (hx - 0.2, hy - 0.2, 0.16), KHAKI, bevel=0.0))
    # walls: olive lower skirt, white canvas upper, with poles showing through
    for side in (-1, 1):
        y0, y1 = sorted((side * hy, side * (hy - 0.06)))
        parts.append(pbox((-hx, y0, 0.1), (hx, y1, 0.9), OLIVE, bevel=0.0))
        parts.append(pbox((-hx, y0, 0.9), (hx, y1, TENT_EAVE), WHITE, bevel=0.0))
        for k in range(5):  # frame uprights
            x = -hx + 0.1 + k * (hx * 2.0 - 0.2) / 4.0
            parts.append(box((x - 0.045, y0 - side * 0.02, 0.0), (x + 0.045, y1 + side * 0.03, TENT_EAVE), METAL))
        # gable roof slopes
        slope = math.hypot(hy, TENT_RIDGE - TENT_EAVE)
        ang = math.atan2(TENT_RIDGE - TENT_EAVE, hy)
        panel = common.make_box((hx * 2.0 + 0.2, slope + 0.16, 0.05), color=WHITE)
        common.transform(panel, rot=(side * ang, 0.0, 0.0),
                         loc=(0.0, side * hy / 2.0, TENT_EAVE + (TENT_RIDGE - TENT_EAVE) / 2.0))
        parts.append(panel)
        # ribs on the roof and a khaki repair patch
        for k in range(5):
            x = -hx + 0.1 + k * (hx * 2.0 - 0.2) / 4.0
            parts.append(_beam((x, side * hy, TENT_EAVE), (x, 0.0, TENT_RIDGE), 0.03, color=METAL, segments=4))
        if side > 0:
            patch = common.make_box((1.4, 1.1, 0.03), color=KHAKI)
            common.transform(patch, rot=(ang, 0.0, 0.0), loc=(-1.6, 1.4, TENT_EAVE + 0.5))
            parts.append(patch)
    # end walls: back closed, front open with the flap rolled up on one side
    parts.append(pbox((-hx, -hy, 0.1), (hx, -hy + 0.06, TENT_EAVE), WHITE, bevel=0.0))
    back_gable = common.new_bm()
    common.add_quad(back_gable, [(-hx, -hy + 0.05, TENT_EAVE), (hx, -hy + 0.05, TENT_EAVE),
                                 (hx, -hy + 0.05, TENT_RIDGE), (-hx, -hy + 0.05, TENT_RIDGE)],
                    (-1.0, 0.0, 0.0), WHITE)
    parts.append(back_gable)
    for x0, x1 in ((-hx, -hx + 0.7), (hx - 0.7, hx)):
        parts.append(pbox((x0, hy - 0.06, 0.1), (x1, hy, TENT_EAVE), WHITE, bevel=0.0))
    parts.append(box((-hx, hy - 0.09, TENT_EAVE), (hx, hy - 0.02, TENT_EAVE + 0.12), METAL))  # lintel pole
    # the flap: rolled up and tied at the right of the opening
    flap = common.make_frustum(0.16, 0.16, -1.1, 1.1, segments=8, color=WHITE)
    common.transform(flap, rot=(math.pi / 2.0, 0.0, 0.0), loc=(-hx + 1.1, hy - 0.2, TENT_EAVE - 0.35))
    parts.append(flap)
    for x in (-hx + 0.55, -hx + 1.65):
        parts.append(box((x - 0.07, hy - 0.28, TENT_EAVE - 0.62), (x + 0.07, hy - 0.12, TENT_EAVE - 0.12), KHAKI))
    # "EVACUATION" stencilled down both long sides (plain lettering, no insignia)
    for side, out in ((-1, -1), (1, 1)):
        parts.append(stencil_on_side("EVACUATION", 0.30, 0.42, side * (hx + 0.008), 1.24, 0.0, "soot",
                                     outward=out))
    parts.append(stencil_on_side("A-1", 0.2, 0.26, -(hx + 0.008), 1.95, 2.6, "soot", outward=-1))
    # guy ropes, stakes and a sandbag ring holding the skirt down
    for side in (-1, 1):
        for x in (-hx + 0.3, 0.0, hx - 0.3):
            top = (x, side * (hy + 0.05), TENT_EAVE + 0.05)
            stake = (x + 0.5 * side, side * (hy + 1.0), 0.0)
            parts.append(_beam(top, stake, 0.014, color=KHAKI, segments=4))
            parts.append(_beam((stake[0], stake[1], 0.0), (stake[0], stake[1], 0.22), 0.025, color="bark_light",
                               segments=4))
    for k in range(7):
        x = -hx + 0.6 + k * 1.1
        for side in (-1, 1):
            parts.append(sandbag(0.5, 0.34, 0.2, (x, side * (hy + 0.1), 0.1), rng.uniform(-0.15, 0.15), KHAKI,
                                 seed=8200 + k))
    # grime and a torn seam along the roof ridge
    for _ in range(8):
        x = rng.uniform(-hx + 0.3, hx - 0.3)
        side = rng.choice((-1, 1))
        parts.append(box((x, side * rng.uniform(0.2, hy - 0.2), TENT_EAVE + 0.02),
                         (x + 0.35, side * rng.uniform(0.2, hy - 0.2) + 0.3, TENT_EAVE + 0.06),
                         rng.choice(("concrete_stain", "grime_green", "dirt"))))
    parts.append(_beam((-hx, 0.0, TENT_RIDGE), (hx, 0.0, TENT_RIDGE), 0.04, color=METAL, segments=5))
    for bm in parts:
        model.add(bm)
    model.add_collider("wall_n", box((-hx, hy - 0.12, 0.0), (hx, hy, TENT_EAVE), "white"))
    model.add_collider("wall_s", box((-hx, -hy, 0.0), (hx, -hy + 0.12, TENT_EAVE), "white"))
    for side in (-1, 1):
        x0, x1 = sorted((side * hx, side * (hx - 0.12)))
        model.add_collider("end", box((x0, -hy, 0.0), (x1, hy, TENT_EAVE), "white"))


# --- field_kitchen -----------------------------------------------------------------------------
def _build_field_kitchen(model, lod=False):
    """Trailer field kitchen: an olive box body on two wheels with a serving hatch, stoves and pots."""
    hx, hy = 1.35, 2.2  # body 2.7 x 4.4 m, 1.9 m tall
    parts = []
    if lod:
        parts.append(box((-hx, -hy, 0.5), (hx, hy, 1.9), OLIVE))
        for side in (-1, 1):
            parts.append(box((side * (hx - 0.1) - 0.15, 0.6, 0.0), (side * (hx + 0.1), 1.4, 0.8), DARK))
        for bm in parts:
            model.add(bm)
        model.add_collider("body", box((-hx, -hy, 0.0), (hx, hy, 1.95), "white"))
        return
    rng = random.Random(8301)
    # chassis, drawbar and jockey wheel
    parts.append(pbox((-hx + 0.1, -hy, 0.42), (hx - 0.1, hy, 0.62), DARK, bevel=0.02))
    parts.append(_beam((0.0, hy, 0.5), (0.0, hy + 0.9, 0.44), 0.06, color=METAL, segments=6))
    parts.append(_beam((0.0, hy + 0.9, 0.44), (0.0, hy + 0.95, 0.3), 0.05, color=METAL, segments=6))
    parts.append(common.make_frustum(0.14, 0.14, 0.0, 0.12, segments=8, center=(0.0, hy + 0.85), color=DARK))
    # body shell with a chamfered roof
    parts.append(_panel_loft([[(x * hx, -hy, 0.6) for x in (1, 1, -1, -1)],
                              [(x * hx, hy, 0.6) for x in (1, 1, -1, -1)]], OLIVE))
    parts.append(pbox((-hx, -hy, 0.62), (hx, hy, 1.85), OLIVE, bevel=0.03))
    parts.append(pbox((-hx - 0.04, -hy - 0.04, 1.85), (hx + 0.04, hy + 0.04, 1.95), KHAKI, bevel=0.03))
    # serving hatch on the right flank: a lifted panel on two struts
    parts.append(pbox((hx - 0.02, -0.9, 1.0), (hx + 0.02, 0.9, 1.75), SOOT, bevel=0.0))
    parts.append(pbox((hx - 0.03, -0.86, 1.04), (hx + 0.03, 0.86, 1.12), METAL, bevel=0.0))  # counter
    lid = common.make_box((0.05, 1.8, 0.78), color=OLIVE)
    common.transform(lid, rot=(0.0, -0.75, 0.0), loc=(hx + 0.34, 0.0, 1.98))
    parts.append(lid)
    for y in (-0.7, 0.7):
        parts.append(_beam((hx + 0.02, y, 1.4), (hx + 0.4, y, 1.9), 0.02, color=METAL, segments=4))
    # stoves on the counter with pots and a kettle
    for y, r in ((-0.45, 0.19), (0.05, 0.15), (0.55, 0.21)):
        parts.append(common.make_frustum(r, r, 0.0, 0.26, segments=10, center=(hx + 0.24, y), color=METAL))
        common.transform(parts[-1], loc=(0.0, 0.0, 1.12))
        parts.append(common.make_frustum(r * 0.85, r * 0.85, 0.0, 0.03, segments=10, center=(hx + 0.24, y),
                                         color="rock_dark"))
        common.transform(parts[-1], loc=(0.0, 0.0, 1.38))
        parts.append(_beam((hx + 0.24, y - r, 1.38), (hx + 0.24, y + r, 1.38), 0.012, color=METAL, segments=4))
    # propane bottles under the body, a stack of trays and two crates
    for y in (-0.5, 0.1):
        parts.append(common.make_frustum(0.16, 0.16, 0.0, 0.6, segments=10, center=(hx + 0.3, y), color=RUST))
    parts.append(pbox((-hx + 0.2, -hy + 0.3, 1.96), (hx - 0.2, -hy + 1.0, 2.12), METAL, bevel=0.02))  # tray stack
    for k in range(3):
        parts.append(box((-hx + 0.25, -hy + 0.32 + k * 0.24, 2.12), (hx - 0.25, -hy + 0.5 + k * 0.24, 2.14),
                         "chrome"))
    # wheels: one tyre, one bare rim
    for side, tyre in ((-1, True), (1, False)):
        x = side * (hx + 0.1)
        if tyre:
            parts.append(common.make_frustum(0.38, 0.38, -0.14, 0.14, segments=12, color="tire_black"))
            common.transform(parts[-1], rot=(0.0, math.pi / 2.0, 0.0), loc=(x, 0.0, 0.38))
        rim = ground_rim(0.0, 0.0, 0.38, 0.14)
        common.transform(rim, rot=(0.0, math.pi / 2.0, 0.0), loc=(x, 0.0, 0.38))
        parts.append(rim)
        for y in (-1.0, 1.0):  # folding legs
            parts.append(_beam((side * (hx - 0.2), y, 0.42), (side * (hx + 0.1), y, 0.0), 0.03, color=METAL,
                               segments=4))
    # soot up the back of the body, stencilled number, spilled flour and a ladle
    for _ in range(6):
        x = rng.uniform(-hx + 0.2, hx - 0.2)
        parts.append(box((x, -hy - 0.045, 0.7), (x + 0.3, -hy - 0.02, rng.uniform(1.2, 1.8)), SOOT))
    parts.append(stencil_on_side("K-3", 0.16, 0.24, -hx - 0.008, 1.0, 1.4, "white", outward=-1))
    for fx, fy, fw in ((-0.3, -hy - 0.75, 0.45), (0.3, -hy - 0.5, 0.3)):
        parts.append(box((fx, fy, 0.0), (fx + fw, fy + 0.22, 0.012), WHITE))
    for bm in parts:
        model.add(bm)
    model.add_collider("body", box((-hx, -hy, 0.62), (hx, hy, 1.95), "white"))
    model.add_collider("wheel", box((-hx - 0.25, -0.4, 0.0), (-hx + 0.1, 0.4, 0.8), "white"))


# --- loudspeaker_pole --------------------------------------------------------------------------
def _build_loudspeaker_pole(model):
    """6 m pole in a concrete footing with four horn speakers at the top, a feed cable and a busted one."""
    height = 6.0  # new: DECISIONS 2026-11-26 (loudspeaker_pole 6 m, 4 horns)
    parts = []
    parts.append(common.make_frustum(0.28, 0.24, 0.0, 0.34, segments=8, color="concrete_grey"))
    parts.append(common.make_frustum(0.1, 0.085, 0.34, height, segments=8, color=METAL))
    parts.append(common.make_frustum(0.11, 0.11, 0.34, 0.45, segments=8, color="rust_dark"))  # base collar
    # cross arms carrying the horns at 5.0 m and 5.7 m
    for z, arm in ((5.0, 0.62), (5.7, 0.5)):
        parts.append(_beam((-arm, 0.0, z), (arm, 0.0, z), 0.028, color=METAL, segments=5))
        for side in (-1, 1):
            hx = side * arm
            horn = common.make_frustum(0.05, 0.2, 0.0, 0.34, segments=10, color=KHAKI)
            common.transform(horn, rot=(0.0, side * (math.pi / 2.0 - 0.42), 0.0), loc=(hx, 0.0, z))
            parts.append(horn)
            parts.append(_beam((hx, 0.0, z), (hx + side * 0.06, 0.0, z + 0.14), 0.03, color=METAL, segments=4))
            parts.append(_beam((hx, 0.0, z), (hx, 0.0, z + 0.1), 0.022, color=DARK, segments=4))
    # one horn has come off and hangs by its cable
    parts.append(_beam((0.5, 0.0, 5.7), (0.72, 0.06, 4.6), 0.016, color=DARK, segments=4))
    horn = common.make_frustum(0.05, 0.18, 0.0, 0.3, segments=8, color=KHAKI)
    common.transform(horn, rot=(0.6, 1.1, 0.0), loc=(0.74, 0.08, 4.42))
    parts.append(horn)
    # feed cable strapped down the pole into the ground, with a loose loop
    for k in range(10):
        z = 0.5 + k * 0.5
        parts.append(pbox((0.08, -0.04, z - 0.03), (0.12, 0.04, z + 0.03), DARK, bevel=0.0))
    cable_pts = [(0.1, 0.0, 0.4), (0.4, 0.25, 0.15), (0.9, 0.4, 0.05), (1.4, 0.2, 0.03), (1.9, -0.1, 0.02)]
    for a, b in zip(cable_pts, cable_pts[1:]):
        parts.append(_beam(a, b, 0.018, color=DARK, segments=4))
    # a rotting notice board and a bent guy wire with a ground anchor
    parts.append(pbox((-0.34, -0.02, 3.3), (0.34, 0.02, 3.85), "white", bevel=0.01))
    for k in range(3):
        parts.append(box((-0.26, 0.02, 3.4 + k * 0.16), (0.2, 0.025, 3.45 + k * 0.16), "soot"))
    parts.append(_beam((0.0, 0.0, 5.3), (1.1, 0.0, 0.1), 0.012, color=METAL, segments=4))
    parts.append(_beam((1.1, 0.0, 0.0), (1.1, 0.0, 0.24), 0.025, color="bark_light", segments=4))
    # weeds at the footing
    for k in range(5):
        a = k * 1.3
        parts.append(_beam((math.cos(a) * 0.22, math.sin(a) * 0.22, 0.2),
                           (math.cos(a) * 0.34, math.sin(a) * 0.34, 0.55), 0.02, color="grass", segments=4))
    for bm in parts:
        model.add(bm)
    model.add_collider("pole", box((-0.3, -0.3, 0.0), (0.3, 0.3, height), "white"))


# --- checkpoint_booth --------------------------------------------------------------------------
def _build_checkpoint_booth(model, lod=False):
    """Small 1.8 x 2.2 m checkpoint booth, a sliding window, and a lifted boom gate arm."""
    hx, hy, h = 0.9, 1.1, 2.55  # new: DECISIONS 2026-11-26 (booth 1.8 x 2.2 m, 2.55 m tall)
    parts = []
    if lod:
        parts.append(box((-hx, -hy, 0.0), (hx, hy, h), KHAKI))
        parts.append(box((-hx - 0.1, -hy - 0.1, h), (hx + 0.1, hy + 0.1, h + 0.12), "concrete_grey"))
        parts.append(_beam((hx + 0.1, 0.0, 1.05), (hx + 3.5, 0.0, 2.3), 0.05, color=WHITE, segments=4))
        for bm in parts:
            model.add(bm)
        model.add_collider("booth", box((-hx, -hy, 0.0), (hx, hy, h + 0.12), "white"))
        return
    # base plinth and walls
    parts.append(pbox((-hx - 0.08, -hy - 0.08, 0.0), (hx + 0.08, hy + 0.08, 0.14), "concrete_grey", bevel=0.02))
    parts.append(pbox((-hx, -hy, 0.14), (hx, hy, h), KHAKI, bevel=0.03))
    parts.append(pbox((-hx - 0.1, -hy - 0.1, h), (hx + 0.1, hy + 0.1, h + 0.1), "concrete_grey", bevel=0.02))
    parts.append(box((-hx, -hy, 0.14), (hx, hy, 0.5), OLIVE))
    # window on the front (+Y): a dark opening, a slid-open glass in its track and a sill
    parts.append(pbox((-0.62, hy - 0.05, 1.15), (0.62, hy + 0.01, 1.95), "glass_dark", bevel=0.0))
    parts.append(pbox((-0.7, hy - 0.02, 1.95), (0.7, hy + 0.06, 2.02), METAL, bevel=0.0))
    parts.append(pbox((-0.7, hy - 0.02, 1.06), (0.7, hy + 0.12, 1.15), METAL, bevel=0.01))  # sill
    parts.append(pbox((0.62, hy + 0.02, 1.15), (1.3, hy + 0.06, 1.95), "glass_tint", bevel=0.0))
    parts.append(pbox((1.28, hy + 0.01, 1.15), (1.34, hy + 0.07, 1.95), METAL, bevel=0.0))
    # door on the left flank, ajar, and a step
    parts.append(pbox((-hx - 0.04, -0.5, 0.14), (-hx + 0.01, 0.5, 1.95), "brownstone", bevel=0.0))
    door = pbox((-0.03, 0.0, 0.14), (0.03, 1.0, 1.95), OLIVE, bevel=0.02)  # hinge edge at y = 0
    common.transform(door, rot=(0.0, 0.0, -0.7), loc=(-hx - 0.05, 0.5, 0.0))
    parts.append(door)
    parts.append(pbox((-hx - 0.6, -0.9, 0.0), (-hx - 0.05, 0.0, 0.14), "concrete_stain", bevel=0.0))
    # lifted boom gate: a pivot post, the arm at 40 degrees and its counterweight
    px = 1.55
    parts.append(common.make_frustum(0.14, 0.12, 0.0, 1.0, segments=8, center=(px, 0.0), color=WHITE))
    parts.append(common.make_frustum(0.2, 0.2, 0.0, 0.06, segments=8, center=(px, 1.1), color="red_paint"))
    arm_len = 3.6
    ang = math.radians(38.0)
    tip = (px + math.cos(ang) * arm_len, 0.0, 1.1 + math.sin(ang) * arm_len)
    parts.append(_beam((px, 0.0, 1.1), tip, 0.055, color=WHITE, segments=6))
    for k in range(1, 7):  # red hazard bands along the arm
        t0, t1 = k / 7.6, (k + 0.5) / 7.6
        a = (px + (tip[0] - px) * t0, 0.0, 1.1 + (tip[2] - 1.1) * t0)
        b = (px + (tip[0] - px) * t1, 0.0, 1.1 + (tip[2] - 1.1) * t1)
        parts.append(_beam(a, b, 0.062, color="red_paint", segments=6))
    parts.append(_beam((px, 0.0, 1.1), (px - 0.55, 0.0, 0.85), 0.07, color=METAL, segments=5))  # counterweight arm
    parts.append(pbox((px - 0.72, -0.16, 0.7), (px - 0.4, 0.16, 0.98), "rock_dark", bevel=0.02))
    # a barrier plinth set across the lane and a striped sign
    parts.append(pbox((2.0, -0.5, 0.0), (2.9, 0.5, 0.95), "concrete_stain", bevel=0.02))
    parts.append(pbox((2.03, -0.46, 0.3), (2.87, 0.46, 0.62), WHITE, bevel=0.0))
    for k in range(3):
        parts.append(box((2.4 + k * 0.16, -0.47, 0.3), (2.48 + k * 0.16, 0.47, 0.62), "red_paint"))
    # paperwork, a field phone and grime on the booth
    parts.append(pbox((-0.3, hy - 0.3, 1.15), (0.3, hy - 0.05, 1.19), WHITE, bevel=0.0))
    parts.append(pbox((0.25, hy - 0.28, 1.19), (0.5, hy - 0.06, 1.3), DARK, bevel=0.02))
    rng = random.Random(8401)
    for _ in range(6):
        x = rng.uniform(-hx + 0.1, hx - 0.1)
        parts.append(box((x, hy + 0.005, rng.uniform(0.5, 2.3)), (x + 0.2, hy + 0.02, rng.uniform(0.6, 2.4)),
                         rng.choice(("concrete_stain", "grime_green", SOOT))))
    parts.append(stencil_on_side("CP-2", 0.14, 0.2, -hx - 0.008, 1.5, 0.0, "white", outward=-1))
    for bm in parts:
        model.add(bm)
    model.add_collider("booth", box((-hx - 0.1, -hy - 0.1, 0.0), (hx + 0.1, hy + 0.1, h + 0.1), "white"))
    model.add_collider("post", box((px - 0.18, -0.18, 0.0), (px + 0.18, 0.18, 1.2), "white"))
    model.add_collider("block", box((2.0, -0.5, 0.0), (2.9, 0.5, 0.95), "white"))


# --- army_gear_scatter -------------------------------------------------------------------------
def _helmet(model_parts, x, y, z, yaw, color=OLIVE):
    dome = common.make_icosphere(subdiv=1, jitter=0.05, seed=int(x * 100) + 3, radius=(0.16, 0.19, 0.13),
                                loc=(x, y, z + 0.11), color=color)
    model_parts.append(dome)
    model_parts.append(common.make_frustum(0.2, 0.2, 0.0, 0.03, segments=10, center=(x, y), color=color))
    common.transform(model_parts[-1], loc=(0.0, 0.0, z + 0.02))
    band = common.make_frustum(0.19, 0.19, 0.0, 0.045, segments=10, center=(x, y), color=OLIVE)
    common.transform(band, loc=(0.0, 0.0, z + 0.14))
    model_parts.append(band)


def _build_army_gear_scatter(model):
    """4 x 3 m of dropped kit: helmets, packs, ammo cans, a stretcher, a rifle and papers."""
    rng = random.Random(8501)
    parts = []
    # scattered ground stains so the kit sits on the road (not a floating island or a plate)
    for sx, sy, sw, sh in ((-1.3, 0.4, 1.3, 0.8), (0.7, -0.5, 1.5, 0.9), (0.2, 1.0, 1.1, 0.6),
                           (-0.4, -1.1, 1.2, 0.7)):
        parts.append(pbox((sx, sy, 0.0), (sx + sw, sy + sh, 0.01), "soil_dark", bevel=0.0))
    for k, (x, y) in enumerate(((-1.4, 0.8), (-0.5, -0.9), (0.9, 0.6), (1.5, -0.8), (0.1, 1.1))):
        _helmet(parts, x, y, 0.012, rng.uniform(0, 3.1))
    # packs: two upright, one slumped with the flap open
    for x, y, yaw in ((-1.0, -0.2, 0.4), (0.6, -0.9, -0.9), (1.3, 0.9, 1.2)):
        parts.append(pbox_at((x - 0.26, y - 0.18, 0.012), (x + 0.26, y + 0.18, 0.42), KHAKI,
                             rot=(0.0, 0.0, yaw), bevel=0.05))
        parts.append(beam_at((x - 0.24, y - 0.18, 0.3), (x + 0.24, y + 0.18, 0.3), (0.0, 0.0, yaw), 0.03,
                             color="leather", segments=4))
        parts.append(pbox_at((x - 0.24, y - 0.16, 0.4), (x + 0.24, y + 0.16, 0.46), OLIVE,
                             rot=(0.6 if yaw > 0 else -0.6, 0.0, yaw), bevel=0.02))
    # ammo cans, some tipped over
    for x, y, yaw, tip in ((-0.2, 0.4, 0.2, 0.0), (0.3, 0.2, -0.4, 0.0), (0.9, -0.2, 1.1, 1.4),
                           (-1.7, -1.1, 0.6, 0.0)):
        parts.append(pbox_at((x - 0.19, y - 0.1, 0.012), (x + 0.19, y + 0.1, 0.272), OLIVE,
                             rot=(tip, 0.0, yaw), bevel=0.02))
        parts.append(pbox_at((x - 0.14, y - 0.06, 0.272), (x + 0.14, y + 0.06, 0.312), METAL,
                             rot=(tip, 0.0, yaw), bevel=0.0))
    # a stretcher lying on the ground: two poles, canvas and a strap
    sx, sy = 0.35, -1.25
    for side in (-1, 1):
        parts.append(_beam((sx - 0.9, sy + side * 0.24, 0.08), (sx + 0.9, sy + side * 0.24, 0.08), 0.026,
                           color=METAL, segments=5))
    parts.append(pbox_at((sx - 0.82, sy - 0.26, 0.05), (sx + 0.82, sy + 0.26, 0.09), KHAKI,
                         rot=(0.0, 0.0, 0.07), bevel=0.0))
    parts.append(box((sx - 0.1, sy - 0.28, 0.09), (sx + 0.1, sy + 0.28, 0.1), "blood_dark"))
    # a dropped rifle and loose rounds, plus paper blowing about
    parts.append(_beam((1.5, -1.2, 0.05), (1.5, -0.5, 0.05), 0.035, color=DARK, segments=6))
    parts.append(pbox((1.46, -0.75, 0.03), (1.54, -0.45, 0.11), OLIVE, bevel=0.01))
    parts.append(pbox((1.47, -0.55, 0.11), (1.53, -0.4, 0.2), DARK, bevel=0.0))
    for _ in range(10):
        x = rng.uniform(-1.9, 1.9)
        y = rng.uniform(-1.4, 1.4)
        parts.append(box((x, y, 0.012), (x + 0.05, y + 0.02, 0.03), "brass"))
    for _ in range(5):
        x = rng.uniform(-1.8, 1.8)
        y = rng.uniform(-1.3, 1.3)
        parts.append(pbox_at((x, y, 0.012), (x + 0.22, y + 0.3, 0.021), WHITE,
                             rot=(0.0, 0.0, rng.uniform(0, 3.1)), bevel=0.0))
    model.add(common.combine(parts))
    model.add_collider("gear", box((-2.0, -1.5, 0.0), (2.0, 1.5, 0.45), "white"))


# --- flood_light -------------------------------------------------------------------------------
def _build_flood_light(model):
    """Light tower on a two-wheel trailer: telescopic mast, four broken lamps, a smashed one hanging."""
    parts = []
    # trailer frame and wheels
    parts.append(pbox((-0.7, -1.3, 0.34), (0.7, 1.3, 0.46), OLIVE, bevel=0.02))
    for side in (-1, 1):
        parts.append(_beam((side * 0.72, -1.0, 0.3), (side * 0.72, 1.0, 0.3), 0.04, color=METAL, segments=5))
        wheel = common.make_frustum(0.28, 0.28, -0.1, 0.1, segments=12, color="tire_black")
        common.transform(wheel, rot=(0.0, math.pi / 2.0, 0.0), loc=(side * 0.78, 0.25, 0.28))
        parts.append(wheel)
        rim = ground_rim(0.0, 0.0, 0.28, 0.1)
        common.transform(rim, rot=(0.0, math.pi / 2.0, 0.0), loc=(side * 0.78, 0.25, 0.28))
        parts.append(rim)
    parts.append(_beam((0.0, -1.3, 0.4), (0.0, -1.9, 0.3), 0.05, color=METAL, segments=5))  # drawbar
    parts.append(common.make_frustum(0.16, 0.16, 0.0, 0.12, segments=8, center=(0.0, -1.85), color=DARK))
    parts.append(_beam((0.0, 1.1, 0.4), (0.0, 1.35, 0.02), 0.035, color=METAL, segments=4))  # prop leg
    # mast: three telescoped sections, crank box, and the lamp frame on top
    parts.append(common.make_frustum(0.15, 0.12, 0.46, 1.0, segments=8, color=METAL))
    parts.append(pbox((-0.24, -0.24, 0.46), (0.24, 0.24, 0.72), OLIVE, bevel=0.02))
    parts.append(common.make_frustum(0.1, 0.085, 1.0, 2.2, segments=8, color=METAL))
    parts.append(common.make_frustum(0.075, 0.06, 2.2, 3.5, segments=8, color=METAL))
    parts.append(common.make_frustum(0.09, 0.09, 1.0, 1.12, segments=8, color="rust_dark"))
    parts.append(common.make_frustum(0.075, 0.075, 2.2, 2.3, segments=8, color="rust_dark"))
    # lamp bar with four lamp housings; three are dark and cracked, one hangs off by its cable
    parts.append(_beam((-0.75, 0.0, 3.6), (0.75, 0.0, 3.6), 0.045, color=METAL, segments=5))
    for x in (-0.62, -0.21, 0.21):
        parts.append(pbox((x - 0.17, -0.12, 3.5), (x + 0.17, 0.12, 3.86), "metal", bevel=0.02))
        parts.append(pbox((x - 0.14, 0.12, 3.53), (x + 0.14, 0.15, 3.83), "glass_dark", bevel=0.0))
        parts.append(box((x - 0.05, 0.15, 3.68), (x + 0.05, 0.16, 3.7), "white"))  # cracked white line
    parts.append(pbox((0.45, -0.12, 3.5), (0.79, 0.12, 3.86), METAL, bevel=0.02))  # empty housing
    parts.append(_beam((0.62, 0.0, 3.5), (0.72, 0.1, 2.95), 0.016, color=DARK, segments=4))  # hanging cable
    lamp = pbox((-0.16, -0.11, -0.18), (0.16, 0.11, 0.18), METAL, bevel=0.02)
    common.transform(lamp, rot=(0.7, 0.4, 0.0), loc=(0.74, 0.14, 2.78))
    parts.append(lamp)
    parts.append(_beam((-0.6, -0.02, 3.6), (-0.6, -0.1, 3.9), 0.03, color=METAL, segments=4))  # bent bracket
    # cable wound on a reel at the base and a jerrycan
    parts.append(common.make_frustum(0.22, 0.22, -0.1, 0.1, segments=10, color=DARK))
    common.transform(parts[-1], rot=(0.0, math.pi / 2.0, 0.0), loc=(0.55, 0.85, 0.55))
    for a in (0.0, 1.2, 2.4):
        parts.append(_beam((0.45, 0.85, 0.55), (0.45, 0.85 + math.cos(a) * 0.2, 0.55 + math.sin(a) * 0.2),
                           0.016, color=DARK, segments=4))
    parts.append(pbox((-0.55, 0.6, 0.46), (-0.22, 0.9, 0.78), OLIVE, bevel=0.03))
    # soot stains and a stencilled number
    for _ in range(5):
        parts.append(box((-0.6, -0.02, 0.5 + _ * 0.12), (0.62, 0.02, 0.56 + _ * 0.12), SOOT))
    parts.append(stencil_on_side("L-4", 0.12, 0.18, 0.71, 0.55, 0.2, "white", outward=1))
    model.add(common.combine(parts))
    model.add_collider("trailer", box((-0.85, -1.9, 0.0), (0.85, 1.35, 0.85), "white"))
    model.add_collider("mast", box((-0.2, -0.2, 0.0), (0.2, 0.2, 3.9), "white"))


# --- group -------------------------------------------------------------------------------------
def build_all():
    """Builds the army line camp props. Returns [(name, ok)]."""
    return [
        ("evac_tent", common.run_model("evac_tent", "props", PROP_BUDGET, _build_evac_tent,
                                       view=(0.9, -1.0, 0.45))),
        ("evac_tent_lod1", common.run_model("evac_tent_lod1", "props", PROP_LOD1_BUDGET,
                                             lambda m: _build_evac_tent(m, lod=True))),
        ("field_kitchen", common.run_model("field_kitchen", "props", PROP_BUDGET, _build_field_kitchen,
                                           view=(1.0, -1.0, 0.55))),
        ("field_kitchen_lod1", common.run_model("field_kitchen_lod1", "props", PROP_LOD1_BUDGET,
                                                lambda m: _build_field_kitchen(m, lod=True))),
        ("loudspeaker_pole", common.run_model("loudspeaker_pole", "props", PROP_BUDGET,
                                              _build_loudspeaker_pole, view=(0.8, -1.0, 0.3))),
        ("checkpoint_booth", common.run_model("checkpoint_booth", "props", PROP_BUDGET,
                                             _build_checkpoint_booth, view=(1.0, -1.0, 0.5))),
        ("checkpoint_booth_lod1", common.run_model("checkpoint_booth_lod1", "props", PROP_LOD1_BUDGET,
                                                   lambda m: _build_checkpoint_booth(m, lod=True))),
        ("army_gear_scatter", common.run_model("army_gear_scatter", "props", PROP_BUDGET,
                                               _build_army_gear_scatter, top_preview=False,
                                               view=(1.0, -1.0, 1.0))),
        ("flood_light", common.run_model("flood_light", "props", PROP_BUDGET, _build_flood_light,
                                         view=(0.9, -1.0, 0.4))),
    ]
