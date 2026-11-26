"""Stone (pre-war office) tower kit: 24 m and 32 m footprints."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402
import roof_parts  # noqa: E402
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
    # Shared roof equipment
    roof_parts.water_tank(k, -h * 0.40, -h * 0.35, 2.95)
    roof_parts.vent_stack(k, h * 0.35, -h * 0.35, 2.95, 2.0)


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
    model.add(common.make_frustum(1.1, 1.1, 3.2, 5.8, segments=6,
                                  center=(-h * 0.40, -h * 0.35), color="bark_light"))


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
