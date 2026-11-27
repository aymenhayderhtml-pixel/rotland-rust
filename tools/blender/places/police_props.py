"""R42-04 police props: the two police cars, desk, holding cell, locker row, armoury cage,
riot shield pile and the striped barricade, plus the small helpers `police.py` shares.

Style follows the R38 jam cars (`metro/jam_car_shell.py`, `metro/jam_util.py`): bevelled panels,
tyres and rims from `jam_util.wheel_parts`, stroke lettering from `traces/graffiti_font.py`.
No brands: the only words are the generic stencil words ("POLICE").

Run: `flock /tmp/rotland-blender.lock blender -b --python-expr
      "import sys; sys.path.insert(0,'tools/blender'); from places import police_props; police_props.build_all()"`
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
for _p in (HERE, os.path.join(HERE, ".."), os.path.join(HERE, "..", "metro"),
           os.path.join(HERE, "..", "vehicles"), os.path.join(HERE, "..", "traces")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402
import graffiti_font  # noqa: E402
import jam_car_shell  # noqa: E402
import jam_util  # noqa: E402
from car_parts import _beam  # noqa: E402
from graffiti_font import quad_bar  # noqa: E402
from jam_util import add_bbox_collider, box, ground_rim, pbox  # noqa: E402

# new: DECISIONS 2026-11-26 (R42 budgets: props 4000, vehicles 10000 / _lod1 1000)
PROP_BUDGET = 4000
CAR_BUDGET = 10000
CAR_LOD_BUDGET = 1000

FRONT = (1.0, 1.0, 0.5)  # preview camera: front right, above (+Y is the front)

# graffiti_font stops at W; POLICE needs the P (and the car door stencil uses nothing else new).
FONT = dict(graffiti_font.FONT)
FONT.update({
    "P": [(0, 0, 0, 1), (0, 1, 0.82, 1), (0.82, 1, 1, 0.78), (1, 0.78, 0.82, 0.55), (0.82, 0.55, 0, 0.55)],
    "Y": [(0, 1, 0.5, 0.45), (1, 1, 0.5, 0.45), (0.5, 0.45, 0.5, 0)],
})


def text_bm(text, letter_w, letter_h, color, thick_scale=1.0):
    """Flat stroke letters on the XZ plane at y = 0, centred on x = 0, resting on z = 0, reading from +Y.

    Returns one combined bmesh; rotate it onto a side face (rot z = -90 deg faces +X, +90 deg faces -X).
    """
    gap = letter_w * 0.35
    space = letter_w * 0.6
    total = sum(space if ch == " " else letter_w for ch in text) + gap * (len(text) - 1)
    thick = letter_h * 0.14 * thick_scale
    x = -total / 2.0
    bms = []
    for ch in text:
        if ch == " ":
            x += space + gap
            continue
        for u0, v0, u1, v1 in FONT[ch]:
            bms.append(quad_bar(-(x + u0 * letter_w), v0 * letter_h, -(x + u1 * letter_w),
                                v1 * letter_h, 0.0, thick, color))
        x += letter_w + gap
    return common.combine(bms)


# ---------------------------------------------------------------- police cars


# Parked-patrol sedan: same shell helpers as the R38 jam cars (hollow lower body with door holes,
# greenhouse with windows and pillars), in a black-and-white patrol livery.
SPEC = dict(
    half_l=2.18, W=0.90, wheel_y=1.38, sill=0.26, belt=0.95, roof=1.45, lid_open=0.0,
    flat=[1, 3], shelf=True,
    cab=dict(bf=0.90, bb=-0.98, tf=0.40, tb=-0.62, xb=0.88, xt=0.78),
    doors=[(-1, 0.90, 0.02, 64.0), (1, 0.90, 0.02, 14.0), (-1, -0.02, -0.98, 0.0), (1, -0.02, -0.98, 72.0)],
    windows=[(1, (0.10, 0.44, 0.16, 0.84), "glass"), (1, (0.52, 0.88, 0.16, 0.84), "hole"),
             (3, (0.12, 0.46, 0.16, 0.84), "hole"), (3, (0.56, 0.90, 0.16, 0.84), "crack"),
             (2, (0.12, 0.88, 0.10, 0.90), "crack"), (0, (0.14, 0.86, 0.12, 0.84), "glass")],
)

W, HL = SPEC["W"], SPEC["half_l"]
SILL, BELT, ROOF = SPEC["sill"], SPEC["belt"], SPEC["roof"]
CAB = SPEC["cab"]
WHEEL_R = 0.34  # new: DECISIONS 2026-11-26 (same wheel as the jam cars)
TYRE_HW = 0.12


def _wheel_x2(x, y, radius, half_w, flat=1.0):
    """Tyre plus a steel rim, 12 segments on the tyre so a wrecked car stays inside its budget."""
    parts = []
    tyre = jam_util.tyre_bm(radius, half_w, seg=12)
    rim = common.make_frustum(radius * 0.60, radius * 0.60, -half_w * 0.55, half_w * 0.55, segments=10,
                              color="metal")
    hub = common.make_frustum(radius * 0.26, radius * 0.26, -half_w * 0.6, half_w * 0.68, segments=8,
                              color="chrome")
    for bm in (tyre, rim, hub):
        common.transform(bm, rot=(0.0, math.pi / 2.0, 0.0))
        common.transform(bm, scale=(1.0, 1.0, flat), loc=(x, y, radius * flat))
        parts.append(bm)
    return parts


def _wheels(burnt):
    """Four wheels; a burnt car keeps only the rims, lying by the body."""
    out = []
    for i, (sx, sy) in enumerate(((-(W - 0.05), SPEC["wheel_y"]), (W - 0.05, SPEC["wheel_y"]),
                                  (-(W - 0.05), -SPEC["wheel_y"]), (W - 0.05, -SPEC["wheel_y"]))):
        if burnt:
            out.append(ground_rim(sx * 1.55, sy * 0.75, WHEEL_R, TYRE_HW, yaw=0.45 * i))
            continue
        out += _wheel_x2(sx, sy, WHEEL_R, TYRE_HW, flat=0.74 if i in SPEC["flat"] else 1.0)
    return out


def _letters_on_door(side, hinge_y, back_y, open_deg, color="car_white", burnt=False):
    """POLICE stencil on a door's outer skin, in the door's own frame, then swung shut like the door."""
    length = back_y - hinge_y
    bm = text_bm("POLICE", 0.082, 0.17, "rust_dark" if burnt else color)
    common.transform(bm, rot=(0.0, 0.0, -side * math.pi / 2.0),
                     loc=(side * 0.030, length / 2.0, SILL + 0.30))
    common.transform(bm, rot=(0.0, 0.0, side * math.radians(open_deg)),
                     loc=(side * (SPEC["W"] - 0.05 / 2.0), hinge_y, 0.0))
    return bm


def _light_bar(burnt):
    """Roof light bar: dark base, a red and a blue segment, two end caps."""
    parts = [pbox((-CAB["xt"] + 0.14, -0.24, ROOF), (CAB["xt"] - 0.14, 0.18, ROOF + 0.05), "gun_black",
                  bevel=0.01)]
    red, blue = ("soot", "rust_dark") if burnt else ("red_paint", "water_blue")
    parts.append(pbox((-0.40, -0.19, ROOF + 0.04), (-0.02, 0.13, ROOF + 0.15), red, bevel=0.02))
    parts.append(pbox((0.02, -0.19, ROOF + 0.04), (0.40, 0.13, ROOF + 0.15), blue, bevel=0.02))
    parts.append(box((-0.03, -0.20, ROOF + 0.04), (0.03, 0.14, ROOF + 0.16), "gun_black"))
    for sx in (-1, 1):
        parts.append(box((sx * 0.42 - 0.03, -0.20, ROOF + 0.03), (sx * 0.42 + 0.03, 0.14, ROOF + 0.14),
                         "gun_black"))
    return parts


def _push_bar(metal="metal"):
    """Bull bar in front of the grille: two uprights, two rails, two brackets."""
    hx = HL - 0.02
    parts = [_beam((-W + 0.22, hx + 0.12, SILL + 0.08), (-W + 0.22, hx + 0.12, BELT - 0.10), 0.035, color=metal),
             _beam((W - 0.22, hx + 0.12, SILL + 0.08), (W - 0.22, hx + 0.12, BELT - 0.10), 0.035, color=metal)]
    parts.append(_beam((-W + 0.18, hx + 0.13, SILL + 0.30), (W - 0.18, hx + 0.13, SILL + 0.30), 0.042, color=metal))
    parts.append(_beam((-W + 0.18, hx + 0.13, SILL + 0.08), (W - 0.18, hx + 0.13, SILL + 0.08), 0.030, color=metal))
    for sx in (-0.42, 0.42):
        parts.append(_beam((sx, hx + 0.13, SILL + 0.10), (sx, hx + 0.01, SILL + 0.10), 0.028, color=metal))
    return parts


def _car_extras(burnt, rng):
    """Livery patches, roof spotlight, antenna and (burnt) charring."""
    parts = []
    # spotlight by the driver's A-pillar
    parts.append(_beam((-W - 0.02, 0.82, ROOF - 0.20), (-W - 0.02, 0.82, ROOF - 0.04), 0.028, color="gun_black"))
    parts.append(pbox((-W - 0.11, 0.76, ROOF - 0.24), (-W - 0.02, 0.90, ROOF - 0.12), "chrome", bevel=0.02))
    # radio antenna on the boot lid
    parts.append(_beam((-W + 0.26, -1.44, BELT + 0.01), (-W + 0.22, -1.56, BELT + 0.82), 0.011, color="metal"))
    if burnt:
        for _ in range(9):
            y = rng.uniform(-HL + 0.3, HL - 0.4)
            z = rng.uniform(SILL + 0.02, BELT - 0.25)
            for side in (-1, 1):
                x0, x1 = sorted((side * (W + 0.001), side * (W + 0.010)))
                parts.append(box((x0, y, z), (x1, y + rng.uniform(0.18, 0.44), z + rng.uniform(0.1, 0.22)),
                                 rng.choice(("rust_dark", "soot", "rust"))))
        for _ in range(4):
            cx, cy = rng.uniform(-0.6, 0.6), rng.uniform(-1.4, 1.0)
            parts.append(box((cx - 0.2, cy - 0.2, ROOF - 0.005), (cx + 0.25, cy + 0.22, ROOF + 0.012),
                             rng.choice(("rust_dark", "soot"))))
    return parts


def _build_car(burnt):
    def build(model):
        rng = random.Random(77 if burnt else 71)
        body = "gun_black" if burnt else "car_white"
        shell = jam_car_shell.lower_parts(SPEC, body, burnt)
        parts = [(bm, False) for bm in shell]
        parts += [(bm, True) for bm in jam_car_shell.greenhouse_parts(SPEC, body, burnt)]
        parts += [(bm, False) for bm in _wheels(burnt)]
        parts += [(bm, True) for bm in _light_bar(burnt)]
        parts += [(bm, False) for bm in _push_bar("rust_dark" if burnt else "metal")]
        parts += [(bm, True) for bm in _car_extras(burnt, rng)]
        # Black door skins (a patrol livery): repaint the door outer panel, keep the glass and trim.
        for side, hinge_y, back_y, open_deg in SPEC["doors"]:
            door = jam_car_shell._door(SPEC, side, hinge_y, back_y, open_deg, body, burnt)
            common.paint(door[0], "soot" if burnt else "gun_black")
            parts += [(bm, False) for bm in door]
            if abs(open_deg) < 30.0:  # the shut doors carry the stencil so it reads from outside
                parts.append((_letters_on_door(side, hinge_y, back_y, open_deg, burnt=burnt), False))
        for bm, roof in parts:
            model.add(bm, roof=roof)
        solid = jam_car_shell.lower_parts(SPEC, body, burnt) + jam_car_shell.greenhouse_parts(SPEC, body, burnt)
        add_bbox_collider(model, solid, inset=0.06)

    return build


def _build_car_lod1(burnt):
    def build(model):
        body = "gun_black" if burnt else "car_white"
        w, hl = SPEC["W"], SPEC["half_l"]
        p = [pbox((-w, -hl, SILL), (w, hl, BELT), body, bevel=0.05),
             pbox((-SPEC["cab"]["xt"], SPEC["cab"]["tb"], BELT), (SPEC["cab"]["xt"], SPEC["cab"]["tf"], ROOF),
                  "soot" if burnt else "glass_dark", bevel=0.03),
             pbox((-w - 0.03, hl - 0.06, SILL - 0.12), (w + 0.03, hl + 0.04, SILL + 0.04), "gun_black", bevel=0.02),
             pbox((-w - 0.03, -hl - 0.04, SILL - 0.12), (w + 0.03, -hl + 0.06, SILL + 0.04), "gun_black", bevel=0.02)]
        for sx in (-(w - 0.05), w - 0.05):
            for sy in (SPEC["wheel_y"], -SPEC["wheel_y"]):
                p.append(common.make_frustum(WHEEL_R, WHEEL_R, -TYRE_HW, TYRE_HW, segments=8,
                                             color="tire_black"))
                common.transform(p[-1], rot=(0.0, math.pi / 2.0, 0.0), loc=(sx, sy, WHEEL_R))
        p.append(pbox((-0.40, -0.22, ROOF), (0.40, 0.16, ROOF + 0.11),
                      "rust_dark" if burnt else "red_paint", bevel=0.02))
        for bm in p:
            model.add(bm, roof=False)

    return build


# ---------------------------------------------------------------- props


def build_desk(model):
    """1.7 x 0.85 m office desk: top, two pedestals with a kneehole, dead monitor, keyboard, papers, phone."""
    parts = []
    top_z = 0.74
    # top with a proud front lip and a dark modesty panel at the back
    parts.append(pbox((-0.85, -0.42, top_z), (0.85, 0.42, top_z + 0.05), "bark_light", bevel=0.012))
    parts.append(pbox((-0.85, 0.40, top_z + 0.03), (0.85, 0.43, top_z + 0.06), "wood_crate", bevel=0.006))
    parts.append(pbox((-0.83, -0.40, 0.10), (0.83, -0.36, top_z - 0.02), "metal", bevel=0.008))
    # two drawer pedestals, leaving a kneehole between them
    for px0, px1 in ((-0.83, -0.40), (0.40, 0.83)):
        parts.append(pbox((px0, -0.38, 0.0), (px1, 0.36, top_z - 0.02), "metal", bevel=0.012))
        for z in (0.10, 0.30, 0.50):
            parts.append(pbox((px0 - 0.008, 0.30, z), (px1 + 0.008, 0.365, z + 0.17), "rock_light", bevel=0.008))
            parts.append(box(((px0 + px1) / 2 - 0.09, 0.365, z + 0.06), ((px0 + px1) / 2 + 0.09, 0.395,
                             z + 0.10), "chrome"))
        parts.append(box((px0, -0.36, 0.0), (px1, 0.36, 0.05), "soot"))
    # monitor on the right of the kneehole, keyboard, mouse
    parts.append(pbox((0.14, -0.06, top_z + 0.05), (0.42, 0.12, top_z + 0.09), "gun_black", bevel=0.006))
    parts.append(box((0.24, -0.03, top_z + 0.09), (0.29, 0.03, top_z + 0.25), "gun_black"))
    parts.append(pbox((0.06, -0.10, top_z + 0.25), (0.62, 0.04, top_z + 0.58), "gun_black", bevel=0.012))
    parts.append(box((0.09, -0.11, top_z + 0.28), (0.59, -0.09, top_z + 0.55), "soot"))
    parts.append(pbox((-0.06, 0.16, top_z + 0.05), (0.34, 0.32, top_z + 0.08), "gun_black", bevel=0.005))
    parts.append(pbox((-0.16, 0.20, top_z + 0.05), (-0.06, 0.30, top_z + 0.07), "gun_black", bevel=0.004))
    # phone and mug
    parts.append(pbox((-0.62, 0.12, top_z + 0.05), (-0.42, 0.30, top_z + 0.10), "white", bevel=0.01))
    parts.append(box((-0.60, 0.14, top_z + 0.10), (-0.48, 0.18, top_z + 0.16), "white"))
    mug = common.make_frustum(0.05, 0.05, top_z + 0.05, top_z + 0.16, segments=8, center=(-0.30, 0.34),
                              color="white")
    parts.append(mug)
    # papers on the top and a few fallen on the floor
    for cx, cy, rz, col in ((-0.30, -0.10, 0.2, "white"), (-0.22, 0.02, -0.3, "bone_white"),
                            (0.44, -0.22, 0.5, "white"), (-0.70, 0.02, 0.15, "white")):
        sheet = pbox((-0.11, -0.15, -0.003), (0.11, 0.15, 0.003), col, bevel=0.001)
        common.transform(sheet, rot=(0.0, 0.0, rz), loc=(cx, cy, top_z + 0.052))
        parts.append(sheet)
    for cx, cy, rz in ((-0.35, 0.60, 0.7), (0.20, 0.72, -0.4), (0.90, -0.10, 1.1)):
        sheet = pbox((-0.11, -0.15, -0.002), (0.11, 0.15, 0.002), "bone_white", bevel=0.001)
        common.transform(sheet, rot=(0.0, 0.0, rz), loc=(cx, cy, 0.004))
        parts.append(sheet)
    # an office chair tipped over behind the desk
    chair = [pbox((-0.22, -0.22, 0.40), (0.22, 0.22, 0.48), "gun_black", bevel=0.02),
             pbox((-0.22, -0.24, 0.48), (-0.14, 0.22, 0.92), "gun_black", bevel=0.02),
             box((0.0, 0.0, 0.05), (0.06, 0.06, 0.42), "metal"),
             box((-0.26, -0.26, 0.0), (0.26, 0.26, 0.05), "metal")]
    for bm in chair:
        common.transform(bm, rot=(math.radians(96.0), 0.0, 0.5), loc=(0.10, -0.90, 0.22))
        parts.append(bm)
    for bm in parts:
        model.add(bm)
    model.add_collider("desk", common.make_box((1.70, 0.86, top_z + 0.05), loc=(0.0, 0.0, (top_z + 0.05) / 2),
                                               color="white"))


def build_holding_cell(model):
    """3 x 3 x 2.8 m standalone cell: concrete back and sides, barred front, open barred door, bunk."""
    hw, h = 1.5, 2.8
    wall_t = 0.14
    parts = []
    # slab
    parts.append(pbox((-hw, -hw, 0.0), (hw, hw, 0.16), "concrete_grey", bevel=0.02))
    for label, lo, hi in (("back", (-hw, -hw, 0.16), (hw, -hw + wall_t, h)),
                          ("left", (-hw, -hw + wall_t, 0.16), (-hw + wall_t, hw, h)),
                          ("right", (hw - wall_t, -hw + wall_t, 0.16), (hw, hw, h))):
        parts.append(pbox(lo, hi, "concrete_grey", bevel=0.015))
        model.add_collider("cell_" + label, common.make_box((hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]),
                                                            loc=((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2,
                                                                 (lo[2] + hi[2]) / 2), color="white"))
    # barred front (door gap 0.30..1.45 in x) and a top frame
    bars = "gun_black"
    x = -hw + 0.12
    while x < hw - 0.10:
        if not (0.30 <= x <= 1.45):
            parts.append(box((x - 0.02, hw - 0.10, 0.16), (x + 0.02, hw - 0.04, h), bars))
        x += 0.17
    parts.append(box((-hw, hw - 0.12, h - 0.10), (hw, hw - 0.02, h), bars))
    parts.append(box((-hw, hw - 0.12, 0.16), (hw, hw - 0.02, 0.32), bars))
    parts.append(box((-hw, hw - 0.12, 1.55), (hw, hw - 0.02, 1.63), bars))
    for lo, hi in (((-hw, hw - 0.14, 0.16), (0.30, hw, h)), ((1.45, hw - 0.14, 0.16), (hw, hw, h))):
        model.add_collider("cell_bars", common.make_box((hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]),
                                                        loc=((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2,
                                                             (lo[2] + hi[2]) / 2), color="white"))
    # top grid so nobody climbs in
    for y in (-hw + 0.3, -0.6, 0.2, 1.0):
        parts.append(box((-hw, y - 0.02, h), (hw, y + 0.02, h + 0.05), bars))
    # barred door hanging open around its hinge at x = 0.30
    door = []
    for i in range(7):
        door.append(box((-0.02 + i * 0.19, -0.02, 0.16), (0.02 + i * 0.19, 0.02, h - 0.10), bars))
    door.append(box((-0.02, -0.04, h - 0.14), (1.18, 0.04, h - 0.06), bars))
    door.append(box((-0.02, -0.04, 0.20), (1.18, 0.04, 0.28), bars))
    door.append(box((-0.02, -0.04, 1.45), (1.18, 0.04, 1.53), bars))
    door.append(box((1.02, -0.05, 0.9), (1.14, 0.05, 1.1), "metal"))
    for bm in door:
        common.transform(bm, rot=(0.0, 0.0, math.radians(-96.0)), loc=(0.30, hw - 0.08, 0.0))
        parts.append(bm)
    # bunk, toilet, bench
    parts.append(pbox((-hw + 0.16, -hw + 0.18, 0.42), (-0.30, hw - 0.30, 0.52), "metal", bevel=0.01))
    parts.append(pbox((-hw + 0.20, -hw + 0.22, 0.52), (-0.34, hw - 0.34, 0.64), "rock_light", bevel=0.02))
    parts.append(pbox((-hw + 0.24, -hw + 0.30, 0.64), (-0.86, -hw + 0.72, 0.74), "bone_white", bevel=0.03))
    for lx in (-hw + 0.18, -0.34):
        for ly in (-hw + 0.20, hw - 0.34):
            parts.append(box((lx, ly, 0.16), (lx + 0.06, ly + 0.06, 0.52), "metal"))
    parts.append(pbox((0.62, -hw + 0.14, 0.16), (1.16, -hw + 0.56, 0.50), "metal", bevel=0.02))
    parts.append(box((0.62, -hw + 0.14, 0.50), (1.16, -hw + 0.30, 0.78), "metal"))
    parts.append(box((-hw + 0.20, hw - 0.52, 0.16), (-0.40, hw - 0.22, 0.58), "wood_crate"))
    # dark stains and scratch marks on the back wall
    parts.append(box((-1.0, -hw + wall_t - 0.01, 0.20), (-0.2, -hw + wall_t + 0.012, 0.7), "blood_dark"))
    for i in range(5):
        z = 0.9 + i * 0.14
        parts.append(box((-1.3 + i * 0.08, -hw + wall_t - 0.01, z), (-1.05 + i * 0.08, -hw + wall_t + 0.012, z + 0.02),
                         "rock_dark"))
    for bm in parts:
        model.add(bm)


def build_locker_row(model):
    """5 steel lockers, 2.25 x 0.5 x 1.85 m; two doors dented open, clothes still on the shelves."""
    parts = []
    w, d, h = 2.25, 0.5, 1.85
    cell = w / 5.0
    blue = "shirt_blue"
    rng = random.Random(5)
    # carcass: back panel, top, bottom, two ends and four dividers (hollow, so the open doors show inside)
    parts.append(box((-w / 2, -d / 2, 0.0), (w / 2, -d / 2 + 0.03, h), blue))
    parts.append(box((-w / 2, -d / 2, h - 0.04), (w / 2, d / 2, h), blue))
    parts.append(box((-w / 2, -d / 2, 0.0), (w / 2, d / 2, 0.10), "rock_dark"))
    for i in range(6):
        x0 = -w / 2 + i * cell
        x1 = x0 + (0.03 if i in (0, 5) else 0.022)
        parts.append(box((x0, -d / 2 + 0.02, 0.0), (min(x1, w / 2), d / 2, h), blue))
    opens = (0.0, -74.0, 0.0, 0.0, -92.0)
    for i in range(5):
        x0 = -w / 2 + i * cell
        open_deg = opens[i]
        # shelf and a hanging rod inside every locker
        parts.append(box((x0 + 0.04, -d / 2 + 0.04, 1.42), (x0 + cell - 0.04, d / 2 - 0.02, 1.46), "metal"))
        parts.append(_beam((x0 + 0.08, 0.02, 1.30), (x0 + cell - 0.08, 0.02, 1.30), 0.012, color="metal"))
        if open_deg != 0.0:
            parts.append(pbox((x0 + 0.07, -0.20, 1.46), (x0 + cell - 0.08, d / 2 - 0.05, 1.52), blue,
                              bevel=0.008))
            parts.append(pbox((x0 + 0.10, -0.16, 1.52), (x0 + 0.38, 0.14, 1.66),
                              rng.choice(("cloth_olive", "pants_denim", "shirt_red", "leather")), bevel=0.02))
            parts.append(pbox((x0 + 0.14, -0.05, 0.10), (x0 + 0.34, 0.18, 0.26), "pants_dark", bevel=0.02))
        else:
            parts.append(pbox((x0 + 0.08, -0.14, 1.44), (x0 + cell - 0.10, 0.12, 1.48), "metal", bevel=0.006))
        # the door leaf, hinged on its left edge, swung by open_deg
        dw = cell - 0.05
        door = [pbox((0.0, -0.014, 0.06), (dw, 0.014, h - 0.05), "metal", bevel=0.008)]
        for k in range(4):  # louvre vents
            z = h - 0.24 - k * 0.075
            door.append(box((0.06, -0.022, z), (dw - 0.10, -0.012, z + 0.035), "gun_black"))
        door.append(box((0.05, -0.022, 0.28), (dw - 0.08, -0.012, 0.62), "gun_black"))  # handle recess
        door.append(box((dw - 0.14, -0.040, 0.86), (dw - 0.07, -0.012, 1.06), "chrome"))  # handle
        door.append(box((0.06, -0.036, h - 0.13), (0.32, -0.014, h - 0.07), "white"))  # number plate
        if open_deg != 0.0:  # dented: a proud buckle and a bent corner
            door.append(box((0.12, -0.046, 0.50), (dw - 0.20, -0.026, 0.92), "shirt_blue"))
            door.append(box((dw - 0.10, -0.040, 0.10), (dw, -0.020, 0.26), "rock_light"))
        for bm in door:
            common.transform(bm, rot=(0.0, 0.0, math.radians(open_deg)),
                             loc=(x0 + 0.035, d / 2 - 0.005, 0.0))
            parts.append(bm)
    # top rail and feet
    parts.append(box((-w / 2, -d / 2 - 0.01, h - 0.03), (w / 2, d / 2 + 0.01, h + 0.03), "rock_dark"))
    for sx in (-w / 2 + 0.08, w / 2 - 0.08):
        for sy in (-d / 2 + 0.08, d / 2 - 0.08):
            parts.append(box((sx - 0.03, sy - 0.03, -0.01), (sx + 0.03, sy + 0.03, 0.03), "soot"))
    for bm in parts:
        model.add(bm)
    model.add_collider("lockers", common.make_box((w, d + 0.06, h), loc=(0.0, 0.0, h / 2), color="white"))


def build_armory_cage(model):
    """3.6 x 2.6 x 2.5 m mesh cage with a forced door and empty gun racks inside."""
    w, d, h = 1.8, 1.3, 2.5  # half sizes
    parts = []
    mesh = "metal"
    # frame posts
    for x in (-w, 0.0, w):
        for y in (-d, d):
            parts.append(_beam((x, y, 0.0), (x, y, h), 0.035, color="rock_dark"))
    for y in (-d, d):
        parts.append(_beam((-w, y, h), (w, y, h), 0.035, color="rock_dark"))
        parts.append(_beam((-w, y, 0.04), (w, y, 0.04), 0.035, color="rock_dark"))
    for x in (-w, w):
        parts.append(_beam((x, -d, h), (x, d, h), 0.035, color="rock_dark"))
    # mesh: uprights and rails on the front (door gap between x -0.30 and 0.95) and both sides
    def mesh_run(x0, x1, y, z0, z1, gaps=()):
        x = x0 + 0.10
        while x < x1 - 0.04:
            if not any(g0 <= x <= g1 for g0, g1 in gaps):
                parts.append(box((x - 0.012, y - 0.02, z0), (x + 0.012, y + 0.02, z1), mesh))
            x += 0.15
        for z in (z0 + (z1 - z0) * 0.35, z0 + (z1 - z0) * 0.72, z1 - 0.05):
            parts.append(box((x0, y - 0.022, z - 0.012), (x1, y + 0.022, z + 0.012), mesh))

    mesh_run(-w, w, d, 0.05, h - 0.05, gaps=((-0.34, 0.99),))
    for y in (1.15, 0.45, -0.25, -0.95):
        parts.append(box((-w - 0.02, y - 0.012, 0.05), (-w + 0.02, y + 0.012, h - 0.05), mesh))
        parts.append(box((w - 0.02, y - 0.012, 0.05), (w + 0.02, y + 0.012, h - 0.05), mesh))
    for x0, x1 in ((-w, -w), (w, w)):
        for z in (0.75, 1.5, h - 0.05):
            parts.append(box((x0 - 0.022, -d, z - 0.012), (x1 + 0.022, d, z + 0.012), mesh))
    # back mesh panel
    mesh_run(-w, w, -d, 0.05, h - 0.05)
    # forced door: bent leaf leaning out of the frame
    door = []
    lw = 1.3
    for i in range(9):
        door.append(box((0.04 + i * 0.145, -0.02, 0.06), (0.075 + i * 0.145, 0.02, h - 0.16), mesh))
    for z in (0.6, 1.5, h - 0.20):
        door.append(box((0.02, -0.025, z - 0.015), (lw, 0.025, z + 0.015), mesh))
    door.append(box((0.02, -0.03, 0.02), (lw, 0.03, 0.09), mesh))
    door.append(box((1.02, -0.05, 0.95), (1.16, 0.05, 1.15), "metal"))
    for bm in door:
        common.transform(bm, rot=(0.0, 0.0, math.radians(64.0)), loc=(-0.30, d, 0.0))
        parts.append(bm)
    # bent frame stub where the lock was forced
    parts.append(_beam((-0.34, d + 0.02, 1.0), (-0.44, d + 0.22, 1.06), 0.02, color="metal"))
    # empty gun racks inside: two upright racks with empty slots
    for y in (0.75, -0.55):
        for side in (-1, 1):
            parts.append(box((side * (w - 0.12) - 0.05, y - 0.05, 0.06), (side * (w - 0.12) + 0.05, y + 0.05, 1.7),
                             "bark_dark"))
        parts.append(box((-w + 0.14, y - 0.06, 1.62), (w - 0.14, y + 0.06, 1.70), "bark_dark"))
        parts.append(box((-w + 0.14, y - 0.06, 0.12), (w - 0.14, y + 0.06, 0.20), "bark_dark"))
        parts.append(box((-w + 0.14, y - 0.05, 0.95), (w - 0.14, y + 0.05, 1.0), "bark_dark"))
        for i in range(9):  # empty slots
            x = -w + 0.30 + i * 0.36
            parts.append(box((x - 0.02, y - 0.07, 1.0), (x + 0.02, y + 0.07, 1.62), "gun_black"))
    # a shelf of empty ammo boxes, one tipped
    parts.append(box((-w + 0.16, -0.90, 0.06), (w - 0.16, -0.20, 0.10), "metal"))
    for i in range(3):
        x = -w + 0.24 + i * 0.55
        parts.append(pbox((x, -0.85, 0.10), (x + 0.42, -0.42, 0.30), "cloth_olive", bevel=0.02))
    tipped = pbox((-0.5, -1.0, 0.0), (-0.1, -0.6, 0.2), "cloth_olive", bevel=0.02)
    common.transform(tipped, rot=(0.0, 0.5, 0.4), loc=(0.2, -0.2, 0.0))
    parts.append(tipped)
    for bm in parts:
        model.add(bm)
    model.add_collider("cage", common.make_box((2 * w + 0.1, d + 0.1, h), loc=(0.0, 0.6, h / 2), color="white"))


def build_riot_shield_pile(model):
    """A heap of police riot shields: a stack lying flat, three dumped against it, a helmet and a baton."""
    parts = []
    rng = random.Random(21)
    sw, sh = 0.52, 1.16  # one shield
    dark = ("glass_dark", "tarp_blue", "rock_light", "gun_black")

    def shield(col):
        """One shield lying flat in its own frame: face up, length along Y, top edge at +Y."""
        return [pbox((-sw / 2, -sh / 2, -0.025), (sw / 2, sh / 2, 0.025), col, bevel=0.025),
                pbox((-sw / 2 + 0.05, -sh / 2 + 0.05, 0.022), (sw / 2 - 0.05, -sh / 2 + 0.30, 0.040), "soot"),
                pbox((-0.09, -0.05, 0.022), (0.09, 0.10, 0.045), "bark_dark")]

    # four flat in a shallow stack, each turned a little
    for i in range(4):
        bm = common.combine(shield(dark[i % len(dark)]))
        common.transform(bm, rot=(0.0, 0.0, rng.uniform(0.02, 0.30)),
                         loc=(rng.uniform(-0.10, 0.10), rng.uniform(-0.10, 0.10), 0.028 + i * 0.052))
        parts.append(bm)
    # three leaning against the stack: bottom edge on the ground, tipped back by `lean` degrees
    half_sh = sh / 2.0
    for i, (dx, dy, lean, spin) in enumerate(((-0.60, 0.10, 22.0, -0.30), (0.10, 0.72, 30.0, 0.24),
                                             (0.62, 0.02, 18.0, 0.42))):
        bm = common.combine(shield(dark[(i + 1) % len(dark)]))
        a = math.radians(lean)
        common.transform(bm, rot=(math.pi / 2.0 - a, 0.0, spin),
                         loc=(dx, dy + half_sh * math.sin(a), half_sh * math.cos(a) + 0.05))
        parts.append(bm)
    # helmet and baton beside the pile
    parts.append(common.make_icosphere(subdiv=1, radius=(0.14, 0.16, 0.12), loc=(-1.05, -0.55, 0.12),
                                       color="gun_black"))
    parts.append(common.make_frustum(0.20, 0.17, 0.0, 0.11, segments=10, center=(-1.05, -0.55),
                                     color="gun_black"))
    baton = common.make_frustum(0.022, 0.022, -0.30, 0.30, segments=6, color="gun_black")
    common.transform(baton, rot=(0.0, math.pi / 2.0, 0.5), loc=(0.85, -0.55, 0.03))
    parts.append(baton)
    for bm in parts:
        model.add(bm)
    model.add_collider("pile", common.make_box((2.0, 1.7, 0.55), loc=(0.0, 0.15, 0.275), color="white"))


def build_barricade(model):
    """2.5 m striped sawhorse barrier: two A-frame legs, two striped planks, a hazard lamp."""
    parts = []
    half = 1.25
    for sx in (-half + 0.18, half - 0.18):
        for lean in (-1, 1):
            leg = box((-0.045, -0.045, 0.0), (0.045, 0.045, 1.10), "bark_light")
            common.transform(leg, rot=(math.radians(18.0 * lean), 0.0, 0.0), loc=(sx, 0.0, 0.0))
            parts.append(leg)
        parts.append(box((sx - 0.06, -0.30, 0.94), (sx + 0.06, 0.30, 1.02), "bark_dark"))
    for z in (1.02, 0.62):
        parts.append(box((-half, -0.055, z), (half, 0.055, z + 0.20), "white"))
        stripes = 7
        for i in range(stripes):
            if i % 2:
                continue
            x0 = -half + i * (2 * half / stripes)
            parts.append(box((x0, -0.065, z + 0.005), (x0 + 2 * half / stripes, 0.065, z + 0.195),
                             "red_paint"))
    # amber hazard lamp on the right post
    parts.append(box((half - 0.30, -0.07, 1.02), (half - 0.16, 0.07, 1.10), "gun_black"))
    parts.append(common.make_frustum(0.055, 0.045, 1.10, 1.22, segments=8, center=(half - 0.23, 0.0),
                                     color="brass"))
    # a plastic foot on each A-frame
    for sx in (-half + 0.18, half - 0.18):
        parts.append(box((sx - 0.09, -0.34, 0.0), (sx + 0.09, 0.34, 0.05), "soot"))
    for bm in parts:
        model.add(bm)
    model.add_collider("barrier", common.make_box((2 * half, 0.6, 1.15), loc=(0.0, 0.0, 0.575), color="white"))


# ---------------------------------------------------------------- group


def build_all():
    """Builds every police prop. Returns [(name, ok)]."""
    results = [
        ("police_car_wreck", common.run_model("police_car_wreck", "vehicles", CAR_BUDGET,
                                             _build_car(False), top_preview=True, view=FRONT)),
        ("police_car_wreck_lod1", common.run_model("police_car_wreck_lod1", "vehicles", CAR_LOD_BUDGET,
                                                  _build_car_lod1(False), view=FRONT)),
        ("police_car_burnt", common.run_model("police_car_burnt", "vehicles", CAR_BUDGET,
                                             _build_car(True), top_preview=True, view=FRONT)),
        ("police_car_burnt_lod1", common.run_model("police_car_burnt_lod1", "vehicles", CAR_LOD_BUDGET,
                                                  _build_car_lod1(True), view=FRONT)),
        ("police_desk", common.run_model("police_desk", "props", PROP_BUDGET, build_desk, view=FRONT)),
        ("holding_cell", common.run_model("holding_cell", "props", PROP_BUDGET, build_holding_cell,
                                          top_preview=True, view=FRONT)),
        ("locker_row", common.run_model("locker_row", "props", PROP_BUDGET, build_locker_row, view=FRONT)),
        ("armory_cage", common.run_model("armory_cage", "props", PROP_BUDGET, build_armory_cage,
                                         top_preview=True, view=FRONT)),
        ("riot_shield_pile", common.run_model("riot_shield_pile", "props", PROP_BUDGET, build_riot_shield_pile,
                                              view=FRONT)),
        ("police_barricade", common.run_model("police_barricade", "props", PROP_BUDGET, build_barricade,
                                              view=FRONT)),
    ]
    return results


if __name__ == "__main__":
    failed = [name for name, ok in build_all() if not ok]
    if failed:
        print("POLICE PROPS FAILED:", ", ".join(failed))
        sys.exit(1)
    print("POLICE PROPS OK")
