"""Stone (pre-war office) tower kit: 24 m and 32 m footprints."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import common  # noqa: E402
import damaged_parts  # noqa: E402
import kit  # noqa: E402
import roof_parts  # noqa: E402
import roof_props  # noqa: E402
import shell  # noqa: E402
import stone_parts  # noqa: E402

FLOOR_H = 3.8
BASE_H = 7.6
CROWN_H = 5.0

BUDGETS = {
    (24, "floor"): (4500, 450),
    (32, "floor"): (6000, 600),
    (24, "base"): (15000, 1500),
    (32, "base"): (15000, 1500),
    (24, "crown"): (8000, 800),
    (32, "crown"): (8000, 800),
}


def _bay_centers(size):
    bays = size // 4
    return [-size / 2 + 2.0 + 4.0 * i for i in range(bays)]


def _window_variant(idx, variant_b, row_boarded=False):
    if row_boarded:
        return "boarded"
    if not variant_b:
        table = ("tint", "dead", "tint", "cracked", "boarded", "tint")
    else:
        table = ("dead", "boarded", "cracked", "dead", "tint", "boarded")
    return table[idx % len(table)]


def _build_windows(k, size, z_bot, z_top, variant_b=False, skip_front_center=False,
                   balconies=False, roof=False):
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
            row_board = variant_b and fi == 0
            var = _window_variant(fi * 7 + bi, variant_b, row_boarded=row_board)
            has_bal = balconies and axis == 1 and sign == 1 and bi in (1, len(bays) - 2)
            stone_parts.add_stone_window(
                k, coord, axis, sign, uc, z_bot, z_top,
                variant=var, has_balcony=has_bal, roof=roof, detailed=detailed)


def build_floor(model, size, variant_b=False):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.16), (h, h, FLOOR_H - 0.18), "limestone")
    k.box((-h - 0.04, -h - 0.04, 0.0), (h + 0.04, h + 0.04, 0.16), "limestone_dark")
    # Fallen cornice piece on floor_b
    top_hi = h + (0.04 if not variant_b else -0.02)
    k.box((-h - 0.04, -h - 0.04, FLOOR_H - 0.18), (top_hi, h + 0.04, FLOOR_H), "limestone_dark")
    stone_parts.add_pilasters(k, size, 0.0, FLOOR_H, broken_tops=variant_b)
    _build_windows(k, size, 0.85, 2.95, variant_b=variant_b, balconies=not variant_b)


def build_base(model, size):
    k = kit.Kit(model)
    h = size / 2
    shell.lobby_shell(k, size, "limestone_dark")

    # Rusticated base floor (z 0.1..3.7): ashlar bands every 0.6 m with 0.05 m deep grooves
    z = 0.12
    while z < 3.65:
        z_hi = min(z + 0.54, 3.66)
        if z < 3.15:
            k.box((-h - 0.05, h - 0.02, z), (-1.45, h + 0.05, z_hi), "limestone")
            k.box((1.45, h - 0.02, z), (h + 0.05, h + 0.05, z_hi), "limestone")
        else:
            k.box((-h - 0.05, h - 0.02, z), (h + 0.05, h + 0.05, z_hi), "limestone")
        k.box((-h - 0.05, -h - 0.05, z), (h + 0.05, -h + 0.02, z_hi), "limestone")
        k.box((-h - 0.05, -h - 0.02, z), (-h + 0.02, h + 0.02, z_hi), "limestone")
        k.box((h - 0.02, -h - 0.02, z), (h + 0.05, h + 0.02, z_hi), "limestone")
        z += 0.60

    # Belt course above GF
    k.box((-h - 0.14, -h - 0.14, 3.68), (h + 0.14, h + 0.14, 3.92), "limestone_dark", roof=True)

    # Two columns flanking front door (strictly outside x -1.4..1.4) + entablature above gap
    for cx in (-1.95, 1.95):
        cy = h + 0.55
        k.box((cx - 0.32, cy - 0.32, 0.0), (cx + 0.32, cy + 0.32, 0.45), "limestone_dark")
        model.add(common.make_frustum(0.26, 0.22, 0.45, 3.20, segments=8,
                                      center=(cx, cy), color="limestone"))
        k.box((cx - 0.32, cy - 0.32, 3.20), (cx + 0.32, cy + 0.32, 3.45), "limestone_dark")
    k.box((-2.45, h, 3.45), (2.45, h + 0.92, 3.95), "limestone")
    k.box((-2.55, h, 3.88), (2.55, h + 1.00, 4.05), "limestone_dark")

    # Broken glass doors: frame only (no collider, no glass inside walkable door gap)
    door = shell.door_opening(size)
    k.frame(door, h + 0.05, 1, color="metal", t=0.10, depth=0.08, sill=False)

    # 2nd floor facade (z 3.92..7.6)
    k.box((-h, -h, 3.92), (h, h, BASE_H), "limestone", roof=True)
    stone_parts.add_pilasters(k, size, 3.92, BASE_H, roof=True)
    _build_windows(k, size, 0.95, 2.85, variant_b=False, skip_front_center=True, roof=False)
    _build_windows(k, size, 4.70, 6.75, variant_b=True, skip_front_center=False, roof=True)


def build_crown(model, size):
    k = kit.Kit(model)
    h = size / 2
    # Attic frieze (z 0..2.2)
    k.box((-h, -h, 0.0), (h, h, 2.2), "limestone")
    stone_parts.add_pilasters(k, size, 0.0, 2.2)
    # Heavy dentil cornice (z 2.2..3.2)
    stone_parts.add_dentil_cornice(k, size, 2.2)
    # Roof slab at z=2.9
    k.box((-h + 0.3, -h + 0.3, 2.75), (h - 0.3, h - 0.3, 2.95), "rock_dark")
    # Balustrade up to CROWN_H (5.0 m)
    stone_parts.add_balustrade(k, size, 3.2, 4.8)
    # Corner flagpoles on +Y corners
    stone_parts.add_flagpole(k, -h + 0.65, h - 0.65, 3.2)
    stone_parts.add_flagpole(k, h - 0.65, h - 0.65, 3.2)
    # Shared roof equipment (NY water tanks, stair hut, machinery, antennas, AC, vent)
    if size == 24:
        roof_props.roof_water_tank_ny(k, -6.2, -5.6, 2.95, r=1.30, leg_h=1.25, tank_h=2.00)
        roof_props.roof_stair_hut(k, 6.0, -5.2, 2.95, wall_color="limestone")
        roof_props.roof_machinery(k, -0.2, -5.6, 2.95)
        roof_props.roof_antenna(k, 6.2, 5.6, 2.95, h=3.6)
        roof_parts.ac_unit(k, -5.8, 5.4, 2.95)
        roof_parts.ac_unit(k, 0.4, 5.6, 2.95)
        roof_parts.vent_stack(k, 4.8, 0.2, 2.95, 2.0)
    else:
        roof_props.roof_water_tank_ny(k, -9.2, -8.4, 2.95, r=1.40, leg_h=1.40, tank_h=2.20)
        roof_props.roof_water_tank_ny(k, -9.2, 7.8, 2.95, r=1.30, leg_h=1.25, tank_h=2.05)
        roof_props.roof_stair_hut(k, 8.8, -8.0, 2.95, wall_color="limestone")
        roof_props.roof_machinery(k, 0.0, -8.4, 2.95)
        roof_props.roof_antenna(k, 9.2, 8.6, 2.95, h=4.2)
        roof_props.roof_antenna(k, -1.2, 8.8, 2.95, h=3.4)
        roof_parts.ac_unit(k, 8.4, 0.8, 2.95)
        roof_parts.vent_stack(k, -2.5, 0.4, 2.95, 2.0)


def build_floor_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.0), (h, h, FLOOR_H), "limestone")
    k.box((-h - 0.04, -h - 0.04, 0.0), (h + 0.04, h + 0.04, 0.18), "limestone_dark")
    for uc in _bay_centers(size):
        k.box((uc - 1.0, h - 0.01, 0.85), (uc + 1.0, h + 0.03, 2.95), "glass_tint")
        k.box((uc - 1.0, -h - 0.03, 0.85), (uc + 1.0, -h + 0.01, 2.95), "glass_tint")
        k.box((h - 0.01, uc - 1.0, 0.85), (h + 0.03, uc + 1.0, 2.95), "glass_tint")
        k.box((-h - 0.03, uc - 1.0, 0.85), (-h + 0.01, uc + 1.0, 2.95), "glass_tint")


def build_base_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.0), (h, h, 3.8), "limestone_dark")
    k.box((-h, -h, 3.8), (h, h, BASE_H), "limestone")
    k.box((-1.4, h - 0.01, 0.1), (1.4, h + 0.04, 3.1), "soot")
    for uc in _bay_centers(size):
        k.box((uc - 1.0, h - 0.01, 4.70), (uc + 1.0, h + 0.03, 6.75), "glass_tint")
        k.box((uc - 1.0, -h - 0.03, 4.70), (uc + 1.0, -h + 0.01, 6.75), "glass_tint")


def build_crown_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    k.box((-h, -h, 0.0), (h, h, 2.2), "limestone")
    k.box((-h - 0.65, -h - 0.65, 2.2), (h + 0.65, h + 0.65, 3.2), "limestone_dark")
    k.box((-h, -h, 3.2), (h, h, CROWN_H), "limestone")
    if size == 24:
        model.add(common.make_frustum(1.3, 1.3, 3.2, 6.8, segments=6, center=(-6.2, -5.6), color="bark_light"))
        k.box((4.5, -6.9, 3.0), (7.5, -3.5, 5.4), "limestone")
        k.box((-1.8, -6.7, 3.0), (1.4, -4.5, 4.6), "concrete_grey")
        k.box((6.0, 5.4, 3.0), (6.4, 5.8, 6.6), "metal")
    else:
        model.add(common.make_frustum(1.4, 1.4, 3.2, 7.2, segments=6, center=(-9.2, -8.4), color="bark_light"))
        model.add(common.make_frustum(1.3, 1.3, 3.2, 6.8, segments=6, center=(-9.2, 7.8), color="bark_light"))
        k.box((7.3, -9.7, 3.0), (10.3, -6.3, 5.4), "limestone")
        k.box((-1.6, -9.5, 3.0), (1.6, -7.3, 4.6), "concrete_grey")
        k.box((9.0, 8.4, 3.0), (9.4, 8.8, 7.1), "metal")
        k.box((-1.4, 8.6, 3.0), (-1.0, 9.0, 6.4), "metal")


def build_floor_c(model, size):
    """Damaged stone floor variant: dark/missing panes, shattered crosses, soot/rust streaks, torn corner."""
    k = kit.Kit(model)
    h = size / 2
    damaged_parts.damaged_wall_core(k, size, 0.16, FLOOR_H - 0.18, "limestone", trim_color="limestone_dark")
    k.box((-h - 0.04, -h - 0.04, 0.0), (h + 0.04, h + 0.04, 0.16), "limestone_dark")
    k.box((-h - 0.04, -h - 0.04, FLOOR_H - 0.18), (h + 0.04, h + 0.04, FLOOR_H), "limestone_dark")

    bays_n = size // 4
    positions = [-h + 4.0 * i for i in range(bays_n + 1)]
    for idx, p in enumerate(positions):
        top_f = FLOOR_H - 0.38 if idx == bays_n else (FLOOR_H - 0.55 if idx == 1 else FLOOR_H)
        k.box((p - 0.25, h - 0.05, 0.0), (p + 0.25, h + 0.40, top_f), "limestone")
        k.box((p - 0.25, -h - 0.40, 0.0), (p + 0.25, -h + 0.05, FLOOR_H), "limestone")
    for idx, p in enumerate(positions[1:-1], start=1):
        top_s = FLOOR_H - 0.55 if idx == 2 else FLOOR_H
        k.box((h - 0.05, p - 0.25, 0.0), (h + 0.40, p + 0.25, top_s), "limestone")
        k.box((-h - 0.40, p - 0.25, 0.0), (-h + 0.05, p + 0.25, top_s), "limestone")

    bays = _bay_centers(size)
    faces = ((h, 1, 1, True), (h, 0, 1, True), (-h, 0, -1, True), (-h, 1, -1, False))
    for fi, (coord, axis, sign, detailed) in enumerate(faces):
        for bi, uc in enumerate(bays):
            idx = fi * len(bays) + bi
            if (fi == 0 and bi == 1) or (fi == 1 and bi == len(bays) - 2):
                damaged_parts.missing_frame(k, coord, axis, sign, uc, 0.85, 2.95, 1.05)
                damaged_parts.soot_streak(k, coord, axis, sign, uc, 2.95, width=1.15)
                continue
            shattered = (fi == 0 and bi == len(bays) - 2) or (fi == 1 and bi == 1) or (fi == 2 and bi == 2)
            has_bal = (axis == 1 and sign == 1 and bi in (1, len(bays) - 2) and bi != 1)
            if shattered:
                stone_parts.add_stone_window(k, coord, axis, sign, uc, 0.85, 2.95,
                                             variant="dead", has_balcony=has_bal, detailed=detailed)
                damaged_parts.shattered_pane(k, coord, axis, sign, uc, 0.85, 2.95, 1.05, d_base=0.028)
            else:
                var = ("dead", "tint", "boarded", "dead", "tint", "dead")[idx % 6]
                stone_parts.add_stone_window(k, coord, axis, sign, uc, 0.85, 2.95,
                                             variant=var, has_balcony=has_bal, detailed=detailed)
            if (fi == 0 and bi == 3) or (fi == 2 and bi == 1):
                damaged_parts.soot_streak(k, coord, axis, sign, uc, 2.95, width=1.15)
            if detailed and (idx % 2 == 0):
                damaged_parts.rust_streak(k, coord, axis, sign, uc, 0.85, width=0.70)


def build_floor_c_lod1(model, size):
    k = kit.Kit(model)
    h = size / 2
    damaged_parts.damaged_wall_core(k, size, 0.16, FLOOR_H - 0.18, "limestone", patch=1.2, short_h=0.40)
    k.box((-h - 0.04, -h - 0.04, 0.0), (h + 0.04, h + 0.04, 0.18), "limestone_dark")
    k.box((-h - 0.04, -h - 0.04, FLOOR_H - 0.18), (h + 0.04, h + 0.04, FLOOR_H), "limestone_dark")
    for bi, uc in enumerate(_bay_centers(size)):
        col = "soot" if bi % 2 == 1 else "glass_dark"
        k.box((uc - 1.0, h - 0.01, 0.85), (uc + 1.0, h + 0.03, 2.95), col)
        k.box((uc - 1.0, -h - 0.03, 0.85), (uc + 1.0, -h + 0.01, 2.95), "glass_dark")
        k.box((h - 0.01, uc - 1.0, 0.85), (h + 0.03, uc + 1.0, 2.95), "soot" if bi == 2 else "glass_tint")
        k.box((-h - 0.03, uc - 1.0, 0.85), (-h + 0.01, uc + 1.0, 2.95), "glass_tint")


def build_r40_all():
    results = []
    for size in (24, 32):
        specs = [
            (f"tw_stone_{size}_floor_c", BUDGETS[(size, "floor")][0],
             lambda m, s=size: build_floor_c(m, s)),
            (f"tw_stone_{size}_floor_c_lod1", BUDGETS[(size, "floor")][1],
             lambda m, s=size: build_floor_c_lod1(m, s)),
            (f"tw_stone_{size}_crown", BUDGETS[(size, "crown")][0],
             lambda m, s=size: build_crown(m, s)),
            (f"tw_stone_{size}_crown_lod1", BUDGETS[(size, "crown")][1],
             lambda m, s=size: build_crown_lod1(m, s)),
        ]
        for name, budget, fn in specs:
            ok = common.run_model(name, "buildings", budget, fn,
                                  grip=(0.0, 0.0, 0.0), top_preview=False, view=kit.FRONT_VIEW)
            results.append((name, ok))
    return results


def build_all():
    results = []
    for size in (24, 32):
        specs = [
            (f"tw_stone_{size}_floor", BUDGETS[(size, "floor")][0],
             lambda m, s=size: build_floor(m, s, False), False),
            (f"tw_stone_{size}_floor_lod1", BUDGETS[(size, "floor")][1],
             lambda m, s=size: build_floor_lod1(m, s), False),
            (f"tw_stone_{size}_floor_b", BUDGETS[(size, "floor")][0],
             lambda m, s=size: build_floor(m, s, True), False),
            (f"tw_stone_{size}_floor_b_lod1", BUDGETS[(size, "floor")][1],
             lambda m, s=size: build_floor_lod1(m, s), False),
            (f"tw_stone_{size}_base", BUDGETS[(size, "base")][0],
             lambda m, s=size: build_base(m, s), True),
            (f"tw_stone_{size}_base_lod1", BUDGETS[(size, "base")][1],
             lambda m, s=size: build_base_lod1(m, s), False),
            (f"tw_stone_{size}_crown", BUDGETS[(size, "crown")][0],
             lambda m, s=size: build_crown(m, s), False),
            (f"tw_stone_{size}_crown_lod1", BUDGETS[(size, "crown")][1],
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
        print("STONE FAILED:", failed)
        sys.exit(1)
