"""R43-04: Stop-off floor interiors (office, flat, camp) in 24 m and 32 m footprints.

Exports `build_all()` -> `[(name, ok), ...]`.
Standalone: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/towers/interiors.py`
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER = os.path.dirname(HERE)
for _p in (HERE, BLENDER, os.path.join(BLENDER, "buildings"),
           os.path.join(BLENDER, "metro"), os.path.join(BLENDER, "places")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import bpy  # noqa: E402
from mathutils import Vector  # noqa: E402

import common  # noqa: E402
import interior_props as ip  # noqa: E402
import kit  # noqa: E402
from jam_util import pbox  # noqa: E402

CEIL_Z = 3.80
WALL_IN = 0.15
WALL_T = 0.12
BUDGET = {24: 30000, 32: 45000}


def _shaft_bounds(size):
    sy = -0.25 * size
    return (-2.2, sy - 3.2, 2.2, sy + 3.2), sy


def _shell_surfaces(k, size, floor_col, floor_alt):
    """Floor (minus shaft), ceiling (roof=True, minus shaft), and inner window walls (no colliders)."""
    half = size * 0.5
    in_h = half - WALL_IN
    (sx0, sy0, sx1, sy1), _ = _shaft_bounds(size)

    # 4 floor & ceiling strips around the shaft rect
    strips = [
        (-in_h, -in_h, sx0, in_h),
        (sx1, -in_h, in_h, in_h),
        (sx0, sy1, sx1, in_h),
        (sx0, -in_h, sx1, sy0),
    ]
    for i, (x0, y0, x1, y1) in enumerate(strips):
        col = floor_col if i % 2 == 0 else floor_alt
        k.box((x0, y0, -0.02), (x1, y1, 0.0), col)
        k.box((x0, y0, 3.72), (x1, y1, CEIL_Z), "white", roof=True)

    # Ceiling light fixtures & dangling wires (roof=True)
    rng = random.Random(size * 7)
    for lx in (-half * 0.55, 0.0, half * 0.55):
        for ly in (-half * 0.55, 0.0, half * 0.55):
            if sx0 - 0.5 < lx < sx1 + 0.5 and sy0 - 0.5 < ly < sy1 + 0.5:
                continue
            k.box((lx - 0.55, ly - 0.15, 3.64), (lx + 0.55, ly + 0.15, 3.72), "rock_light", roof=True)
            if rng.random() < 0.45:
                w = pbox((-0.015, -0.015, -0.55), (0.015, 0.015, 0.0), "gun_black", bevel=0.002)
                common.transform(w, loc=(lx + 0.3, ly, 3.70), rot=(0.25, -0.18, rng.uniform(0, 3.1)))
                k.model.add(w, roof=True)

    # Outer window skin walls (visual only; colliders come from R43-01)
    win_c = list(range(-size // 2 + 4, size // 2 - 2, 4))
    hw = 0.90
    curtain_cols = ("gown_blue", "canvas_khaki", "cloth_olive", "shirt_red")
    for side in range(4):
        # Build along local U in [-in_h, in_h], then orient per side
        # side 0: +Y, side 1: -Y, side 2: +X, side 3: -X
        def _wbox(u0, u1, z0, z1, d0, d1, col):
            if side == 0:
                k.box((u0, in_h + d0, z0), (u1, in_h + d1, z1), col)
            elif side == 1:
                k.box((u0, -in_h - d1, z0), (u1, -in_h - d0, z1), col)
            elif side == 2:
                k.box((in_h + d0, u0, z0), (in_h + d1, u1, z1), col)
            else:
                k.box((-in_h - d1, u0, z0), (-in_h - d0, u1, z1), col)

        _wbox(-in_h, in_h, 0.0, 0.85, 0.0, WALL_T, "limestone")
        _wbox(-in_h, in_h, 2.65, 3.72, 0.0, WALL_T, "limestone")
        prev = -in_h
        for idx, uc in enumerate(win_c):
            _wbox(prev, uc - hw, 0.85, 2.65, 0.0, WALL_T, "limestone")
            # Sill & frame
            _wbox(uc - hw - 0.08, uc + hw + 0.08, 0.80, 0.86, -0.04, WALL_T, "rock_light")
            _wbox(uc - hw, uc + hw, 2.60, 2.66, -0.02, WALL_T, "metal")
            _wbox(uc - 0.03, uc + 0.03, 0.86, 2.60, 0.02, 0.06, "metal")
            # Broken glass shards + occasional curtain + vine
            if (idx + side) % 3 == 0:
                _wbox(uc - hw, uc - hw * 0.35, 0.86, 1.30, 0.03, 0.05, "glass_tint")
                vx, vy = ((uc, in_h - 0.1) if side == 0 else
                          (uc, -in_h + 0.1) if side == 1 else
                          (in_h - 0.1, uc) if side == 2 else (-in_h + 0.1, uc))
                ip.vine_cluster(k.model, vx, vy, 2.55, seed=side * 10 + idx)
            else:
                _wbox(uc - hw, uc + hw, 0.86, 2.60, 0.03, 0.05, "glass_dark")
            if (idx + side) % 2 == 0:
                _wbox(uc - hw, uc - hw + 0.32, 0.92, 2.58, -0.05, 0.0, curtain_cols[(idx + side) % 4])
            prev = uc + hw
        _wbox(prev, in_h, 0.85, 2.65, 0.0, WALL_T, "limestone")


def _stain_patches(k, size, count=5):
    rng = random.Random(size * 13)
    (sx0, sy0, sx1, sy1), sy = _shaft_bounds(size)
    half = size * 0.5 - 2.0
    for _ in range(count):
        px, py = rng.uniform(-half, half), rng.uniform(-half, half)
        if sx0 - 1.0 < px < sx1 + 1.0 and sy0 - 1.0 < py < sy1 + 1.0:
            continue
        k.box((px - 0.7, py - 0.5, 0.02), (px + 0.7, py + 0.5, 0.028), rng.choice(("concrete_stain", "soot")))


def _add_spawns(k, pts):
    for i, (x, y) in enumerate(pts, 1):
        k.model.add_empty(f"spawn_{i}", (x, y, 0.1))


def _build_office(model, size):
    k = kit.Kit(model)
    _shell_surfaces(k, size, "concrete_grey", "rock_mid")
    _stain_patches(k, size, 6)
    half = size * 0.5
    in_h = half - WALL_IN
    _, sy = _shaft_bounds(size)
    door_y = sy + 3.2

    # Meeting room in front-left corner with glass walls & open 1.8 m doorway
    mx, my = -3.4, 2.8
    k.wall((-in_h, my, 0.0), (mx, my + 0.18, 3.72),
           openings=[((-7.5, my - 0.05, 0.0), (-5.5, my + 0.25, 2.3))], color="white", base="meet_y")
    k.wall((mx, my, 0.0), (mx + 0.18, in_h, 3.72), openings=[], color="white", base="meet_x")
    k.box((-in_h + 0.4, my - 0.02, 1.0), (mx - 0.2, my + 0.20, 2.4), "glass_tint")
    ip.meeting_table(model, "meet_tbl", (-in_h + mx) * 0.5, (my + in_h) * 0.5, length=3.4 if size == 24 else 4.6)

    # Kitchenette in far back-right (well east of hatch at x=2.4..4.4)
    ip.kitchenette(model, "kitchen", in_h - 1.6, -in_h + 0.55)
    ip.copier(model, "copier", -3.4, -in_h + 1.2, rot_z=math.pi * 0.5)
    for i, fy in enumerate((-1.2, 0.0, 1.2)):
        ip.filing_cabinet(model, f"fc_{i}", -in_h + 0.55, fy, rot_z=-math.pi * 0.5)

    # Open-plan cubicle rows (front-right & centre-right)
    xs = (-0.5, 4.2, 8.2) if size == 24 else (-1.0, 3.5, 8.0, 12.2)
    ys = (2.5, 7.2) if size == 24 else (1.5, 6.2, 10.8)
    idx = 0
    for r, dy in enumerate(ys):
        ip.cubicle_wall(model, f"cub_{r}", -1.2, dy + 0.85, in_h - 1.8, dy + 0.85)
        for c, dx in enumerate(xs):
            idx += 1
            ip.desk_station(model, f"desk_{idx}", dx, dy, rot_z=0.0 if r % 2 == 0 else math.pi,
                            tipped_chair=(idx % 3 == 0), seed=idx)
            ip.clutter_papers(model, dx + 0.4, dy - 1.1, 0.0, count=4, seed=idx * 5)

    # West wing desks
    for w_i, wy in enumerate((-1.5, -5.5) if size == 24 else (-2.0, -6.5, -10.5)):
        ip.desk_station(model, f"wdesk_{w_i}", -6.8, wy, rot_z=math.pi * 0.5, tipped_chair=(w_i == 1), seed=20 + w_i)

    # Angled desk barricade near shaft door (keeps 2 m corridor open)
    ip.desk_station(model, "barricade_desk", -1.9, door_y + 1.6, rot_z=0.35, tipped_chair=True, seed=99)

    spawns = [(-7.5, 6.8), (1.8, 4.8), (7.5, 4.8), (-6.0, -3.5), (7.0, -2.5)]
    if size == 32:
        spawns.append((10.5, 9.0))
    _add_spawns(k, spawns)


def _build_flat(model, size):
    k = kit.Kit(model)
    _shell_surfaces(k, size, "wood_crate", "bark_light")
    _stain_patches(k, size, 5)
    half = size * 0.5
    in_h = half - WALL_IN
    _, sy = _shaft_bounds(size)
    door_y = sy + 3.2

    # Central N-S corridor x in [-1.8, 1.8] from door_y to in_h - 1.5; cross corridor y in [1.0, 3.6]
    # Left apartment walls (x = -1.8) and Right apartment walls (x = 1.8)
    cw_y0 = door_y + 0.2
    cy_lo, cy_hi = 1.0, 3.6
    # Cross walls at y = cy_lo and y = cy_hi, leaving x in [-1.8, 1.8] open, plus hatch corridor x in [2.2, 5.0]
    k.wall((-in_h, cy_lo, 0.0), (-1.8, cy_lo + 0.2, 3.72),
           openings=[((-7.5, cy_lo - 0.1, 0.0), (-5.8, cy_lo + 0.3, 2.3))], color="white", base="apt_w1")
    k.wall((-in_h, cy_hi, 0.0), (-1.8, cy_hi + 0.2, 3.72),
           openings=[((-7.5, cy_hi - 0.1, 0.0), (-5.8, cy_hi + 0.3, 2.3))], color="white", base="apt_w2")
    k.wall((5.0, cy_lo, 0.0), (in_h, cy_lo + 0.2, 3.72),
           openings=[((6.5, cy_lo - 0.1, 0.0), (8.2, cy_lo + 0.3, 2.3))], color="white", base="apt_w3")
    k.wall((1.8, cy_hi, 0.0), (in_h, cy_hi + 0.2, 3.72),
           openings=[((6.5, cy_hi - 0.1, 0.0), (8.2, cy_hi + 0.3, 2.3))], color="white", base="apt_w4")

    # Corridor walls along x = -1.8 and x = 1.8 (for y > cy_hi)
    k.wall((-1.9, cy_hi + 0.2, 0.0), (-1.7, in_h, 3.72),
           openings=[((-2.0, 6.2, 0.0), (-1.6, 7.9, 2.3))], color="white", base="cor_l")
    k.wall((1.7, cy_hi + 0.2, 0.0), (1.9, in_h, 3.72),
           openings=[((1.6, 6.2, 0.0), (2.0, 7.9, 2.3))], color="white", base="cor_r")

    # Boarded door on far end of corridor (visual + collider)
    for bz in (0.6, 1.2, 1.8):
        k.solid((-1.7, in_h - 0.35, bz), (1.7, in_h - 0.25, bz + 0.16), "bark_light", base=f"board_{int(bz*10)}")

    # Front-left apt: bed, sofa, coffee table, kitchenette
    ip.bed(model, "bed_fl", -in_h + 1.5, in_h - 1.5, rot_z=math.pi * 0.5, blanket="tarp_blue")
    ip.sofa(model, "sofa_fl", -4.5, in_h - 1.2, rot_z=math.pi)
    ip.coffee_table(model, "ct_fl", -4.5, in_h - 2.6)
    ip.kitchenette(model, "kit_fl", -in_h + 0.6, 5.5, rot_z=-math.pi * 0.5)

    # Front-right apt: bed, crib & toys, sofa
    ip.bed(model, "bed_fr", in_h - 1.5, in_h - 1.5, rot_z=-math.pi * 0.5, blanket="shirt_red")
    ip.crib_and_toys(model, "crib_fr", 4.8, in_h - 1.4, rot_z=math.pi)
    ip.sofa(model, "sofa_fr", in_h - 2.2, 5.8, rot_z=-math.pi * 0.5, color="tarp_blue")

    # Back-left apt: bed, bathroom, kitchenette
    ip.bed(model, "bed_bl", -in_h + 1.5, -in_h + 1.5, rot_z=math.pi * 0.5, blanket="cloth_olive")
    ip.bathroom_set(model, "bath_bl", -5.2, -in_h + 0.8)
    ip.sofa(model, "sofa_bl", -5.0, -1.5, rot_z=0.0)

    # Back-right apt (east of hatch x > 5.0): bed, coffee table
    ip.bed(model, "bed_br", in_h - 1.5, -in_h + 1.8, rot_z=-math.pi * 0.5, blanket="canvas_khaki")
    ip.coffee_table(model, "ct_br", in_h - 3.5, -in_h + 2.0)

    spawns = [(0.0, 5.0), (-6.8, 7.5), (6.5, 7.5), (-6.5, -4.5), (7.5, -3.5)]
    if size == 32:
        spawns.append((0.0, 10.5))
    _add_spawns(k, spawns)


def _build_camp(model, size):
    k = kit.Kit(model)
    _shell_surfaces(k, size, "concrete_grey", "concrete_stain")
    _stain_patches(k, size, 8)
    half = size * 0.5
    in_h = half - WALL_IN
    _, sy = _shaft_bounds(size)
    door_y = sy + 3.2

    # Barricades flanking the shaft door (1.8 m clear gap in the middle)
    ip.camp_barricade(model, "bar_l", -2.1, door_y + 2.2, rot_z=0.25, length=2.4)
    ip.camp_barricade(model, "bar_r", 2.1, door_y + 2.2, rot_z=-0.25, length=2.4)
    # Perimeter barricades
    ip.camp_barricade(model, "bar_w", -in_h + 3.0, 1.5, rot_z=math.pi * 0.5, length=3.0)
    ip.camp_barricade(model, "bar_e", in_h - 3.0, 2.0, rot_z=-math.pi * 0.5, length=3.0)

    # Sleeping / living area (west wing)
    for i, (mx, my, col) in enumerate(((-6.5, 5.5, "cloth_olive"), (-8.2, 5.2, "tarp_blue"),
                                       (-6.2, 2.5, "canvas_khaki"), (-8.0, -2.5, "shirt_red"))):
        ip.mattress_bed(model, mx, my, rot_z=0.15 * i, bag_col=col)

    # Bodies under sheets
    ip.body_under_sheet(model, -6.5, -5.5, rot_z=0.4)
    ip.body_under_sheet(model, 7.2, -5.2, rot_z=-0.3)

    # Supply pile & camp stove
    ip.supply_pile(model, "sup_1", -3.8, 6.2, rot_z=0.3)
    ip.supply_pile(model, "sup_2", 6.8, 5.5, rot_z=-0.5)

    # Radio / planning table
    ip.radio_table(model, "radio_tbl", 1.8, 6.8, rot_z=0.1)

    # "HELP" painted on front (+Y) windows
    ip.help_sign(model, -4.0, in_h - 0.12, 1.55, rot_z=math.pi, color="red_paint")
    ip.help_sign(model, 4.0, in_h - 0.12, 1.55, rot_z=math.pi, color="white")

    spawns = [(-4.2, 1.5), (0.0, 4.5), (5.5, 1.8), (-5.0, -4.5), (7.0, -2.0)]
    if size == 32:
        spawns.append((9.5, 8.0))
    _add_spawns(k, spawns)


BUILDERS = {"office": _build_office, "flat": _build_flat, "camp": _build_camp}


def _render_eye_preview(name, size):
    """Eye-level shot from just outside the shaft door looking into the floor."""
    if name in bpy.data.objects:
        bpy.data.objects[name].hide_render = False
    _, sy = _shaft_bounds(size)
    door_y = sy + 3.2
    eye = Vector((0.0, door_y + 0.4, 1.65))
    target = Vector((1.2, door_y + 7.5, 1.15))

    scene = bpy.context.scene
    for old in ("PreviewCamera", "PreviewSun"):
        if old in bpy.data.objects:
            bpy.data.objects.remove(bpy.data.objects[old], do_unlink=True)
    cam_data = bpy.data.cameras.new("PreviewCamera")
    cam_data.angle = math.radians(65)
    cam = bpy.data.objects.new("PreviewCamera", cam_data)
    scene.collection.objects.link(cam)
    cam.location = eye
    cam.rotation_euler = (target - eye).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam

    sun_data = bpy.data.lights.new("PreviewSun", "SUN")
    sun_data.energy = 3.0
    sun = bpy.data.objects.new("PreviewSun", sun_data)
    scene.collection.objects.link(sun)
    sun.rotation_euler = (math.radians(45), 0.0, math.radians(-25))

    path = os.path.join(common.PREVIEW_DIR, f"{name}_eye.png")
    scene.render.resolution_x = common.PREVIEW_SIZE
    scene.render.resolution_y = common.PREVIEW_SIZE
    scene.render.filepath = path
    common._setup_workbench(scene)
    bpy.ops.render.render(write_still=True)
    print(f"preview {name}_eye -> {path}")


def build_all():
    results = []
    for kind, fn in BUILDERS.items():
        for size in (24, 32):
            name = f"tw_interior_{kind}_{size}"
            ok = common.run_model(name, "buildings", BUDGET[size],
                                  lambda m, f=fn, s=size: f(m, s),
                                  grip=(0.0, 0.0, 0.0), top_preview=True, view=kit.FRONT_VIEW)
            _render_eye_preview(name, size)
            results.append((name, ok))
    return results


if __name__ == "__main__":
    out = build_all()
    failed = [n for n, ok in out if not ok]
    for n, ok in out:
        print(f"interiors: {n} {'OK' if ok else 'FAILED'}")
    sys.exit(1 if failed else 0)
