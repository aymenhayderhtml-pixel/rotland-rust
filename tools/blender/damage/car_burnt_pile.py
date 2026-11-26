"""Burnt stacked car pile (`car_burnt_pile`), 3 burnt car shells tipped and stacked ~6 x 5 x 3 m.

Reuses `jam_cars.CARS` specs + `jam_car_shell.greenhouse_parts` and un-bevelled `jam_util.box`
lower body shells so all 3 wrecks fit cleanly under the 4000-triangle prop budget.
One solid convex hull collider `pile`. Origin: bottom centre.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))

import bmesh  # noqa: E402

import common  # noqa: E402
import jam_car_shell  # noqa: E402
import jam_cars  # noqa: E402
from jam_util import box  # noqa: E402

BUDGET = 4000  # new: DECISIONS 2026-11-26 (car_burnt_pile <= 4000 tris)


def _burnt_shell(key, seed, on_ground=False):
    """Builds a burnt car shell (un-bevelled lower body + greenhouse + rust/soot patches) <= ~1200 tris."""
    spec = dict(jam_cars.CARS[key])
    spec["wipers"] = False  # new: DECISIONS 2026-11-26 (no floating wiper twigs when tipped)
    spec["doors"] = [(s, h, b, 0.0) for s, h, b, _ in spec["doors"]]  # new: DECISIONS 2026-11-26 (shut jammed doors)
    w, half, sill, belt = spec["W"], spec["half_l"], spec["sill"], spec["belt"]
    cab = spec["cab"]
    bed = spec.get("bed")
    body = "gun_black"
    rng = random.Random(seed)  # new: DECISIONS 2026-11-26

    parts = []
    # Lower pan, bonnet, rear closure, bumpers, grille, side walls, wheel arches
    parts.append(box((-w + 0.04, -half, sill - 0.06), (w - 0.04, half, sill), body))
    parts.append(box((-w + 0.02, cab["bf"] - 0.02, belt - 0.07), (w - 0.02, half - 0.04, belt), body))  # bonnet
    rear_top = bed["top"] if bed else belt
    parts.append(box((-w, -half, sill), (w, -half + 0.07, rear_top), body))
    if bed:
        parts.append(box((-w, bed["front"] - 0.05, sill), (w, bed["front"], bed["top"]), body))
        parts.append(box((-w + 0.05, -half + 0.07, sill), (w - 0.05, bed["front"], sill + 0.06), "metal"))
    else:
        parts.append(box((-w + 0.02, -half + 0.04, belt - 0.05), (w - 0.02, cab["bb"], belt), body))  # boot

    # Front & rear bumpers + grille
    parts.append(box((-w - 0.03, half - 0.03, sill - 0.12), (w + 0.03, half + 0.05, sill + 0.02), "rock_dark"))
    parts.append(box((-w - 0.03, -half - 0.05, sill - 0.12), (w + 0.03, -half + 0.03, sill + 0.02), "rock_dark"))
    parts.append(box((-0.30, half - 0.02, sill + 0.10), (0.30, half + 0.03, sill + 0.30), "soot"))

    # Side body segments + shut jammed doors + fenders
    for side in (-1, 1):
        x0, x1 = sorted((side * w, side * (w - 0.05)))
        gaps = [(min(h, b), max(h, b)) for s, h, b, _ in spec["doors"] if s == side]
        for y0, y1, top in jam_car_shell._side_segments(spec):
            for p0, p1 in jam_car_shell._carve(y0, y1, gaps):
                parts.append(box((x0, p0, sill), (x1, p1, top), body))
        for axle in (spec["wheel_y"], -spec["wheel_y"]):
            parts.append(box((side * (w - 0.01), axle - 0.40, sill - 0.04),
                             (side * (w + 0.06), axle + 0.40, sill + 0.28), body))
    for side, hinge, back, open_deg in spec["doors"]:
        ang = side * math.radians(open_deg)  # new: DECISIONS 2026-11-26
        door = box((-0.025, back - hinge, sill), (0.025, 0.0, belt), body)
        common.transform(door, rot=(0.0, 0.0, ang), loc=(side * (w - 0.025), hinge, 0.0))
        parts.append(door)

    # Greenhouse (cabin loft + burnt hole windows + pillars) from jam_car_shell
    parts.extend(jam_car_shell.greenhouse_parts(spec, body, burnt=True))

    # Rust & soot scorch patches (un-bevelled, cheap on triangles)
    for side in (-1, 1):
        for _ in range(5):
            y = rng.uniform(-half + 0.4, half - 0.6)
            z = rng.uniform(sill, belt - 0.25)
            x0, x1 = sorted((side * (w + 0.003), side * (w + 0.018)))
            parts.append(box((x0, y, z), (x1, y + rng.uniform(0.3, 0.7), z + rng.uniform(0.12, 0.28)), "rust"))
    for _ in range(4):
        x = rng.uniform(-cab["xt"] + 0.15, cab["xt"] - 0.35)
        y = rng.uniform(cab["tb"] + 0.2, cab["tf"] - 0.4)
        parts.append(box((x, y, spec["roof"] - 0.004),
                         (x + rng.uniform(0.3, 0.65), y + rng.uniform(0.25, 0.55), spec["roof"] + 0.014),
                         "rust" if rng.random() < 0.7 else "soil_dark"))

    if on_ground:
        for sx in (-1, 1):
            rim = common.make_frustum(0.22, 0.22, -0.08, 0.08, segments=8, color="metal")  # new: DECISIONS 2026-11-26
            common.transform(rim, rot=(0.0, math.pi / 2.0, 0.0), loc=(sx * (w - 0.06), spec["wheel_y"], 0.22))
            parts.append(rim)
    return common.combine(parts)


def build(model):
    # 3 burnt wrecks stacked and tipped into a ~6 x 5 x 3 m pile resting directly on the ground
    placements = [
        ("sedan", 4101, True, (0.04, 0.06, 0.42), (-0.70, 0.20, 0.0)),      # base sedan
        ("hatch", 4102, False, (-0.12, -0.24, 1.52), (0.85, -0.30, 0.12)),   # hatchback jammed against sedan
        ("pickup", 4103, False, (0.38, -0.44, -0.82), (-0.15, 0.25, 0.92)),  # pickup resting on top (~2.9 m)
    ]  # new: DECISIONS 2026-11-26
    all_bms = []
    for key, seed, on_ground, rot, loc in placements:
        bm = _burnt_shell(key, seed, on_ground=on_ground)
        common.transform(bm, rot=rot, loc=loc)
        all_bms.append(bm)

    # Low scorched ash/rims tucked at ground edge (no square pedestal)
    for rx, ry in ((-1.9, -1.2), (1.8, 1.1), (0.3, -1.7)):
        rim = common.make_frustum(0.24, 0.24, 0.0, 0.12, segments=8, center=(rx, ry), color="rust_dark")  # new: DECISIONS 2026-11-26
        all_bms.append(rim)

    combined = common.combine(all_bms)
    hull = common.new_bm()
    for v in combined.verts:
        hull.verts.new(v.co)
    hull.verts.ensure_lookup_table()
    bmesh.ops.convex_hull(hull, input=list(hull.verts))
    bmesh.ops.recalc_face_normals(hull, faces=list(hull.faces))

    model.add(combined)
    model.add_collider("pile", hull)


def build_all():
    ok = common.run_model("car_burnt_pile", "props", BUDGET, build, view=(1.0, -1.0, 0.65))
    return [("car_burnt_pile", ok)]
