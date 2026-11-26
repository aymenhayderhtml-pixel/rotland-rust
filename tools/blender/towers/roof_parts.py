"""Shared roof kit for New Halden tower crowns: water tanks, AC units, vents, antennas, roof decks.

Every function takes a `kit.Kit` and adds visual-only parts (crowns have no colliders; R39-06 adds a solid roof box).
"""
import math

import bmesh
from mathutils import Matrix, Vector

import common


def _angled_box(k, a, b, width, thick, color):
    """Visual box stretched between 3D points `a` and `b`."""
    va, vb = Vector(a), Vector(b)
    d = vb - va
    if d.length < 1e-4:
        return
    z_axis = d.normalized()
    up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    bm = common.make_box((width, thick, d.length), color=color)
    mat = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    k.model.add(bm)


def water_tank(k, x, y, z0, r=1.2, h=2.4):
    """Wooden rooftop water tank on 4 steel legs with conical roof and a broken hoop."""
    leg_h = 1.2
    leg_off = r * 0.72
    plat_z = z0 + leg_h

    # 4 steel legs + diagonal cross braces
    for sx in (-1.0, 1.0):
        for sy in (-1.0, 1.0):
            lx, ly = x + sx * leg_off, y + sy * leg_off
            k.box((lx - 0.07, ly - 0.07, z0), (lx + 0.07, ly + 0.07, plat_z), "metal")
    for sy in (-1.0, 1.0):
        ly = y + sy * leg_off
        _angled_box(k, (x - leg_off, ly, z0 + 0.15), (x + leg_off, ly, plat_z - 0.1), 0.05, 0.05, "rust")
        _angled_box(k, (x + leg_off, ly, z0 + 0.15), (x - leg_off, ly, plat_z - 0.1), 0.05, 0.05, "rust")

    # Support beams + wooden deck
    k.box((x - r - 0.1, y - r - 0.1, plat_z), (x + r + 0.1, y + r + 0.1, plat_z + 0.12), "bark_dark")
    tz0 = plat_z + 0.12
    tz1 = tz0 + h

    # Wooden cylindrical tank + conical roof
    k.model.add(common.make_frustum(r, r, tz0, tz1, segments=12, center=(x, y), color="bark_light"))
    k.model.add(common.make_frustum(r + 0.15, 0.0, tz1, tz1 + 0.75, segments=12, center=(x, y), color="bark_dark"))

    # Two intact metal hoops
    for hz in (tz0 + h * 0.22, tz0 + h * 0.80):
        k.model.add(common.make_frustum(r + 0.03, r + 0.03, hz, hz + 0.06, segments=12, center=(x, y), color="metal"))

    # Broken middle hoop: arc segments with a gap + dangling end
    mid_z = tz0 + h * 0.52
    segs = 12
    for i in range(segs):
        if i in (2, 3):  # missing section
            continue
        a0 = 2.0 * math.pi * i / segs
        a1 = 2.0 * math.pi * (i + 1) / segs
        p0 = (x + math.cos(a0) * (r + 0.03), y + math.sin(a0) * (r + 0.03), mid_z)
        dz = -0.18 if i == 4 else 0.0
        p1 = (x + math.cos(a1) * (r + 0.03), y + math.sin(a1) * (r + 0.03), mid_z + dz)
        _angled_box(k, p0, p1, 0.04, 0.05, "rust_dark")


def ac_unit(k, x, y, z0):
    """Rooftop HVAC unit with louver grille, top fan ring, and two rusty corner posts."""
    hw, hd, hh = 0.70, 0.50, 0.85
    k.box((x - hw, y - hd, z0), (x + hw, y + hd, z0 + hh), "concrete_grey")
    # Recessed dark grille on +Y and -Y faces
    for sy in (-1.0, 1.0):
        gy0 = y + sy * hd - (0.02 if sy > 0 else 0.03)
        gy1 = y + sy * hd + (0.03 if sy > 0 else 0.02)
        k.box((x - hw + 0.14, gy0, z0 + 0.14), (x + hw - 0.14, gy1, z0 + hh - 0.14), "rock_dark")
        for s in range(3):
            sz = z0 + 0.24 + s * 0.18
            k.box((x - hw + 0.16, gy0 - 0.01, sz), (x + hw - 0.16, gy1 + 0.01, sz + 0.03), "metal")

    # Top fan ring + hub + blades
    k.model.add(common.make_frustum(0.32, 0.32, z0 + hh, z0 + hh + 0.08, segments=10, center=(x, y), color="rock_dark"))
    k.model.add(common.make_frustum(0.08, 0.08, z0 + hh, z0 + hh + 0.10, segments=8, center=(x, y), color="gun_black"))
    k.box((x - 0.28, y - 0.03, z0 + hh + 0.02), (x + 0.28, y + 0.03, z0 + hh + 0.05), "metal")
    k.box((x - 0.03, y - 0.28, z0 + hh + 0.02), (x + 0.03, y + 0.28, z0 + hh + 0.05), "metal")

    # Two rusty corner posts
    for cx, cy in ((x - hw + 0.04, y + hd - 0.04), (x + hw - 0.04, y - hd + 0.04)):
        k.box((cx - 0.05, cy - 0.05, z0 - 0.02), (cx + 0.05, cy + 0.05, z0 + hh + 0.04), "rust")


def vent_stack(k, x, y, z0, h):
    """Metal rooftop vent pipe with a 90-degree bend and rain cap."""
    bend_z = z0 + max(0.3, h - 0.30)
    k.model.add(common.make_frustum(0.16, 0.16, z0, bend_z, segments=8, center=(x, y), color="metal"))
    k.model.add(common.make_frustum(0.20, 0.20, z0, z0 + 0.08, segments=8, center=(x, y), color="rock_dark"))
    # Top elbow bend toward +Y + flanged cap
    k.box((x - 0.16, y - 0.16, bend_z), (x + 0.16, y + 0.42, bend_z + 0.26), "metal")
    k.model.add(common.make_frustum(0.26, 0.04, bend_z + 0.24, z0 + h, segments=8, center=(x, y), color="rust_dark"))


def antenna(k, x, y, z0, h):
    """Thin rooftop antenna mast with two cross bars."""
    k.box((x - 0.14, y - 0.14, z0), (x + 0.14, y + 0.14, z0 + 0.12), "rock_dark")
    k.model.add(common.make_frustum(0.045, 0.022, z0 + 0.12, z0 + h, segments=6, center=(x, y), color="metal"))
    for frac, span in ((0.62, 0.55), (0.84, 0.36)):
        cz = z0 + h * frac
        k.box((x - span, y - 0.02, cz - 0.02), (x + span, y + 0.02, cz + 0.02), "metal")
        k.box((x - span, y - 0.02, cz - 0.12), (x - span + 0.03, y + 0.02, cz + 0.12), "rust")
        k.box((x + span - 0.03, y - 0.02, cz - 0.12), (x + span, y + 0.02, cz + 0.12), "rust")


def roof_deck(k, size, z, color):
    """Roof slab over `size x size` footprint (z..z+0.2) with a 0.6 m perimeter parapet."""
    hs = size / 2.0
    pt = 0.30
    top = z + 0.20
    parapet_top = top + 0.60
    k.box((-hs, -hs, z), (hs, hs, top), color)
    k.box((-hs, hs - pt, top), (hs, hs, parapet_top), color)
    k.box((-hs, -hs, top), (hs, -hs + pt, parapet_top), color)
    k.box((-hs, -hs + pt, top), (-hs + pt, hs - pt, parapet_top), color)
    k.box((hs - pt, -hs + pt, top), (hs, hs - pt, parapet_top), color)
