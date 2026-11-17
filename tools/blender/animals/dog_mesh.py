"""Dog body mesh (V11-08, reshaped via AI Studio 2026-11-14): stylized low-poly parts, each rigidly skinned to one bone.

Uses tapered frustums, wedges, and stretched low-poly icospheres instead of plain boxes so
the silhouette reads as a friendly shepherd-lab dog rather than a blocky bear cub.
Every vertex belongs 100% to one bone via its part's vertex range in the combined mesh.
"""
import math
import bmesh
import common  # noqa: E402


def _span(p0, p1, r0, r1, color, segs=8, scale_xy=(1.0, 1.0), rot_z=0.0):
    """Tapered frustum running from 3D point p0 (radius r0) to p1 (radius r1)."""
    dx, dy, dz = p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]
    length = math.sqrt(dx * dx + dy * dy + dz * dz)
    mid = ((p0[0] + p1[0]) * 0.5, (p0[1] + p1[1]) * 0.5, (p0[2] + p1[2]) * 0.5)
    # Keep local X horizontal for centerline/leg spans; tilt in X then Y for lateral spans (ears)
    rx = math.atan2(-dy, dz) if abs(dx) < 1e-6 else -math.asin(max(-1.0, min(1.0, dy / length)))
    ry = 0.0 if abs(dx) < 1e-6 else math.atan2(dx, dz)
    bm = common.make_frustum(
        r0, r1, -0.5 * length, 0.5 * length, segments=segs, rot_z=rot_z, color=color
    )
    return common.transform(bm, loc=mid, rot=(rx, ry, 0.0), scale=(scale_xy[0], scale_xy[1], 1.0))


def _paw(x, y, color):
    """Compact, rounded wedge paw sitting flat on the ground at z=0."""
    bm = common.make_frustum(0.034, 0.026, 0.0, 0.055, segments=6, rot_z=math.pi / 6, color=color)
    return common.transform(bm, loc=(x, y + 0.032, 0.0), rot=(0.0, 0.0, 0.0), scale=(0.92, 1.35, 1.0))


