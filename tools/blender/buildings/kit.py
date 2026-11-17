"""Shared parts for the POI buildings: boxes, walls with openings, roof planes and gable ends.

Walls are cut into cells around their openings (doors, windows). Cells that are not open get a
visual and a collider, so a doorway has no collider and the player can walk through it.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402

FLOOR_TOP = 0.1  # raised floors sit 0.1 m above the ground: a small step the player climbs
DOOR_H = 2.2  # door tops, measured from the ground
FRONT_VIEW = (1.0, 1.0, 0.7)  # preview camera: front right, above (+Y is the front)


def _box_bm(lo, hi, color):
    size = [hi[i] - lo[i] for i in range(3)]
    loc = [(lo[i] + hi[i]) / 2 for i in range(3)]
    return common.make_box(size, loc=loc, color=color)


def _plane_bm(a, b, span, width, thick, color):
    """Flat slab whose top surface lies on the line a-b (ridge to eave). `span` is the width direction."""
    a, b = Vector(a), Vector(b)
    d = b - a
    x_axis = d.normalized()
    y_axis = Vector(span).normalized()
    z_axis = x_axis.cross(y_axis).normalized()
    if z_axis.z < 0:
        z_axis = -z_axis  # keep the slab's thickness pointing down from the surface
    center = (a + b) / 2 - z_axis * (thick / 2)
    bm = common.make_box((d.length, width, thick), color=color)
    matrix = Matrix.Translation(center) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=matrix, verts=list(bm.verts))
    bm.normal_update()
    return bm


def _prism_bm(tri, offset, color):
    """Triangle `tri` (3 points) pushed by `offset`: a gable end wall or a roof gable."""
    bm = common.new_bm()
    a = [bm.verts.new(Vector(p)) for p in tri]
    b = [bm.verts.new(Vector(p) + Vector(offset)) for p in tri]
    bm.faces.new(a)
    bm.faces.new(list(reversed(b)))
    for i in range(3):
        j = (i + 1) % 3
        bm.faces.new([a[i], a[j], b[j], b[i]])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def _cuts(lo, hi, openings, axis, extra=()):
    values = {lo[axis], hi[axis]}
    for o_lo, o_hi in openings:
        for v in (o_lo[axis], o_hi[axis]):
            if lo[axis] < v < hi[axis]:
                values.add(v)
    for v in extra:
        if lo[axis] < v < hi[axis]:
            values.add(v)
    return sorted(values)


def _cells(xs, ys, zs):
    for i in range(len(xs) - 1):
        for j in range(len(ys) - 1):
            for k in range(len(zs) - 1):
                yield (xs[i], ys[j], zs[k]), (xs[i + 1], ys[j + 1], zs[k + 1])


def _is_open(lo, hi, openings):
    center = [(lo[i] + hi[i]) / 2 for i in range(3)]
    for o_lo, o_hi in openings:
        if all(o_lo[i] < center[i] < o_hi[i] for i in range(3)):
            return True
    return False


class Kit:
    """Adds parts to one Model. Collider labels are numbered so they stay unique."""

    def __init__(self, model):
        self.model = model
        self._count = 0

    def _label(self, base):
        self._count += 1
        return f"{base}_{self._count}"

    def box(self, lo, hi, color, roof=False):
        """Visual only (trim, boards, props that can be walked past)."""
        self.model.add(_box_bm(lo, hi, color), roof=roof)

    def collider(self, lo, hi, base="blk"):
        self.model.add_collider(self._label(base), _box_bm(lo, hi, "white"))

    def solid(self, lo, hi, color, base="blk"):
        """Visual box plus one collider box of the same size."""
        self.box(lo, hi, color)
        self.collider(lo, hi, base)

    def wall(self, lo, hi, openings=(), color="white", course=None, colors=None, base="wall"):
        """Wall box lo..hi with openings (boxes cutting through it, e.g. a door or window).

        Colliders: one per cell that is not open. Visual: the same cells, plus extra cuts every
        `course` metres when `colors` is given (stacked logs or boards in alternating colors).
        """
        xs = _cuts(lo, hi, openings, 0)
        ys = _cuts(lo, hi, openings, 1)
        zs = _cuts(lo, hi, openings, 2)
        for c_lo, c_hi in _cells(xs, ys, zs):
            if not _is_open(c_lo, c_hi, openings):
                self.collider(c_lo, c_hi, base)

        extra = []
        if course:
            steps = int((hi[2] - lo[2]) / course) + 1
            extra = [lo[2] + k * course for k in range(1, steps)]
        vzs = _cuts(lo, hi, openings, 2, extra=extra)
        for c_lo, c_hi in _cells(xs, ys, vzs):
            if _is_open(c_lo, c_hi, openings):
                continue
            paint = color
            if colors:
                row = int(((c_lo[2] + c_hi[2]) / 2 - lo[2]) // course)
                paint = colors[row % len(colors)]
            self.box(c_lo, c_hi, paint)

    def roof(self, ridge, eave, span, width, color, thick=0.12):
        """Sloped roof slab from the ridge point to one eave point, `width` wide along `span`."""
        self.model.add(_plane_bm(ridge, eave, span, width, thick, color), roof=True)
        self.model.add_collider(self._label("roof"), _plane_bm(ridge, eave, span, width, thick, "white"))

    def gable(self, tri, offset, color, roof=False):
        """Triangular wall or roof gable, visual and collider. `roof=True` hides it in the top view."""
        self.model.add(_prism_bm(tri, offset, color), roof=roof)
        self.model.add_collider(self._label("gable"), _prism_bm(tri, offset, "white"))

    def empty(self, label, loc, scale=(1.0, 1.0, 1.0), rot_z=0.0):
        self.model.add_empty(label, loc, scale, rot_z)

    def ladder(self, x, y, z_top, color="bark_light", half_w=0.3, rung_gap=0.3):
        """Visual-only ladder in the plane at y, rails at x +- half_w, from the ground to z_top.
        No collider: the climber code moves the player along it (markers ladder_bottom / ladder_top)."""
        t = 0.03
        self.box((x - half_w - t, y - t, 0.0), (x - half_w + t, y + t, z_top), color)
        self.box((x + half_w - t, y - t, 0.0), (x + half_w + t, y + t, z_top), color)
        z = 0.3
        while z < z_top - 0.2:
            self.box((x - half_w, y - t, z), (x + half_w, y + t, z + 0.05), color)
            z += rung_gap

    # ---- Visual plan step 7: detail parts. All visual only, so no doorway or path changes. ----

    def frame(self, opening, face, sign, color="white", t=0.08, depth=0.05, sill=True):
        """Trim round a door or window on the wall face at `face` (along the wall's thin axis),
        sticking out toward `sign` (+1 or -1). Windows (bottom above the floor) also get a sill."""
        lo, hi = opening
        axis = 0 if (hi[0] - lo[0]) < (hi[1] - lo[1]) else 1  # the opening's thin axis crosses the wall
        u = 1 - axis
        a, b = sorted((face, face + sign * depth))

        def strip(u0, u1, z0, z1, extra=0.0):
            p0, p1 = [0.0, 0.0, z0], [0.0, 0.0, z1]
            p0[u], p1[u] = u0, u1
            p0[axis], p1[axis] = (a - extra, b) if sign < 0 else (a, b + extra)
            self.box(tuple(p0), tuple(p1), color)

        window = lo[2] > FLOOR_TOP + 0.05
        z_bottom = lo[2] - (t if window else 0.0)
        strip(lo[u] - t, lo[u], z_bottom, hi[2] + t)
        strip(hi[u], hi[u] + t, z_bottom, hi[2] + t)
        strip(lo[u] - t, hi[u] + t, hi[2], hi[2] + t)
        if window and sill:
            strip(lo[u] - t * 1.5, hi[u] + t * 1.5, lo[2] - t, lo[2], extra=0.04)

    def boards(self, opening, face, sign, color="bark_light", rows=(0.2, 0.5, 0.8)):
        """Planks nailed across a window, a little proud of the trim (a boarded-up look)."""
        lo, hi = opening
        axis = 0 if (hi[0] - lo[0]) < (hi[1] - lo[1]) else 1
        u = 1 - axis
        a, b = sorted((face + sign * 0.06, face + sign * 0.10))
        for k, f in enumerate(rows):
            z = lo[2] + (hi[2] - lo[2]) * f
            shift = 0.06 * (1 if k % 2 else -1)
            p0, p1 = [0.0, 0.0, z - 0.06], [0.0, 0.0, z + 0.06]
            p0[u], p1[u] = lo[u] - 0.1 + shift, hi[u] + 0.1 + shift
            p0[axis], p1[axis] = a, b
            self.box(tuple(p0), tuple(p1), color)

    def ridge_cap(self, center, along, length, color="rock_mid", size=0.16):
        """A cap along the roof ridge (along = 0 for X, 1 for Y)."""
        lo, hi = [center[0] - size / 2, center[1] - size / 2, center[2] - 0.04], \
                 [center[0] + size / 2, center[1] + size / 2, center[2] + size * 0.6]
        lo[along], hi[along] = center[along] - length / 2, center[along] + length / 2
        self.box(tuple(lo), tuple(hi), color, roof=True)

    def chimney(self, x, y, z0, z1, color="red_paint"):
        self.box((x - 0.25, y - 0.25, z0), (x + 0.25, y + 0.25, z1), color, roof=True)
        self.box((x - 0.3, y - 0.3, z1), (x + 0.3, y + 0.3, z1 + 0.1), "rock_dark", roof=True)

    def posts(self, corners, z1, color="bark_dark", size=0.22):
        """Corner posts (log ends) standing a little proud of the wall corners."""
        for x, y in corners:
            self.box((x - size / 2, y - size / 2, 0.0), (x + size / 2, y + size / 2, z1), color)

    def crate(self, x, y, size=0.6, color="wood_crate", z=0.0):
        """A solid crate (visual and collider), for clutter beside walls."""
        h = size
        self.solid((x - size / 2, y - size / 2, z), (x + size / 2, y + size / 2, z + h), color, base="clutter")
        self.box((x - size / 2 - 0.01, y - 0.04, z + 0.05), (x + size / 2 + 0.01, y + 0.04, z + h - 0.05), "bark_dark")

    def barrel(self, x, y, color="red_paint", r=0.3, h=0.9):
        self.model.add(common.make_frustum(r, r, 0.0, h, segments=10, center=(x, y), color=color))
        for z in (0.15, h - 0.2):
            self.model.add(common.make_frustum(r + 0.02, r + 0.02, z, z + 0.05, segments=10, center=(x, y), color="rock_dark"))
        self.collider((x - r, y - r, 0.0), (x + r, y + r, h), "barrel")
