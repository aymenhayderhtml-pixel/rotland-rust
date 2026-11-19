"""Rounded humanoid body (visual plan step 1): lofted limbs, a shaped head, hands with fingers.

Every part is built on exactly one bone of the 22-bone rig (docs/RIG_CONTRACT.md), so the rigid
weights stay exact and every existing clip keeps working. Joint fillers (knees, elbows,
shoulders) are spheres on the lower bone, so bends don't open gaps.
Facing +Y, right is +X, metres, standing at z=0.
"""
from mathutils import Vector

import zombie_details


def _lerp(a, b, t):
    return a + (b - a) * t


def build_body(mb, look, hunched, arm_points):
    """Adds the whole body to MeshBuilder `mb`. `look` holds the palette names (skin, hair...)."""
    hips_z = 0.90 if not hunched else 0.88
    lm = {
        "hips_z": hips_z,
        "spine_z": hips_z + 0.14,
        "chest_z": hips_z + 0.32,
        "neck_z": hips_z + 0.56,
        "head_z": hips_z + 0.64,
        "head_top": 1.80 if not hunched else 1.77,
        # Forward offsets for the zombie hunch: hips, spine, chest, neck, face
        "hy": 0.0 if not hunched else 0.02,
        "sy": 0.0 if not hunched else 0.05,
        "cy": 0.0 if not hunched else 0.10,
        "ny": 0.0 if not hunched else 0.14,
        "fy": 0.0 if not hunched else 0.17,
    }
    _feet(mb, look)
    _legs(mb, look, lm)
    _torso(mb, look, lm)
    _head(mb, look, lm, look.get("zombie", False))
    _arms(mb, look, lm, hunched, arm_points)
    if look.get("zombie"):
        zombie_details.add(mb, look, lm, arm_points)


def _feet(mb, look):
    shoes = look["shoes"]
    for sign, foot, toe in ((1, "foot_R", "toe_R"), (-1, "foot_L", "toe_L")):
        x = sign * 0.10
        # Main shoe body (heel to ball of foot, resting above sole z=0.014)
        mb.add_loft([((x, -0.068, 0.048), 0.036, 0.034), ((x, -0.035, 0.052), 0.046, 0.046),
                     ((x, 0.02, 0.052), 0.048, 0.046), ((x, 0.10, 0.042), 0.047, 0.036)],
                    shoes, foot, segs=10, axis=(0, 1, 0))
        # Toe box (ball of foot to toe tip)
        mb.add_loft([((x, 0.10, 0.040), 0.047, 0.034), ((x, 0.155, 0.032), 0.045, 0.026),
                     ((x, 0.196, 0.025), 0.031, 0.017)], shoes, toe, segs=10, axis=(0, 1, 0))
        # Sole: dark strip all round the bottom from ground (z=0.0) to z=0.014
        mb.add_box((x - 0.049, -0.072, 0.0), (x + 0.049, 0.10, 0.014), "pants_dark", foot)
        mb.add_box((x - 0.047, 0.10, 0.0), (x + 0.047, 0.192, 0.014), "pants_dark", toe)
        # Laces across the instep (offset by ~3 mm so they sit proudly on top of the shoe, REND-003)
        for y, top in ((0.035, 0.096), (0.068, 0.086)):
            mb.add_box((x - 0.022, y - 0.006, top), (x + 0.022, y + 0.006, top + 0.007), "rock_light", foot)


def _legs(mb, look, lm):
    pants, blood = look["pants"], look.get("blood")
    hz = lm["hips_z"]
    for sign, side in ((1, "R"), (-1, "L")):
        x = sign * 0.10
        mb.add_loft([((x, 0.0, hz + 0.03), 0.068, 0.070), ((x, 0.005, 0.80), 0.066, 0.069),
                     ((x, 0.005, 0.66), 0.058, 0.061), ((x, 0.0, 0.53), 0.050, 0.053),
                     ((x, 0.0, 0.49), 0.048, 0.051)], pants, "thigh_" + side, segs=10)
        # Knee cap on the shin, so a bent knee stays closed
        mb.add_ellipsoid((x, 0.006, 0.50), (0.050, 0.053, 0.046), pants, "shin_" + side)
        mb.add_loft([((x, 0.0, 0.50), 0.050, 0.052), ((x, -0.006, 0.40), 0.052, 0.058),
                     ((x, 0.0, 0.26), 0.043, 0.046), ((x, 0.0, 0.12), 0.040, 0.042),
                     ((x, 0.0, 0.07), 0.040, 0.042)], pants, "shin_" + side, segs=10)
        # Pants hem, slightly wider than the shin, over the shoe top
        mb.add_loft([((x, 0.0, 0.135), 0.046, 0.048), ((x, 0.0, 0.085), 0.047, 0.049)], pants, "shin_" + side, segs=10)
        if blood and sign == 1:
            # Zombie knee tear
            mb.add_box((x - 0.035, 0.040, 0.52), (x + 0.035, 0.060, 0.58), blood, "thigh_" + side)


