"""rubble_pile: a 3x3 m pile of broken concrete chunks and bent rebar (<= 1500 tris). Has a collider."""
import math
import os
import random
import sys

from mathutils import Vector

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rubble_pile", "props", 1500


def _slabs():
    """Large broken concrete slabs resting at various tilted angles in the central heap."""
    bms = []
    # (size, loc, rot, color)
    slab_defs = [
        # Central main slab: large flat slab tilted upward
        ((1.10, 0.85, 0.16), (0.10, 0.05, 0.42), (0.25, -0.35, 0.4), "rock_mid"),
        # Secondary slab propped under the first
        ((0.95, 0.75, 0.15), (-0.45, 0.30, 0.28), (-0.30, 0.20, -0.6), "rock_dark"),
        # Front tilted slab
        ((0.85, 0.65, 0.14), (0.35, -0.55, 0.22), (0.40, 0.15, 0.8), "rock_light"),
        # Rear tilted slab
        ((0.90, 0.70, 0.15), (-0.25, 0.75, 0.25), (-0.35, -0.25, 1.2), "rock_mid"),
        # Side leaning slab
        ((0.75, 0.55, 0.12), (0.80, 0.25, 0.20), (0.15, 0.50, -0.3), "rock_mid"),
        # Broken slab fragment sticking out of top
        ((0.60, 0.45, 0.12), (-0.10, -0.15, 0.62), (0.45, -0.40, 0.2), "rock_light"),
    ]
    for size, loc, rot, col in slab_defs:
        slab = common.make_box(size, loc=(0.0, 0.0, 0.0), color=col)
        common.transform(slab, rot=rot, loc=loc)
        bms.append(slab)

    return common.combine(bms)


def _chunks():
    """Medium and smaller broken concrete chunks and masonry rubble scattered around the pile."""
    bms = []
    rng = random.Random(42)

    # 14 medium angular chunks (size 0.25 - 0.45 m)
    for i in range(14):
        angle = rng.uniform(0, 2 * math.pi)
        dist = rng.uniform(0.4, 1.25)
        x = math.cos(angle) * dist
        y = math.sin(angle) * dist
        # Height follows mound contour (highest near center)
        z = max(0.08, (1.3 - dist) * 0.45 + rng.uniform(-0.05, 0.08))
        sx = rng.uniform(0.24, 0.42)
        sy = rng.uniform(0.22, 0.38)
        sz = rng.uniform(0.16, 0.30)
        col = rng.choice(["rock_mid", "rock_dark", "rock_light"])
        chunk = common.make_box((sx, sy, sz), loc=(0.0, 0.0, 0.0), color=col)
        rot = (rng.uniform(-0.6, 0.6), rng.uniform(-0.6, 0.6), rng.uniform(0, 2 * math.pi))
        common.transform(chunk, rot=rot, loc=(x, y, z))
        bms.append(chunk)

    # 22 small rubble rocks around perimeter and in crevices (size 0.12 - 0.22 m)
    for i in range(22):
        angle = rng.uniform(0, 2 * math.pi)
        dist = rng.uniform(0.6, 1.45)
        x = math.cos(angle) * dist
        y = math.sin(angle) * dist
        z = max(0.05, (1.5 - dist) * 0.25 + rng.uniform(-0.03, 0.05))
        s = rng.uniform(0.10, 0.22)
        col = rng.choice(["rock_mid", "rock_dark", "rock_light", "dirt"])
        chunk = common.make_box((s * rng.uniform(0.8, 1.2), s * rng.uniform(0.8, 1.2), s * rng.uniform(0.6, 1.0)),
                                loc=(0.0, 0.0, 0.0), color=col)
        rot = (rng.uniform(-0.8, 0.8), rng.uniform(-0.8, 0.8), rng.uniform(0, 2 * math.pi))
        common.transform(chunk, rot=rot, loc=(x, y, z))
        bms.append(chunk)

    # A few broken red bricks mixed into the concrete rubble
    brick_defs = [
        ((0.24, 0.12, 0.07), (0.45, 0.55, 0.12), (0.2, 0.4, 0.8)),
        ((0.22, 0.11, 0.07), (-0.65, -0.40, 0.10), (-0.3, 0.2, 1.5)),
        ((0.24, 0.12, 0.07), (0.15, -0.85, 0.08), (0.1, -0.3, -0.4)),
        ((0.20, 0.10, 0.06), (-0.35, 0.85, 0.14), (0.4, -0.2, 0.6)),
    ]
    for size, loc, rot in brick_defs:
        brick = common.make_box(size, loc=(0.0, 0.0, 0.0), color="red_paint")
        common.transform(brick, rot=rot, loc=loc)
        bms.append(brick)

    return common.combine(bms)


