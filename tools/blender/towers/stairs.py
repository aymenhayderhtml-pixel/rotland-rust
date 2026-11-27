"""R43 stairwell modules: tw_stair_flight, _b, _z, tw_stair_open, tw_stair_top.

Origin = shaft centre on the floor landing (z = 0 is the landing top).
Blender +Y = game -Z (front, where the shaft door sits).
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "traces"))

import bpy  # noqa: E402
from mathutils import Vector  # noqa: E402

import common  # noqa: E402
import graffiti_font  # noqa: E402
import jam_util  # noqa: E402
import kit  # noqa: E402

graffiti_font.FONT.setdefault("X", [(0, 0, 1, 1), (0, 1, 1, 0)])

BUDGET = 9000
SHAFT_W = 4.4  # outer X: -2.2..+2.2 (R43-common.md)
SHAFT_D = 6.4  # outer Y: -3.2..+3.2 (R43-common.md)
WALL_T = 0.20  # wall thickness (R43-common.md)
STOREY_H = 3.80  # floor-to-floor height (R43-common.md)
MID_Z = 1.90  # mid landing height (R43-common.md)
SLAB_T = 0.18  # landing slab thickness (underside down from top)
DOOR_W = 1.40  # x -0.7..+0.7 (R43-common.md)
DOOR_H = 2.20  # z 0..2.2 (R43-common.md)
STEPS_PER_FLIGHT = 10  # 0.19 m risers over 1.90 m


def _add_walls(k, door_open, top_only=False):
    """Builds the 4 concrete shaft walls (two-tone with 1.2 m stripe) + colliders."""
    hx = SHAFT_W / 2.0  # 2.2
    hy = SHAFT_D / 2.0  # 3.2
    ix = hx - WALL_T  # 2.0
    iy = hy - WALL_T  # 3.0
    hw = DOOR_W / 2.0  # 0.7

    # Colliders: 4 outer walls, front (+Y) split around door gap
    k.collider((-hx, -hy, 0.0), (hx, -iy, STOREY_H), "wall_back")
    k.collider((-hx, -iy, 0.0), (-ix, iy, STOREY_H), "wall_left")
    k.collider((ix, -iy, 0.0), (hx, iy, STOREY_H), "wall_right")
    k.collider((-hx, iy, 0.0), (-hw, hy, STOREY_H), "wall_front_l")
    k.collider((hw, iy, 0.0), (hx, hy, STOREY_H), "wall_front_r")
    k.collider((-hw, iy, DOOR_H), (hw, hy, STOREY_H), "wall_front_top")

    # Visual walls: exterior grey, interior two-tone (lower concrete_grey, stripe concrete_stain, upper rock_light)
    def wall_band(z0, z1, col):
        k.box((-hx, -hy, z0), (hx, -iy, z1), col)
        k.box((-hx, -iy, z0), (-ix, iy, z1), col)
        k.box((ix, -iy, z0), (hx, iy, z1), col)
        k.box((-hx, iy, z0), (-hw, hy, z1), col)
        k.box((hw, iy, z0), (hx, hy, z1), col)
        if z1 > DOOR_H:
            k.box((-hw, iy, max(z0, DOOR_H)), (hw, hy, z1), col)

    wall_band(0.0, 1.15, "concrete_grey")
    wall_band(1.15, 1.25, "concrete_stain")
    wall_band(1.25, STOREY_H, "rock_light")

    # Exterior grime/scuff bands + interior cracked plaster patches
    k.box((-hx - 0.01, -hy - 0.01, 0.0), (hx + 0.01, hy + 0.01, 0.14), "concrete_stain")
    for px, py, pz, pw, ph in [
        (-ix, -0.6, 0.65, 0.7, 0.45),
        (ix - 0.01, 0.4, 2.35, 0.65, 0.50),
        (-0.8, -iy, 2.25, 0.9, 0.40),
    ]:
        if abs(px) >= ix - 0.02:
            k.box((px, py - pw / 2, pz), (px + 0.015, py + pw / 2, pz + ph), "concrete_stain")
        else:
            k.box((px - pw / 2, py, pz), (px + pw / 2, py + 0.015, pz + ph), "concrete_stain")

    # Standpipe running continuously up the (-X, -Y) back-left corner
    px, py = -ix + 0.10, -iy + 0.10
    k.model.add(common.make_frustum(0.045, 0.045, 0.0, STOREY_H, segments=6, center=(px, py), color="red_paint"))
    for cz in (0.06, 1.90, STOREY_H - 0.06):
        k.model.add(common.make_frustum(0.065, 0.065, cz - 0.03, cz + 0.03, segments=6, center=(px, py), color="metal"))
    if not top_only:
        # Valve wheel at z = 1.25 (on floor/mid level)
        k.box((px - 0.02, py, 1.25), (px + 0.08, py + 0.10, 1.29), "metal")
        k.model.add(common.make_frustum(0.08, 0.08, 1.24, 1.30, segments=8, center=(px + 0.06, py + 0.10), color="red_paint"))

    # Emergency light box above the door lintel (inside)
    k.box((-0.20, iy - 0.08, 2.65), (0.20, iy, 2.80), "rock_dark")
    for sx in (-0.13, 0.13):
        k.box((sx - 0.04, iy - 0.11, 2.68), (sx + 0.04, iy - 0.06, 2.77), "white")

    # Fire hose cabinet on the -X landing wall on open/jammed variants
    if not door_open and not top_only:
        k.box((-ix, 1.95, 0.90), (-ix + 0.14, 2.55, 1.65), "red_paint")
        k.box((-ix + 0.13, 2.04, 1.05), (-ix + 0.15, 2.46, 1.50), "rock_dark")
        k.box((-ix + 0.14, 2.48, 1.22), (-ix + 0.16, 2.52, 1.34), "metal")


def _add_door(k, jammed, has_exit_sign):
    """Door frame + closed/open steel leaf on the +Y wall (x -0.7..+0.7, z 0..2.2)."""
    iy = SHAFT_D / 2.0 - WALL_T  # 3.0
    hy = SHAFT_D / 2.0  # 3.2
    hw = DOOR_W / 2.0  # 0.7

    # Steel frame
    k.box((-hw, iy, 0.0), (-hw + 0.05, hy, DOOR_H), "gun_black")
    k.box((hw - 0.05, iy, 0.0), (hw, hy, DOOR_H), "gun_black")
    k.box((-hw, iy, DOOR_H - 0.05), (hw, hy, DOOR_H), "gun_black")

    if jammed:
        # Shut steel door + collider
        k.solid((-hw + 0.05, iy + 0.06, 0.0), (hw - 0.05, iy + 0.13, DOOR_H - 0.05), "metal", base="door")
        k.box((hw - 0.18, iy + 0.02, 0.96), (hw - 0.10, iy + 0.07, 1.06), "gun_black")
        # Wedged plank & broken chair leg against the inside of the door
        plank = common.make_box((0.12, 0.04, 1.35), color="wood_crate")
        common.transform(plank, rot=(0.55, 0.0, 0.18), loc=(-0.15, iy - 0.34, 0.52))
        k.model.add(plank)
        bar = common.make_box((1.15, 0.05, 0.08), color="bark_dark")
        common.transform(bar, rot=(0.0, 0.12, 0.04), loc=(0.05, iy + 0.03, 0.92))
        k.model.add(bar)
    else:
        # Swung open ~100 deg inward around x=+0.65 (out of Flight A's path)
        leaf = common.make_box((DOOR_W - 0.10, 0.06, DOOR_H - 0.05), color="metal")
        common.transform(leaf, rot=(0.0, 0.0, 0.0))
        # Pivot at (+0.65, iy + 0.06): swing 100 deg toward -Y/+X
        common.transform(leaf, loc=(-(DOOR_W - 0.10) / 2.0, 0.0, (DOOR_H - 0.05) / 2.0))
        common.transform(leaf, rot=(0.0, 0.0, math.radians(100.0)), loc=(hw - 0.05, iy + 0.06, 0.0))
        k.model.add(leaf)
        # Wooden door wedge
        k.box((0.36, iy - 1.15, 0.0), (0.48, iy - 0.98, 0.05), "wood_crate")

    if has_exit_sign:
        # Green EXIT sign box above the door (inside, -Y facing)
        k.box((-0.34, iy - 0.08, DOOR_H + 0.08), (0.34, iy, DOOR_H + 0.32), "gun_black")
        k.box((-0.30, iy - 0.09, DOOR_H + 0.11), (0.30, iy - 0.07, DOOR_H + 0.29), "leaf_mid")
        for bm in graffiti_font.text_bms("EXIT", 0.0, DOOR_H + 0.14, 0.08, 0.12, 0.0, "white", flat=True):
            common.transform(bm, rot=(0.0, 0.0, math.pi), loc=(0.0, iy - 0.095, 0.0))
            k.model.add(bm)


def _add_flight(k, x0, x1, y_start, y_end, z_start, z_end, rail_x, wall_rail_x):
    """Builds one flight of 10 visual steps + 1 smooth ramp collider + steel handrails."""
    # Ramp collider: top surface on line (y_start, z_start) -> (y_end, z_end)
    cx = (x0 + x1) / 2.0
    width = abs(x1 - x0)
    k.model.add_collider(
        k._label("ramp"),
        kit._plane_bm((cx, y_start, z_start), (cx, y_end, z_end), (1.0, 0.0, 0.0), width, 0.16, "white"),
    )

    # Visual steps with nosing
    dy = (y_end - y_start) / STEPS_PER_FLIGHT
    dz = (z_end - z_start) / STEPS_PER_FLIGHT
    sign_y = 1.0 if y_end > y_start else -1.0
    for i in range(STEPS_PER_FLIGHT):
        sy0 = y_start + i * dy
        sy1 = y_start + (i + 1) * dy
        sz_top = z_start + (i + 1) * dz
        sz_bot = max(z_start - 0.14, z_start + i * dz - 0.14)
        ylo, yhi = min(sy0, sy1), max(sy0, sy1)
        k.box((x0, ylo, sz_bot), (x1, yhi, sz_top), "concrete_grey")
        # Step nosing lip
        nose_y0 = sy0 - 0.02 * sign_y
        nose_y1 = sy0 + 0.04 * sign_y
        k.box((x0, min(nose_y0, nose_y1), sz_top - 0.03),
              (x1, max(nose_y0, nose_y1), sz_top), "concrete_stain")

    # Open-side steel handrail (posts + top rail + mid rail)
    rail_h = 0.92
    for frac in (0.05, 0.35, 0.65, 0.95):
        py = y_start + (y_end - y_start) * frac
        pz = z_start + (z_end - z_start) * frac
        k.box((rail_x - 0.025, py - 0.025, pz), (rail_x + 0.025, py + 0.025, pz + rail_h), "metal")
    for h_off, thick, col in ((rail_h, 0.04, "metal"), (rail_h * 0.5, 0.025, "rust_dark")):
        a = (rail_x, y_start, z_start + h_off)
        b = (rail_x, y_end, z_end + h_off)
        k.model.add(kit._plane_bm(a, b, (1.0, 0.0, 0.0), thick, thick, col))

    # Wall-side handrail
    a = (wall_rail_x, y_start, z_start + rail_h)
    b = (wall_rail_x, y_end, z_end + rail_h)
    k.model.add(kit._plane_bm(a, b, (1.0, 0.0, 0.0), 0.035, 0.035, "metal"))


def _add_clutter(k, with_body):
    """Rubble on Flight A's lower steps (<0.08 m), blood smear, backpack, papers, optional body."""
    # Low rubble on Flight A (x -2.0..-0.1, y +1.6..-1.6, kept <= 0.07 m above ramp)
    for rx, ry, rz, col in [
        (-1.75, 1.35, 0.22, "rock_dark"),
        (-1.60, 1.10, 0.36, "concrete_stain"),
        (-0.35, 1.42, 0.18, "brick_red"),
    ]:
        chunk = common.make_box((0.18, 0.14, 0.06), color=col)
        common.transform(chunk, rot=(0.1, -0.1, 0.4), loc=(rx, ry, rz))
        k.model.add(chunk)

    # Blood smear on floor landing & lower step
    k.box((-1.35, 1.72, 0.002), (-0.85, 2.25, 0.006), "blood_dark")
    k.box((-1.25, 1.50, 0.192), (-0.95, 1.62, 0.196), "blood_dark")

    # Dropped backpack against the -X wall on the floor landing (Sheet A/C colors only)
    k.box((-1.88, 1.85, 0.0), (-1.52, 2.28, 0.20), "bark_dark")
    k.box((-1.85, 1.90, 0.20), (-1.55, 2.24, 0.26), "wood_crate")
    k.box((-1.62, 1.98, 0.05), (-1.50, 2.15, 0.18), "rock_dark")

    # Scattered papers
    for px, py, rot in [(-0.4, 2.35, 0.3), (-0.15, 2.15, -0.5), (0.55, 2.45, 0.9)]:
        p = common.make_box((0.20, 0.26, 0.004), color="bone_white")
        common.transform(p, rot=(0.0, 0.0, rot), loc=(px, py, 0.004))
        k.model.add(p)

    if with_body:
        # Low-poly slumped body on the mid landing (-X/-Y corner, z = 1.90)
        bx, by, bz = -1.45, -2.55, MID_Z
        k.model.add(common.make_icosphere(subdiv=1, radius=(0.22, 0.15, 0.14),
                                          loc=(bx, by, bz + 0.14), color="shirt_blue"))
        k.model.add(common.make_icosphere(subdiv=1, radius=(0.11, 0.11, 0.11),
                                          loc=(bx + 0.06, by + 0.14, bz + 0.24), color="skin_pale"))
        k.box((bx - 0.12, by + 0.10, bz), (bx - 0.02, by + 0.55, bz + 0.10), "pants_denim")
        k.box((bx + 0.04, by + 0.10, bz), (bx + 0.14, by + 0.52, bz + 0.10), "pants_denim")
        k.box((bx - 0.14, by - 0.08, bz + 0.002), (bx + 0.22, by + 0.20, bz + 0.006), "blood_dark")


