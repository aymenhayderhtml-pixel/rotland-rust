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


def water_tank_ny(k, x, y, z0, r=1.40, leg_h=1.45, tank_h=2.25):
    """Bigger New York wooden-barrel rooftop water tank on steel legs with dunnage, hoops and ladder."""
    leg_off = r * 0.74
    plat_z = z0 + leg_h

    # 4 concrete/steel foot pads, 4 legs, mid-tie beams, and diagonal braces
    for sx in (-1.0, 1.0):
        for sy in (-1.0, 1.0):
            lx, ly = x + sx * leg_off, y + sy * leg_off
            k.box((lx - 0.16, ly - 0.16, z0), (lx + 0.16, ly + 0.16, z0 + 0.12), "rock_dark")
            k.box((lx - 0.08, ly - 0.08, z0 + 0.12), (lx + 0.08, ly + 0.08, plat_z), "metal")

    mid_z = z0 + leg_h * 0.52
    for sy in (-1.0, 1.0):
        ly = y + sy * leg_off
        k.box((x - leg_off, ly - 0.04, mid_z), (x + leg_off, ly + 0.04, mid_z + 0.08), "metal")
        _angled_box(k, (x - leg_off, ly, z0 + 0.14), (x + leg_off, ly, plat_z - 0.06), 0.045, 0.045, "rust_dark")
    for sx in (-1.0, 1.0):
        lx = x + sx * leg_off
        k.box((lx - 0.04, y - leg_off, mid_z), (lx + 0.04, y + leg_off, mid_z + 0.08), "metal")
        _angled_box(k, (lx, y - leg_off, z0 + 0.14), (lx, y + leg_off, plat_z - 0.06), 0.045, 0.045, "metal")

    # Steel dunnage beams + wooden catwalk deck + partial rail
    for sx in (-0.65, 0.65):
        bx = x + sx * leg_off
        k.box((bx - 0.08, y - r - 0.12, plat_z), (bx + 0.08, y + r + 0.12, plat_z + 0.14), "metal")
    deck_z0, deck_z1 = plat_z + 0.14, plat_z + 0.24
    k.box((x - r - 0.15, y - r - 0.15, deck_z0), (x + r + 0.15, y + r + 0.15, deck_z1), "bark_dark")
    ry = y + r + 0.10
    k.box((x - r, ry - 0.03, deck_z1 + 0.65), (x + r, ry + 0.03, deck_z1 + 0.70), "metal")
    for rx in (x - r, x, x + r):
        k.box((rx - 0.03, ry - 0.03, deck_z1), (rx + 0.03, ry + 0.03, deck_z1 + 0.70), "metal")

    # Wooden stave barrel + conical roof + finial cap
    tz0 = deck_z1
    tz1 = tz0 + tank_h
    k.model.add(common.make_frustum(r, r, tz0, tz1, segments=10, center=(x, y), color="bark_light"))
    k.model.add(common.make_frustum(r + 0.18, 0.06, tz1, tz1 + 0.78, segments=10, center=(x, y), color="bark_dark"))
    k.model.add(common.make_frustum(0.10, 0.0, tz1 + 0.78, tz1 + 0.96, segments=6, center=(x, y), color="metal"))

    # Steel tension hoops (3 intact + 1 broken)
    for frac in (0.14, 0.42, 0.84):
        hz = tz0 + tank_h * frac
        k.model.add(common.make_frustum(r + 0.03, r + 0.03, hz, hz + 0.06, segments=10, center=(x, y), color="metal"))
    mid_h = tz0 + tank_h * 0.63
    for i in range(8):
        if i in (1, 2):
            continue
        a0 = 2.0 * math.pi * i / 8
        a1 = 2.0 * math.pi * (i + 1) / 8
        p0 = (x + math.cos(a0) * (r + 0.03), y + math.sin(a0) * (r + 0.03), mid_h)
        dz = -0.16 if i == 3 else 0.0
        p1 = (x + math.cos(a1) * (r + 0.03), y + math.sin(a1) * (r + 0.03), mid_h + dz)
        _angled_box(k, p0, p1, 0.04, 0.05, "rust_dark")

    # Side ladder on +X
    lad_x = x + r + 0.10
    lad_top = tz1 + 0.10
    for ly in (y - 0.22, y + 0.22):
        k.box((lad_x - 0.025, ly - 0.025, z0), (lad_x + 0.025, ly + 0.025, lad_top), "metal")
    rz = z0 + 0.45
    while rz < lad_top - 0.15:
        k.box((lad_x - 0.02, y - 0.22, rz), (lad_x + 0.02, y + 0.22, rz + 0.035), "rust")
        rz += 0.70


