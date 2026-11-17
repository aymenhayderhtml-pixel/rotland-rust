"""Mesh for the wild animals (V12-01, rounded in visual plan step 5). Every part is rigidly skinned to one bone.

Part kinds (see wild_species.py):
- ("seg", bone, p0, p1, width, depth, colour): a round tube along a bone, tapering a little toward p1.
- ("tube", bone, [(point, half_width, half_depth), ...], colour): a round tube through several stations.
- ("ell", bone, (rx, ry, rz), center, colour): a low-poly ellipsoid (bodies, heads, eyes).
- ("box", bone, size, loc, colour): a plain box (hooves, small hard parts).
Each part's vertices are counted as it is built, so the bone groups always match the mesh.
"""
import math

import bmesh
import common  # noqa: E402
from mathutils import Vector


def tube(stations, colour, segs=8):
    """Closed round tube through (point, half_width, half_depth) stations. Width is sideways (X)."""
    pts = [Vector(p) for p, _, _ in stations]
    axis = (pts[-1] - pts[0]).normalized()
    ref = Vector((1.0, 0.0, 0.0)) if abs(axis.x) < 0.9 else Vector((0.0, 1.0, 0.0))
    up = axis.cross(ref).normalized()
    side = up.cross(axis).normalized()
    bm = common.new_bm()
    rings = []
    for p, (_, rw, rd) in zip(pts, stations):
        rings.append([bm.verts.new(p + side * (math.cos(a) * max(rw, 0.002)) + up * (math.sin(a) * max(rd, 0.002)))
                      for a in (2.0 * math.pi * (i + 0.5) / segs for i in range(segs))])
    for r0, r1 in zip(rings, rings[1:]):
        for i in range(segs):
            j = (i + 1) % segs
            bm.faces.new([r0[i], r0[j], r1[j], r1[i]])
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, colour)
    return bm


def segment(p0, p1, width, depth, colour):
    """Round tube from p0 to p1, a little thinner at p1 (limbs, necks and tails taper)."""
    return tube([(p0, width / 2, depth / 2), (p1, width * 0.42, depth * 0.42)], colour)


def ellipsoid(radii, center, colour, segs=10, rings=6):
    c = Vector(center)
    stations = []
    for k in range(rings):
        th = math.pi * (k + 0.5) / rings
        stations.append(((c.x, c.y, c.z - radii[2] * math.cos(th)), radii[0] * math.sin(th), radii[1] * math.sin(th)))
    return tube(stations, colour, segs=segs)


def _leg_parts(spec):
    """Thigh, shin and hoof for each leg, on the leg's own bones. Hind thighs are heavier."""
    out = []
    knee, ankle = spec["knee_z"], spec["ankle_z"]
    uw, lw = spec["upper_w"], spec["lower_w"]
    for leg, (_, x, y, top) in spec["legs"].items():
        thigh = 1.35 if leg.startswith("B") else 1.15
        out.append(("tube", f"upper_{leg}", [((x, y, top + 0.02), uw * thigh / 2, uw * thigh * 0.6),
                                             ((x, y, (top + knee) / 2), uw * 0.55, uw * 0.6),
                                             ((x, y, knee), lw * 0.5, lw * 0.55)], spec["coat"]))
        out.append(("tube", f"lower_{leg}", [((x, y, knee + 0.01), lw * 0.5, lw * 0.55),
                                             ((x, y + 0.01, ankle + 0.03), lw * 0.32, lw * 0.36),
                                             ((x, y + 0.01, ankle), lw * 0.36, lw * 0.4)], spec["coat"]))
        # Rounded hoof or paw: wide at the heel, tapering toward the toe, flat on the ground
        pw, pl = spec["paw_w"], spec["paw_len"]
        out.append(("tube", f"paw_{leg}", [((x, y + 0.005, 0.03), pw * 0.45, 0.03),
                                           ((x, y + 0.01 + pl * 0.6, 0.026), pw * 0.5, 0.026),
                                           ((x, y + 0.01 + pl, 0.015), pw * 0.3, 0.015)], spec["hoof"]))
    return out


def part_list(spec):
    """All parts of one species: its body and head parts, then the legs."""
    return list(spec["parts"]) + _leg_parts(spec)


def _part_mesh(part):
    kind = part[0]
    if kind == "seg":
        _, _, p0, p1, width, depth, colour = part
        return segment(p0, p1, width, depth, colour)
    if kind == "tube":
        return tube(part[2], part[3])
    if kind == "ell":
        _, _, radii, center, colour = part
        if max(radii) < 0.05:
            return ellipsoid(radii, center, colour, segs=6, rings=4)  # small parts (eyes, noses, rabbit) stay cheap
        return ellipsoid(radii, center, colour)
    _, _, size, loc, colour = part
    return common.make_box(size, loc=loc, color=colour)


def build_mesh(arm_obj, name, spec):
    """Skinned mesh parented to the armature. Returns the object and the number of parts."""
    parts = part_list(spec)
    bms, ranges, start = [], [], 0
    for part in parts:
        bm = _part_mesh(part)
        ranges.append((part[1], start, start + len(bm.verts)))
        start += len(bm.verts)
        bms.append(bm)
    combined = common.combine(bms)
    bmesh.ops.triangulate(combined, faces=list(combined.faces))  # keeps vertex order, so the groups hold
    mesh_obj = common.object_from_bm(combined, name)
    mesh_obj.parent = arm_obj
    for bone, v0, v1 in ranges:
        group = mesh_obj.vertex_groups.get(bone) or mesh_obj.vertex_groups.new(name=bone)
        group.add(list(range(v0, v1)), 1.0, "REPLACE")
    mod = mesh_obj.modifiers.new(name="Armature", type="ARMATURE")
    mod.object = arm_obj
    return mesh_obj, len(parts)


def triangle_count(mesh_obj):
    return len(mesh_obj.data.polygons)
