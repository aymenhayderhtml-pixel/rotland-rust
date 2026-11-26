"""Four 24 m landmark crowns (+ _lod1 each) for New Halden towers."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import kit  # noqa: E402
import roof_parts  # noqa: E402

FOOTPRINT = 24.0
HALF = FOOTPRINT / 2.0


def _strut(model, a, b, thick, color):
    """Thin box strut between 3D points `a` and `b`."""
    va, vb = Vector(a), Vector(b)
    d = vb - va
    if d.length < 1e-4:
        return
    z_axis = d.normalized()
    up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    bm = common.make_box((thick, thick, d.length), color=color)
    mat = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    model.add(bm)


def _angled_panel(model, center, size, rot, color):
    bm = common.make_box(size, color=color)
    common.transform(bm, loc=center, rot=rot)
    model.add(bm)


def _build_spire(model):
    k = kit.Kit(model)
    # 24 m base stone plinth (0..1.5 m)
    k.box((-HALF, -HALF, 0.0), (HALF, HALF, 0.9), "limestone")
    k.box((-HALF - 0.12, -HALF - 0.12, 0.9), (HALF + 0.12, HALF + 0.12, 1.25), "limestone_dark")
    k.box((-HALF + 0.4, -HALF + 0.4, 1.25), (HALF - 0.4, HALF - 0.4, 1.50), "mortar")

    # Tapered octagonal core + 8 continuous tapering ribs (1.5..15.5 m)
    model.add(common.make_frustum(10.2, 1.8, 1.5, 15.5, segments=8, rot_z=math.pi / 8, color="limestone"))
    for i in range(8):
        ang = 2.0 * math.pi * i / 8.0
        ca, sa = math.cos(ang), math.sin(ang)
        _strut(model, (ca * 10.2, sa * 10.2, 1.5), (ca * 1.9, sa * 1.9, 15.5), 0.85, "limestone_dark")
        _strut(model, (ca * 10.5, sa * 10.5, 1.5), (ca * 2.05, sa * 2.05, 15.4), 0.42, "mortar")
        # Dead windows on lower/mid rib faces
        for wf in (0.18, 0.42, 0.66):
            wz = 1.5 + wf * 14.0
            wr = 10.35 * (1.0 - wf) + 2.0 * wf + 0.35
            col = "soot" if (i + int(wf * 10)) % 3 == 0 else "glass_dark"
            bm = common.make_box((0.14, 0.50, 1.35), color=col)
            common.transform(bm, rot=(0.0, 0.0, ang), loc=(ca * wr, sa * wr, wz))
            model.add(bm)

    # Two lamp rings (at z=10.5 and z=15.3)
    model.add(common.make_frustum(4.8, 4.8, 10.4, 10.75, segments=12, color="brass"))
    model.add(common.make_frustum(2.2, 2.2, 15.2, 15.55, segments=12, color="brass"))

    # 2.5 m spire (15.5..18.0 m) + finial
    model.add(common.make_frustum(1.3, 0.14, 15.5, 17.6, segments=8, color="metal"))
    model.add(common.make_frustum(0.28, 0.0, 17.6, 18.0, segments=8, color="brass"))


def _build_mast(model):
    k = kit.Kit(model)
    # Base plinth (0..0.8 m) + parapet + water tank at base
    roof_parts.roof_deck(k, FOOTPRINT, 0.0, "limestone_dark")
    roof_parts.water_tank(k, -8.2, -8.2, 0.2, r=1.15, h=2.2)

    # 4 square lattice legs tapering from +-5.5 m at z=0.2 to +-2.2 m at z=18.0 m (struts every 1.5 m)
    bays_lower = 12  # 12 * 1.483 m ~ 1.5 m per bay
    for i in range(bays_lower):
        t0, t1 = i / bays_lower, (i + 1) / bays_lower
        z0 = 0.2 + t0 * 17.8
        z1 = 0.2 + t1 * 17.8
        h0 = 5.5 + (2.2 - 5.5) * t0
        h1 = 5.5 + (2.2 - 5.5) * t1
        c0 = [(-h0, -h0, z0), (h0, -h0, z0), (h0, h0, z0), (-h0, h0, z0)]
        c1 = [(-h1, -h1, z1), (h1, -h1, z1), (h1, h1, z1), (-h1, h1, z1)]
        for leg in range(4):
            nxt = (leg + 1) % 4
            _strut(model, c0[leg], c1[leg], 0.22, "metal")
            _strut(model, c1[leg], c1[nxt], 0.11, "metal")
            _strut(model, c0[leg], c1[nxt], 0.08, "rust" if i % 4 == 0 else "metal")
            _strut(model, c0[nxt], c1[leg], 0.08, "metal")

    # 6 x 6 m platform at z=18.0 m with railings
    pz = 18.0
    k.box((-3.0, -3.0, pz), (3.0, 3.0, pz + 0.22), "concrete_grey")
    for post in ((-2.9, -2.9), (0.0, -2.9), (2.9, -2.9), (2.9, 0.0),
                 (2.9, 2.9), (0.0, 2.9), (-2.9, 2.9), (-2.9, 0.0)):
        k.box((post[0] - 0.04, post[1] - 0.04, pz + 0.22), (post[0] + 0.04, post[1] + 0.04, pz + 1.25), "metal")
    for rz in (pz + 0.72, pz + 1.20):
        k.box((-2.95, 2.86, rz), (2.95, 2.94, rz + 0.06), "metal")
        k.box((-2.95, -2.94, rz), (2.95, -2.86, rz + 0.06), "metal")
        k.box((-2.94, -2.95, rz), (-2.86, 2.95, rz + 0.06), "metal")
        k.box((2.86, -2.95, rz), (2.94, 2.95, rz + 0.06), "metal")

    # Radio hut as separate node `radio_hut` (R43 goal)
    hut_parts = [
        common.make_box((2.6, 2.4, 2.3), loc=(-1.1, 0.6, pz + 0.22 + 1.15), color="rock_mid"),
        common.make_box((2.85, 2.65, 0.16), loc=(-1.1, 0.6, pz + 0.22 + 2.38), color="rock_dark"),
        common.make_box((0.95, 0.06, 1.95), loc=(-1.1, 1.82, pz + 0.22 + 0.98), color="bark_dark"),
        common.make_box((0.06, 0.90, 0.75), loc=(0.22, 0.6, pz + 0.22 + 1.45), color="glass_dark"),
    ]
    model.add_part("radio_hut", common.combine(hut_parts))

    # Dish on platform
    model.add(common.make_frustum(0.08, 0.08, pz + 0.22, pz + 1.1, segments=8, center=(1.8, 1.5), color="metal"))
    dish = common.make_frustum(0.85, 0.12, 0.0, 0.38, segments=12, color="white")
    common.transform(dish, rot=(math.radians(55), 0.0, math.radians(-25)), loc=(1.8, 1.5, pz + 1.25))
    model.add(dish)

    # Upper lattice mast (18.2..27.5 m) + spire to 30.0 m
    bays_upper = 6
    for i in range(bays_upper):
        t0, t1 = i / bays_upper, (i + 1) / bays_upper
        z0, z1 = 18.2 + t0 * 9.3, 18.2 + t1 * 9.3
        h0 = 1.1 + (0.35 - 1.1) * t0
        h1 = 1.1 + (0.35 - 1.1) * t1
        c0 = [(-h0, -h0, z0), (h0, -h0, z0), (h0, h0, z0), (-h0, h0, z0)]
        c1 = [(-h1, -h1, z1), (h1, -h1, z1), (h1, h1, z1), (-h1, h1, z1)]
        col = "red_paint" if i >= 4 else "metal"
        for leg in range(4):
            nxt = (leg + 1) % 4
            _strut(model, c0[leg], c1[leg], 0.12, col)
            _strut(model, c1[leg], c1[nxt], 0.07, col)
            _strut(model, c0[leg], c1[nxt], 0.05, col)

    model.add(common.make_frustum(0.09, 0.04, 27.5, 29.7, segments=8, color="white"))
    model.add(common.make_frustum(0.11, 0.0, 29.7, 30.0, segments=8, color="red_paint"))

    # 4 guy-cable runs from platform to crown base
    for sx, sy in ((-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)):
        _strut(model, (sx * 2.8, sy * 2.8, pz), (sx * 10.2, sy * 10.2, 0.4), 0.06, "gun_black")


def _build_stepped(model):
    k = kit.Kit(model)
    tiers = ((24.0, 0.0, 4.0), (18.0, 4.0, 7.5), (12.0, 7.5, 10.2))
    for idx, (w, z0, z1) in enumerate(tiers):
        hw = w / 2.0
        k.box((-hw, -hw, z0), (hw, hw, z1 - 0.3), "limestone")
        k.box((-hw - 0.16, -hw - 0.16, z1 - 0.3), (hw + 0.16, hw + 0.16, z1), "limestone_dark")
        # Vertical piers & dead windows on each tier
        n = int(round(w / 3.0))
        for i in range(n + 1):
            u = -hw + i * (w / n)
            k.box((u - 0.22, hw - 0.05, z0), (u + 0.22, hw + 0.14, z1 - 0.3), "limestone_dark")
            k.box((u - 0.22, -hw - 0.14, z0), (u + 0.22, -hw + 0.05, z1 - 0.3), "limestone_dark")
            k.box((-hw - 0.14, u - 0.22, z0), (-hw + 0.05, u + 0.22, z1 - 0.3), "limestone_dark")
            k.box((hw - 0.05, u - 0.22, z0), (hw + 0.14, u + 0.22, z1 - 0.3), "limestone_dark")
        for i in range(n):
            uc = -hw + (i + 0.5) * (w / n)
            col = "soot" if (i + idx) % 3 == 0 else "glass_dark"
            k.box((uc - 0.85, hw - 0.02, z0 + 0.6), (uc + 0.85, hw + 0.04, z1 - 0.7), col)
            k.box((uc - 0.85, -hw - 0.04, z0 + 0.6), (uc + 0.85, -hw + 0.02, z1 - 0.7), col)
            k.box((-hw - 0.04, uc - 0.85, z0 + 0.6), (-hw + 0.02, uc + 0.85, z1 - 0.7), col)
            k.box((hw - 0.02, uc - 0.85, z0 + 0.6), (hw + 0.04, uc + 0.85, z1 - 0.7), col)

    # Water tank on first terrace + small top spire (10.2..12.0 m)
    roof_parts.water_tank(k, -9.8, -9.8, 4.0, r=1.0, h=2.0)
    model.add(common.make_frustum(1.4, 0.25, 10.2, 11.6, segments=8, color="metal"))
    model.add(common.make_frustum(0.32, 0.0, 11.6, 12.0, segments=8, color="brass"))


def _build_glass_top(model):
    k = kit.Kit(model)
    # Stone ring base (0..1.5 m) + water tank at base terrace
    roof_parts.roof_deck(k, FOOTPRINT, 0.0, "limestone")
    k.box((-HALF - 0.12, -HALF - 0.12, 1.25), (HALF + 0.12, HALF + 0.12, 1.50), "limestone_dark")
    roof_parts.water_tank(k, -9.4, -9.4, 0.2, r=1.05, h=2.0)

    # Intact lower glass crown (1.5..6.8 m) as 4 perimeter curtain walls (open shattered top)
    gh = 8.4
    wt = 0.25
    k.box((-gh, -gh, 1.5), (gh, gh, 1.7), "soot")
    k.box((-gh, gh - wt, 1.5), (gh, gh, 6.8), "glass_dark")
    k.box((-gh, -gh, 1.5), (gh, -gh + wt, 6.8), "glass_dark")
    k.box((-gh, -gh + wt, 1.5), (-gh + wt, gh - wt, 6.8), "glass_dark")
    k.box((gh - wt, -gh + wt, 1.5), (gh, gh - wt, 6.8), "glass_dark")
    # Exposed interior steel cross-trusses
    _strut(model, (-gh, -gh, 6.6), (gh, gh, 6.6), 0.14, "rust")
    _strut(model, (-gh, gh, 6.6), (gh, -gh, 6.6), 0.14, "metal")
    for s in (-1.0, 1.0):
        for u in (-6.0, -3.0, 0.0, 3.0, 6.0):
            k.box((u - 0.09, s * gh - 0.09, 1.5), (u + 0.09, s * gh + 0.09, 6.8), "metal")
            k.box((s * gh - 0.09, u - 0.09, 1.5), (s * gh + 0.09, u + 0.09, 6.8), "metal")
    for rz in (3.2, 5.0, 6.7):
        k.box((-gh - 0.1, gh - 0.2, rz), (gh + 0.1, gh + 0.1, rz + 0.14), "metal")
        k.box((-gh - 0.1, -gh - 0.1, rz), (gh + 0.1, -gh + 0.2, rz + 0.14), "metal")
        k.box((-gh - 0.1, -gh, rz), (-gh + 0.2, gh, rz + 0.14), "metal")
        k.box((gh - 0.2, -gh, rz), (gh + 0.1, gh, rz + 0.14), "metal")

    # Shattered top third (6.8..10.0 m): jagged glass shards + bent steel frames
    shards = [
        ((-6.5, gh, 7.9), (2.2, 0.06, 2.4), (0.05, 0.18, 0.0), "glass_tint"),
        ((-2.2, gh, 8.3), (2.0, 0.06, 3.2), (-0.08, -0.22, 0.04), "glass_dark"),
        ((3.4, gh, 7.6), (2.4, 0.06, 1.8), (0.06, 0.15, -0.03), "glass_tint"),
        ((gh, 4.0, 8.4), (0.06, 2.3, 3.1), (0.20, -0.06, 0.0), "glass_dark"),
        ((gh, -3.5, 7.8), (0.06, 2.6, 2.2), (-0.18, 0.08, 0.0), "glass_tint"),
        ((-gh, 2.5, 8.1), (0.06, 2.2, 2.8), (0.14, 0.08, 0.02), "glass_dark"),
        ((-4.0, -gh, 7.7), (2.4, 0.06, 2.1), (-0.05, 0.25, 0.0), "glass_tint"),
    ]
    for loc, sz, rot, col in shards:
        _angled_panel(model, loc, sz, rot, col)

    # Bent steel frames poking up to 10.0 m
    beams = [
        ((-gh, gh, 6.8), (-gh + 0.8, gh - 0.5, 9.9), "rust"),
        ((gh, gh, 6.8), (gh - 0.6, gh - 0.9, 10.0), "metal"),
        ((gh, -gh, 6.8), (gh - 1.1, -gh + 0.4, 9.3), "rust"),
        ((-gh, -gh, 6.8), (-gh + 0.5, -gh + 0.7, 9.5), "metal"),
        ((0.0, gh, 6.8), (-0.7, gh - 0.4, 9.6), "rust"),
    ]
    for a, b, col in beams:
        _strut(model, a, b, 0.16, col)


def _build_lod1(model, kind):
    k = kit.Kit(model)
    if kind == "spire":
        k.box((-HALF, -HALF, 0.0), (HALF, HALF, 1.5), "limestone_dark")
        model.add(common.make_frustum(10.0, 1.5, 1.5, 15.5, segments=8, color="limestone"))
        model.add(common.make_frustum(1.0, 0.05, 15.5, 18.0, segments=6, color="metal"))
    elif kind == "mast":
        k.box((-HALF, -HALF, 0.0), (HALF, HALF, 0.8), "limestone_dark")
        model.add(common.make_frustum(5.5, 2.0, 0.8, 18.0, segments=4, rot_z=math.pi / 4, color="metal"))
        k.box((-3.0, -3.0, 18.0), (3.0, 3.0, 20.5), "rock_mid")
        model.add(common.make_frustum(1.0, 0.1, 20.5, 30.0, segments=4, color="red_paint"))
    elif kind == "stepped":
        k.box((-12.0, -12.0, 0.0), (12.0, 12.0, 4.0), "limestone")
        k.box((-9.0, -9.0, 4.0), (9.0, 9.0, 7.5), "limestone_dark")
        k.box((-6.0, -6.0, 7.5), (6.0, 6.0, 10.2), "limestone")
        model.add(common.make_frustum(1.2, 0.0, 10.2, 12.0, segments=6, color="metal"))
    else:  # glass_top
        k.box((-HALF, -HALF, 0.0), (HALF, HALF, 1.5), "limestone_dark")
        k.box((-8.4, -8.4, 1.5), (8.4, 8.4, 6.8), "glass_dark")
        k.box((-6.0, 7.0, 6.8), (-4.0, 8.0, 9.8), "glass_tint")
        k.box((5.0, 6.5, 6.8), (7.5, 7.5, 10.0), "metal")


def build_all():
    results = []
    specs = [
        ("tw_landmark_spire", 25000, _build_spire),
        ("tw_landmark_spire_lod1", 2500, lambda m: _build_lod1(m, "spire")),
        ("tw_landmark_mast", 25000, _build_mast),
        ("tw_landmark_mast_lod1", 2500, lambda m: _build_lod1(m, "mast")),
        ("tw_landmark_stepped", 25000, _build_stepped),
        ("tw_landmark_stepped_lod1", 2500, lambda m: _build_lod1(m, "stepped")),
        ("tw_landmark_glass_top", 25000, _build_glass_top),
        ("tw_landmark_glass_top_lod1", 2500, lambda m: _build_lod1(m, "glass_top")),
    ]
    for name, budget, fn in specs:
        ok = common.run_model(name, "buildings", budget, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)
        results.append((name, ok))
    return results


if __name__ == "__main__":
    res = build_all()
    failed = [n for n, ok in res if not ok]
    print(f"LANDMARKS: {len(res) - len(failed)}/{len(res)} OK")
    sys.exit(1 if failed else 0)