def _rebar():
    """Bent rusted steel rebar rods embedded in the concrete slabs and protruding out."""
    bms = []

    # Rebar 1: bent arched rod emerging from central slab and looping downward
    spine1 = [
        Vector((0.05, 0.0, 0.45)),
        Vector((0.12, -0.05, 0.68)),
        Vector((0.22, -0.15, 0.82)),
        Vector((0.38, -0.28, 0.74)),
        Vector((0.48, -0.40, 0.50)),
        Vector((0.55, -0.52, 0.28)),
    ]
    for i in range(len(spine1) - 1):
        p1 = spine1[i]
        p2 = spine1[i + 1]
        mid = (p1 + p2) / 2
        diff = p2 - p1
        length = diff.length
        rod = common.make_box((0.025, 0.025, length), loc=(0.0, 0.0, 0.0), color="rust")
        rot = (Vector((0.0, 0.0, 1.0)).cross(diff)).normalized()
        angle = math.acos(max(-1.0, min(1.0, diff.z / length)))
        matrix = common.Matrix.Rotation(angle, 4, rot) if rot.length > 1e-4 else common.Matrix.Identity(4)
        common.bmesh.ops.transform(rod, matrix=matrix, verts=list(rod.verts))
        common.transform(rod, loc=(mid.x, mid.y, mid.z))
        bms.append(rod)

    # Rebar 2: zigzag bent rod extending across the rear slab
    spine2 = [
        Vector((-0.35, 0.25, 0.32)),
        Vector((-0.45, 0.45, 0.52)),
        Vector((-0.30, 0.65, 0.60)),
        Vector((-0.15, 0.80, 0.42)),
        Vector((-0.05, 0.95, 0.20)),
    ]
    for i in range(len(spine2) - 1):
        p1 = spine2[i]
        p2 = spine2[i + 1]
        mid = (p1 + p2) / 2
        diff = p2 - p1
        length = diff.length
        rod = common.make_box((0.025, 0.025, length), loc=(0.0, 0.0, 0.0), color="rust")
        rot = (Vector((0.0, 0.0, 1.0)).cross(diff)).normalized()
        angle = math.acos(max(-1.0, min(1.0, diff.z / length)))
        matrix = common.Matrix.Rotation(angle, 4, rot) if rot.length > 1e-4 else common.Matrix.Identity(4)
        common.bmesh.ops.transform(rod, matrix=matrix, verts=list(rod.verts))
        common.transform(rod, loc=(mid.x, mid.y, mid.z))
        bms.append(rod)

    # Rebar 3: short jagged bent spike sticking out near front-right
    rod3a = common.make_box((0.025, 0.025, 0.28), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(rod3a, rot=(0.4, 0.6, 0.2), loc=(0.70, -0.25, 0.30))
    rod3b = common.make_box((0.025, 0.025, 0.22), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(rod3b, rot=(-0.3, 0.8, -0.4), loc=(0.82, -0.32, 0.42))
    bms.extend([rod3a, rod3b])

    return common.combine(bms)


def build(model):
    model.add(_slabs())
    model.add(_chunks())
    model.add(_rebar())
    # Convex collider covering the 3x3 rubble pile
    model.add_collider("rubble", common.make_box((2.85, 2.85, 0.78), loc=(0.0, 0.0, 0.39), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