def build_stair(model, jammed=True, clutter=False, body=False, exit_sign=False, top_only=False,
                hide_right_wall=False):
    """Shared builder for all 5 stairwell variants."""
    k = kit.Kit(model)
    ix = SHAFT_W / 2.0 - WALL_T  # 2.0
    iy = SHAFT_D / 2.0 - WALL_T  # 3.0

    if hide_right_wall:
        # Cut-away preview helper: omit the +X wall so the interior reads clearly
        hx = SHAFT_W / 2.0
        hy = SHAFT_D / 2.0
        hw = DOOR_W / 2.0
        for z0, z1, col in ((0.0, 1.15, "concrete_grey"), (1.15, 1.25, "concrete_stain"), (1.25, STOREY_H, "rock_light")):
            k.box((-hx, -hy, z0), (hx, -iy, z1), col)
            k.box((-hx, -iy, z0), (-ix, iy, z1), col)
            k.box((-hx, iy, z0), (-hw, hy, z1), col)
            k.box((hw, iy, z0), (hx, hy, z1), col)
            if z1 > DOOR_H:
                k.box((-hw, iy, max(z0, DOOR_H)), (hw, hy, z1), col)
        px, py = -ix + 0.10, -iy + 0.10
        k.model.add(common.make_frustum(0.045, 0.045, 0.0, STOREY_H, segments=6, center=(px, py), color="red_paint"))
    else:
        _add_walls(k, door_open=not jammed, top_only=top_only)

    _add_door(k, jammed=jammed, has_exit_sign=exit_sign)

    # Floor landing: y +1.6..+3.0, full width x -2.0..+2.0, top at z = 0.0
    k.solid((-ix, 1.6, -SLAB_T), (ix, iy, 0.0), "concrete_grey", base="floor_landing")

    if top_only:
        # Ceiling slab at 3.6..3.8 m + painted "ROOF ->" arrow pointing toward +X (the hatch side)
        k.solid((-SHAFT_W / 2.0, -SHAFT_D / 2.0, 3.6), (SHAFT_W / 2.0, SHAFT_D / 2.0, STOREY_H),
                "concrete_grey", base="ceiling", )
        # Painted "ROOF" + arrow on the -Y back wall facing +Y
        for bm in graffiti_font.text_bms("ROOF", 0.0, 1.75, 0.22, 0.34, -iy + 0.015, "red_paint", flat=True):
            k.model.add(bm)
        # Arrow pointing +X (from x=-0.6 to +0.7 at z=1.45)
        k.box((-0.6, -iy + 0.01, 1.41), (0.5, -iy + 0.02, 1.49), "red_paint")
        k.box((0.35, -iy + 0.01, 1.32), (0.65, -iy + 0.02, 1.58), "red_paint")
        return

    # Mid landing: y -3.0..-1.6, full width x -2.0..+2.0, top at z = 1.9
    k.solid((-ix, -iy, MID_Z - SLAB_T), (ix, -1.6, MID_Z), "concrete_grey", base="mid_landing")

    # Flight A: x -2.0..-0.1, rises y +1.6 -> -1.6, z 0 -> 1.9
    _add_flight(k, -ix, -0.1, 1.6, -1.6, 0.0, MID_Z, rail_x=-0.10, wall_rail_x=-ix + 0.04)

    # Flight B: x +0.1..+2.0, rises y -1.6 -> +1.6, z 1.9 -> 3.8
    _add_flight(k, 0.1, ix, -1.6, 1.6, MID_Z, STOREY_H, rail_x=0.10, wall_rail_x=ix - 0.04)

    if clutter:
        _add_clutter(k, with_body=body)


