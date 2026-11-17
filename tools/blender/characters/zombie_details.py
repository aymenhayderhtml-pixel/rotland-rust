"""Zombie-only details on top of the rounded body (visual plan step 2).

All three zombies get sunken eyes, an open mouth with teeth and a torn shirt hem. Then each look
gets its own story: zombie_01 (hunched) has torn ribs and wild hair, zombie_02 (big, bare
left arm) has a bitten arm, a torn trouser leg and a bald patch, and zombie_03 (thin, hospital gown)
has a long gown, a wrist tag and long stringy hair. Every part sits on one bone (rigid weights).
"""
import math

from mathutils import Vector


def add(mb, look, lm, arm_points):
    _face(mb, lm)
    _torn_hem(mb, look, lm)
    kind = look["name"]
    if kind == "zombie_01":
        _ribs(mb, look, lm)
        _wild_hair(mb, look, lm)
    elif kind == "zombie_02":
        _bitten_arm(mb, lm, arm_points)
        _torn_leg(mb, look)
        _bald_patch(mb, look, lm)
    elif kind == "zombie_03":
        _gown(mb, look, lm)
        _wrist_tag(mb, lm, arm_points)
        _long_hair(mb, look, lm)


def _face(mb, lm):
    h, f = lm["head_z"], lm["fy"]
    # Dark, sunken rings round the eyes
    for s in (1, -1):
        mb.add_ellipsoid((s * 0.041, f + 0.073, h + 0.111), (0.026, 0.009, 0.020), "bark_dark", "head", segs=8, rings=4)
    # Open mouth: a dark hole with an upper and a lower row of teeth
    # Kept just under the face surface so it reads as a hole, not a block
    mb.add_box((-0.028, f + 0.070, h + 0.028), (0.028, f + 0.091, h + 0.062), "blood_dark", "head")
    mb.add_box((-0.022, f + 0.084, h + 0.054), (0.022, f + 0.093, h + 0.060), "rock_light", "head")
    mb.add_box((-0.018, f + 0.080, h + 0.030), (0.018, f + 0.090, h + 0.035), "rock_light", "head")


def _torn_hem(mb, look, lm):
    """Jagged shirt shreds hanging over the belt, all round the waist."""
    sz, hy = lm["spine_z"], lm["hy"]
    for i, (ang, drop) in enumerate(((20, 0.07), (55, 0.045), (90, 0.08), (125, 0.05), (160, 0.065),
                                     (215, 0.06), (270, 0.075), (325, 0.05))):
        a = math.radians(ang)
        x, y = 0.152 * math.cos(a), hy + 0.091 * math.sin(a)
        top = sz - 0.012
        mb.add_loft([((x, y, top), 0.020, 0.012), ((x * 1.02, y * 1.02, top - drop), 0.003, 0.003)],
                    look["shirt"], "spine", segs=5)


def _ribs(mb, look, lm):
    """A torn hole in the shirt on the right side of the chest, with ribs showing."""
    cz, pc = lm["chest_z"], (lm["sy"] + lm["cy"]) / 2 + 0.006
    mb.add_box((0.03, pc + 0.074, cz + 0.02), (0.135, pc + 0.100, cz + 0.15), "blood_dark", "chest")
    for k in range(4):
        z = cz + 0.04 + k * 0.03
        mb.add_box((0.035, pc + 0.094, z), (0.130, pc + 0.106, z + 0.012), "skin_pale", "chest")


