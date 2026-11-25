"""radio_tower: 25 m lattice radio mast with red/white top and enterable equipment hut at the base (<= 4000 tris).

Mast is centered at (0.0, -1.3) and is not enterable (base legs have colliders).
Equipment hut sits at the front (+Y) at (0.0, 1.8) and is walk-in with a door on +Y.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "radio_tower", "buildings", 4000
F = kit.FLOOR_TOP
MAST_Y = -1.3
MAST_H = 25.0


def _strut(model, a, b, thick, color):
    """Thin box strut between 3D points `a` and `b`."""
    va, vb = Vector(a), Vector(b)
    d = vb - va
    z_axis = d.normalized()
    up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    bm = common.make_box((thick, thick, d.length), color=color)
    matrix = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=matrix, verts=list(bm.verts))
    bm.normal_update()
    model.add(bm)


def _corner(half, z):
    return [(-half, MAST_Y - half, z), (half, MAST_Y - half, z),
            (half, MAST_Y + half, z), (-half, MAST_Y + half, z)]


def build(model):
    k = kit.Kit(model)

    # ---- 25 m lattice mast (5 bays of 5 m, narrowing from 3.0 m base to 0.7 m top) ----
    bays = 5
    base_half = 1.50
    top_half = 0.35
    for i in range(bays):
        t0, t1 = i / bays, (i + 1) / bays
        z0, z1 = t0 * MAST_H, t1 * MAST_H
        h0 = base_half + (top_half - base_half) * t0
        h1 = base_half + (top_half - base_half) * t1
        c0 = _corner(h0, z0)
        c1 = _corner(h1, z1)
        # Red/white top section (top two bays alternate white/red), lower bays metal
        if i == 3:
            col = "white"
        elif i == 4:
            col = "red_paint"
        else:
            col = "metal"
        brace_col = col if i >= 3 else "rock_light"

        for leg in range(4):
            _strut(model, c0[leg], c1[leg], 0.14 if i < 2 else 0.10, col)
            nxt = (leg + 1) % 4
            _strut(model, c1[leg], c1[nxt], 0.08, col)
            _strut(model, c0[leg], c1[nxt], 0.06, brace_col)
            _strut(model, c0[nxt], c1[leg], 0.06, brace_col)

    # Top platform, antenna spire (25..27.5 m), and red beacon tip
    k.box((-0.42, MAST_Y - 0.42, MAST_H), (0.42, MAST_Y + 0.42, MAST_H + 0.08), "metal")
    k.box((-0.05, MAST_Y - 0.05, MAST_H + 0.08), (0.05, MAST_Y + 0.05, MAST_H + 2.4), "white")
    k.box((-0.09, MAST_Y - 0.09, MAST_H + 2.4), (0.09, MAST_Y + 0.09, MAST_H + 2.65), "red_paint")

    # Concrete pads and colliders for the 4 mast legs at ground level
    for cx, cy, _ in _corner(base_half, 0.0):
        k.solid((cx - 0.30, cy - 0.30, 0.0), (cx + 0.30, cy + 0.30, 0.35), "rock_mid", base="pad")
        k.collider((cx - 0.15, cy - 0.15, 0.35), (cx + 0.15, cy + 0.15, 3.0), base="mast_leg")

    # ---- Walk-in equipment hut at the front base (X: -1.8..1.8, Y: 0.4..3.2) ----
    TOP = 2.5
    k.solid((-1.8, 0.4, 0.0), (1.8, 3.2, F), "rock_mid", base="hut_floor")
    door = ((-0.55, 3.0, F), (0.55, 3.3, kit.DOOR_H))
    win_r = ((1.6, 1.3, 1.0), (1.95, 2.3, 1.8))
    k.wall((-1.8, 3.05, F), (1.8, 3.2, TOP), openings=[door], color="metal", base="hut_front")
    k.wall((-1.8, 0.4, F), (1.8, 0.55, TOP), color="metal", base="hut_back")
    k.wall((-1.8, 0.55, F), (-1.65, 3.05, TOP), color="metal", base="hut_left")
    k.wall((1.65, 0.55, F), (1.8, 3.05, TOP), openings=[win_r], color="metal", base="hut_right")

    # Corrugated metal roof
    k.solid((-1.95, 0.25, TOP), (1.95, 3.35, TOP + 0.14), "rock_dark", base="hut_roof")
    k.frame(door, 3.22, 1, "rock_dark", sill=False)
    k.frame(win_r, 1.82, 1, "rock_dark")

    # Interior: radio desk + server rack + crate
    k.solid((-1.6, 0.6, F), (0.2, 1.2, 0.85), "bark_dark", base="desk")
    k.box((-1.3, 0.65, 0.85), (-0.4, 1.05, 1.15), "gun_black")
    k.solid((0.6, 0.6, F), (1.5, 1.2, 1.9), "gun_black", base="rack")
    k.box((0.7, 1.21, 1.4), (0.85, 1.23, 1.5), "red_paint")
    k.crate(-1.2, 2.2, 0.55, z=F)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(6.0, 7.0, 1.0))
    k.empty("door_1", (0.0, 3.125, F))
    k.empty("loot_1", (-0.8, 0.9, 0.85))  # radio desk
    k.empty("loot_2", (-1.2, 2.2, F + 0.55))  # crate
    k.empty("spawn_1", (0.2, 2.0, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
