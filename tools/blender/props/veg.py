"""Shared builders for the vegetation models (visual plan step 4).

Pines are stacks of jagged tiers (long drooping tips between short notches) instead of smooth
cones. Leafy trees get a flared trunk with roots, limbs that reach into the crown, and a lumpy
crown made of many small leaf clusters instead of three big balls.
"""
import math
import random

import bmesh
from mathutils import Vector

import common


def tier(r, z0, height, color, seed, points=11, droop=0.35, inner=0.62, center=(0.0, 0.0)):
    """One jagged pine tier: a cone whose rim alternates drooping tips and short notches."""
    rng = random.Random(seed)
    bm = common.new_bm()
    cx, cy = center
    rot = rng.uniform(0.0, 2.0 * math.pi)
    apex = bm.verts.new((cx, cy, z0 + height))
    rim = []
    for i in range(points * 2):
        a = rot + math.pi * i / points
        if i % 2 == 0:
            rr, z = r * rng.uniform(0.9, 1.08), z0 - droop * rng.uniform(0.7, 1.2)
        else:
            rr, z = r * inner, z0 + 0.08 * height
        rim.append(bm.verts.new((cx + math.cos(a) * rr, cy + math.sin(a) * rr, z)))
    # The underside rises to a point inside the cone, so the tier reads as a hollow skirt
    under = bm.verts.new((cx, cy, z0 + 0.25 * height))
    n = len(rim)
    for i in range(n):
        bm.faces.new([rim[i], rim[(i + 1) % n], apex])
        bm.faces.new([rim[(i + 1) % n], rim[i], under])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def limb(start, end, r0, r1, color, segs=6):
    """Tapered branch from `start` to `end`."""
    s, e = Vector(start), Vector(end)
    d = e - s
    bm = common.make_frustum(r0, r1, 0.0, d.length, segments=segs, color=color)
    rot = Vector((0.0, 0.0, 1.0)).rotation_difference(d.normalized()).to_matrix()
    bmesh.ops.rotate(bm, verts=list(bm.verts), cent=(0.0, 0.0, 0.0), matrix=rot)
    bmesh.ops.translate(bm, verts=list(bm.verts), vec=s)
    return bm


def trunk(model, r0, r1, height, color, seed, roots=5, segs=8):
    """Trunk with a flared foot and root wedges spreading over the ground."""
    rng = random.Random(seed)
    model.add(common.make_frustum(r0, r1, 0.0, height, segments=segs, color=color))
    model.add(common.make_frustum(r0 * 1.35, r0, 0.0, min(0.45, height * 0.15), segments=segs, color=color))
    for i in range(roots):
        a = 2.0 * math.pi * i / roots + rng.uniform(-0.3, 0.3)
        length = r0 * rng.uniform(1.2, 2.0)
        d = Vector((math.cos(a), math.sin(a), 0.0))
        side = Vector((-d.y, d.x, 0.0))
        pts = [d * r0 * 0.7 + side * r0 * 0.35, d * r0 * 0.7 - side * r0 * 0.35,
               d * r0 * 0.8 + Vector((0, 0, r0 * 1.3)),
               d * (r0 + length) + Vector((0, 0, 0.02)), d * (r0 + length * 0.4) + Vector((0, 0, r0 * 0.5))]
        hull = common.new_bm()
        verts = [hull.verts.new(p) for p in pts]
        bmesh.ops.convex_hull(hull, input=verts)
        bmesh.ops.recalc_face_normals(hull, faces=list(hull.faces))
        common.paint(hull, color)
        model.add(hull)


def leafy_crown(model, seed, center, size, colors, count, blob_r, branch_from, bark, limbs=4, limb_r=0.11):
    """Limbs from the trunk top into the crown, then `count` leaf clusters around an inner core.

    colors: (low, mid, high) palette names; low clusters are darker, high ones lighter.
    """
    rng = random.Random(seed)
    c = Vector(center)
    sx, sy, sz = size
    for i in range(limbs):
        yaw = 2.0 * math.pi * i / limbs + rng.uniform(-0.4, 0.4)
        start = Vector((0.0, 0.0, branch_from * rng.uniform(0.85, 1.0)))
        end = c + Vector((math.cos(yaw) * sx * 0.65, math.sin(yaw) * sy * 0.65, -sz * rng.uniform(0.0, 0.3)))
        model.add(limb(start, end, limb_r, limb_r * 0.45, bark))
    model.add(common.make_icosphere(subdiv=2, jitter=0.2, seed=seed, radius=(sx * 0.72, sy * 0.72, sz * 0.7),
                                    loc=tuple(c), color=colors[1]))
    for k in range(count):
        # Even spread over the crown surface (golden-angle spiral), biased a little upward
        t = (k + 0.5) / count
        z = 1.0 - 1.6 * t
        ring = math.sqrt(max(0.0, 1.0 - z * z))
        a = k * 2.399963 + rng.uniform(-0.3, 0.3)
        d = Vector((math.cos(a) * ring, math.sin(a) * ring, z))
        pos = c + Vector((d.x * sx, d.y * sy, d.z * sz)) * rng.uniform(0.62, 0.8)
        color = colors[2] if d.z > 0.35 else colors[0] if d.z < -0.2 else colors[1]
        r = blob_r * rng.uniform(0.8, 1.15)
        model.add(common.make_icosphere(subdiv=2, jitter=0.22, seed=seed * 31 + k, radius=(r, r, r * 0.85),
                                        loc=tuple(pos), color=color))


def blade(x, y, height, lean, yaw, color, width=0.035):
    """One grass blade or frond: a thin three-sided cone, leaning out."""
    bm = common.make_frustum(width, 0.0, 0.0, height, segments=3, color=color)
    common.transform(bm, rot=(lean, 0.0, yaw), loc=(x, y, 0.0))
    return bm