def _wild_hair(mb, look, lm):
    """Tufts sticking out of the top and back of the head."""
    h, top, f = lm["head_z"], lm["head_top"], lm["fy"]
    for ax, ay, az, tip in ((0.05, -0.02, 0.92, (0.030, 0.008, 0.032)), (-0.06, -0.01, 0.89, (-0.034, 0.01, 0.026)),
                            (0.0, -0.07, 0.86, (0.004, -0.036, 0.020)), (0.08, -0.04, 0.74, (0.030, -0.02, -0.012)),
                            (-0.08, -0.05, 0.72, (-0.028, -0.02, -0.016)), (0.02, 0.05, 0.93, (0.010, 0.030, 0.012)),
                            (-0.03, 0.03, 0.96, (-0.016, 0.012, 0.028)), (0.04, -0.08, 0.66, (0.012, -0.026, -0.022))):
        base = Vector((ax, f + ay, h + az * (top - h)))
        mb.add_loft([(base, 0.024, 0.020), (base + Vector(tip) * 0.6, 0.014, 0.012), (base + Vector(tip), 0.004, 0.004)],
                    look["hair"], "head", segs=5)


def _bitten_arm(mb, lm, arm_points):
    shoulder, elbow, _, _ = arm_points(-1, 0.0, lm["hips_z"] + 0.52)
    bite = shoulder.lerp(elbow, 0.55) + Vector((0.0, 0.040, 0.006))
    mb.add_ellipsoid(bite, (0.028, 0.016, 0.022), "blood_dark", "upper_arm_L", segs=8, rings=4)
    mb.add_ellipsoid(bite + Vector((-0.005, 0.006, 0.0)), (0.012, 0.012, 0.010), "red_paint", "upper_arm_L", segs=6, rings=3)


def _torn_leg(mb, look):
    """The right trouser leg is ripped open at the shin: bare skin with a bloody edge."""
    x = 0.10
    mb.add_box((x - 0.036, 0.034, 0.22), (x + 0.036, 0.058, 0.36), "blood_dark", "shin_R")
    mb.add_box((x - 0.030, 0.040, 0.23), (x + 0.030, 0.062, 0.35), look["skin"], "shin_R")


def _bald_patch(mb, look, lm):
    """A torn scalp wound on the right side of the head."""
    h, top, f = lm["head_z"], lm["head_top"], lm["fy"]
    c = (0.088, f - 0.02, h + 0.78 * (top - h))
    mb.add_ellipsoid(c, (0.022, 0.040, 0.034), "blood_dark", "head", segs=8, rings=4)
    mb.add_ellipsoid((c[0] + 0.006, c[1], c[2]), (0.020, 0.026, 0.022), look["skin"], "head", segs=6, rings=3)


def _gown(mb, look, lm):
    """Hospital gown hanging from the waist to mid-thigh (on the hips, so the legs swing under it)."""
    hz, hy = lm["hips_z"], lm["hy"]
    mb.add_loft([((0, hy, hz + 0.10), 0.157, 0.095), ((0, hy, hz - 0.08), 0.172, 0.112),
                 ((0, hy, hz - 0.26), 0.185, 0.125)], look["shirt"], "hips", segs=12)
    # Dirty hem
    mb.add_loft([((0, hy, hz - 0.23), 0.187, 0.127), ((0, hy, hz - 0.265), 0.188, 0.128)], "dirt", "hips", segs=12)


def _wrist_tag(mb, lm, arm_points):
    _, elbow, wrist, _ = arm_points(1, 0.0, lm["hips_z"] + 0.52)
    c0, c1 = elbow.lerp(wrist, 0.80), elbow.lerp(wrist, 0.90)
    mb.add_loft([(c0, 0.037, 0.035), (c1, 0.036, 0.034)], "white", "forearm_R", segs=8)


def _long_hair(mb, look, lm):
    h, top, f = lm["head_z"], lm["head_top"], lm["fy"]
    span = top - h
    mb.add_loft([((0, f - 0.040, h + 0.70 * span), 0.095, 0.070), ((0, f - 0.055, h + 0.10 * span), 0.090, 0.055),
                 ((0, f - 0.065, h - 0.10 * span), 0.078, 0.038)], look["hair"], "head", segs=10)
    # Stringy strands over the sides of the face
    for s in (1, -1):
        mb.add_loft([((s * 0.085, f + 0.02, h + 0.80 * span), 0.012, 0.016),
                     ((s * 0.090, f + 0.035, h + 0.10 * span), 0.006, 0.008)], look["hair"], "head", segs=5)
