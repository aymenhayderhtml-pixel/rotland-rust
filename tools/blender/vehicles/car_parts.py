"""Geometry for the beat-up hatchback (V11-08). Pure bmesh: no export here.

Each part is (name, bmesh, location). Visual parts keep their own origin, so code can
spin or steer the wheels. Coordinates are metres; the origin sits on the ground between
the axles. Front is +Y in Blender (Godot -Z), left is -X, right is +X.
Contract: docs/VEHICLE_CONTRACT.md.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
import bmesh  # noqa: E402
from mathutils import Vector  # noqa: E402
import common  # noqa: E402

WHEEL_RADIUS = 0.30
WHEEL_HALF_WIDTH = 0.11
WHEEL_X = 0.72  # track centre, +-X
WHEEL_Y = 1.22  # axle positions, +-Y (wheelbase 2.44 m)
BODY_HALF_WIDTH = 0.80
BODY_HALF_LENGTH = 1.90
SILL_Z = 0.26  # bottom of the body = ground clearance
BELT_Z = 0.84  # top of the lower body = bottom of the cabin
ROOF_Z = 1.42
DOOR_HINGE = (-BODY_HALF_WIDTH, 0.95, 0.55)  # front edge of the driver door, mid height

# Empties exported as Godot Node3D markers (name, location)
EMPTIES = [
    ("seat_driver", (-0.32, 0.20, 0.50)),
    ("exit_L", (-2.00, 0.50, 0.0)),
    ("trunk", (0.0, -2.20, 0.60)),
    ("headlight_L", (-0.55, 1.93, 0.66)),
    ("headlight_R", (0.55, 1.93, 0.66)),
    ("exhaust", (0.30, -1.97, 0.20)),
]


def _loft(bm, bottom, top, side_colors, top_color, bottom_color=None):
    """Closed shape between a bottom and a top quad (4 corners each, same order)."""
    centre = Vector((0.0, 0.0, 0.0))
    for p in bottom + top:
        centre += Vector(p) / 8.0
    for i in range(4):
        j = (i + 1) % 4
        quad = [bottom[i], bottom[j], top[j], top[i]]
        mid = sum((Vector(p) for p in quad), Vector((0.0, 0.0, 0.0))) / 4.0
        common.add_quad(bm, quad, outward=mid - centre, color=side_colors[i])
    common.add_quad(bm, top, outward=(0.0, 0.0, 1.0), color=top_color)
    if bottom_color:
        common.add_quad(bm, bottom, outward=(0.0, 0.0, -1.0), color=bottom_color)


def _body(body_color):
    """Lower body, cabin with glass sides, bumpers, lights, and a rust patch on the right side."""
    lower = common.make_box((2 * BODY_HALF_WIDTH, 2 * BODY_HALF_LENGTH, BELT_Z - SILL_Z),
                            loc=(0.0, 0.0, (SILL_Z + BELT_Z) / 2), color=body_color)
    cabin = common.new_bm()
    _loft(cabin,
          bottom=[(-0.70, 0.90, BELT_Z), (0.70, 0.90, BELT_Z), (0.66, -1.85, BELT_Z), (-0.66, -1.85, BELT_Z)],
          top=[(-0.62, 0.30, ROOF_Z), (0.62, 0.30, ROOF_Z), (0.60, -1.20, ROOF_Z), (-0.60, -1.20, ROOF_Z)],
          side_colors=["glass_tint"] * 4, top_color=body_color)
    bumpers = [common.make_box((2 * BODY_HALF_WIDTH + 0.02, 0.06, 0.14), loc=(0.0, s * 1.93, 0.33), color="metal")
               for s in (1, -1)]
    lights = [common.make_box((0.22, 0.02, 0.12), loc=(x, 1.915, 0.66), color="white") for x in (-0.55, 0.55)]
    lights += [common.make_box((0.20, 0.02, 0.12), loc=(x, -1.915, 0.66), color="red_paint") for x in (-0.60, 0.60)]
    # Rust decal sits on the right side: make it a thin closed box so no open boundary edges exist,
    # offset by ~3 mm outside body surface (x=0.80) to x=0.803 to eliminate z-fighting (REND-003)
    rust = common.make_box((0.006, 0.50, 0.18), loc=(0.803, -1.05, 0.41), color="rust")
    # Detail (visual plan step 7/8): grille, plates, mirrors, wheel arches, door seams, handle,
    # side trim and roof rails - ensure all parts touch and sit flush on the body without floating
    extra = [common.make_box((0.9, 0.03, 0.2), loc=(0.0, 1.905, 0.55), color="gun_black")]
    for x in (-0.3, -0.15, 0.0, 0.15, 0.3):
        extra.append(common.make_box((0.02, 0.035, 0.18), loc=(x, 1.91, 0.55), color="chrome"))
    extra += [common.make_box((0.34, 0.02, 0.11), loc=(0.0, s * 1.965, 0.38), color="white") for s in (1, -1)]
    for sx in (-1, 1):
        # Mirror mount and mirror glass: sits flush on the cabin side
        extra.append(common.make_box((0.08, 0.06, 0.08), loc=(sx * (BODY_HALF_WIDTH + 0.03), 0.82, 0.92),
                                     color=body_color))
        extra.append(common.make_box((0.015, 0.05, 0.06), loc=(sx * (BODY_HALF_WIDTH + 0.07), 0.82, 0.92),
                                     color="glass_tint"))
        for wy in (WHEEL_Y, -WHEEL_Y):
            # Kept above the ground line: the car's lowest point must stay at its origin (ModelsOriginAtGround)
            arch = common.make_frustum(WHEEL_RADIUS + 0.05, WHEEL_RADIUS + 0.05, -0.06, 0.06, segments=12,
                                       color="tire_black")
            common.transform(arch, rot=(0.0, math.pi / 2, 0.0), loc=(sx * (BODY_HALF_WIDTH - 0.02), wy, WHEEL_RADIUS + 0.06))
            extra.append(arch)
        # Side trim strip: sits touching the body surface (width 0.012, centered at BODY_HALF_WIDTH + 0.004)
        extra.append(common.make_box((0.012, 3.0, 0.04), loc=(sx * (BODY_HALF_WIDTH + 0.004), 0.0, 0.5), color="chrome"))
        for y in (0.95, -0.05, -0.95):
            extra.append(common.make_box((0.012, 0.015, BELT_Z - SILL_Z - 0.04), loc=(sx * (BODY_HALF_WIDTH + 0.004), y,
                                                                                     (SILL_Z + BELT_Z) / 2), color="gun_black"))
        # Door handle
        extra.append(common.make_box((0.016, 0.12, 0.03), loc=(sx * (BODY_HALF_WIDTH + 0.006), 0.2, 0.72), color="chrome"))
        # Roof rails: resting solidly on the roof surface
        extra.append(common.make_box((0.04, 1.3, 0.04), loc=(sx * 0.5, -0.45, ROOF_Z + 0.02), color="gun_black"))
    return common.combine([lower, cabin, rust] + bumpers + lights + extra)


def _door(body_color):
    """Driver door panel, origin at its front (hinge) edge. Sits just outside the left side."""
    return common.make_box((0.03, 0.95, BELT_Z - SILL_Z), loc=(-0.015, -0.475, 0.0), color=body_color)


def _wheel():
    """Tyre with a chrome hub cap on each side. Origin at the wheel centre, axle along X."""
    tyre = common.make_frustum(WHEEL_RADIUS, WHEEL_RADIUS, -WHEEL_HALF_WIDTH, WHEEL_HALF_WIDTH,
                               segments=16, color="tire_black")
    hub = common.make_frustum(0.17, 0.17, -WHEEL_HALF_WIDTH - 0.01, WHEEL_HALF_WIDTH + 0.01,
                              segments=12, color="chrome")
    # Rim lip, a dark centre and five lug nuts on each side
    lip = common.make_frustum(0.19, 0.19, -WHEEL_HALF_WIDTH - 0.005, WHEEL_HALF_WIDTH + 0.005, segments=12, color="metal")
    centre = common.make_frustum(0.05, 0.05, -WHEEL_HALF_WIDTH - 0.02, WHEEL_HALF_WIDTH + 0.02, segments=8, color="gun_black")
    parts = [tyre, lip, hub, centre]
    for k in range(5):
        a = 2.0 * math.pi * k / 5
        # Lug nuts sit proudly outside hub cap (half-width WHEEL_HALF_WIDTH + 0.012 vs hub 0.010, REND-003)
        parts.append(common.make_box((0.025, 0.025, 2 * (WHEEL_HALF_WIDTH + 0.012)),
                                     loc=(math.cos(a) * 0.1, math.sin(a) * 0.1, 0.0), color="gun_black"))
    for bm in parts:
        common.transform(bm, rot=(0.0, math.pi / 2, 0.0))  # cylinder axis Z -> X
    return common.combine(parts)


def build_parts(body_color):
    """All visual parts plus the collider. Returns a list of (name, bmesh, location)."""
    wheel_points = {"wheel_FL": (-WHEEL_X, WHEEL_Y), "wheel_FR": (WHEEL_X, WHEEL_Y),
                    "wheel_RL": (-WHEEL_X, -WHEEL_Y), "wheel_RR": (WHEEL_X, -WHEEL_Y)}
    parts = [("body", _body(body_color), (0.0, 0.0, 0.0)),
             ("door_L", _door(body_color), DOOR_HINGE)]
    for name, (x, y) in wheel_points.items():
        parts.append((name, _wheel(), (x, y, WHEEL_RADIUS)))
    collision = common.make_box((2 * (BODY_HALF_WIDTH - 0.02), 2 * (BODY_HALF_LENGTH - 0.02), ROOF_Z - SILL_Z),
                                loc=(0.0, 0.0, (SILL_Z + ROOF_Z) / 2), color="white")
    parts.append(("collision-convcolonly", collision, (0.0, 0.0, 0.0)))
    for _, bm, _ in parts:
        bmesh.ops.triangulate(bm, faces=list(bm.faces))
    return parts
