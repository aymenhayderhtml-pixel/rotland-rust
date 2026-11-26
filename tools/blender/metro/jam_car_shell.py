"""Car shell for the R38 jam cars: bevelled hollow lower body with door holes, full doors, greenhouse with
frames and pillars, fenders, lights, bumpers, grille and mirrors.

Window states: "glass", "crack" (darker cracked pane), "hole" (empty frame, dark inside). Burnt cars: holes only.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import car_parts  # noqa: E402
import common  # noqa: E402
from mathutils import Vector  # noqa: E402
from jam_util import box, pbox  # noqa: E402

WALL_T = 0.05  # side panel thickness (m)
GLASS_OUT = 0.012  # glass sits this far outside the greenhouse face (m)
FRAME_W = 0.035  # window frame width (m)
# Outward direction per greenhouse face: 0 rear, 1 right, 2 front, 3 left
FACE_OUT = {0: (0.0, -1.0, 0.3), 1: (1.0, 0.0, 0.0), 2: (0.0, 1.0, 0.4), 3: (-1.0, 0.0, 0.0)}


def greenhouse_corners(spec):
    """Bottom (at belt height) and top (at roof height) quads, both CCW from +Z: back-left, back-right, front-right, front-left."""
    cab = spec["cab"]
    bl = [(-cab["xb"], cab["bb"], spec["belt"]), (cab["xb"], cab["bb"], spec["belt"]),
          (cab["xb"], cab["bf"], spec["belt"]), (-cab["xb"], cab["bf"], spec["belt"])]
    tp = [(-cab["xt"], cab["tb"], spec["roof"]), (cab["xt"], cab["tb"], spec["roof"]),
          (cab["xt"], cab["tf"], spec["roof"]), (-cab["xt"], cab["tf"], spec["roof"])]
    return bl, tp


def face_point(bl, tp, face, u, v):
    """Point on a greenhouse face: u runs along the face (first corner to second), v from belt (0) to roof (1)."""
    a, c = Vector(bl[face]), Vector(bl[(face + 1) % 4])
    at, ct = Vector(tp[face]), Vector(tp[(face + 1) % 4])
    return a.lerp(c, u).lerp(at.lerp(ct, u), v)


def _outward(face, offset):
    return Vector(FACE_OUT[face]).normalized() * offset


def _quad_on_face(bl, tp, face, rect, offset, color):
    u0, u1, v0, v1 = rect
    out = _outward(face, offset)
    pts = [face_point(bl, tp, face, u, v) + out for u, v in ((u0, v0), (u1, v0), (u1, v1), (u0, v1))]
    bm = common.new_bm()
    common.add_quad(bm, [tuple(p) for p in pts], outward=FACE_OUT[face], color=color)
    return bm


def _strip(p1, p2, width, outward, color):
    """A thin strip from p1 to p2 lying on the face, `width` wide and pushed out along `outward`."""
    edge = (p2 - p1).normalized()
    side = edge.cross(outward).normalized() * (width / 2)
    pts = [p1 - side, p1 + side, p2 + side, p2 - side]
    bm = common.new_bm()
    common.add_quad(bm, [tuple(p) for p in pts], outward=tuple(outward), color=color)
    return bm


def _frame(bl, tp, face, rect, color):
    """Four bars around a window rectangle, sitting just outside the glass."""
    u0, u1, v0, v1 = rect
    out = _outward(face, GLASS_OUT + 0.006)
    c = [face_point(bl, tp, face, u, v) + out for u, v in ((u0, v0), (u1, v0), (u1, v1), (u0, v1))]
    return [_strip(c[i], c[(i + 1) % 4], FRAME_W, out, color) for i in range(4)]


def _crack(bl, tp, face, rect):
    """Two dark crack lines across a cracked pane."""
    u0, u1, v0, v1 = rect
    du, dv = u1 - u0, v1 - v0
    out = _outward(face, GLASS_OUT + 0.02)
    lines = [((u0 + 0.15 * du, v1 - 0.02 * dv), (u0 + 0.62 * du, v0 + 0.05 * dv)),
             ((u0 + 0.70 * du, v1 - 0.02 * dv), (u1 - 0.04 * du, v0 + 0.40 * dv))]
    out_bms = []
    for (a, b) in lines:
        p1 = face_point(bl, tp, face, a[0], a[1]) + out
        p2 = face_point(bl, tp, face, b[0], b[1]) + out
        out_bms.append(_strip(p1, p2, 0.018, out, "gun_black"))
    return out_bms


def window_parts(bl, tp, face, rect, state, burnt):
    """Pane, crack lines or hole, plus the frame, on one greenhouse face. Burnt cars: holes only."""
    if burnt or state == "hole":
        pane = [_quad_on_face(bl, tp, face, rect, GLASS_OUT, "gun_black")]
    else:
        pane = [_quad_on_face(bl, tp, face, rect, GLASS_OUT, "glass_tint")]
        if state == "crack":
            pane += _crack(bl, tp, face, rect)
    return pane + _frame(bl, tp, face, rect, "gun_black")


def _pillars(bl, tp, spec, body):
    """Corner pillars and roof gutters of the greenhouse (bevel-free beams, body colour)."""
    out = []
    for i in range(4):
        out.append(car_parts._beam(bl[i], tp[i], 0.035, color=body, segments=6))
    for i in range(4):
        out.append(car_parts._beam(tp[i], tp[(i + 1) % 4], 0.03, color=body, segments=6))
    return out


def greenhouse_parts(spec, body, burnt):
    """Closed cabin loft, its windows and frames, pillars, wipers and roof gutters. Roof-flagged by the caller."""
    bl, tp = greenhouse_corners(spec)
    parts = [car_parts._closed_loft(bl, tp, [body] * 4, body, "gun_black")]
    for face, rect, state in spec["windows"]:
        parts.extend(window_parts(bl, tp, face, rect, state, burnt))
    parts.extend(_pillars(bl, tp, spec, body))
    if spec.get("wipers", True):
        for x in (-0.25, 0.25):
            a = face_point(bl, tp, 2, 0.5 + x, 0.1)
            parts.append(car_parts._beam(tuple(a + Vector((0.0, 0.03, 0.02))),
                                         tuple(a + Vector((0.0, 0.03, 0.02)) + Vector((0.0, 0.05, 0.02))),
                                         0.008, color="gun_black", segments=4))
    return parts


def _carve(y0, y1, gaps):
    """The parts of [y0, y1] left after removing the gap ranges."""
    pieces = [(y0, y1)]
    for g0, g1 in sorted(gaps):
        kept = []
        for a, b in pieces:
            if g1 <= a or g0 >= b:
                kept.append((a, b))
                continue
            if g0 > a:
                kept.append((a, g0))
            if g1 < b:
                kept.append((g1, b))
        pieces = kept
    return pieces


def _side_segments(spec):
    """Side wall ranges along Y with their top height (the bed of a pickup is lower)."""
    half = spec["half_l"]
    bed = spec.get("bed")
    if not bed:
        return [(-half, half, spec["belt"])]
    return [(bed["front"], half, spec["belt"]), (-half, bed["front"], bed["top"])]


def _door(spec, side, hinge, back, open_deg, body, burnt):
    """A full door hinged on its front edge (`hinge`): outer skin, inner trim, window with frame, handle, seam.
    Built around the hinge line, then swung open by `open_deg` (0 = shut)."""
    belt, sill = spec["belt"], spec["sill"]
    parts = []
    parts.append(pbox((-WALL_T / 2, back - hinge, sill), (WALL_T / 2, 0.0, belt), body, bevel=0.012))
    inner_lo, inner_hi = sorted((-side * 0.026, -side * 0.042))
    parts.append(pbox((inner_lo, back - hinge + 0.05, sill + 0.08), (inner_hi, -0.05, belt - 0.02),
                      "gun_black" if burnt else "leather", bevel=0.008))  # inner trim panel
    outer_x = side * 0.026
    gy0, gy1 = back - hinge + 0.12, -0.12
    gz0, gz1 = belt - 0.36, belt - 0.08
    if burnt:
        parts.append(box((outer_x - 0.004, gy0, gz0), (outer_x + 0.004, gy1, gz1), "gun_black"))
    else:
        parts.append(box((outer_x - 0.002, gy0, gz0), (outer_x + 0.002, gy1, gz1), "glass_tint"))
    for ya, yb, za, zb in ((gy0 - 0.03, gy1 + 0.03, gz1, gz1 + 0.03), (gy0 - 0.03, gy1 + 0.03, gz0 - 0.03, gz0),
                           (gy0 - 0.03, gy0, gz0, gz1), (gy1, gy1 + 0.03, gz0, gz1)):
        parts.append(box((outer_x - 0.004, ya, za), (outer_x + 0.006, yb, zb), "metal"))  # window frame
    parts.append(box((outer_x, back - hinge + 0.16, belt - 0.22), (outer_x + side * 0.012, back - hinge + 0.26,
                                                                     belt - 0.18), "chrome"))  # handle
    for part in parts:
        common.transform(part, rot=(0.0, 0.0, side * math.radians(open_deg)),
                         loc=(side * (spec["W"] - WALL_T / 2), hinge, 0.0))
    return parts


def _boot_lid(spec, body):
    """Rear closure at belt height; hinged on its front edge, so `lid_open` lifts the back."""
    half, belt, cb = spec["half_l"], spec["belt"], spec["cab"]["bb"]
    lid = pbox((-spec["W"] + 0.02, -(cb + half), -0.05), (spec["W"] - 0.02, 0.0, 0.0), body, bevel=0.02)
    common.transform(lid, rot=(-math.radians(spec.get("lid_open", 0.0)), 0.0, 0.0), loc=(0.0, cb, belt))
    return lid


def _front_end(spec, body, burnt):
    """Bonnet, bumper with plate, grille, headlights, bumper rear, taillights, mirrors, fenders."""
    w, half, sill, belt = spec["W"], spec["half_l"], spec["sill"], spec["belt"]
    cab = spec["cab"]
    parts = [pbox((-w + 0.02, cab["bf"] - 0.02, belt - 0.07), (w - 0.02, half - 0.05, belt), body, bevel=0.03)]
    parts.append(box((-0.3, half - 0.1, belt), (0.3, half - 0.05, belt + 0.01), "chrome"))  # bonnet ridge
    parts.append(pbox((-w - 0.03, half - 0.03, sill - 0.14), (w + 0.03, half + 0.05, sill + 0.02), "rock_dark",
                      bevel=0.03))  # front bumper
    parts.append(box((-0.13, half + 0.05, sill - 0.08), (0.13, half + 0.06, sill - 0.01), "white"))  # plate
    parts.append(pbox((-0.3, half - 0.02, sill + 0.1), (0.3, half + 0.03, sill + 0.32), "gun_black", bevel=0.01))
    for k in range(4):
        x = -0.24 + k * 0.16
        parts.append(box((x - 0.02, half + 0.03, sill + 0.12), (x + 0.02, half + 0.04, sill + 0.3), "chrome"))
    for sx in (-1, 1):
        parts.append(pbox((sx * (w - 0.22) - 0.12, half - 0.04, sill + 0.2), (sx * (w - 0.22) + 0.12, half + 0.02,
                                                                             sill + 0.36), "white", bevel=0.01))
        parts.append(box((sx * (w - 0.12) - 0.05, -half - 0.02, sill + 0.2), (sx * (w - 0.12) + 0.05, -half,
                                                                              sill + 0.3), "red_paint"))
    parts.append(pbox((-w - 0.03, -half - 0.05, sill - 0.14), (w + 0.03, -half + 0.03, sill + 0.02), "rock_dark",
                      bevel=0.03))  # rear bumper
    parts.append(box((-0.13, -half - 0.06, sill - 0.08), (0.13, -half - 0.05, sill - 0.01), "white"))
    return parts


def _mirror(spec, side, body):
    w, belt = spec["W"], spec["belt"]
    hinge = max(h for _, h, _, _ in spec["doors"] if h > 0)
    x0, x1 = sorted((side * (w + 0.02), side * (w + 0.13)))
    return pbox((x0, hinge - 0.02, belt - 0.12), (x1, hinge + 0.12, belt + 0.02), body, bevel=0.01)


def lower_parts(spec, body, burnt):
    """Hollow lower body: pan, closures, bonnet, bed (pickup), side panels with door holes, doors, fenders, mirrors."""
    w, half, sill, belt = spec["W"], spec["half_l"], spec["sill"], spec["belt"]
    bed = spec.get("bed")
    parts = [box((-w + 0.03, -half, sill - 0.07), (w - 0.03, half, sill), "gun_black")]
    parts += _front_end(spec, body, burnt)
    rear_top = bed["top"] if bed else belt
    parts.append(pbox((-w, -half, sill), (w, -half + 0.07, rear_top), body, bevel=0.02))  # rear closure
    if bed:
        parts.append(pbox((-w, bed["front"] - 0.05, sill), (w, bed["front"], bed["top"]), body, bevel=0.02))
        parts.append(pbox((-w + 0.05, -half + 0.07, sill), (w - 0.05, bed["front"], sill + 0.06), "metal",
                          bevel=0.01))  # bed floor
    else:
        parts.append(_boot_lid(spec, body))
    for side in (-1, 1):
        x0, x1 = sorted((side * w, side * (w - WALL_T)))
        gaps = [(min(h, b), max(h, b)) for s, h, b, _ in spec["doors"] if s == side]
        for y0, y1, top in _side_segments(spec):
            for p0, p1 in _carve(y0, y1, gaps):
                parts.append(pbox((x0, p0, sill), (x1, p1, top), body, bevel=0.015))
        for axle in (spec["wheel_y"], -spec["wheel_y"]):
            parts.append(pbox((side * (w - 0.01), axle - 0.42, sill - 0.04), (side * (w + 0.07), axle + 0.42,
                              sill + 0.3), body, bevel=0.025))  # fender flare over the wheel
            parts.append(box((side * (w - 0.04), axle - 0.36, sill - 0.02), (side * (w + 0.01), axle + 0.36,
                              sill + 0.24), "gun_black"))  # dark arch liner
        if spec.get("doors"):
            parts.append(_mirror(spec, side, body))
    for side, hinge, back, open_deg in spec["doors"]:
        parts += _door(spec, side, hinge, back, open_deg, body, burnt)
    return parts