def _render_cutaway(name, kwargs):
    """Renders `<name>_cutaway.png` with the +X wall omitted so the interior is visible."""
    common.reset_scene()
    m = common.Model(name + "_cutaway", "buildings", BUDGET, grip=(0.0, 0.0, 0.0), view=(1.0, 0.35, 0.45))
    build_stair(m, hide_right_wall=True, **kwargs)
    visual = common.combine([bm for bm, _ in m._visual])
    obj = common._to_object(visual, name + "_cutaway", painted=True)
    common.render_preview(name + "_cutaway", obj, view=(1.0, 0.35, 0.45))


def _render_stack_preview():
    """Stacks 3 tw_stair_flight cut-aways vertically to verify rails, standpipe, and landing join."""
    common.reset_scene()
    objs = []
    for idx in range(3):
        m = common.Model(f"stack_{idx}", "buildings", BUDGET, grip=(0.0, 0.0, 0.0))
        build_stair(m, jammed=True, hide_right_wall=True)
        visual = common.combine([bm for bm, _ in m._visual])
        obj = common._to_object(visual, f"stack_{idx}", painted=True)
        obj.location.z = idx * STOREY_H
        objs.append(obj)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objs:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    joined = bpy.context.view_layer.objects.active
    joined.name = "stack_tw_stair_flight"
    common.render_preview("stack_tw_stair_flight", joined, view=(1.0, 0.35, 0.35))


