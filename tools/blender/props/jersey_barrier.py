"""jersey_barrier: a 3 m concrete road barrier, chipped and weathered (<= 300 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "jersey_barrier", "props", 300


def _barrier_profile_verts(bm, y, scale_width=1.0, z_offset=0.0):
    """Creates a ring of 8 profile vertices of a standard Jersey barrier cross-section at coordinate y."""
    coords = [
        (-0.30 * scale_width, 0.0 + z_offset),
        (-0.28 * scale_width, 0.08 + z_offset),
        (-0.21 * scale_width, 0.28 + z_offset),
        (-0.09 * scale_width, 0.82 + z_offset),
        (0.09 * scale_width, 0.82 + z_offset),
        (0.21 * scale_width, 0.28 + z_offset),
        (0.28 * scale_width, 0.08 + z_offset),
        (0.30 * scale_width, 0.0 + z_offset),
    ]
    return [bm.verts.new((x, y, z)) for x, z in coords]


def _build_barrier_mesh():
    bm = common.new_bm()
    layer = bm.faces.layers.int[common.LAYER]

    # Stations along length (Y):
    # y = -1.5 (clean end)
    # y = 0.5  (mid)
    # y = 1.0  (start of chipped damage)
    y_stations = [-1.5, 0.5, 1.0]
    rings = [_barrier_profile_verts(bm, y) for y in y_stations]

    # Clean end face at y = -1.5 (facing -Y)
    f_end1 = bm.faces.new(list(reversed(rings[0])))
    f_end1[layer] = common.palette.key_of("rock_mid")

    # Connect segments between rings 0->1 and 1->2
    for s in range(len(rings) - 1):
        r1 = rings[s]
        r2 = rings[s + 1]
        for i in range(7):
            v_quad = [r1[i], r2[i], r2[i + 1], r1[i + 1]]
            f = bm.faces.new(v_quad)
            if i in (0, 6):
                col = "rock_mid" if s == 1 else "dirt"
            elif i == 3:
                col = "rock_light"
            else:
                col = "rock_mid" if (i % 2 == 0) else "rock_light"
            f[layer] = common.palette.key_of(col)

        # Bottom face
        f_bot = bm.faces.new([r1[7], r2[7], r2[0], r1[0]])
        f_bot[layer] = common.palette.key_of("rock_dark")

    # Damaged section: y from 1.0 to 1.5
    # The right top corner is broken off / gouged down heavily
    r3_coords = [
        (-0.30, 1.5, 0.0),
        (-0.28, 1.5, 0.08),
        (-0.21, 1.5, 0.28),
        (-0.09, 1.5, 0.78),
        (0.00, 1.5, 0.50),   # chipped top right
        (0.12, 1.5, 0.22),   # broken shoulder
        (0.24, 1.5, 0.08),
        (0.30, 1.5, 0.0),
    ]
    r3 = [bm.verts.new(c) for c in r3_coords]

    # Mid gouge vertex for jagged fracture cavity
    r2 = rings[2]
    cavity_v = bm.verts.new((0.04, 1.25, 0.45))

    # Loft left side and base normally
    for i in (0, 1, 2):
        f = bm.faces.new([r2[i], r3[i], r3[i + 1], r2[i + 1]])
        f[layer] = common.palette.key_of("rock_mid")

    for i in (5, 6):
        f = bm.faces.new([r2[i], r3[i], r3[i + 1], r2[i + 1]])
        f[layer] = common.palette.key_of("rock_mid")

    # Triangulated jagged chip faces connecting to cavity_v
    f_c1 = bm.faces.new([r2[3], r3[3], r3[4]])
    f_c1[layer] = common.palette.key_of("rock_light")

    f_c2 = bm.faces.new([r2[3], r3[4], cavity_v])
    f_c2[layer] = common.palette.key_of("rock_dark")

    f_c3 = bm.faces.new([r2[3], cavity_v, r2[4]])
    f_c3[layer] = common.palette.key_of("rock_dark")

    f_c4 = bm.faces.new([r2[4], cavity_v, r2[5]])
    f_c4[layer] = common.palette.key_of("rock_dark")

    f_c5 = bm.faces.new([r2[5], cavity_v, r3[5]])
    f_c5[layer] = common.palette.key_of("rock_mid")

    f_c6 = bm.faces.new([r3[4], r3[5], cavity_v])
    f_c6[layer] = common.palette.key_of("rock_light")

    # Bottom face
    f_bot2 = bm.faces.new([r2[7], r3[7], r3[0], r2[0]])
    f_bot2[layer] = common.palette.key_of("rock_dark")

    # Broken end face at y = 1.5 (facing +Y)
    f_end2 = bm.faces.new(r3)
    f_end2[layer] = common.palette.key_of("rock_dark")

    common.bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def _details():
    """Bent rebar anchored inside the broken cavity, chipped debris chunk, and forklift slots."""
    bms = []
    # Rebar rod 1: firmly anchored inside the concrete cavity (y=1.2, z=0.48) and bent out towards y=1.55
    rod1 = common.make_box((0.02, 0.38, 0.02), loc=(0.04, 1.36, 0.54), color="rust")
    common.transform(rod1, rot=(0.28, 0.12, 0.0))
    bms.append(rod1)

    # Rebar rod 2: lower horizontal bend poking out of side chip
    rod2 = common.make_box((0.02, 0.26, 0.02), loc=(0.08, 1.30, 0.38), color="rust")
    common.transform(rod2, rot=(-0.20, 0.22, 0.15))
    bms.append(rod2)

    # Broken concrete chunk on ground next to damaged end
    chunk1 = common.make_box((0.18, 0.16, 0.12), loc=(0.24, 1.45, 0.06), color="rock_light")
    common.transform(chunk1, rot=(0.2, -0.3, 0.4))
    bms.append(chunk1)

    chunk2 = common.make_box((0.12, 0.10, 0.08), loc=(0.28, 1.25, 0.04), color="rock_mid")
    common.transform(chunk2, rot=(-0.1, 0.4, 0.2))
    bms.append(chunk2)

    # Drainage / forklift cutout slots at the bottom (visual cutouts)
    notch1 = common.make_box((0.62, 0.28, 0.06), loc=(0.0, -0.6, 0.03), color="rock_dark")
    bms.append(notch1)
    notch2 = common.make_box((0.62, 0.28, 0.06), loc=(0.0, 0.6, 0.03), color="rock_dark")
    bms.append(notch2)

    return common.combine(bms)


def build(model):
    model.add(_build_barrier_mesh())
    model.add(_details())
    # Collider: convex box covering the barrier
    model.add_collider("barrier", common.make_box((0.60, 3.0, 0.82), loc=(0.0, 0.0, 0.41), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