def _torso(mb, look, lm):
    shirt, pants, skin, blood = look["shirt"], look["pants"], look["skin"], look.get("blood")
    hz, sz, cz, nz = lm["hips_z"], lm["spine_z"], lm["chest_z"], lm["neck_z"]
    hy, sy, cy, ny, fy = lm["hy"], lm["sy"], lm["cy"], lm["ny"], lm["fy"]
    # Pelvis and seat
    mb.add_loft([((0, hy - 0.005, hz - 0.075), 0.105, 0.070), ((0, hy, hz - 0.03), 0.140, 0.086),
                 ((0, hy, hz + 0.05), 0.150, 0.090), ((0, hy, sz), 0.150, 0.088)], pants, "hips", segs=12)
    # Belt wraps around waist (a ring matching hips size); buckle sits proudly on front (REND-003)
    mb.add_loft([((0, hy, sz - 0.042), 0.155, 0.093), ((0, hy, sz + 0.006), 0.155, 0.093)], "bark_dark", "hips", segs=12)
    mb.add_box((-0.025, hy + 0.093, sz - 0.035), (0.025, hy + 0.101, sz - 0.003), "metal", "hips")
    # Midriff: the shirt hem hangs just over the belt top
    my = (hy + sy) / 2
    mb.add_loft([((0, hy, sz - 0.022), 0.150, 0.089), ((0, my, sz + 0.09), 0.152, 0.090),
                 ((0, sy, cz), 0.162, 0.096)], shirt, "spine", segs=12)
    if blood:
        mb.add_box((0.02, sy + 0.074, sz + 0.03), (0.12, sy + 0.097, sz + 0.12), blood, "spine")
    # Chest: ribcage, then shoulders sloping into the neck
    pc = (sy + cy) / 2 + 0.006
    mb.add_loft([((0, sy, cz), 0.162, 0.096), ((0, pc, cz + 0.12), 0.180, 0.106),
                 ((0, cy, nz - 0.05), 0.188, 0.100), ((0, cy - 0.005, nz - 0.005), 0.120, 0.075)],
                shirt, "chest", segs=12)
    # Collar round the neck, and a breast pocket on the left
    mb.add_loft([((0, cy, nz - 0.03), 0.070, 0.066), ((0, cy + 0.004, nz + 0.022), 0.063, 0.059)], shirt, "chest", segs=10)
    mb.add_box((-0.135, pc + 0.082, cz + 0.07), (-0.055, pc + 0.100, cz + 0.15), shirt, "chest")
    if blood:
        mb.add_box((-0.14, pc + 0.086, cz + 0.08), (-0.02, pc + 0.110, nz - 0.07), blood, "chest")
    # Neck
    mb.add_loft([((0, ny, nz - 0.03), 0.047, 0.045), ((0, (ny + fy) / 2, lm["head_z"] + 0.03), 0.044, 0.042)],
                skin, "neck", segs=10)


# Head profile: (height fraction, half-width, half-depth, forward offset). Chin at the bottom, crown at the top.
_HEAD = [(0.02, 0.050, 0.050, 0.022), (0.13, 0.070, 0.074, 0.016), (0.31, 0.084, 0.090, 0.006),
         (0.54, 0.090, 0.098, 0.0), (0.77, 0.088, 0.096, -0.006), (0.91, 0.070, 0.080, -0.010),
         (1.0, 0.034, 0.040, -0.012)]
# Hair cap sits on top and back of the head, above the brow line (no eye band, closed crown)
_HAIR_CAP = [(0.60, 0.095, 0.100, -0.012), (0.76, 0.095, 0.098, -0.014), (0.90, 0.080, 0.086, -0.015),
             (1.00, 0.048, 0.052, -0.014), (1.035, 0.020, 0.024, -0.012)]