def build_all():
    variants = [
        ("tw_stair_flight", dict(jammed=True)),
        ("tw_stair_flight_b", dict(jammed=True, clutter=True)),
        ("tw_stair_flight_z", dict(jammed=True, clutter=True, body=True)),
        ("tw_stair_open", dict(jammed=False, exit_sign=True)),
        ("tw_stair_top", dict(jammed=False, exit_sign=True, top_only=True)),
    ]
    results = []
    for name, kw in variants:
        def _fn(m, _kw=kw, _name=name):
            build_stair(m, **_kw)
            if _name == "tw_stair_flight_z":
                # Spawn empties 0.1 m above floor landing and mid landing
                m.add_empty("spawn_1", (0.0, 2.2, 0.1))
                m.add_empty("spawn_2", (0.4, -2.3, MID_Z + 0.1))

        ok = common.run_model(name, "buildings", BUDGET, _fn, grip=(0.0, 0.0, 0.0), view=(1.0, 1.0, 0.5))
        results.append((name, ok))
        _render_cutaway(name, kw)

    _render_stack_preview()
    return results


if __name__ == "__main__":
    failed = [n for n, ok in build_all() if not ok]
    if failed:
        print("STAIRS FAILED:", ", ".join(failed))
        sys.exit(1)
    print("STAIRS OK")
