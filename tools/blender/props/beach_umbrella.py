"""beach_umbrella: a broken beach sun umbrella with tilted pole and torn canopy (<= 300 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "beach_umbrella", "props", 300


def build(model):
    # Central metal pole tilted in the sand (tilt around X and Y)
    rot_tilt = (0.12, -0.10, 0.0)
    pole_h = 2.10
    pole = common.make_box((0.06, 0.06, pole_h), color="white")
    common.transform(pole, rot=rot_tilt, loc=(0.0, 0.0, pole_h / 2))
    model.add(pole)

    # Runner hub mechanism on the pole
    hub_z = 1.70
    hub = common.make_frustum(0.08, 0.08, -0.06, 0.06, segments=6, color="metal")
    common.transform(hub, rot=rot_tilt, loc=(0.0, 0.0, hub_z))
    model.add(hub)

    # Octagonal sun canopy: diameter ~2.1 m, apex at Z=2.15
    apex = (0.0, 0.0, 2.15)
    radius = 1.05
    eave_z = 1.75
    segments = 8
    colors = ["red_paint", "white"]

    # One side is broken/collapsed: ribs 4 and 5 sag down lower
    bm = common.new_bm()
    verts_ring = []
    for i in range(segments):
        ang = 2 * math.pi * i / segments
        r = radius
        z = eave_z
        if i in (4, 5):
            z -= 0.35
            r -= 0.15
        elif i == 3:
            z -= 0.15
        vx = math.cos(ang) * r
        vy = math.sin(ang) * r
        verts_ring.append(bm.verts.new((vx, vy, z)))

    v_apex = bm.verts.new(apex)

    for i in range(segments):
        j = (i + 1) % segments
        col = colors[i % len(colors)]
        face = bm.faces.new([v_apex, verts_ring[i], verts_ring[j]])
        common.paint_faces(bm, [face], col)

    # Tilt canopy with pole
    common.transform(bm, rot=rot_tilt)
    model.add(bm)

    # Bent metal spoke rib hanging down from the broken canopy section
    broken_rib = common.make_box((0.025, 0.025, 0.50), color="metal")
    common.transform(broken_rib, rot=(0.45, 0.10, 0.0), loc=(0.10, -0.80, 1.45))
    model.add(broken_rib)

    # Sand anchor base at ground level
    base = common.make_frustum(0.18, 0.08, 0.0, 0.16, segments=6, color="white")
    model.add(base)

    # Collider
    model.add_collider("umbrella", common.make_box((1.80, 1.80, 2.20), loc=(0.10, -0.10, 1.10), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
