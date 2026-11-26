"""Brick (brownstone & tenement) tower kit: 16 m and 24 m footprints."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import common  # noqa: E402
import damaged_parts  # noqa: E402
import kit  # noqa: E402
import brick_parts  # noqa: E402
import roof_parts  # noqa: E402
import roof_props  # noqa: E402
import shell  # noqa: E402

FLOOR_H = 3.8
BASE_H = 7.6
CROWN_H = 3.5

BUDGETS = {
    (16, "floor"): (3000, 300),
    (24, "floor"): (4500, 450),
    (16, "base"): (15000, 1500),
    (24, "base"): (15000, 1500),
    (16, "crown"): (8000, 800),
    (24, "crown"): (8000, 800),
}


def _bay_centers(size):
    bays = size // 4
    return [-size / 2 + 2.0 + 4.0 * i for i in range(bays)]


def _pier_positions(size):
    bays = size // 4
    return [-size / 2 + 4.0 * i for i in range(1, bays)]


def _add_piers_and_bands(k, size, z0, z1, roof=False):
    h = size / 2
    # Bottom and top slab bands so stacked modules have no gap
    k.box((-h, -h, z0), (h, h, z0 + 0.14), "brick_dark", roof=roof)
    k.box((-h, -h, z1 - 0.14), (h, h, z1), "brick_dark", roof=roof)
    # Corner piers
    for cx in (-h, h):
        for cy in (-h, h):
            k.box((cx - 0.22, cy - 0.22, z0), (cx + 0.22, cy + 0.22, z1), "brick_dark", roof=roof)
    # Mid-bay piers on all 4 faces
    for p in _pier_positions(size):
        k.box((p - 0.16, h - 0.05, z0), (p + 0.16, h + 0.10, z1), "brick_dark", roof=roof)
        k.box((p - 0.16, -h - 0.10, z0), (p + 0.16, -h + 0.05, z1), "brick_dark", roof=roof)
        k.box((h - 0.05, p - 0.16, z0), (h + 0.10, p + 0.16, z1), "brick_dark", roof=roof)
        k.box((-h - 0.10, p - 0.16, z0), (-h + 0.05, p + 0.16, z1), "brick_dark", roof=roof)


def _window_variant(idx, variant_b):
    if not variant_b:
        table = ("normal", "ac", "boarded", "cracked", "rust", "empty")
    else:
        table = ("boarded", "cracked", "empty", "ac", "boarded", "rust")
    return table[idx % len(table)]


def _build_floor_windows(k, size, z_bot, z_top, variant_b=False, skip_front_center=False, roof=False):
    h = size / 2
    bays = _bay_centers(size)
    faces = (
        (h, 1, 1, True),
        (h, 0, 1, True),
        (-h, 0, -1, True),
        (-h, 1, -1, False),
    )
    for fi, (coord, axis, sign, detailed) in enumerate(faces):
        for bi, uc in enumerate(bays):
            if skip_front_center and axis == 1 and sign == 1 and abs(uc) < 2.2:
                continue
            var = _window_variant(fi * 5 + bi, variant_b)
            brick_parts.add_window(k, coord, axis, sign, uc, z_bot, z_top,
                                   variant=var, roof=roof, detailed=detailed)


def build_floor(model, size, variant_b=False):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.14), (h, h, FLOOR_H - 0.14), "brick_red")
    _add_piers_and_bands(k, size, 0.0, FLOOR_H)
    _build_floor_windows(k, size, 0.95, 2.85, variant_b=variant_b)
    # Fire escapes: +Y front and +X side
    fe_u = -2.0 if size == 16 else -4.0
    brick_parts.add_fire_escape_floor(
        k, h, 1, 1, fe_u, 0.05, FLOOR_H, zig_dir=1, has_landing=not variant_b)
    brick_parts.add_fire_escape_floor(
        k, h, 0, 1, 0.0, 0.05, FLOOR_H, zig_dir=-1, has_landing=True)


def build_base(model, size):
    k = kit.Kit(model)
    h = size / 2
    shell.lobby_shell(k, size, "brick_red")

    # Brownstone base cladding (z 0..3.8, split around front door x -1.4..1.4)
    k.box((-h - 0.03, h - 0.02, 0.0), (-1.42, h + 0.06, 3.8), "brownstone")
    k.box((1.42, h - 0.02, 0.0), (h + 0.03, h + 0.06, 3.8), "brownstone")
    k.box((-h - 0.03, -h - 0.06, 0.0), (h + 0.03, -h + 0.02, 3.8), "brownstone")
    k.box((-h - 0.06, -h - 0.03, 0.0), (-h + 0.02, h + 0.03, 3.8), "brownstone")
    k.box((h - 0.02, -h - 0.03, 0.0), (h + 0.06, h + 0.03, 3.8), "brownstone")
    # Belt course between GF and 2F
    k.box((-h - 0.08, -h - 0.08, 3.68), (h + 0.08, h + 0.08, 3.88), "brownstone", roof=True)

    # 3-step brownstone stoop on +Y (within 1.2 m overhang, visual only so lobby stays walkable)
    for step in range(3):
        y_out = h + 1.15 - step * 0.38
        z_hi = 0.035 * (step + 1)
        k.box((-1.65, h, 0.0), (1.65, y_out, z_hi), "brownstone")
    # Flanking brownstone cheek walls / pedestals
    for sx in (-1.65, 1.42):
        k.box((sx, h, 0.0), (sx + 0.23, h + 1.15, 0.95), "brownstone")

    # Stone arch frame over front door (strictly outside x -1.4..1.4, z 0.1..3.1)
    door = shell.door_opening(size)
    k.frame(door, h + 0.06, 1, color="brownstone", t=0.18, depth=0.12, sill=False)
    k.box((-1.75, h, 3.12), (1.75, h + 0.24, 3.42), "limestone")
    k.box((-1.45, h, 3.42), (1.45, h + 0.22, 3.65), "brownstone")
    k.box((-0.24, h + 0.04, 3.12), (0.24, h + 0.28, 3.72), "limestone")

    # Upper base piers & windows (GF and 2F)
    _add_piers_and_bands(k, size, 3.8, BASE_H, roof=True)
    _build_floor_windows(k, size, 0.95, 2.85, variant_b=False, skip_front_center=True, roof=False)
    _build_floor_windows(k, size, 4.75, 6.65, variant_b=True, skip_front_center=False, roof=True)

    # 2nd floor fire escape landing + drop ladder
    fe_u = -2.0 if size == 16 else -4.0
    brick_parts.add_fire_escape_floor(k, h, 1, 1, fe_u, 3.85, BASE_H, zig_dir=1, roof=True)
    k.box((fe_u - 2.3, h + 0.8, 1.9), (fe_u - 2.25, h + 0.85, 3.85), "rust_dark")
    k.box((fe_u - 1.9, h + 0.8, 1.9), (fe_u - 1.85, h + 0.85, 3.85), "rust_dark")
    for rz in (2.1, 2.5, 2.9, 3.3, 3.7):
        k.box((fe_u - 2.3, h + 0.8, rz), (fe_u - 1.85, h + 0.85, rz + 0.04), "rust_dark")


def build_crown(model, size):
    k = kit.Kit(model)
    h = size / 2
    # Lower brick frieze (z 0..2.0)
    k.box((-h, -h, 0.0), (h, h, 2.0), "brick_red")
    _add_piers_and_bands(k, size, 0.0, 2.0)

    # Projecting 0.6 m limestone cornice (z 2.0..2.6) with mortar lines
    k.box((-h - 0.15, -h - 0.15, 2.0), (h + 0.15, h + 0.15, 2.22), "limestone")
    k.box((-h - 0.35, -h - 0.35, 2.22), (h + 0.35, h + 0.35, 2.42), "mortar")
    k.box((-h - 0.58, -h - 0.58, 2.42), (h + 0.58, h + 0.58, 2.60), "limestone")

    # Roof deck slab at z=2.35 and brick parapet z 2.6..3.35 + coping to 3.5
    k.box((-h + 0.25, -h + 0.25, 2.20), (h - 0.25, h - 0.25, 2.40), "rock_dark")
    par_t = 0.32
    k.box((-h, h - par_t, 2.60), (h, h, 3.36), "brick_red")
    k.box((-h, -h, 2.60), (h, -h + par_t, 3.36), "brick_red")
    k.box((-h, -h + par_t, 2.60), (-h + par_t, h - par_t, 3.36), "brick_red")
    k.box((h - par_t, -h + par_t, 2.60), (h, h - par_t, 3.36), "brick_red")
    # Limestone coping
    k.box((-h - 0.05, h - par_t - 0.04, 3.36), (h + 0.05, h + 0.04, CROWN_H), "limestone")
    k.box((-h - 0.05, -h - 0.04, 3.36), (h + 0.05, -h + par_t + 0.04, CROWN_H), "limestone")
    k.box((-h - 0.05, -h + par_t, 3.36), (-h + par_t + 0.04, h - par_t, CROWN_H), "limestone")
    k.box((h - par_t - 0.04, -h + par_t, 3.36), (h + 0.05, h - par_t, CROWN_H), "limestone")

    # Roof equipment from shared roof_props / roof_parts
    if size == 16:
        roof_props.roof_water_tank_ny(k, -3.6, -3.4, 2.40, r=1.25, leg_h=1.20, tank_h=1.95)
        roof_props.roof_stair_hut(k, 3.6, -3.2, 2.40, wall_color="brick_red")
        roof_props.roof_machinery(k, -3.4, 3.6, 2.40)
        roof_props.roof_antenna(k, 4.2, 3.8, 2.40, h=3.4)
        roof_parts.ac_unit(k, 0.2, 4.6, 2.40)
        roof_parts.vent_stack(k, 0.2, 0.2, 2.40, 1.5)
    else:
        roof_props.roof_water_tank_ny(k, -6.4, -5.8, 2.40, r=1.35, leg_h=1.35, tank_h=2.15)
        roof_props.roof_water_tank_ny(k, -6.4, 5.4, 2.40, r=1.30, leg_h=1.25, tank_h=2.05)
        roof_props.roof_stair_hut(k, 6.2, -5.4, 2.40, wall_color="brick_red")
        roof_props.roof_machinery(k, 0.0, -5.8, 2.40)
        roof_props.roof_antenna(k, 6.8, 6.2, 2.40, h=3.8)
        roof_props.roof_antenna(k, -0.8, 6.4, 2.40, h=3.2)
        roof_parts.ac_unit(k, 6.2, 0.6, 2.40)
        roof_parts.ac_unit(k, 2.6, 6.4, 2.40)
        roof_parts.vent_stack(k, -1.8, 0.2, 2.40, 1.5)


def build_floor_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.0), (h, h, FLOOR_H), "brick_red")
    k.box((-h - 0.04, -h - 0.04, 0.0), (h + 0.04, h + 0.04, 0.16), "brick_dark")
    for uc in _bay_centers(size):
        k.box((uc - 0.6, h - 0.01, 0.95), (uc + 0.6, h + 0.03, 2.85), "glass_dark")
        k.box((uc - 0.6, -h - 0.03, 0.95), (uc + 0.6, -h + 0.01, 2.85), "glass_dark")
        k.box((h - 0.01, uc - 0.6, 0.95), (h + 0.03, uc + 0.6, 2.85), "glass_dark")
        k.box((-h - 0.03, uc - 0.6, 0.95), (-h + 0.01, uc + 0.6, 2.85), "glass_dark")


def build_base_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.0), (h, h, 3.8), "brownstone")
    k.box((-h, -h, 3.8), (h, h, BASE_H), "brick_red")
    k.box((-1.4, h - 0.01, 0.1), (1.4, h + 0.04, 3.1), "soot")
    for uc in _bay_centers(size):
        k.box((uc - 0.6, h - 0.01, 4.75), (uc + 0.6, h + 0.03, 6.65), "glass_dark")
        k.box((uc - 0.6, -h - 0.03, 4.75), (uc + 0.6, -h + 0.01, 6.65), "glass_dark")


def build_crown_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.0), (h, h, 2.0), "brick_red")
    k.box((-h - 0.55, -h - 0.55, 2.0), (h + 0.55, h + 0.55, 2.6), "limestone")
    k.box((-h, -h, 2.6), (h, h, CROWN_H), "brick_red")
    if size == 16:
        model.add(common.make_frustum(1.3, 1.3, 2.6, 6.2, segments=6, center=(-3.6, -3.4), color="bark_light"))
        k.box((2.1, -4.9, 2.4), (5.1, -1.5, 4.7), "brick_red")
        k.box((-5.0, 2.5, 2.4), (-1.8, 4.7, 4.0), "concrete_grey")
        k.box((4.0, 3.6, 2.4), (4.4, 4.0, 5.8), "metal")
    else:
        model.add(common.make_frustum(1.35, 1.35, 2.6, 6.4, segments=6, center=(-6.4, -5.8), color="bark_light"))
        model.add(common.make_frustum(1.30, 1.30, 2.6, 6.2, segments=6, center=(-6.4, 5.4), color="bark_light"))
        k.box((4.7, -7.1, 2.4), (7.7, -3.7, 4.7), "brick_red")
        k.box((-1.6, -6.9, 2.4), (1.6, -4.7, 4.0), "concrete_grey")
        k.box((6.6, 6.0, 2.4), (7.0, 6.4, 6.2), "metal")
        k.box((-1.0, 6.2, 2.4), (-0.6, 6.6, 5.6), "metal")


def build_floor_c(model, size):
    """Damaged brick floor variant: 25-40% dead/missing panes, shattered crosses, soot/rust streaks, torn corner."""
    k = kit.Kit(model)
    h = size / 2
    damaged_parts.damaged_wall_core(k, size, 0.14, FLOOR_H - 0.14, "brick_red", trim_color="brick_dark")
    # Bottom and top slab bands
    k.box((-h, -h, 0.0), (h, h, 0.14), "brick_dark")
    k.box((-h, -h, FLOOR_H - 0.14), (h, h, FLOOR_H), "brick_dark")
    # Corner piers (shortened at +X/+Y torn corner) + mid-bay piers
    for cx in (-h, h):
        for cy in (-h, h):
            z_top = FLOOR_H - 0.38 if (cx > 0 and cy > 0) else FLOOR_H
            k.box((cx - 0.22, cy - 0.22, 0.0), (cx + 0.22, cy + 0.22, z_top), "brick_dark")
    for p in _pier_positions(size):
        k.box((p - 0.16, h - 0.05, 0.0), (p + 0.16, h + 0.10, FLOOR_H), "brick_dark")
        k.box((p - 0.16, -h - 0.10, 0.0), (p + 0.16, -h + 0.05, FLOOR_H), "brick_dark")
        k.box((h - 0.05, p - 0.16, 0.0), (h + 0.10, p + 0.16, FLOOR_H), "brick_dark")
        k.box((-h - 0.10, p - 0.16, 0.0), (-h + 0.05, p + 0.16, FLOOR_H), "brick_dark")

    bays = _bay_centers(size)
    faces = ((h, 1, 1, True), (h, 0, 1, True), (-h, 0, -1, True), (-h, 1, -1, False))
    for fi, (coord, axis, sign, detailed) in enumerate(faces):
        for bi, uc in enumerate(bays):
            idx = fi * len(bays) + bi
            if (fi == 0 and bi == 1) or (fi == 1 and bi == len(bays) - 2):
                damaged_parts.missing_frame(k, coord, axis, sign, uc, 0.95, 2.85, 0.65)
                damaged_parts.soot_streak(k, coord, axis, sign, uc, 2.85)
                continue
            shattered = (fi == 0 and bi == len(bays) - 1) or (fi == 1 and bi == 0) or (fi == 2 and bi == 1)
            if shattered:
                brick_parts.add_window(k, coord, axis, sign, uc, 0.95, 2.85, variant="empty", detailed=detailed)
                damaged_parts.shattered_pane(k, coord, axis, sign, uc, 0.95, 2.85, 0.65, d_base=0.028)
            else:
                var = ("empty", "normal", "boarded", "rust", "empty", "ac")[idx % 6]
                brick_parts.add_window(k, coord, axis, sign, uc, 0.95, 2.85, variant=var, detailed=detailed)
            if (fi == 0 and bi == 2) or (fi == 2 and bi == 2):
                damaged_parts.soot_streak(k, coord, axis, sign, uc, 2.85)
            if detailed and (idx % 2 == 0):
                damaged_parts.rust_streak(k, coord, axis, sign, uc, 0.95)

    fe_u = -2.0 if size == 16 else -4.0
    brick_parts.add_fire_escape_floor(k, h, 1, 1, fe_u, 0.05, FLOOR_H, zig_dir=1, has_landing=True)
    brick_parts.add_fire_escape_floor(k, h, 0, 1, 0.0, 0.05, FLOOR_H, zig_dir=-1, has_landing=False)


def build_floor_c_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    damaged_parts.damaged_wall_core(k, size, 0.14, FLOOR_H - 0.14, "brick_red", patch=1.2, short_h=0.40)
    k.box((-h - 0.04, -h - 0.04, 0.0), (h + 0.04, h + 0.04, 0.16), "brick_dark")
    k.box((-h - 0.04, -h - 0.04, FLOOR_H - 0.14), (h + 0.04, h + 0.04, FLOOR_H), "brick_dark")
    for bi, uc in enumerate(_bay_centers(size)):
        col_f = "soot" if bi % 2 == 1 else "glass_dark"
        k.box((uc - 0.6, h - 0.01, 0.95), (uc + 0.6, h + 0.03, 2.85), col_f)
        if size > 16 or bi % 2 == 0:
            k.box((uc - 0.6, -h - 0.03, 0.95), (uc + 0.6, -h + 0.01, 2.85), "glass_dark")
        k.box((h - 0.01, uc - 0.6, 0.95), (h + 0.03, uc + 0.6, 2.85), "soot" if bi == 1 else "glass_dark")
        if size > 16 or bi % 2 == 0:
            k.box((-h - 0.03, uc - 0.6, 0.95), (-h + 0.01, uc + 0.6, 2.85), "glass_dark")


def build_r40_all():
    results = []
    for size in (16, 24):
        specs = [
            (f"tw_brick_{size}_floor_c", BUDGETS[(size, "floor")][0],
             lambda m, s=size: build_floor_c(m, s)),
            (f"tw_brick_{size}_floor_c_lod1", BUDGETS[(size, "floor")][1],
             lambda m, s=size: build_floor_c_lod1(m, s)),
            (f"tw_brick_{size}_crown", BUDGETS[(size, "crown")][0],
             lambda m, s=size: build_crown(m, s)),
            (f"tw_brick_{size}_crown_lod1", BUDGETS[(size, "crown")][1],
             lambda m, s=size: build_crown_lod1(m, s)),
        ]
        for name, budget, fn in specs:
            ok = common.run_model(name, "buildings", budget, fn,
                                  grip=(0.0, 0.0, 0.0), top_preview=False, view=kit.FRONT_VIEW)
            results.append((name, ok))
    return results


def build_all():
    results = []
    for size in (16, 24):
        specs = [
            (f"tw_brick_{size}_floor", BUDGETS[(size, "floor")][0],
             lambda m, s=size: build_floor(m, s, False), False),
            (f"tw_brick_{size}_floor_lod1", BUDGETS[(size, "floor")][1],
             lambda m, s=size: build_floor_lod1(m, s), False),
            (f"tw_brick_{size}_floor_b", BUDGETS[(size, "floor")][0],
             lambda m, s=size: build_floor(m, s, True), False),
            (f"tw_brick_{size}_floor_b_lod1", BUDGETS[(size, "floor")][1],
             lambda m, s=size: build_floor_lod1(m, s), False),
            (f"tw_brick_{size}_base", BUDGETS[(size, "base")][0],
             lambda m, s=size: build_base(m, s), True),
            (f"tw_brick_{size}_base_lod1", BUDGETS[(size, "base")][1],
             lambda m, s=size: build_base_lod1(m, s), False),
            (f"tw_brick_{size}_crown", BUDGETS[(size, "crown")][0],
             lambda m, s=size: build_crown(m, s), False),
            (f"tw_brick_{size}_crown_lod1", BUDGETS[(size, "crown")][1],
             lambda m, s=size: build_crown_lod1(m, s), False),
        ]
        for name, budget, fn, top in specs:
            ok = common.run_model(name, "buildings", budget, fn,
                                  grip=(0.0, 0.0, 0.0), top_preview=top, view=kit.FRONT_VIEW)
            results.append((name, ok))
    return results


if __name__ == "__main__":
    out = build_all()
    failed = [n for n, ok in out if not ok]
    if failed:
        print("BRICK FAILED:", failed)
        sys.exit(1)