def _parts(coat, accent):
    """(bone_name, bmesh) for every body part in rest-pose coordinates (metres)."""
    parts = [
        # Hips & pelvis: rounded rump cap and trim pelvis that tuck up above the flank
        ("hips", _span((0.0, -0.28, 0.492), (0.0, -0.22, 0.492), 0.045, 0.080, coat, 8, (1.05, 1.0))),
        ("hips", _span((0.0, -0.23, 0.492), (0.0, -0.085, 0.492), 0.080, 0.088, coat, 8, (1.05, 1.0))),
        # Spine: level backline with a tucked-in belly sloping down toward the chest, plus saddle
        ("spine", _span((0.0, -0.105, 0.492), (0.0, 0.095, 0.472), 0.088, 0.105, coat, 8, (1.0, 1.0))),
        ("spine", _span((0.0, -0.17, 0.530), (0.0, 0.09, 0.530), 0.060, 0.066, accent, 8, (1.35, 1.0))),
        # Chest: deep brisket, slim ribcage flush with the shoulders, and pale cream bib
        ("chest", _span((0.0, 0.05, 0.472), (0.0, 0.225, 0.480), 0.105, 0.092, coat, 8, (1.0, 1.05))),
        ("chest", common.make_icosphere(subdiv=1, radius=(0.098, 0.090, 0.105), loc=(0.0, 0.185, 0.470), color=coat)),
        ("chest", _span((0.0, 0.215, 0.385), (0.0, 0.282, 0.490), 0.040, 0.052, "dog_cream", 6, (1.12, 0.58))),
        # Neck: tapered cylinder angled into the skull, wrapped with a leather collar and brass tag
        ("neck", _span((0.0, 0.180, 0.468), (0.0, 0.315, 0.620), 0.088, 0.066, coat, 8, (0.95, 1.04))),
        ("neck", _span((0.0, 0.258, 0.548), (0.0, 0.278, 0.572), 0.076, 0.072, "leather", 8, (0.94, 1.06))),
        ("neck", common.make_box((0.018, 0.014, 0.022), loc=(0.0, 0.332, 0.515), color="brass")),
        # Head: rounded icosphere cranium, long tapered snout, pale muzzle tip, and dark nose
        ("head", common.make_icosphere(subdiv=1, radius=(0.084, 0.080, 0.074), loc=(0.0, 0.330, 0.648), color=coat)),
        ("head", _span((0.0, 0.365, 0.622), (0.0, 0.485, 0.612), 0.042, 0.027, coat, 8, (0.96, 0.78), math.pi / 8)),
        ("head", _span((0.0, 0.478, 0.612), (0.0, 0.540, 0.608), 0.027, 0.018, "dog_cream", 8, (0.96, 0.78), math.pi / 8)),
        ("head", _span((0.0, 0.534, 0.614), (0.0, 0.558, 0.616), 0.014, 0.007, "tire_black", 6, (1.20, 0.85))),
        # Jaw: tapered lower mandible with pale chin tip and tiny tongue peeking out
        ("jaw", _span((0.0, 0.345, 0.584), (0.0, 0.470, 0.578), 0.032, 0.020, coat, 6, (0.88, 0.72))),
        ("jaw", _span((0.0, 0.465, 0.578), (0.0, 0.515, 0.578), 0.020, 0.013, "dog_cream", 6, (0.88, 0.70))),
        ("jaw", common.make_box((0.022, 0.036, 0.008), loc=(0.0, 0.502, 0.590), color="blood_dark")),
        # Tail: three angled segments across tail_1 and tail_2 that curve visibly upward
        ("tail_1", _span((0.0, -0.23, 0.512), (0.0, -0.345, 0.602), 0.038, 0.030, coat, 6)),
        ("tail_2", _span((0.0, -0.34, 0.598), (0.0, -0.415, 0.665), 0.030, 0.022, coat, 6)),
        ("tail_2", _span((0.0, -0.41, 0.660), (0.0, -0.460, 0.730), 0.022, 0.008, accent, 6)),
    ]
    # Dark eyes, cream eyebrow spots, and pointed triangular wedge ears with pale inner detail
    for side, s in (("L", -1.0), ("R", 1.0)):
        parts.append(("head", common.make_box((0.016, 0.024, 0.022), loc=(s * 0.058, 0.385, 0.656), color="tire_black")))
        parts.append(("head", common.make_box((0.016, 0.018, 0.009), loc=(s * 0.050, 0.390, 0.682), color="dog_cream")))
        parts.append((f"ear_{side}", _span((s * 0.048, 0.30, 0.665), (s * 0.092, 0.28, 0.770), 0.040, 0.010, accent, 4, (1.0, 0.45), math.pi / 4)))
        parts.append((f"ear_{side}", _span((s * 0.050, 0.31, 0.672), (s * 0.086, 0.290, 0.755), 0.024, 0.006, "dog_cream", 4, (1.0, 0.35), math.pi / 4)))
    # Slim, tapered canine legs (muscular hind thighs, slender shins) and neat dark paws
    for leg, x, y, top_y, r_top, sy in (
        ("FL", -0.09, 0.16, 0.16, 0.050, 1.08),
        ("FR", 0.09, 0.16, 0.16, 0.050, 1.08),
        ("BL", -0.09, -0.22, -0.20, 0.056, 1.30),
        ("BR", 0.09, -0.22, -0.20, 0.056, 1.30),
    ):
        parts.append((f"upper_{leg}", _span((x, top_y, 0.495), (x, y, 0.268), r_top, 0.034, coat, 6, (0.95, sy))))
        parts.append((f"lower_{leg}", _span((x, y, 0.274), (x, y + 0.01, 0.055), 0.032, 0.027, coat, 6, (1.0, 1.05))))
        parts.append((f"paw_{leg}", _paw(x, y, accent)))
    return parts


def build_mesh(arm_obj, name, coat, accent):
    """Skinned dog mesh parented to the armature. Returns the mesh object and the part count."""
    parts = _parts(coat, accent)
    ranges = []
    start = 0
    for bone, bm in parts:
        count = len(bm.verts)
        ranges.append((bone, start, start + count))
        start += count
    combined = common.combine([bm for _, bm in parts])
    bmesh.ops.triangulate(combined, faces=list(combined.faces))  # keeps vertex order for rigid groups
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


def part_bones():
    """Bone names of the coat-independent parts, for the self-test contract."""
    parts = _parts("dog_tan", "dog_dark")
    bones = sorted({bone for bone, _ in parts})
    for _, bm in parts:
        bm.free()
    return bones
