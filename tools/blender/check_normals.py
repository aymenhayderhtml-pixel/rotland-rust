"""Check script for Rotland 3D models (.glb).

Checks for:
1. Open edges (boundary edges with only 1 adjacent face)
2. Non-manifold edges (> 2 adjacent faces)
3. Inward-facing / reversed normals (face normal opposite to vertex normals or inward-pointing)

Can be run via Blender or standalone python:
    blender -b --python tools/blender/check_normals.py -- assets/models/vehicles/car_hatchback_red.glb
    python3 tools/blender/check_normals.py assets/models/vehicles/car_hatchback_red.glb
"""
import json
import math
import os
import struct
import sys
from collections import defaultdict


def parse_glb(path):
    with open(path, "rb") as f:
        magic, ver, length = struct.unpack("<III", f.read(12))
        if magic != 0x46546C67:
            raise ValueError(f"Not a valid GLB file: magic={hex(magic)}")
        chunk_len, chunk_type = struct.unpack("<II", f.read(8))
        if chunk_type != 0x4E4F534A:
            raise ValueError(f"Expected JSON chunk, got {hex(chunk_type)}")
        gltf = json.loads(f.read(chunk_len))
        bin_chunk_len, bin_chunk_type = struct.unpack("<II", f.read(8))
        bin_data = f.read(bin_chunk_len)

    def get_buff(accessor_idx):
        acc = gltf["accessors"][accessor_idx]
        bv = gltf["bufferViews"][acc["bufferView"]]
        offset = bv.get("byteOffset", 0) + acc.get("byteOffset", 0)
        count = acc["count"]
        ctype = acc["componentType"]
        atype = acc["type"]
        if ctype == 5126:  # FLOAT
            n_comp = 3 if atype == "VEC3" else (2 if atype == "VEC2" else (4 if atype == "VEC4" else 1))
            fmt = f"<{count * n_comp}f"
            data = struct.unpack_from(fmt, bin_data, offset)
            return [data[i * n_comp:(i + 1) * n_comp] for i in range(count)]
        elif ctype == 5123:  # UNSIGNED_SHORT
            fmt = f"<{count}H"
            return list(struct.unpack_from(fmt, bin_data, offset))
        elif ctype == 5125:  # UNSIGNED_INT
            fmt = f"<{count}I"
            return list(struct.unpack_from(fmt, bin_data, offset))
        return []

    return gltf, get_buff


def check_mesh(mesh_name, primitives, get_buff):
    total_tris = 0
    total_verts = 0
    edge_faces = defaultdict(list)
    reversed_normals = 0

    for p in primitives:
        if "indices" not in p or "POSITION" not in p["attributes"]:
            continue
        indices = get_buff(p["indices"])
        pos = get_buff(p["attributes"]["POSITION"])
        norm = get_buff(p["attributes"]["NORMAL"]) if "NORMAL" in p["attributes"] else []
        n_tris = len(indices) // 3
        total_tris += n_tris
        total_verts += len(pos)

        for i in range(n_tris):
            i0, i1, i2 = indices[i * 3:i * 3 + 3]
            p0 = tuple(round(x, 4) for x in pos[i0])
            p1 = tuple(round(x, 4) for x in pos[i1])
            p2 = tuple(round(x, 4) for x in pos[i2])

            # Triangle edges
            for u, v in ((p0, p1), (p1, p2), (p2, p0)):
                edge = (min(u, v), max(u, v))
                edge_faces[edge].append((p0, p1, p2))

            # Face normal vs vertex normal check
            if norm:
                n0 = norm[i0]
                v0 = [p1[j] - p0[j] for j in range(3)]
                v1 = [p2[j] - p0[j] for j in range(3)]
                fn = [
                    v0[1] * v1[2] - v0[2] * v1[1],
                    v0[2] * v1[0] - v0[0] * v1[2],
                    v0[0] * v1[1] - v0[1] * v1[0],
                ]
                dot = sum(fn[j] * n0[j] for j in range(3))
                if dot < -1e-6:
                    reversed_normals += 1

    open_edges = [edge for edge, faces in edge_faces.items() if len(faces) == 1]
    non_manifold_edges = [edge for edge, faces in edge_faces.items() if len(faces) > 2]

    return {
        "name": mesh_name,
        "tris": total_tris,
        "verts": total_verts,
        "open_edges": len(open_edges),
        "non_manifold_edges": len(non_manifold_edges),
        "reversed_normals": reversed_normals,
    }


def check_glb_file(path):
    if not os.path.isfile(path):
        print(f"File not found: {path}")
        return False

    print(f"Checking model: {path}")
    gltf, get_buff = parse_glb(path)
    all_ok = True

    for m_idx, mesh in enumerate(gltf.get("meshes", [])):
        m_name = mesh.get("name", f"mesh_{m_idx}")
        res = check_mesh(m_name, mesh.get("primitives", []), get_buff)
        status_parts = []
        if res["open_edges"] > 0:
            status_parts.append(f"{res['open_edges']} open edges")
        if res["non_manifold_edges"] > 0:
            status_parts.append(f"{res['non_manifold_edges']} non-manifold edges")
        if res["reversed_normals"] > 0:
            status_parts.append(f"{res['reversed_normals']} reversed normal faces")
            all_ok = False

        status = ", ".join(status_parts) if status_parts else "manifold & normals consistent"
        print(f"  [{m_name}] {res['tris']} tris, {res['verts']} verts -> {status}")

    return all_ok


def main():
    args = sys.argv[1:]
    if "--" in args:
        args = args[args.index("--") + 1:]

    if not args:
        print("Usage: check_normals.py <path_to_glb> [<path_to_glb> ...]")
        return 1

    overall_ok = True
    for path in args:
        ok = check_glb_file(path)
        if not ok:
            overall_ok = False

    return 0 if overall_ok else 1


if __name__ == "__main__":
    sys.exit(main())