def machinery(k, x, y, z0):
    """Rooftop compressor machinery block (~3.2 x 2.2 x 1.75 m) with pipes and fan grille."""
    # Concrete skid & rubber mounts
    k.box((x - 1.60, y - 1.10, z0), (x + 1.60, y + 1.10, z0 + 0.16), "rock_dark")
    for sx in (-1.40, 1.40):
        for sy in (-0.90, 0.90):
            k.box((x + sx - 0.12, y + sy - 0.12, z0 + 0.16),
                  (x + sx + 0.12, y + sy + 0.12, z0 + 0.24), "gun_black")

    # Main compressor housing + rusted hatch + control box
    z_base = z0 + 0.24
    k.box((x - 1.35, y - 0.85, z_base), (x + 0.55, y + 0.85, z_base + 1.25), "concrete_grey")
    k.box((x - 1.38, y - 0.55, z_base + 0.20), (x - 1.32, y + 0.25, z_base + 0.98), "rust")
    k.box((x - 0.90, y + 0.84, z_base + 0.35), (x - 0.20, y + 0.95, z_base + 0.95), "metal")
    k.box((x - 0.82, y + 0.94, z_base + 0.72), (x - 0.52, y + 0.98, z_base + 0.86), "red_paint")
    k.box((x - 0.44, y + 0.94, z_base + 0.72), (x - 0.26, y + 0.98, z_base + 0.86), "brass")

    # Front (+Y) louvered fan grille + internal blades
    k.box((x - 0.05, y + 0.82, z_base + 0.15), (x + 0.45, y + 0.90, z_base + 1.05), "rock_dark")
    for i in range(4):
        lz = z_base + 0.26 + i * 0.20
        k.box((x - 0.01, y + 0.86, lz), (x + 0.41, y + 0.93, lz + 0.04), "metal")

    # Side cylindrical pressure tank + top exhaust + connecting pipes
    tx, ty = x + 1.05, y - 0.15
    k.model.add(common.make_frustum(0.38, 0.38, z_base, z_base + 1.22, segments=8, center=(tx, ty), color="metal"))
    for hz in (z_base + 0.25, z_base + 0.92):
        k.model.add(common.make_frustum(0.40, 0.40, hz, hz + 0.06, segments=8, center=(tx, ty), color="rust_dark"))
    k.box((x + 0.55, ty - 0.08, z_base + 0.90), (tx, ty + 0.08, z_base + 1.04), "rust")
    k.box((x + 0.55, y + 0.30, z_base + 0.50), (tx + 0.25, y + 0.44, z_base + 0.64), "metal")
    k.box((tx + 0.12, y + 0.30, z_base), (tx + 0.25, y + 0.44, z_base + 0.64), "metal")
    k.model.add(common.make_frustum(0.14, 0.18, z_base + 1.25, z_base + 1.52, segments=8,
                                    center=(x - 0.45, y), color="rock_dark"))


