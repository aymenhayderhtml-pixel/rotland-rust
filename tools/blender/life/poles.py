"""R41-04 power poles (standing, leaning, fallen) and power wires (span, down) + _lod1s."""
import math
import os
import sys

import bmesh
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import common  # noqa: E402

POLE_BUDGET = 1500
POLE_LOD1_BUDGET = 150
WIRE_BUDGET = 600
WIRE_LOD1_BUDGET = 60

# Insulator wire attachment height & Y offsets (cross-arm runs along Y so spans run along +X)
WIRE_Z = 8.50
WIRE_YS = (-0.90, 0.0, 0.90)
SPAN_LEN = 20.0
SPAN_SAG = 1.20


def _tube_between(p0, p1, r0, r1, segs, color):
    """Closed tapered tube from p0 to p1."""
    s = Vector(p0)
    e = Vector(p1)
    d = e - s
    if d.length < 1e-4:
        d = Vector((0.0, 0.0, 0.01))
    bm = common.make_frustum(r0, r1, 0.0, d.length, segments=segs, color=color)
    rot = Vector((0.0, 0.0, 1.0)).rotation_difference(d.normalized()).to_matrix()
    bmesh.ops.rotate(bm, verts=list(bm.verts), cent=(0.0, 0.0, 0.0), matrix=rot)
    bmesh.ops.translate(bm, verts=list(bm.verts), vec=s)
    return bm


def _wire_curve(points, radius, segs, color):
    """Continuous segmented tube along `points`."""
    bm = common.new_bm()
    rings = []
    for idx, pt in enumerate(points):
        p = Vector(pt)
        if idx == 0:
            tangent = (Vector(points[1]) - p).normalized()
        elif idx == len(points) - 1:
            tangent = (p - Vector(points[-2])).normalized()
        else:
            tangent = (Vector(points[idx + 1]) - Vector(points[idx - 1])).normalized()
        up = Vector((0.0, 0.0, 1.0)) if abs(tangent.z) < 0.95 else Vector((1.0, 0.0, 0.0))
        side = tangent.cross(up).normalized()
        up_ortho = side.cross(tangent).normalized()
        ring = []
        for s in range(segs):
            ang = 2.0 * math.pi * s / segs
            offset = (side * math.cos(ang) + up_ortho * math.sin(ang)) * radius
            ring.append(bm.verts.new(p + offset))
        rings.append(ring)

    for k in range(len(rings) - 1):
        r0, r1 = rings[k], rings[k + 1]
        for s in range(segs):
            ns = (s + 1) % segs
            bm.faces.new((r0[s], r0[ns], r1[ns], r1[s]))
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def _pole_parts(lean_deg=0.0, include_transformer=True, lod1=False):
    """Builds standing or leaning wooden power pole parts (origin at ground center)."""
    parts = []
    pole_segs = 5 if lod1 else 8
    parts.append(common.make_frustum(0.17, 0.12, 0.0, 9.0, segments=pole_segs, color="bark_dark"))

    # Cross arm along Y at z=8.28 so insulators sit at z=8.50
    arm = common.make_box((0.14, 2.20, 0.14), loc=(0.0, 0.0, 8.28), color="bark_light")
    parts.append(arm)

    if not lod1:
        # Steel diagonal braces
        for sy in (-1.0, 1.0):
            parts.append(_tube_between((0.0, 0.0, 7.75), (0.0, sy * 0.65, 8.22), 0.022, 0.022, 4, "metal"))
        # 3 insulators (pin + glass bell) with groove at z=8.50
        for iy in WIRE_YS:
            parts.append(common.make_frustum(0.02, 0.02, 8.34, 8.42, segments=6, center=(0.0, iy), color="metal"))
            parts.append(common.make_frustum(0.055, 0.038, 8.41, 8.53, segments=8, center=(0.0, iy), color="glass_tint"))
        # Climbing spikes alternating on +Y and -Y
        for idx, z in enumerate(z * 0.55 + 2.4 for z in range(10)):
            sy = 0.18 if idx % 2 == 0 else -0.18
            parts.append(common.make_box((0.03, 0.12, 0.03), loc=(0.0, sy, z), color="metal"))
        # Pole-mounted transformer can on -X side
        if include_transformer:
            parts.append(common.make_frustum(0.22, 0.22, 6.40, 7.25, segments=10, center=(-0.36, 0.0), color="rock_dark"))
            parts.append(common.make_frustum(0.24, 0.24, 7.22, 7.28, segments=10, center=(-0.36, 0.0), color="metal"))
            for bz in (6.58, 7.08):
                parts.append(common.make_box((0.20, 0.10, 0.06), loc=(-0.18, 0.0, bz), color="rust_dark"))
            for by in (-0.09, 0.09):
                parts.append(common.make_frustum(0.025, 0.02, 7.28, 7.46, segments=6, center=(-0.36, by), color="glass_tint"))
    else:
        for iy in WIRE_YS:
            parts.append(common.make_box((0.08, 0.08, 0.14), loc=(0.0, iy, 8.44), color="glass_tint"))

    if lean_deg != 0.0:
        # Negative X rotation tilts +Z toward +Y
        rot_x = -math.radians(lean_deg)
        for bm in parts:
            common.transform(bm, rot=(rot_x, 0.0, 0.0), loc=(0.0, 0.0, -0.04))
    return parts


def _build_power_pole(model):
    for bm in _pole_parts(lean_deg=0.0, include_transformer=True, lod1=False):
        model.add(bm)