def _head(mb, look, lm, zombie):
    skin, hair = look["skin"], look["hair"]
    h, top, f = lm["head_z"], lm["head_top"], lm["fy"]
    span = top - h

    def prof(rows, extra=0.0):
        return [((0, f + yc, h + fr * span), rx + extra, ry + extra) for fr, rx, ry, yc in rows]

    mb.add_loft(prof(_HEAD), skin, "head", segs=12)
    mb.add_loft(prof(_HAIR_CAP), hair, "head", segs=12)
    # Ears
    for s in (1, -1):
        mb.add_ellipsoid((s * 0.090, f - 0.004, h + 0.11), (0.013, 0.022, 0.030), skin, "head", segs=6, rings=4)
    # Eyes, pupils and brows
    eye = "white" if not zombie else "rock_light"
    pupil = "hair_dark" if not zombie else "blood_dark"
    brow = hair if not zombie else "bark_dark"
    for s in (1, -1):
        mb.add_ellipsoid((s * 0.041, f + 0.080, h + 0.112), (0.019, 0.009, 0.013), eye, "head", segs=8, rings=4)
        mb.add_ellipsoid((s * 0.039, f + 0.087, h + 0.111), (0.0085, 0.005, 0.0095), pupil, "head", segs=6, rings=3)
        mb.add_box((s * 0.042 - 0.022, f + 0.074, h + 0.132), (s * 0.042 + 0.022, f + 0.090, h + 0.143), brow, "head")
    # Nose: bridge to tip, with a flat underside
    mb.add_loft([((0, f + 0.094, h + 0.130), 0.010, 0.010), ((0, f + 0.110, h + 0.088), 0.017, 0.014),
                 ((0, f + 0.104, h + 0.075), 0.015, 0.010)], skin, "head", segs=6)
    # Mouth
    mouth = "bark_dark" if not zombie else "blood_dark"
    if not zombie:
        mb.add_box((-0.026, f + 0.086, h + 0.048), (0.026, f + 0.098, h + 0.056), mouth, "head")


# Fingers across the palm, thumb side (+Y) first: (offset across the palm, length)
_FINGERS = [(0.026, 0.066), (0.009, 0.074), (-0.009, 0.069), (-0.026, 0.055)]


def _arms(mb, look, lm, hunched, arm_points):
    skin, shirt = look["skin"], look["shirt"]
    sh_y = 0.0 if not hunched else 0.08
    sh_z = lm["hips_z"] + 0.52
    for sign, side in ((1, "R"), (-1, "L")):
        shoulder, elbow, wrist, tip = arm_points(sign, sh_y, sh_z)
        sleeve = skin if look.get("bare_arm") == side else shirt
        a = (tip - wrist).normalized()
        back = Vector((sign * 0.5, 0.0, 0.866))  # back of the hand (perpendicular to the arm)
        across = Vector((0.0, 1.0, 0.0))
        mb.add_ellipsoid(shoulder, (0.056, 0.058, 0.052), sleeve, "shoulder_" + side, segs=10, rings=6)
        # Upper arm: from shoulder extending slightly past the elbow joint (overlap at joint)
        mb.add_loft([(shoulder, 0.054, 0.054), (_lerp(shoulder, elbow, 0.45), 0.052, 0.055),
                     (elbow, 0.045, 0.045), (_lerp(elbow, wrist, 0.04), 0.044, 0.044)], sleeve, "upper_arm_" + side, segs=10)
        if sleeve != skin:
            # Rolled sleeve cuff just above the elbow
            mb.add_loft([(_lerp(shoulder, elbow, 0.80), 0.051, 0.051), (_lerp(shoulder, elbow, 0.97), 0.050, 0.050)],
                        sleeve, "upper_arm_" + side, segs=10)
        # Elbow joint sphere attached to forearm (slightly enlarged to 0.046 for seamless coverage)
        mb.add_ellipsoid(elbow, (0.046, 0.046, 0.046), skin, "forearm_" + side, segs=8, rings=5)
        # Forearm starting slightly above elbow to overlap cleanly
        mb.add_loft([(_lerp(elbow, shoulder, 0.03), 0.044, 0.044), (elbow, 0.043, 0.043),
                     (_lerp(elbow, wrist, 0.3), 0.046, 0.044), (wrist, 0.032, 0.030)],
                    skin, "forearm_" + side, segs=10)
        _hand(mb, skin, "hand_" + side, wrist, a, back, across)


def _hand(mb, skin, bone, wrist, a, back, across):
    mb.add_loft([(wrist, 0.030, 0.018), (wrist + a * 0.03, 0.040, 0.020), (wrist + a * 0.080, 0.039, 0.017)],
                skin, bone, segs=8)
    base_c = wrist + a * 0.074
    for dy, length in _FINGERS:
        base = base_c + across * dy
        mid = base + a * (length * 0.55) - back * 0.003
        tip = base + a * length - back * 0.012  # slight curl toward the palm
        mb.add_loft([(base, 0.0095, 0.0085), (mid, 0.009, 0.008), (tip, 0.0072, 0.0068)], skin, bone, segs=6)
    t0 = wrist + a * 0.02 + across * 0.028 - back * 0.006
    t1 = wrist + a * 0.065 + across * 0.048 - back * 0.012
    mb.add_loft([(t0, 0.012, 0.011), (t1, 0.010, 0.009)], skin, bone, segs=6)