def stair_hut(k, x, y, z0, wall_color="concrete_grey"):
    """Rooftop stair access hut (~3.0 x 3.4 x 2.55 m) with sloped roof and door."""
    hw, hd = 1.50, 1.70
    # Concrete curb
    k.box((x - hw - 0.08, y - hd - 0.08, z0), (x + hw + 0.08, y + hd + 0.08, z0 + 0.14), "concrete_stain")
    # Main hut walls (flat top under sloped roof)
    zb = z0 + 0.14
    k.box((x - hw, y - hd, zb), (x + hw, y + hd, zb + 2.05), wall_color)
    # Sloped roof from z0+2.10 at +Y front up to z0+2.52 at -Y back
    roof_bm = common.make_box((hw * 2.0 + 0.32, hd * 2.0 + 0.34, 0.16), color="rock_dark")
    common.transform(roof_bm, rot=(0.12, 0.0, 0.0), loc=(x, y, zb + 2.18))
    k.model.add(roof_bm)
    # Metal coping lip on back high edge
    k.box((x - hw - 0.16, y - hd - 0.16, zb + 2.32), (x + hw + 0.16, y - hd + 0.02, zb + 2.46), "metal")

    # Recessed metal access door on +Y front + frame + handle + threshold step
    dy = y + hd
    k.box((x - 0.56, dy - 0.02, zb), (x + 0.56, dy + 0.06, zb + 1.98), "gun_black")
    k.box((x - 0.48, dy + 0.02, zb + 0.02), (x + 0.48, dy + 0.08, zb + 1.90), "metal")
    k.box((x - 0.22, dy + 0.06, zb + 1.25), (x + 0.22, dy + 0.10, zb + 1.65), "glass_dark")
    k.box((x + 0.30, dy + 0.07, zb + 0.88), (x + 0.40, dy + 0.13, zb + 1.02), "brass")
    k.box((x - 0.70, dy, z0), (x + 0.70, dy + 0.32, zb), "rock_mid")

    # Side vent louver + corner rust conduit
    k.box((x + hw - 0.02, y - 0.40, zb + 1.20), (x + hw + 0.05, y + 0.40, zb + 1.75), "rock_dark")
    for i in range(3):
        vz = zb + 1.28 + i * 0.16
        k.box((x + hw + 0.02, y - 0.34, vz), (x + hw + 0.07, y + 0.34, vz + 0.04), "metal")
    k.box((x - hw - 0.04, y + hd - 0.14, zb), (x - hw + 0.02, y + hd - 0.06, zb + 1.90), "rust_dark")


def lattice_antenna(k, x, y, z0, h=4.2):
    """Tapering 4-legged lattice rooftop antenna mast (3-5 m) with crossbars and microwave dish."""
    # Mounting plate
    k.box((x - 0.50, y - 0.50, z0), (x + 0.50, y + 0.50, z0 + 0.12), "rock_dark")

    bays = 3
    lat_top = z0 + h * 0.74
    lat_z0 = z0 + 0.12
    for i in range(bays):
        t0, t1 = i / bays, (i + 1) / bays
        bz0 = lat_z0 + (lat_top - lat_z0) * t0
        bz1 = lat_z0 + (lat_top - lat_z0) * t1
        h0 = 0.40 + (0.14 - 0.40) * t0
        h1 = 0.40 + (0.14 - 0.40) * t1
        c0 = [(x - h0, y - h0, bz0), (x + h0, y - h0, bz0), (x + h0, y + h0, bz0), (x - h0, y + h0, bz0)]
        c1 = [(x - h1, y - h1, bz1), (x + h1, y - h1, bz1), (x + h1, y + h1, bz1), (x - h1, y + h1, bz1)]
        for leg in range(4):
            nxt = (leg + 1) % 4
            _angled_box(k, c0[leg], c1[leg], 0.055, 0.055, "metal")
            _angled_box(k, c1[leg], c1[nxt], 0.035, 0.035, "metal")
            _angled_box(k, c0[leg], c1[nxt], 0.028, 0.028, "rust_dark" if (i + leg) % 3 == 0 else "metal")

    # Upper mast pole + red/white aircraft tip + crossbars + small dish
    k.model.add(common.make_frustum(0.05, 0.03, lat_top, z0 + h * 0.92, segments=6, center=(x, y), color="white"))
    k.model.add(common.make_frustum(0.03, 0.015, z0 + h * 0.92, z0 + h, segments=6, center=(x, y), color="red_paint"))
    for frac, span in ((0.78, 0.60), (0.90, 0.38)):
        cz = z0 + h * frac
        k.box((x - span, y - 0.025, cz - 0.025), (x + span, y + 0.025, cz + 0.025), "metal")
        k.box((x - span, y - 0.02, cz - 0.14), (x - span + 0.03, y + 0.02, cz + 0.14), "rust")
        k.box((x + span - 0.03, y - 0.02, cz - 0.14), (x + span, y + 0.02, cz + 0.14), "rust")

    dish_z = z0 + h * 0.55
    k.box((x - 0.03, y + 0.16, dish_z - 0.03), (x + 0.03, y + 0.34, dish_z + 0.03), "metal")
    dish = common.make_frustum(0.26, 0.05, 0.0, 0.12, segments=8, color="white")
    common.transform(dish, rot=(1.15, 0.0, 0.3), loc=(x, y + 0.36, dish_z))
    k.model.add(dish)
