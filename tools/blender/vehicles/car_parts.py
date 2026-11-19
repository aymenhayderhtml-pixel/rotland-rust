"""Geometry for the Rotland 4x4 Outdoor Survival SUV. Pure bmesh: no export here.

Split into animatable parts so Rust can independently steer/spin/bounce all 4 wheels,
swing open the driver door, and swing open the rear tailgate with its mounted spare tire.

Coordinates are metres in Blender space:
  +Y = forward (Godot/Rotland -Z)
  +X = right   (Godot/Rotland +X)
  -X = left    (Godot/Rotland -X, driver side)
  +Z = up      (Godot/Rotland +Y)
Origin sits on the ground at the centre between the axles.
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

# Core 4x4 dimensions (metres) — cleared for +-0.15 m suspension travel & +-0.5 rad steering
WHEEL_RADIUS = 0.34
WHEEL_HALF_WIDTH = 0.13
WHEEL_X = 0.78          # track centre +-X (track 1.56 m)
WHEEL_Y = 1.26          # axle centres +-Y (wheelbase 2.52 m)
BODY_HALF_WIDTH = 0.86  # main body width 1.72 m (flares out to 0.94 m)
BODY_HALF_LENGTH = 1.94
SILL_Z = 0.40           # body rocker sill ground clearance
ARCH_TOP_Z = 0.90       # wheel well ceiling (> 0.34 + 0.15 + 0.34 = 0.83 m max bump!)
BELT_Z = 1.00           # beltline / hood shoulder height
ROOF_Z = 1.62           # roof surface height
INNER_WELL_X = 0.48     # inner wheel well wall (< 0.78 - 0.28 steered envelope!)

DOOR_HINGE = (-0.84, 0.80, 0.72)      # front edge of driver door, mid height
TAILGATE_HINGE = (0.74, -1.90, 0.90)  # right-side vertical hinge of rear swing tailgate

WHEEL_POINTS = {
    "wheel_FL": (-WHEEL_X, WHEEL_Y),
    "wheel_FR": (WHEEL_X, WHEEL_Y),
    "wheel_RL": (-WHEEL_X, -WHEEL_Y),
    "wheel_RR": (WHEEL_X, -WHEEL_Y),
}

# Empties exported as glTF / pack Markers (name, Blender location)
EMPTIES = [
    ("seat_driver", (-0.36, 0.12, 0.72)),
    ("exit_L", (-2.05, 0.30, 0.0)),
    ("trunk", (0.0, -2.35, 0.75)),
    ("headlight_L", (-0.60, 1.98, 0.80)),
    ("headlight_R", (0.60, 1.98, 0.80)),
    ("taillight_L", (-0.79, -1.93, 0.84)),
    ("taillight_R", (0.79, -1.93, 0.84)),
    ("lightbar", (0.0, 0.38, 1.68)),
    ("exhaust", (0.46, -2.04, 0.34)),
    ("hood_smoke", (0.0, 1.45, 1.08)),
    ("door_hinge_L", DOOR_HINGE),
    ("tailgate_hinge", TAILGATE_HINGE),
    ("wheel_FL", (-WHEEL_X, WHEEL_Y, WHEEL_RADIUS)),
    ("wheel_FR", (WHEEL_X, WHEEL_Y, WHEEL_RADIUS)),
    ("wheel_RL", (-WHEEL_X, -WHEEL_Y, WHEEL_RADIUS)),
    ("wheel_RR", (WHEEL_X, -WHEEL_Y, WHEEL_RADIUS)),
]


def _box(size, loc=(0.0, 0.0, 0.0), rot=(0.0, 0.0, 0.0), color="metal"):
    """Closed outward-facing box with optional Euler XYZ rotation."""
    bm = common.make_box(size, loc=(0.0, 0.0, 0.0), color=color)
    if rot != (0.0, 0.0, 0.0) or loc != (0.0, 0.0, 0.0):
        common.transform(bm, loc=loc, rot=rot)
    return bm


def _closed_loft(bottom, top, side_colors, top_color, bottom_color):
    """Watertight shared-vertex hexahedron between bottom and top quads.

    Both `bottom` and `top` must be 4 corners ordered CCW when viewed from +Z:
    [back-left, back-right, front-right, front-left].
    `side_colors` order: [rear (-Y), right (+X), front (+Y), left (-X)].
    """
    bm = common.new_bm()
    layer = bm.faces.layers.int[common.LAYER]
    b = [bm.verts.new(p) for p in bottom]
    t = [bm.verts.new(p) for p in top]

    def make_face(verts, col_name):
        f = bm.faces.new(verts)
        f[layer] = common.palette.key_of(col_name)
        return f

    for i in range(4):
        j = (i + 1) % 4
        make_face([b[i], b[j], t[j], t[i]], side_colors[i])
    make_face([t[0], t[1], t[2], t[3]], top_color)
    make_face([b[3], b[2], b[1], b[0]], bottom_color)
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def _loft(bm, bottom, top, side_colors, top_color, bottom_color=None):
    """Closed shape between a bottom and a top quad (4 corners each, same order).

    Kept for tools/blender/props/car_wreck.py, which reuses the old hatchback cabin loft.
    New geometry should use `_closed_loft` (shared verts, watertight, recalculated normals).
    """
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


def _beam(p0, p1, radius, color="gun_black", segments=6):
    """Strut / tube / pillar connecting two 3D points `p0` -> `p1`."""
    v0 = Vector(p0)
    v1 = Vector(p1)
    delta = v1 - v0
    length = delta.length
    bm = common.make_frustum(radius, radius, -length * 0.5, length * 0.5, segments=segments, color=color)
    mid = (v0 + v1) * 0.5
    quat = Vector((0.0, 0.0, 1.0)).rotation_difference(delta.normalized())
    bmesh.ops.rotate(bm, cent=(0.0, 0.0, 0.0), matrix=quat.to_matrix(), verts=list(bm.verts))
    bmesh.ops.translate(bm, vec=mid, verts=list(bm.verts))
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def _fender_flare(sx, wy):
    """Polygonal off-road trapezoidal fender arch wrapping cleanly around a wheel well."""
    parts = []
    # Inner x overlaps slightly into body shoulder; outer x flares out to +-0.94
    x_center = sx * 0.875
    w = 0.13
    # 5 crisp segments forming a flat-top 4x4 arch (zero overlap with tire envelope!)
    segs = [
        ((wy - 0.44, 0.55), (0.07, 0.34), 0.0),
        ((wy - 0.33, 0.82), (0.07, 0.28), -0.78),
        ((wy, 0.935), (0.48, 0.07), 0.0),
        ((wy + 0.33, 0.82), (0.07, 0.28), 0.78),
        ((wy + 0.44, 0.55), (0.07, 0.34), 0.0),
    ]
    for (cy, cz), (sy, sz), rx in segs:
        parts.append(_box((w, sy, sz), loc=(x_center, cy, cz), rot=(rx, 0.0, 0.0), color="tire_black"))
    return parts


def _build_wheel_mesh(include_brake=True):
    """Chunky off-road wheel centered at (0,0,0), axle along X, outer decorative face on -X."""
    parts = []
    segs = 12
    hw = WHEEL_HALF_WIDTH
    r_outer = WHEEL_RADIUS - 0.015  # base carcass; tread lugs bring outer radius to WHEEL_RADIUS
    r_side = 0.23

    # 3-ring crowned tire profile along Z (then rotated Z -> X)
    bm_tire = common.new_bm()
    layer = bm_tire.faces.layers.int[common.LAYER]
    tire_key = common.palette.key_of("tire_black")
    z_rings = [(-hw, r_side), (-hw * 0.72, r_outer), (hw * 0.72, r_outer), (hw, r_side)]
    rings = []
    for z, rad in z_rings:
        ring = []
        for i in range(segs):
            a = 2.0 * math.pi * i / segs
            ring.append(bm_tire.verts.new((math.cos(a) * rad, math.sin(a) * rad, z)))
        rings.append(ring)

    for k in range(len(rings) - 1):
        r0, r1 = rings[k], rings[k + 1]
        for i in range(segs):
            j = (i + 1) % segs
            f = bm_tire.faces.new([r0[i], r0[j], r1[j], r1[i]])
            f[layer] = tire_key
    # End caps
    f_bot = bm_tire.faces.new(list(reversed(rings[0])))
    f_bot[layer] = tire_key
    f_top = bm_tire.faces.new(rings[-1])
    f_top[layer] = tire_key
    bmesh.ops.recalc_face_normals(bm_tire, faces=list(bm_tire.faces))
    parts.append(bm_tire)

    # 8 chunky off-road tread lugs wrapping around the tire crown
    num_lugs = 8
    for i in range(num_lugs):
        a = 2.0 * math.pi * i / num_lugs
        cx = math.cos(a) * (WHEEL_RADIUS - 0.012)
        cy = math.sin(a) * (WHEEL_RADIUS - 0.012)
        parts.append(_box((0.026, 0.11, hw * 1.65), loc=(cx, cy, 0.0), rot=(0.0, 0.0, a), color="tire_black"))

    # Outer deep-dish wheel rim on -Z side (becomes -X after rotation)
    rim_dish = common.make_frustum(0.21, 0.21, -hw - 0.004, hw * 0.4, segments=8, color="rock_dark")
    parts.append(rim_dish)

    # 5 sturdy star spokes on outer face (-Z)
    for k in range(5):
        a = 2.0 * math.pi * k / 5.0
        sx = math.cos(a) * 0.105
        sy = math.sin(a) * 0.105
        parts.append(_box((0.185, 0.048, 0.020), loc=(sx, sy, -hw - 0.015), rot=(0.0, 0.0, a), color="chrome"))

    # Locking hub cap protruding proudly on outer face
    hub = common.make_frustum(0.062, 0.052, -hw - 0.042, -hw - 0.004, segments=6, color="gun_black")
    parts.append(hub)

    # Inner brake rotor backing on +Z side (+X after rotation)
    if include_brake:
        brake = common.make_frustum(0.16, 0.16, hw - 0.01, hw + 0.02, segments=8, color="rock_dark")
        parts.append(brake)

    combined = common.combine(parts)
    # Rotate cylinder axis +Z -> +X so outer decorative face (-Z) faces -X (driver left side)
    common.transform(combined, rot=(0.0, math.pi / 2.0, 0.0))
    bmesh.ops.recalc_face_normals(combined, faces=list(combined.faces))
    return combined


def _body(body_color):
    """Main sculpted 4x4 body: frame, wheel wells, nose/hood, cabin, interior, bullbar, rack, snorkel."""
    parts = []

    # 1. Underbody ladder frame & center floor pan (stays strictly inside |x| <= 0.48 -> zero wheel clip!)
    parts.append(_box((2 * INNER_WELL_X, 3.68, 0.16), loc=(0.0, 0.0, 0.44), color="gun_black"))
    # Front & rear solid axles / differential housings for grounded 4x4 underside look
    for wy in (WHEEL_Y, -WHEEL_Y):
        parts.append(_box((1.16, 0.10, 0.10), loc=(0.0, wy, WHEEL_RADIUS + 0.02), color="gun_black"))
        parts.append(_box((0.24, 0.22, 0.20), loc=(0.0, wy, WHEEL_RADIUS + 0.02), color="rock_dark"))

    # 2. Inner wheel well liners & top shelves (sealed dark arches around all 4 wheels)
    for sx in (-1, 1):
        for wy in (WHEEL_Y, -WHEEL_Y):
            # Vertical inner liner wall at |x| = 0.48
            parts.append(_box((0.03, 0.88, 0.46), loc=(sx * INNER_WELL_X, wy, 0.67), color="tire_black"))
            # Top wheel well ceiling at z = ARCH_TOP_Z (0.90)
            parts.append(_box((0.39, 0.88, 0.03), loc=(sx * 0.665, wy, ARCH_TOP_Z), color="tire_black"))

    # 3. Sculpted front nose fascia (strictly ahead of front wheel well: y in [1.70, 1.96])
    parts.append(_closed_loft(
        bottom=[(-0.84, 1.70, 0.48), (0.84, 1.70, 0.48), (0.80, 1.96, 0.54), (-0.80, 1.96, 0.54)],
        top=[(-0.86, 1.70, BELT_Z), (0.86, 1.70, BELT_Z), (0.82, 1.94, 0.96), (-0.82, 1.94, 0.96)],
        side_colors=[body_color, body_color, body_color, body_color],
        top_color=body_color,
        bottom_color="gun_black",
    ))
    # Angled steel front skid plate protecting underside of nose
    parts.append(_box((0.92, 0.44, 0.04), loc=(0.0, 1.84, 0.44), rot=(0.42, 0.0, 0.0), color="metal"))

    # 4. Sculpted hood & upper front fenders (all above z = 0.91 -> clears full wheel compression!)
    parts.append(_closed_loft(
        bottom=[(-0.86, 0.78, 0.91), (0.86, 0.78, 0.91), (0.84, 1.72, 0.91), (-0.84, 1.72, 0.91)],
        top=[(-0.84, 0.78, 1.03), (0.84, 0.78, 1.03), (0.80, 1.93, 0.97), (-0.80, 1.93, 0.97)],
        side_colors=[body_color, body_color, body_color, body_color],
        top_color=body_color,
        bottom_color="tire_black",
    ))
    # Raised central hood power-dome + cooling louver vent
    parts.append(_closed_loft(
        bottom=[(-0.42, 0.84, 1.02), (0.42, 0.84, 1.02), (0.34, 1.82, 0.97), (-0.34, 1.82, 0.97)],
        top=[(-0.34, 0.88, 1.08), (0.34, 0.88, 1.08), (0.26, 1.76, 1.02), (-0.26, 1.76, 1.02)],
        side_colors=[body_color, body_color, body_color, body_color],
        top_color=body_color,
        bottom_color=body_color,
    ))
    parts.append(_box((0.46, 0.22, 0.02), loc=(0.0, 1.24, 1.065), rot=(-0.05, 0.0, 0.0), color="gun_black"))

    # 5. Center cabin lower body (between front & rear wheel wells: y in [-0.82, 0.82])
    # Cabin floor & rocker sills
    parts.append(_box((2 * (BODY_HALF_WIDTH - 0.015), 1.64, 0.14), loc=(0.0, 0.0, 0.47), color=body_color))
    # Heavy-duty side rock-slider steps (left & right)
    for sx in (-1, 1):
        parts.append(_box((0.12, 1.56, 0.06), loc=(sx * 0.90, 0.0, 0.37), color="gun_black"))
        # Side trim moulding on fixed panels
    # Passenger (+X) full lower side wall (y = -0.82..0.82, z = 0.52..1.00)
    parts.append(_box((0.12, 1.64, 0.60), loc=(BODY_HALF_WIDTH - 0.06, 0.0, 0.70), color=body_color))
    # Passenger door seams, side rub strip, handles, and weathered rust patch
    for y_seam in (0.79, -0.10, -0.80):
        parts.append(_box((0.014, 0.018, 0.52), loc=(BODY_HALF_WIDTH + 0.005, y_seam, 0.72), color="gun_black"))
    parts.append(_box((0.016, 1.58, 0.055), loc=(BODY_HALF_WIDTH + 0.006, 0.0, 0.62), color="tire_black"))
    for y_h in (0.04, -0.66):
        parts.append(_box((0.022, 0.13, 0.032), loc=(BODY_HALF_WIDTH + 0.010, y_h, 0.88), color="chrome"))
    parts.append(_box((0.010, 0.48, 0.16), loc=(BODY_HALF_WIDTH + 0.004, -0.45, 0.50), color="rust"))

    # Driver (-X) side: REAR door fixed (y = -0.82..-0.10); FRONT door opening left hollow for animated `door_L`!
    parts.append(_box((0.12, 0.72, 0.60), loc=(-BODY_HALF_WIDTH + 0.06, -0.46, 0.70), color=body_color))
    parts.append(_box((0.016, 0.68, 0.055), loc=(-BODY_HALF_WIDTH - 0.006, -0.46, 0.62), color="tire_black"))
    parts.append(_box((0.022, 0.13, 0.032), loc=(-BODY_HALF_WIDTH - 0.010, -0.66, 0.88), color="chrome"))
    # Lower door sill threshold under the openable driver door
    parts.append(_box((0.10, 0.90, 0.06), loc=(-BODY_HALF_WIDTH + 0.075, 0.36, 0.435), color="gun_black"))

    # 6. Rear quarter panels & upper rear fenders (y in [-1.92, -0.82])
    for sx in (-1, 1):
        # Upper fender deck above rear wheel well (z = 0.91..1.02)
        parts.append(_box((0.38, 1.10, 0.11), loc=(sx * 0.67, -1.37, 0.965), color=body_color))
        # Rear corner body behind rear wheel well (y in [-1.92, -1.70], z = 0.48..1.02)
        parts.append(_box((0.38, 0.22, 0.54), loc=(sx * 0.67, -1.81, 0.75), color=body_color))
    # Rear cargo floor & tailgate sill beam
    parts.append(_box((1.52, 1.10, 0.12), loc=(0.0, -1.37, 0.54), color="gun_black"))
    parts.append(_box((1.68, 0.12, 0.12), loc=(0.0, -1.86, 0.54), color=body_color))
    # Fuel filler cap on left rear quarter
    parts.append(_box((0.016, 0.16, 0.14), loc=(-BODY_HALF_WIDTH - 0.005, -1.28, 0.96), color="gun_black"))
    # Subtle weathered rust patch on left rear corner
    parts.append(_box((0.010, 0.20, 0.14), loc=(-BODY_HALF_WIDTH - 0.004, -1.80, 0.56), color="rust"))

    # 7. Flared polygonal off-road fender arches on all 4 corners
    for sx in (-1, 1):
        for wy in (WHEEL_Y, -WHEEL_Y):
            parts.extend(_fender_flare(sx, wy))

    # 8. Low-poly cabin interior (visible through open driver door, tailgate, & glass)
    # Firewall + dashboard
    parts.append(_box((1.50, 0.34, 0.44), loc=(0.0, 0.62, 0.78), color="gun_black"))
    parts.append(_box((0.52, 0.08, 0.12), loc=(-0.36, 0.50, 0.98), color="pants_dark"))  # gauge hood
    # Steering column & octagonal steering wheel on driver left
    parts.append(_beam((-0.36, 0.56, 0.86), (-0.36, 0.34, 0.97), 0.025, color="gun_black", segments=6))
    wheel_ring = common.make_frustum(0.15, 0.15, -0.02, 0.02, segments=8, color="tire_black")
    common.transform(wheel_ring, loc=(-0.36, 0.34, 0.97), rot=(1.15, 0.0, 0.0))
    parts.append(wheel_ring)
    # Front bucket seats (driver & passenger)
    for sx in (-1, 1):
        parts.append(_box((0.46, 0.46, 0.12), loc=(sx * 0.36, 0.02, 0.60), color="leather"))
        parts.append(_box((0.44, 0.12, 0.56), loc=(sx * 0.36, -0.18, 0.90), rot=(-0.18, 0.0, 0.0), color="leather"))
        parts.append(_box((0.24, 0.09, 0.16), loc=(sx * 0.36, -0.24, 1.24), color="pants_dark"))
    # Rear bench seat + cargo divider
    parts.append(_box((1.38, 0.44, 0.14), loc=(0.0, -0.68, 0.62), color="leather"))
    parts.append(_box((1.38, 0.12, 0.48), loc=(0.0, -0.90, 0.90), rot=(-0.15, 0.0, 0.0), color="leather"))

    # 9. Cabin Greenhouse: Real A/B/C/D Pillars, Recessed Glass, and Bevelled Roof
    roof_x = 0.68
    belt_x = 0.83
    for sx in (-1, 1):
        # A-pillar (front windshield frame)
        parts.append(_beam((sx * belt_x, 0.78, BELT_Z), (sx * roof_x, 0.40, ROOF_Z), 0.042, color=body_color, segments=6))
        # B-pillar (between front and rear doors)
        parts.append(_beam((sx * belt_x, -0.10, BELT_Z), (sx * roof_x, -0.10, ROOF_Z), 0.046, color=body_color, segments=6))
        # C-pillar (behind rear door)
        parts.append(_beam((sx * belt_x, -0.92, BELT_Z), (sx * roof_x, -0.92, ROOF_Z), 0.052, color=body_color, segments=6))
        # D-pillar (sturdy rear corner)
        parts.append(_beam((sx * 0.80, -1.86, BELT_Z), (sx * roof_x, -1.74, ROOF_Z), 0.068, color=body_color, segments=6))
        # Upper door roof rail
        parts.append(_beam((sx * roof_x, 0.40, ROOF_Z), (sx * roof_x, -1.74, ROOF_Z), 0.040, color=body_color, segments=6))

    # Front & rear header beams across the roofline
    parts.append(_box((2 * roof_x, 0.12, 0.07), loc=(0.0, 0.38, ROOF_Z - 0.02), color=body_color))
    parts.append(_box((2 * roof_x, 0.12, 0.07), loc=(0.0, -1.74, ROOF_Z - 0.02), color=body_color))

    # Sculpted bevelled roof cap (watertight!)
    parts.append(_closed_loft(
        bottom=[(-roof_x, -1.78, ROOF_Z - 0.04), (roof_x, -1.78, ROOF_Z - 0.04),
                (roof_x, 0.42, ROOF_Z - 0.04), (-roof_x, 0.42, ROOF_Z - 0.04)],
        top=[(-0.62, -1.70, ROOF_Z + 0.03), (0.62, -1.70, ROOF_Z + 0.03),
             (0.62, 0.34, ROOF_Z + 0.03), (-0.62, 0.34, ROOF_Z + 0.03)],
        side_colors=[body_color, body_color, body_color, body_color],
        top_color=body_color,
        bottom_color="gun_black",
    ))

    # Recessed Windshield pane + two wiper blades
    parts.append(_closed_loft(
        bottom=[(-0.76, 0.74, BELT_Z + 0.02), (0.76, 0.74, BELT_Z + 0.02),
                (0.76, 0.77, BELT_Z + 0.02), (-0.76, 0.77, BELT_Z + 0.02)],
        top=[(-0.64, 0.39, ROOF_Z - 0.04), (0.64, 0.39, ROOF_Z - 0.04),
             (0.64, 0.42, ROOF_Z - 0.04), (-0.64, 0.42, ROOF_Z - 0.04)],
        side_colors=["glass_tint"] * 4,
        top_color="gun_black",
        bottom_color="gun_black",
    ))
    for wx in (-0.30, 0.22):
        parts.append(_box((0.40, 0.02, 0.02), loc=(wx, 0.78, BELT_Z + 0.05), rot=(0.0, 0.0, 0.14), color="gun_black"))

    # Recessed side windows:
    # Right (+X) side: front + rear + cargo quarter windows
    # Left (-X) side: rear + cargo quarter windows ONLY (front driver window is on `door_L`!)
    def side_glass(sx, y_back, y_front_bot, y_front_top):
        x_b_in, x_b_out = sx * 0.78, sx * 0.81
        x_t_in, x_t_out = sx * 0.65, sx * 0.68
        lo, hi = min(x_b_in, x_b_out), max(x_b_in, x_b_out)
        tlo, thi = min(x_t_in, x_t_out), max(x_t_in, x_t_out)
        return _closed_loft(
            bottom=[(lo, y_back, BELT_Z + 0.02), (hi, y_back, BELT_Z + 0.02),
                    (hi, y_front_bot, BELT_Z + 0.02), (lo, y_front_bot, BELT_Z + 0.02)],
            top=[(tlo, y_back, ROOF_Z - 0.04), (thi, y_back, ROOF_Z - 0.04),
                 (thi, y_front_top, ROOF_Z - 0.04), (tlo, y_front_top, ROOF_Z - 0.04)],
            side_colors=["glass_tint"] * 4,
            top_color="gun_black",
            bottom_color="gun_black",
        )

    parts.append(side_glass(1, -0.07, 0.73, 0.39))    # right front passenger window
    parts.append(side_glass(1, -0.88, -0.14, -0.14))  # right rear passenger window
    parts.append(side_glass(1, -1.72, -0.96, -0.96))  # right cargo quarter window
    parts.append(side_glass(-1, -0.88, -0.14, -0.14)) # left rear passenger window
    parts.append(side_glass(-1, -1.72, -0.96, -0.96)) # left cargo quarter window

    # Right-side mounted side mirror (left mirror lives on animated `door_L`)
    parts.append(_box((0.12, 0.04, 0.04), loc=(0.86, 0.66, 1.05), color="gun_black"))
    parts.append(_box((0.14, 0.09, 0.13), loc=(0.97, 0.65, 1.07), color="gun_black"))
    parts.append(_box((0.11, 0.014, 0.10), loc=(0.97, 0.60, 1.07), color="glass_tint"))

    # 10. Front Grille, Headlights, Indicators & Heavy Apocalypse Bullbar / Zombie Ram-Bumper
    # Recessed grille backing + 5 chrome vertical Jeep/4x4 slats
    parts.append(_box((0.88, 0.04, 0.24), loc=(0.0, 1.955, 0.78), color="gun_black"))
    for gx in (-0.32, -0.16, 0.0, 0.16, 0.32):
        parts.append(_box((0.032, 0.045, 0.21), loc=(gx, 1.965, 0.78), color="chrome"))
    # Dual headlights + amber corner indicators
    for sx in (-1, 1):
        parts.append(_box((0.26, 0.04, 0.17), loc=(sx * 0.60, 1.95, 0.80), color="chrome"))
        parts.append(_box((0.22, 0.045, 0.13), loc=(sx * 0.60, 1.962, 0.80), color="white"))
        parts.append(_box((0.11, 0.05, 0.11), loc=(sx * 0.77, 1.94, 0.80), color="carrot_orange"))

    # Heavy-duty steel off-road front bumper + winch box + shackles + front plate
    parts.append(_box((1.78, 0.18, 0.17), loc=(0.0, 2.02, 0.56), color="gun_black"))
    parts.append(_box((0.46, 0.16, 0.13), loc=(0.0, 2.05, 0.66), color="metal"))  # winch housing
    parts.append(_box((0.08, 0.06, 0.06), loc=(0.0, 2.13, 0.65), color="brass"))  # winch hook
    parts.append(_box((0.32, 0.02, 0.10), loc=(0.0, 2.115, 0.55), color="white"))  # front plate
    for sx in (-1, 1):
        # Tow shackle rings
        parts.append(_box((0.06, 0.09, 0.09), loc=(sx * 0.44, 2.12, 0.55), color="red_paint"))
        # Angled bullbar uprights + outer headlight brush-guard loops
        parts.append(_beam((sx * 0.34, 2.08, 0.62), (sx * 0.28, 2.10, 0.98), 0.028, color="gun_black", segments=6))
        parts.append(_beam((sx * 0.30, 2.10, 0.94), (sx * 0.82, 2.03, 0.90), 0.024, color="gun_black", segments=6))
        parts.append(_beam((sx * 0.82, 2.03, 0.90), (sx * 0.85, 2.01, 0.62), 0.024, color="gun_black", segments=6))
    # Bullbar top & mid crossbars
    parts.append(_beam((-0.28, 2.10, 0.98), (0.28, 2.10, 0.98), 0.028, color="gun_black", segments=6))
    parts.append(_beam((-0.32, 2.09, 0.80), (0.32, 2.09, 0.80), 0.024, color="gun_black", segments=6))

    # 11. Off-Road Raised Snorkel along Right A-pillar
    parts.append(_beam((0.86, 1.12, 0.96), (0.86, 0.78, 1.02), 0.042, color="gun_black", segments=6))
    parts.append(_beam((0.86, 0.78, 1.02), (0.74, 0.44, 1.58), 0.042, color="gun_black", segments=6))
    parts.append(_box((0.10, 0.14, 0.11), loc=(0.74, 0.46, 1.63), color="gun_black"))  # intake scoop

    # 12. Roof Survival Rack + Cargo Gear + 4-Pod LED Roof Lightbar
    rack_z = ROOF_Z + 0.09
    rack_x = 0.56
    rack_y0, rack_y1 = -1.56, 0.22
    # 6 mounting feet connecting roof cleanly to rack
    for sx in (-1, 1):
        for ry in (0.14, -0.68, -1.48):
            parts.append(_box((0.04, 0.06, 0.09), loc=(sx * rack_x, ry, ROOF_Z + 0.045), color="gun_black"))
        # Side rails (upper & lower basket rails)
        parts.append(_box((0.032, rack_y1 - rack_y0, 0.032), loc=(sx * rack_x, (rack_y0 + rack_y1) * 0.5, rack_z), color="gun_black"))
    # Front, rear & middle crossbars
    for ry in (rack_y1, -0.35, -0.95, rack_y0):
        parts.append(_box((2 * rack_x + 0.03, 0.035, 0.032), loc=(0.0, ry, rack_z - 0.02), color="gun_black"))

    # Survival Gear loaded inside roof rack:
    # Red fuel jerry can
    parts.append(_box((0.18, 0.36, 0.26), loc=(-0.36, -1.24, rack_z + 0.09), color="red_paint"))
    parts.append(_box((0.06, 0.06, 0.05), loc=(-0.36, -1.10, rack_z + 0.24), color="gun_black"))
    # Supply crate
    parts.append(_box((0.46, 0.48, 0.22), loc=(0.20, -1.20, rack_z + 0.07), color="wood_crate"))
    parts.append(_box((0.48, 0.05, 0.05), loc=(0.20, -1.20, rack_z + 0.18), color="metal"))
    # Rolled khaki bedroll / tarp + leather straps
    bedroll = common.make_frustum(0.11, 0.11, -0.44, 0.44, segments=8, color="canvas_khaki")
    common.transform(bedroll, loc=(0.0, -0.42, rack_z + 0.08), rot=(0.0, math.pi / 2.0, 0.0))
    parts.append(bedroll)
    for sx in (-0.26, 0.26):
        strap = common.make_frustum(0.116, 0.116, -0.025, 0.025, segments=8, color="leather")
        common.transform(strap, loc=(sx, -0.42, rack_z + 0.08), rot=(0.0, math.pi / 2.0, 0.0))
        parts.append(strap)

    # 4-Pod Off-Road LED Roof Lightbar (above windshield)
    parts.append(_box((1.06, 0.09, 0.07), loc=(0.0, 0.34, ROOF_Z + 0.06), color="gun_black"))
    for lx in (-0.36, -0.12, 0.12, 0.36):
        parts.append(_box((0.18, 0.03, 0.055), loc=(lx, 0.39, ROOF_Z + 0.06), color="white"))

    # 13. Rear Heavy Bumper, Multi-Element Taillights, Tow Hitch & Exhaust Pipe
    parts.append(_box((1.78, 0.16, 0.16), loc=(0.0, -1.96, 0.50), color="gun_black"))
    parts.append(_box((0.16, 0.12, 0.10), loc=(0.0, -2.05, 0.44), color="metal"))  # tow hitch
    for sx in (-1, 1):
        # Vertical taillight bezel on rear D-pillar corner
        parts.append(_box((0.14, 0.05, 0.36), loc=(sx * 0.79, -1.91, 0.84), color="gun_black"))
        parts.append(_box((0.10, 0.03, 0.16), loc=(sx * 0.79, -1.93, 0.92), color="red_paint"))     # tail/brake
        parts.append(_box((0.10, 0.03, 0.07), loc=(sx * 0.79, -1.93, 0.79), color="carrot_orange")) # amber turn
        parts.append(_box((0.10, 0.03, 0.07), loc=(sx * 0.79, -1.93, 0.71), color="white"))         # reverse
    # Angled exhaust pipe exiting under rear right bumper
    parts.append(_beam((0.42, -1.65, 0.39), (0.46, -2.03, 0.34), 0.042, color="metal", segments=6))

    return common.combine(parts)


def _door_left(body_color):
    """Hinged driver door (origin at DOOR_HINGE = (-0.84, 0.80, 0.72)).

    Spans y_local in [-0.90, 0.0] (back to B-pillar), z_local from -0.26 (sill) to +0.88 (roof).
    Includes window frame, recessed glass, chrome handle, rub strip, and mounted side mirror.
    """
    parts = []
    # Lower sculpted door panel
    parts.append(_box((0.08, 0.89, 0.58), loc=(0.02, -0.445, -0.01), color=body_color))
    # Inner dark door card
    parts.append(_box((0.02, 0.82, 0.44), loc=(0.065, -0.445, 0.02), color="gun_black"))
    # Side rub strip & chrome handle
    parts.append(_box((0.018, 0.86, 0.055), loc=(-0.024, -0.445, -0.10), color="tire_black"))
    parts.append(_box((0.024, 0.13, 0.032), loc=(-0.028, -0.76, 0.16), color="chrome"))

    # Upper window frame matching cabin A-pillar & B-pillar taper
    # Front rail (follows A-pillar from (0.01, -0.02, 0.28) to (0.15, -0.40, 0.88))
    parts.append(_beam((0.01, -0.02, 0.28), (0.15, -0.40, 0.88), 0.030, color=body_color, segments=6))
    # Rear rail (matches B-pillar from (0.01, -0.88, 0.28) to (0.15, -0.88, 0.88))
    parts.append(_beam((0.01, -0.88, 0.28), (0.15, -0.88, 0.88), 0.030, color=body_color, segments=6))
    # Top rail
    parts.append(_beam((0.15, -0.40, 0.88), (0.15, -0.88, 0.88), 0.030, color=body_color, segments=6))

    # Recessed driver door window glass
    parts.append(_closed_loft(
        bottom=[(0.01, -0.86, 0.30), (0.04, -0.86, 0.30),
                (0.04, -0.06, 0.30), (0.01, -0.06, 0.30)],
        top=[(0.14, -0.86, 0.85), (0.17, -0.86, 0.85),
             (0.17, -0.41, 0.85), (0.14, -0.41, 0.85)],
        side_colors=["glass_tint"] * 4,
        top_color="gun_black",
        bottom_color="gun_black",
    ))

    # Mounted left side mirror with bracket arm
    parts.append(_box((0.12, 0.04, 0.04), loc=(-0.03, -0.14, 0.33), color="gun_black"))
    parts.append(_box((0.14, 0.09, 0.13), loc=(-0.13, -0.15, 0.35), color="gun_black"))
    parts.append(_box((0.11, 0.014, 0.10), loc=(-0.13, -0.20, 0.35), color="glass_tint"))

    return common.combine(parts)


def _tailgate(body_color):
    """Hinged rear swing tailgate + mounted spare off-road tire (origin at TAILGATE_HINGE)."""
    parts = []
    # Origin at right hinge (0.74, -1.90, 0.90) -> extends left to x_local = -1.48
    # Lower swing gate body panel
    parts.append(_box((1.44, 0.08, 0.46), loc=(-0.74, 0.0, -0.10), color=body_color))
    # Rear plate & handle on left side of tailgate
    parts.append(_box((0.32, 0.02, 0.11), loc=(-1.14, -0.05, -0.14), color="white"))
    parts.append(_box((0.04, 0.03, 0.14), loc=(-1.38, -0.05, 0.02), color="chrome"))

    # Upper rear window frame + recessed rear glass (tapers forward slightly to match D-pillar)
    parts.append(_closed_loft(
        bottom=[(-1.42, -0.03, 0.13), (-0.06, -0.03, 0.13),
                (-0.06, 0.04, 0.13), (-1.42, 0.04, 0.13)],
        top=[(-1.32, 0.10, 0.67), (-0.16, 0.10, 0.67),
             (-0.16, 0.16, 0.67), (-1.32, 0.16, 0.67)],
        side_colors=["glass_tint", body_color, "glass_tint", body_color],
        top_color=body_color,
        bottom_color=body_color,
    ))
    # Side frame posts around rear glass
    parts.append(_beam((-1.40, 0.0, 0.13), (-1.31, 0.13, 0.67), 0.038, color=body_color, segments=6))
    parts.append(_beam((-0.08, 0.0, 0.13), (-0.17, 0.13, 0.67), 0.038, color=body_color, segments=6))
    parts.append(_beam((-1.31, 0.13, 0.67), (-0.17, 0.13, 0.67), 0.038, color=body_color, segments=6))

    # Heavy external spare-tire carrier bracket + mounted chunky spare wheel!
    parts.append(_box((0.26, 0.16, 0.24), loc=(-0.56, -0.10, 0.02), color="gun_black"))
    spare = _build_wheel_mesh(include_brake=False)
    # Rotate wheel so outer rim (-X) faces straight backwards (-Y)
    common.transform(spare, loc=(-0.56, -0.21, 0.04), rot=(0.0, 0.0, math.pi / 2.0), scale=(0.94, 0.94, 0.94))
    parts.append(spare)

    return common.combine(parts)


def build_body_and_collider(body_color):
    """Returns [(name, bmesh, location)] for the main car body + convex physics collider."""
    body = _body(body_color)
    collision = common.make_box(
        (2 * (BODY_HALF_WIDTH - 0.02), 2 * (BODY_HALF_LENGTH - 0.02), ROOF_Z - SILL_Z),
        loc=(0.0, 0.0, (SILL_Z + ROOF_Z) / 2.0),
        color="white",
    )
    parts = [
        ("body", body, (0.0, 0.0, 0.0)),
        ("collision-convcolonly", collision, (0.0, 0.0, 0.0)),
    ]
    for _, bm, _ in parts:
        bmesh.ops.triangulate(bm, faces=list(bm.faces))
    return parts


def build_door_left(body_color):
    bm = _door_left(body_color)
    bmesh.ops.triangulate(bm, faces=list(bm.faces))
    return bm


def build_tailgate(body_color):
    bm = _tailgate(body_color)
    bmesh.ops.triangulate(bm, faces=list(bm.faces))
    return bm


def build_wheel():
    bm = _build_wheel_mesh(include_brake=True)
    bmesh.ops.triangulate(bm, faces=list(bm.faces))
    return bm