def _build_power_pole_lod1(model):
    for bm in _pole_parts(lean_deg=0.0, include_transformer=False, lod1=True):
        model.add(bm)


def _build_power_pole_leaning(model):
    for bm in _pole_parts(lean_deg=12.0, include_transformer=True, lod1=False):
        model.add(bm)


def _build_power_pole_leaning_lod1(model):
    for bm in _pole_parts(lean_deg=12.0, include_transformer=False, lod1=True):
        model.add(bm)


def _fallen_pole_parts(lod1=False):
    parts = []
    segs = 5 if lod1 else 8
    # Short snapped stump at origin
    parts.append(common.make_frustum(0.17, 0.155, 0.0, 0.72, segments=segs, color="bark_dark"))
    if not lod1:
        for ang, r_off, h in ((0.4, 0.08, 0.24), (2.1, 0.07, 0.30), (4.2, 0.09, 0.20), (5.4, 0.06, 0.26)):
            parts.append(common.make_frustum(
                0.045, 0.005, 0.68, 0.68 + h, segments=4,
                center=(math.cos(ang) * r_off, math.sin(ang) * r_off), color="bark_light",
            ))

    # Fallen upper section lying along +Y from y=0.95 to y=9.2
    fallen = []
    fallen.append(common.make_frustum(0.155, 0.12, 0.0, 8.25, segments=segs, color="bark_dark"))
    fallen.append(common.make_box((0.14, 2.20, 0.14), loc=(0.0, 0.0, 7.55), color="bark_light"))
    if not lod1:
        for ang, r_off, h in ((0.8, 0.07, 0.22), (3.0, 0.08, 0.26), (4.9, 0.06, 0.18)):
            fallen.append(common.make_frustum(
                0.04, 0.005, 0.02, -h, segments=4,
                center=(math.cos(ang) * r_off, math.sin(ang) * r_off), color="bark_light",
            ))
        for iy in WIRE_YS:
            fallen.append(common.make_frustum(0.045, 0.035, 7.64, 7.78, segments=6, center=(0.0, iy), color="glass_tint"))
        fallen.append(common.make_frustum(0.21, 0.21, 5.7, 6.5, segments=8, center=(-0.34, 0.0), color="rock_dark"))

    # Tilt so +Z lies along +Y, rolled slightly so cross arm rests naturally
    for bm in fallen:
        common.transform(bm, rot=(-math.radians(88.5), 0.0, math.radians(18.0)), loc=(0.0, 0.95, 0.17))
    parts.extend(fallen)
    return parts


def _build_power_pole_fallen(model):
    for bm in _fallen_pole_parts(lod1=False):
        model.add(bm)


def _build_power_pole_fallen_lod1(model):
    for bm in _fallen_pole_parts(lod1=True):
        model.add(bm)


def _build_wire_span(segs_along, ring_segs):
    def build(model):
        for iy in WIRE_YS:
            pts = []
            for i in range(segs_along + 1):
                t = i / segs_along
                x = t * SPAN_LEN
                z = WIRE_Z - SPAN_SAG * 4.0 * t * (1.0 - t)
                pts.append((x, iy, z))
            model.add(_wire_curve(pts, 0.02, ring_segs, "gun_black"))
    return build


def _build_wire_down(segs_along, ring_segs):
    def build(model):
        offsets = (-0.85, 0.05, 0.95)
        phases = (0.0, 1.4, 2.7)
        amps = (0.55, 0.70, 0.48)
        for iy, ph, amp in zip(offsets, phases, amps):
            pts = []
            for i in range(segs_along + 1):
                t = i / segs_along
                x = t * SPAN_LEN
                # Loose S-curves on the ground (2 cm above ground)
                y = iy + amp * math.sin(t * math.pi * 3.0 + ph) * math.sin(t * math.pi)
                pts.append((x, y, 0.02))
            model.add(_wire_curve(pts, 0.02, ring_segs, "gun_black"))
    return build


def build_all():
    """Exports all pole and power wire models + LOD1s."""
    specs = [
        ("power_pole", POLE_BUDGET, _build_power_pole, (0.0, 0.0, 0.0)),
        ("power_pole_lod1", POLE_LOD1_BUDGET, _build_power_pole_lod1, (0.0, 0.0, 0.0)),
        ("power_pole_leaning", POLE_BUDGET, _build_power_pole_leaning, (0.0, 0.0, 0.0)),
        ("power_pole_leaning_lod1", POLE_LOD1_BUDGET, _build_power_pole_leaning_lod1, (0.0, 0.0, 0.0)),
        ("power_pole_fallen", POLE_BUDGET, _build_power_pole_fallen, (0.0, 0.0, 0.0)),
        ("power_pole_fallen_lod1", POLE_LOD1_BUDGET, _build_power_pole_fallen_lod1, (0.0, 0.0, 0.0)),
        ("power_wire_span", WIRE_BUDGET, _build_wire_span(14, 4), (0.0, 0.0, 0.0)),
        ("power_wire_span_lod1", WIRE_LOD1_BUDGET, _build_wire_span(3, 3), (0.0, 0.0, 0.0)),
        ("power_wire_down", WIRE_BUDGET, _build_wire_down(14, 4), (0.0, 0.0, 0.0)),
        ("power_wire_down_lod1", WIRE_LOD1_BUDGET, _build_wire_down(3, 3), (0.0, 0.0, 0.0)),
    ]
    return [(name, common.run_model(name, "props", budget, fn, grip=grip)) for name, budget, fn, grip in specs]
