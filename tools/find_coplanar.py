#!/usr/bin/env python3
"""Find z-fighting in .glb models: triangles that face the same way, lie in the same plane
(within 5 mm) and overlap. Those flicker in the game (Blender previews don't show it).

Usage: python3 -I tools/find_coplanar.py assets/models/buildings/office.glb [more.glb ...]
Prints each problem (the plane and where) and exits 1 if any model has one.
"""
import json, struct, sys
from collections import defaultdict

import numpy as np

PLANE_TOL = 0.005  # metres: closer than this counts as the same plane
MIN_AREA = 1e-4    # m^2: overlaps smaller than this are ignored (shared edges)
COMP = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
NCOMP = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


def load_glb(path):
    data = open(path, "rb").read()
    _, _, _ = struct.unpack_from("<III", data, 0)
    off, doc, binc = 12, None, b""
    while off < len(data):
        ln, kind = struct.unpack_from("<II", data, off)
        chunk = data[off + 8: off + 8 + ln]
        if kind == 0x4E4F534A:
            doc = json.loads(chunk)
        elif kind == 0x004E4942:
            binc = chunk
        off += 8 + ln
    return doc, binc


def accessor(doc, binc, idx):
    a = doc["accessors"][idx]
    bv = doc["bufferViews"][a["bufferView"]]
    dt = np.dtype(COMP[a["componentType"]])
    n = NCOMP[a["type"]]
    start = bv.get("byteOffset", 0) + a.get("byteOffset", 0)
    stride = bv.get("byteStride", 0) or dt.itemsize * n
    raw = np.frombuffer(binc, dtype=np.uint8, count=stride * (a["count"] - 1) + dt.itemsize * n, offset=start)
    rows = [np.frombuffer(raw[i * stride: i * stride + dt.itemsize * n].tobytes(), dtype=dt) for i in range(a["count"])]
    return np.array(rows).reshape(a["count"], n)


def node_matrix(node):
    if "matrix" in node:
        return np.array(node["matrix"], dtype=float).reshape(4, 4).T
    t = np.array(node.get("translation", [0, 0, 0]), dtype=float)
    x, y, z, w = node.get("rotation", [0, 0, 0, 1])
    s = np.array(node.get("scale", [1, 1, 1]), dtype=float)
    r = np.array([
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ])
    m = np.eye(4)
    m[:3, :3] = r * s
    m[:3, 3] = t
    return m


def triangles(doc, binc):
    """World-space triangles of every drawn mesh, as (N, 3, 3), plus a mesh name per triangle."""
    tris, names = [], []

    def visit(ni, parent):
        node = doc["nodes"][ni]
        m = parent @ node_matrix(node)
        name = node.get("name", "")
        # Colliders and markers are not drawn
        low = name.lower()
        drawn = not (low.endswith(("-colonly", "-convcolonly")) or low.startswith(("lot_", "loot_", "spawn_")))
        if "mesh" in node and drawn:
            for prim in doc["meshes"][node["mesh"]]["primitives"]:
                if prim.get("mode", 4) != 4:
                    continue
                pos = accessor(doc, binc, prim["attributes"]["POSITION"]).astype(float)
                pos = (np.c_[pos, np.ones(len(pos))] @ m.T)[:, :3]
                idx = accessor(doc, binc, prim["indices"]).ravel() if "indices" in prim else np.arange(len(pos))
                t = pos[idx.reshape(-1, 3)]
                tris.append(t)
                names.extend([name] * len(t))
        for c in node.get("children", []):
            visit(c, m)

    for scene in doc.get("scenes", [{"nodes": list(range(len(doc["nodes"])))}]):
        for ni in scene["nodes"]:
            visit(ni, np.eye(4))
    return (np.concatenate(tris) if tris else np.zeros((0, 3, 3))), names


def clip(poly, a, b):
    """Keep the part of convex polygon `poly` (2D) on the left of edge a->b."""
    out = []
    side = lambda p: (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
    for i in range(len(poly)):
        p, q = poly[i], poly[(i + 1) % len(poly)]
        sp, sq = side(p), side(q)
        if sp >= 0:
            out.append(p)
        if (sp >= 0) != (sq >= 0):
            t = sp / (sp - sq)
            out.append((p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])))
    return out


def area(poly):
    return 0.5 * abs(sum(poly[i][0] * poly[(i + 1) % len(poly)][1] - poly[(i + 1) % len(poly)][0] * poly[i][1]
                         for i in range(len(poly))))


def overlap_area(t1, t2):
    def ccw(t):
        t = list(t)
        if (t[1][0] - t[0][0]) * (t[2][1] - t[0][1]) - (t[1][1] - t[0][1]) * (t[2][0] - t[0][0]) < 0:
            t[1], t[2] = t[2], t[1]
        return t
    poly, c = ccw(t1), ccw(t2)
    for i in range(3):
        poly = clip(poly, c[i], c[(i + 1) % 3])
        if len(poly) < 3:
            return 0.0
    return area(poly)


def check(path):
    doc, binc = load_glb(path)
    tris, names = triangles(doc, binc)
    if len(tris) == 0:
        return []
    n = np.cross(tris[:, 1] - tris[:, 0], tris[:, 2] - tris[:, 0])
    ln = np.linalg.norm(n, axis=1)
    ok = ln > 1e-9
    n[ok] /= ln[ok, None]
    d = np.einsum("ij,ij->i", n, tris[:, 0])
    groups = defaultdict(list)
    for i in np.nonzero(ok)[0]:
        key = tuple(np.round(n[i], 2)) + (round(d[i] / PLANE_TOL),)
        groups[key].append(i)
    problems = []
    seen = set()
    for key, idxs in groups.items():
        # neighbours in plane offset too, so 4.9 mm apart across a rounding edge still meet
        others = idxs + groups.get(key[:3] + (key[3] + 1,), [])
        if len(others) < 2:
            continue
        normal = np.array(key[:3])
        ax = np.argmax(np.abs(normal))
        keep = [k for k in range(3) if k != ax]
        flat = {i: tris[i][:, keep] for i in others}
        for i in idxs:
            for j in others:
                if j <= i or (i, j) in seen:
                    continue
                if abs(d[i] - d[j]) > PLANE_TOL or np.dot(n[i], n[j]) < 0.999:
                    continue
                bi, bj = flat[i], flat[j]
                if (bi.max(0) < bj.min(0)).any() or (bj.max(0) < bi.min(0)).any():
                    continue
                a = overlap_area([tuple(p) for p in bi], [tuple(p) for p in bj])
                if a > MIN_AREA:
                    seen.add((i, j))
                    c = tris[i].mean(0)
                    problems.append((a, names[i], names[j], n[i], c))
    return problems


def main():
    bad = 0
    for path in sys.argv[1:]:
        probs = check(path)
        if probs:
            bad += 1
            total = sum(p[0] for p in probs)
            print(f"{path}: {len(probs)} overlapping coplanar triangle pairs ({total:.2f} m2)")
            # one line per spot (rounded to 0.5 m) so the list stays short
            spots = {}
            for a, ni, nj, nrm, c in probs:
                k = (ni, nj, tuple(np.round(c * 2) / 2))
                spots[k] = spots.get(k, 0) + a
            for (ni, nj, c), a in sorted(spots.items(), key=lambda kv: -kv[1])[:15]:
                print(f"   {a:7.3f} m2  '{ni}' vs '{nj}' near x={c[0]:.1f} y={c[1]:.1f} z={c[2]:.1f}")
        else:
            print(f"{path}: OK")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
