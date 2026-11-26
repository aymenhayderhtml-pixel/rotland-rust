"""70s concrete slab tower kit (24 m and 32 m): base, floor, floor_b, crown + LOD1."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER_DIR = os.path.dirname(HERE)
for p in (HERE, BLENDER_DIR, os.path.join(BLENDER_DIR, "buildings"), os.path.join(BLENDER_DIR, "metro")):
    if p not in sys.path:
        sys.path.insert(0, p)

import common  # noqa: E402
import kit  # noqa: E402
import roof_parts  # noqa: E402
import shell  # noqa: E402

CATEGORY = "buildings"
FLOOR_H = 3.8  # new: DECISIONS 2026-11-26
BASE_H = 7.6   # new: DECISIONS 2026-11-26
CROWN_H = 3.0  # new: DECISIONS 2026-11-26
SLAB_T = 0.22
BAY_W = 4.0

BUDGET_FLOOR = {24: 4500, 32: 6000}
BUDGET_BASE = 15000
BUDGET_CROWN = 8000


def _satellite_dish(k, x, y, z0, yaw=0.4):
    """White dish on a metal bracket."""
    k.box((x - 0.04, y - 0.04, z0), (x + 0.04, y + 0.04, z0 + 0.35), "metal")
    k.box((x - 0.03, y - 0.03, z0 + 0.32), (x + 0.03, y + 0.14, z0 + 0.38), "metal")
    dish = common.make_frustum(0.32, 0.05, 0.0, 0.14, segments=8, color="white")
    common.transform(dish, rot=(1.05, 0.0, yaw), loc=(x, y + 0.12, z0 + 0.42))
    k.model.add(dish)
    k.box((x - 0.02, y + 0.22, z0 + 0.40), (x + 0.02, y + 0.28, z0 + 0.46), "metal")


def _balcony(k, cx, fy, z0, has_tarp=True, has_bike=True, has_dish=False):
    """2.4 m wide x 1.2 m deep balcony on +Y with metal rail and junk."""
    bw, bd = 2.4, 1.2
    x0, x1 = cx - bw / 2.0, cx + bw / 2.0
    y0, y1 = fy, fy + bd
    k.box((x0, y0, z0), (x1, y1, z0 + 0.16), "concrete_grey")
    # Stain streak under balcony
    k.box((cx - 0.55, fy - 0.01, z0 - 1.1), (cx - 0.35, fy + 0.04, z0), "concrete_stain")
    k.box((cx + 0.40, fy - 0.01, z0 - 0.8), (cx + 0.56, fy + 0.04, z0), "concrete_stain")

    # Metal rails
    rz = z0 + 0.16
    for px in (x0 + 0.04, cx, x1 - 0.04):
        k.box((px - 0.03, y1 - 0.06, rz), (px + 0.03, y1, rz + 0.95), "metal")
    for py in (y0 + 0.25, y1 - 0.03):
        k.box((x0 + 0.01, py - 0.03, rz), (x0 + 0.07, py + 0.03, rz + 0.95), "metal")
        k.box((x1 - 0.07, py - 0.03, rz), (x1 - 0.01, py + 0.03, rz + 0.95), "metal")
    k.box((x0, y1 - 0.06, rz + 0.90), (x1, y1, rz + 0.96), "metal")
    k.box((x0, y1 - 0.05, rz + 0.45), (x1, y1 - 0.01, rz + 0.49), "metal")
    k.box((x0, y0, rz + 0.90), (x0 + 0.06, y1, rz + 0.96), "metal")
    k.box((x1 - 0.06, y0, rz + 0.90), (x1, y1, rz + 0.96), "metal")

    # Junk: box, bag, dead potted plant, optional bicycle, tarp, satellite dish
    k.box((x0 + 0.15, y0 + 0.15, rz), (x0 + 0.65, y0 + 0.55, rz + 0.38), "wood_crate")
    k.box((x0 + 0.72, y0 + 0.20, rz), (x0 + 1.05, y0 + 0.52, rz + 0.26), "gun_black")
    pot = common.make_frustum(0.14, 0.18, rz, rz + 0.32, segments=8, center=(x1 - 0.35, y0 + 0.35), color="rock_mid")
    k.model.add(pot)
    k.box((x1 - 0.38, y0 + 0.32, rz + 0.30), (x1 - 0.32, y0 + 0.38, rz + 0.68), "dry_brush")
    k.box((x1 - 0.46, y0 + 0.30, rz + 0.48), (x1 - 0.24, y0 + 0.42, rz + 0.56), "dry_brush")

    if has_bike:
        for wx in (cx - 0.35, cx + 0.35):
            wheel = common.make_frustum(0.24, 0.24, -0.03, 0.03, segments=10, color="gun_black")
            common.transform(wheel, rot=(1.5708, 0.0, 0.0), loc=(wx, y0 + 0.75, rz + 0.24))
            k.model.add(wheel)
        k.box((cx - 0.36, y0 + 0.72, rz + 0.24), (cx + 0.36, y0 + 0.76, rz + 0.29), "metal")
        k.box((cx - 0.12, y0 + 0.72, rz + 0.24), (cx - 0.06, y0 + 0.76, rz + 0.62), "metal")
        k.box((cx + 0.28, y0 + 0.62, rz + 0.58), (cx + 0.36, y0 + 0.88, rz + 0.63), "metal")

    if has_tarp:
        k.box((x0 + 0.10, y1 - 0.12, rz + 0.60), (x0 + 1.05, y1 + 0.03, rz + 0.98), "tarp_blue")
        k.box((x0 + 0.15, y1 - 0.02, rz + 0.20), (x0 + 0.95, y1 + 0.04, rz + 0.65), "tarp_blue")

    if has_dish:
        _satellite_dish(k, x1 - 0.32, y1 - 0.32, rz + 0.50)


def _slab_window(k, size, face, u, z0, z1, state, stain=True):
    """Concrete framed strip window with optional boards, soot, and stain streak."""
    half = size / 2.0
    along_x = face in ("+Y", "-Y")
    sign = 1.0 if face in ("+Y", "+X") else -1.0

    def pbox(u0, u1, d0, d1, zb, zt, color):
        if along_x:
            lo = (u0, sign * half + min(d0, d1) * sign, zb)
            hi = (u1, sign * half + max(d0, d1) * sign, zt)
        else:
            lo = (sign * half + min(d0, d1) * sign, u0, zb)
            hi = (sign * half + max(d0, d1) * sign, u1, zt)
        k.box((min(lo[0], hi[0]), min(lo[1], hi[1]), lo[2]),
              (max(lo[0], hi[0]), max(lo[1], hi[1]), hi[2]), color)

    hw = 1.35
    pbox(u - hw - 0.08, u + hw + 0.08, -0.02, 0.06, z0 - 0.08, z0, "concrete_grey")
    pbox(u - hw - 0.08, u + hw + 0.08, -0.02, 0.06, z1, z1 + 0.08, "concrete_grey")
    pbox(u - hw - 0.08, u - hw, -0.02, 0.05, z0, z1, "concrete_grey")
    pbox(u + hw, u + hw + 0.08, -0.02, 0.05, z0, z1, "concrete_grey")
    pane_color = "soot" if state == "soot" else "glass_tint"
    pbox(u - hw, u + hw, -0.02, 0.025, z0, z1, pane_color)
    if state == "boarded":
        if along_x:
            w_op = ((u - hw, sign * half - 0.04, z0), (u + hw, sign * half + 0.04, z1))
        else:
            w_op = ((sign * half - 0.04, u - hw, z0), (sign * half + 0.04, u + hw, z1))
        k.boards(w_op, sign * half, int(sign), color="bark_light", rows=(0.25, 0.55, 0.8))
    if stain:
        pbox(u - 0.45, u + 0.45, -0.01, 0.04, max(0.05, z0 - 0.95), z0 - 0.08, "concrete_stain")


def _floor_windows(k, size, z0, z1, variant="a", skip_center_front=False):
    bays = size // 4
    half = size / 2.0
    for fi, face in enumerate(("+Y", "+X", "-X", "-Y")):
        for i in range(bays):
            u = -half + (i + 0.5) * BAY_W
            if skip_center_front and face == "+Y" and -2.2 < u < 2.2:
                continue
            h = (fi * 19 + i * 7 + (5 if variant == "b" else 0)) % 9
            state = "soot" if h == 1 else ("boarded" if h in (3, 7) else "glass")
            _slab_window(k, size, face, u, z0, z1, state, stain=(face != "-Y" or i % 2 == 0))


def _build_floor(size, variant):
    name = f"tw_slab_{size}_floor" + ("_b" if variant == "b" else "")

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half, -half, SLAB_T), (half, half, FLOOR_H - SLAB_T), "concrete_grey")
        k.box((-half - 0.03, -half - 0.03, 0.0), (half + 0.03, half + 0.03, SLAB_T), "concrete_stain")
        k.box((-half - 0.03, -half - 0.03, FLOOR_H - SLAB_T), (half + 0.03, half + 0.03, FLOOR_H), "concrete_stain")
        _floor_windows(k, size, 1.15, 2.75, variant=variant)

        bays = size // 4
        balc_bays = (1, 4) if (size == 24 and variant == "a") else (
            (2,) if size == 24 else ((1, 4, 6) if variant == "a" else (2, 5))
        )
        for bi in balc_bays:
            if bi < bays:
                cx = -half + (bi + 0.5) * BAY_W
                _balcony(k, cx, half, 0.12, has_tarp=(variant == "a" or bi % 2 == 0),
                         has_bike=(bi % 2 == 1), has_dish=(bi == balc_bays[-1]))

    return common.run_model(name, CATEGORY, BUDGET_FLOOR[size], fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_floor_lod1(size, variant):
    name = f"tw_slab_{size}_floor" + ("_b" if variant == "b" else "") + "_lod1"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half, -half, 0.0), (half, half, FLOOR_H), "concrete_grey")
        k.box((-half - 0.02, -half - 0.02, 1.2), (half + 0.02, half + 0.02, 2.7), "glass_tint")
        balc_bays = (1, 4) if (size == 24 and variant == "a") else (
            (2,) if size == 24 else ((1, 4, 6) if variant == "a" else (2, 5))
        )
        for bi in balc_bays:
            cx = -half + (bi + 0.5) * BAY_W
            k.box((cx - 1.2, half, 0.12), (cx + 1.2, half + 1.2, 0.30), "concrete_grey")

    return common.run_model(name, CATEGORY, BUDGET_FLOOR[size] // 10, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_base(size):
    name = f"tw_slab_{size}_base"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        fy = shell.front_face_y(size)
        shell.lobby_shell(k, size, "concrete_grey")

        # Slab bands at mid-floor (3.8 m) and top (7.6 m), roof=True so top_preview shows open hall
        k.box((-half - 0.03, -half - 0.03, 3.65), (half + 0.03, half + 0.03, 3.95), "concrete_stain", roof=True)
        k.box((-half - 0.03, -half - 0.03, BASE_H - SLAB_T), (half + 0.03, half + 0.03, BASE_H - 0.01), "concrete_stain", roof=True)

        # Ground floor & 2nd floor windows
        _floor_windows(k, size, 1.0, 2.7, variant="a", skip_center_front=True)
        _floor_windows(k, size, 4.95, 6.55, variant="b", skip_center_front=False)

        # Rough portico outside the 2.8 m door gap (1.15 m overhang <= 1.2 m)
        k.box((-2.1, fy, 3.18), (2.1, fy + 1.15, 3.48), "concrete_grey")
        k.box((-2.14, fy, 3.42), (2.14, fy + 1.18, 3.52), "concrete_stain")
        for px in (-1.75, 1.75):
            k.box((px - 0.18, fy + 0.78, 0.0), (px + 0.18, fy + 1.12, 3.18), "concrete_grey")

    return common.run_model(name, CATEGORY, BUDGET_BASE, fn, grip=(0.0, 0.0, 0.0),
                            top_preview=True, view=kit.FRONT_VIEW)


def _build_base_lod1(size):
    name = f"tw_slab_{size}_base_lod1"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half, -half, 0.0), (half, half, BASE_H), "concrete_grey")
        k.box((-1.4, half - 0.05, 0.1), (1.4, half + 0.06, 3.1), "soot")
        k.box((-2.1, half, 3.18), (2.1, half + 1.15, 3.52), "concrete_grey")

    return common.run_model(name, CATEGORY, BUDGET_BASE // 10, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_crown(size):
    name = f"tw_slab_{size}_crown"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half - 0.03, -half - 0.03, 0.0), (half + 0.03, half + 0.03, 0.30), "concrete_stain")
        # Low parapet
        for lo, hi in (
            ((-half, half - 0.25, 0.30), (half, half, 0.60)),
            ((-half, -half, 0.30), (half, -half + 0.25, 0.60)),
            ((-half, -half + 0.25, 0.30), (-half + 0.25, half - 0.25, 0.60)),
            ((half - 0.25, -half + 0.25, 0.30), (half, half - 0.25, 0.60)),
        ):
            k.box(lo, hi, "concrete_grey")

        # Perimeter metal rail (z 0.60..1.10)
        r_inset = half - 0.12
        for z_bar in (0.85, 1.08):
            k.box((-r_inset, r_inset - 0.03, z_bar), (r_inset, r_inset + 0.03, z_bar + 0.04), "metal")
            k.box((-r_inset, -r_inset - 0.03, z_bar), (r_inset, -r_inset + 0.03, z_bar + 0.04), "metal")
            k.box((-r_inset - 0.03, -r_inset, z_bar), (-r_inset + 0.03, r_inset, z_bar + 0.04), "metal")
            k.box((r_inset - 0.03, -r_inset, z_bar), (r_inset + 0.03, r_inset, z_bar + 0.04), "metal")
        steps = 6 if size == 24 else 8
        for i in range(steps + 1):
            u = -r_inset + i * (2.0 * r_inset / steps)
            k.box((u - 0.03, r_inset - 0.03, 0.60), (u + 0.03, r_inset + 0.03, 1.10), "metal")
            k.box((u - 0.03, -r_inset - 0.03, 0.60), (u + 0.03, -r_inset + 0.03, 1.10), "metal")
            k.box((-r_inset - 0.03, u - 0.03, 0.60), (-r_inset + 0.03, u + 0.03, 1.10), "metal")
            k.box((r_inset - 0.03, u - 0.03, 0.60), (r_inset + 0.03, u + 0.03, 1.10), "metal")

        # Lift motor room (concrete_grey, one door, one dead window, z 0.30..CROWN_H)
        k.box((-2.5, -2.0, 0.30), (2.5, 2.0, CROWN_H), "concrete_grey")
        k.box((-2.6, -2.1, CROWN_H - 0.15), (2.6, 2.1, CROWN_H), "concrete_stain")
        k.box((-0.6, 1.98, 0.35), (0.6, 2.05, 2.35), "metal")
        k.box((1.0, 1.98, 1.1), (2.1, 2.04, 2.1), "glass_dark")
        k.box((1.1, 1.99, 0.45), (2.0, 2.05, 1.1), "concrete_stain")

        # Shared roof_parts + two satellite dishes
        roof_parts.vent_stack(k, -half + 2.2, half - 2.2, 0.30, 1.4)
        roof_parts.vent_stack(k, half - 2.2, -half + 2.2, 0.30, 1.4)
        roof_parts.ac_unit(k, -half + 3.0, -half + 3.0, 0.30)
        _satellite_dish(k, half - 1.5, half - 1.5, 0.60, yaw=0.3)
        _satellite_dish(k, -half + 1.5, half - 1.5, 0.60, yaw=-0.4)

    return common.run_model(name, CATEGORY, BUDGET_CROWN, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_crown_lod1(size):
    name = f"tw_slab_{size}_crown_lod1"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half, -half, 0.0), (half, half, 0.6), "concrete_grey")
        k.box((-2.5, -2.0, 0.6), (2.5, 2.0, CROWN_H), "concrete_grey")

    return common.run_model(name, CATEGORY, BUDGET_CROWN // 10, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def build_all():
    results = []
    for size in (24, 32):
        results.append((f"tw_slab_{size}_base", _build_base(size)))
        results.append((f"tw_slab_{size}_base_lod1", _build_base_lod1(size)))
        results.append((f"tw_slab_{size}_floor", _build_floor(size, "a")))
        results.append((f"tw_slab_{size}_floor_lod1", _build_floor_lod1(size, "a")))
        results.append((f"tw_slab_{size}_floor_b", _build_floor(size, "b")))
        results.append((f"tw_slab_{size}_floor_b_lod1", _build_floor_lod1(size, "b")))
        results.append((f"tw_slab_{size}_crown", _build_crown(size)))
        results.append((f"tw_slab_{size}_crown_lod1", _build_crown_lod1(size)))
    return results


if __name__ == "__main__":
    res = build_all()
    failed = [n for n, ok in res if not ok]
    for n, ok in res:
        print(f"slab: {n} {'OK' if ok else 'FAILED'}")
    if failed:
        sys.exit(1)
