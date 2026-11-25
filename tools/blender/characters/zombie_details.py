"""Zombie-only details on top of the rounded body (visual plan step 2).

All zombies get sunken eyes, open mouth details, and torn hems. Then each look gets its own story:
- zombie_01: hunched, torn ribs, wild hair
- zombie_02: big, bare left arm, bitten arm, torn leg, bald patch
- zombie_03: thin, hospital gown, wrist tag, long hair
- zombie_brute: 1.35x scale, very broad shoulders, huge arms, torn work overalls, metal plate strapped to chest
- zombie_screamer: thin, long neck, jaw wide open, pale skin, ragged dress/coat
- zombie_crawler: legs missing below knee (stumps), dragging scrapes and torn rags
- zombie_frozen: pale ice-blue skin, frosted winter jacket collar, ice/snow shards on shoulders and head
- zombie_husk: dried thin desert zombie, bare chest with rib stripes, torn khaki waist/shoulder rags
- zombie_bloater: swollen belly sphere on spine with boils, ragged shirt covering only the shoulders
"""
import math

import bmesh
from mathutils import Vector


def add(mb, look, lm, arm_points):
    kind = look["name"]
    if kind == "zombie_screamer":
        _screamer_face(mb, lm, look)
        _screamer_ragged_dress(mb, look, lm)
    elif kind == "zombie_crawler":
        _face(mb, lm)
        _crawler_torn_hem(mb, look, lm)
    elif kind in ("zombie_husk", "zombie_bloater"):
        _face(mb, lm)
    else:
        _face(mb, lm)
        _torn_hem(mb, look, lm)

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
    elif kind == "zombie_brute":
        _brute_overalls(mb, look, lm)
        _brute_metal_plate(mb, lm)
        _brute_huge_arms(mb, look, lm, arm_points)
    elif kind == "zombie_screamer":
        _screamer_long_neck(mb, look, lm)
        _screamer_hair(mb, look, lm)
    elif kind == "zombie_crawler":
        _crawler_stumps(mb, look, lm)
    elif kind == "zombie_frozen":
        _frozen_frost(mb, lm, arm_points)
    elif kind == "zombie_husk":
        _husk_ribs(mb, lm)
        _husk_torn_clothes(mb, look, lm)
    elif kind == "zombie_bloater":
        _bloater_belly(mb, lm)
        _bloater_ragged_shoulders(mb, lm, arm_points)


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


def _brute_overalls(mb, look, lm):
    """Torn work overalls with front bib, pocket, shoulder straps and torn pant legs."""
    cz, pc = lm["chest_z"], (lm["sy"] + lm["cy"]) / 2 + 0.006
    sz = lm["spine_z"]
    pants = look["pants"]
    # Front bib
    mb.add_box((-0.135, pc + 0.098, sz + 0.01), (0.135, pc + 0.118, cz + 0.165), pants, "chest")
    # Back bib panel
    mb.add_box((-0.125, pc - 0.108, sz + 0.01), (0.125, pc - 0.092, cz + 0.140), pants, "chest")
    # Front center bib pocket
    mb.add_box((-0.075, pc + 0.114, cz + 0.025), (0.075, pc + 0.124, cz + 0.105), "canvas_khaki", "chest")
    # Overalls straps running over shoulders with brass buckle clips
    for s in (-1, 1):
        mb.add_loft([((s * 0.095, pc + 0.106, cz + 0.145), 0.024, 0.010),
                     ((s * 0.100, lm["cy"] + 0.008, lm["neck_z"] + 0.010), 0.024, 0.010),
                     ((s * 0.095, pc - 0.098, cz + 0.120), 0.024, 0.010)], pants, "chest", segs=6)
        mb.add_box((s * 0.095 - 0.015, pc + 0.112, cz + 0.130),
                   (s * 0.095 + 0.015, pc + 0.122, cz + 0.155), "brass", "chest")
    # Torn patches on the legs revealing zombie flesh
    mb.add_box((-0.14, 0.046, 0.58), (-0.06, 0.068, 0.68), "blood_dark", "thigh_L")
    mb.add_box((-0.13, 0.052, 0.60), (-0.07, 0.070, 0.66), look["skin"], "thigh_L")
    mb.add_box((0.06, 0.046, 0.30), (0.14, 0.068, 0.40), "blood_dark", "shin_R")
    mb.add_box((0.07, 0.052, 0.32), (0.13, 0.070, 0.38), look["skin"], "shin_R")


def _brute_metal_plate(mb, lm):
    """Heavy steel armor plate strapped to the chest with rivets and leather bands."""
    cz, pc = lm["chest_z"], (lm["sy"] + lm["cy"]) / 2 + 0.006
    # Metal plate
    mb.add_box((-0.120, pc + 0.118, cz + 0.025), (0.120, pc + 0.138, cz + 0.175), "metal", "chest")
    # Rust stain patch
    mb.add_box((-0.100, pc + 0.128, cz + 0.040), (0.010, pc + 0.140, cz + 0.110), "rust", "chest")
    # Corner rivets
    for cx in (-0.102, 0.102):
        for cz_r in (cz + 0.045, cz + 0.155):
            mb.add_box((cx - 0.009, pc + 0.132, cz_r - 0.009),
                       (cx + 0.009, pc + 0.144, cz_r + 0.009), "rock_light", "chest")
    # Heavy strapping wrapping around torso
    mb.add_loft([((0, pc, cz + 0.135), 0.208, 0.124), ((0, pc, cz + 0.158), 0.208, 0.124)],
                "tire_black", "chest", segs=10)
    mb.add_loft([((0, pc, cz + 0.045), 0.205, 0.122), ((0, pc, cz + 0.068), 0.205, 0.122)],
                "tire_black", "chest", segs=10)
    # Strap buckles on sides
    mb.add_box((-0.212, pc - 0.015, cz + 0.130), (-0.200, pc + 0.020, cz + 0.165), "chrome", "chest")
    mb.add_box((0.200, pc - 0.015, cz + 0.040), (0.212, pc + 0.020, cz + 0.075), "chrome", "chest")


def _brute_huge_arms(mb, look, lm, arm_points):
    """Massive muscular volume wrapping deltoids, upper arms, forearms and fists."""
    skin = look["skin"]
    for sign, side in ((1, "R"), (-1, "L")):
        shoulder, elbow, wrist, tip = arm_points(sign, lm["hy"], lm["hips_z"] + 0.52)
        # Trapezius muscle slope connecting neck to shoulder
        trap_p0 = Vector((sign * 0.08, lm["ny"], lm["neck_z"] + 0.01))
        trap_p1 = Vector((sign * 0.17, lm["hy"], lm["hips_z"] + 0.54))
        mb.add_loft([(trap_p0, 0.065, 0.065), (trap_p1, 0.076, 0.076)], skin, "shoulder_" + side, segs=8)

        # Huge rounded deltoid cap on upper arm
        delt_c = shoulder + Vector((sign * 0.02, 0.0, 0.01))
        mb.add_ellipsoid(delt_c, (0.078, 0.082, 0.074), skin, "upper_arm_" + side, segs=8, rings=5)

        # Thick muscular upper arm loft from shoulder through bicep/tricep to elbow
        p_bicep = shoulder.lerp(elbow, 0.48)
        mb.add_loft([(shoulder, 0.072, 0.074), (p_bicep, 0.076, 0.078), (elbow, 0.062, 0.062)],
                    skin, "upper_arm_" + side, segs=8)

        # Thick muscular forearm loft from elbow through forearm swell to wrist
        p_fore = elbow.lerp(wrist, 0.35)
        mb.add_loft([(elbow, 0.062, 0.062), (p_fore, 0.066, 0.064), (wrist, 0.048, 0.044)],
                    skin, "forearm_" + side, segs=8)

        # Heavy brutal knuckles/fist
        hand_c = wrist.lerp(tip, 0.50)
        mb.add_ellipsoid(hand_c, (0.052, 0.056, 0.042), skin, "hand_" + side, segs=6, rings=4)


def _screamer_face(mb, lm, look):
    """Black hollow eye sockets and a massive, unhinged gaping scream jaw."""
    h, f = lm["head_z"], lm["fy"]
    for s in (1, -1):
        mb.add_ellipsoid((s * 0.040, f + 0.073, h + 0.113), (0.028, 0.012, 0.022), "gun_black", "head", segs=8, rings=4)
        mb.add_box((s * 0.040 - 0.005, f + 0.078, h + 0.110),
                   (s * 0.040 + 0.005, f + 0.084, h + 0.116), "white", "head")
    # Cavernous gaping throat
    mb.add_box((-0.032, f + 0.052, h - 0.018), (0.032, f + 0.092, h + 0.060), "gun_black", "head")
    mb.add_box((-0.028, f + 0.062, h - 0.010), (0.028, f + 0.095, h + 0.056), "blood_dark", "head")
    # Upper sharp teeth
    mb.add_box((-0.024, f + 0.084, h + 0.050), (0.024, f + 0.094, h + 0.060), "rock_light", "head")
    # Distended lower jaw dropped down
    mb.add_loft([((-0.034, f + 0.052, h + 0.015), 0.012, 0.014),
                 ((0.0, f + 0.094, h - 0.024), 0.028, 0.016),
                 ((0.034, f + 0.052, h + 0.015), 0.012, 0.014)], look["skin"], "head", segs=6)
    # Lower sharp teeth
    mb.add_box((-0.022, f + 0.084, h - 0.014), (0.022, f + 0.093, h - 0.006), "rock_light", "head")
    # Bloody sinew connecting corners of open mouth
    for s in (1, -1):
        mb.add_loft([((s * 0.030, f + 0.068, h + 0.048), 0.004, 0.005),
                     ((s * 0.031, f + 0.074, h + 0.002), 0.003, 0.004)], "blood_dark", "head", segs=5)


def _screamer_long_neck(mb, look, lm):
    """Taut, elongated cords and protruding trachea on the neck."""
    skin = look["skin"]
    for s in (1, -1):
        mb.add_loft([((s * 0.046, lm["ny"] - 0.012, lm["neck_z"] - 0.035), 0.014, 0.012),
                     ((s * 0.054, lm["fy"] - 0.018, lm["head_z"] + 0.075), 0.010, 0.010)], skin, "neck", segs=6)
    # Windpipe ridge
    mb.add_loft([((0, lm["ny"] + 0.058, lm["neck_z"] - 0.020), 0.015, 0.014),
                 ((0, lm["fy"] + 0.064, lm["head_z"] + 0.050), 0.017, 0.015)], skin, "neck", segs=6)
    # Throat shadow hollow
    mb.add_ellipsoid((0, lm["ny"] + 0.046, lm["neck_z"] - 0.030), (0.026, 0.010, 0.016), "bark_dark", "neck", segs=6, rings=3)


def _screamer_ragged_dress(mb, look, lm):
    """Long ragged dress/coat hanging down from waist past knees with jagged tattered strips."""
    hz, hy = lm["hips_z"], lm["hy"]
    dress_col = look["shirt"]
    mb.add_loft([((0, hy, hz + 0.06), 0.165, 0.105),
                 ((0, hy, hz - 0.12), 0.185, 0.120),
                 ((0, hy, hz - 0.32), 0.200, 0.135)], dress_col, "hips", segs=10)
    mb.add_loft([((0, hy, hz - 0.28), 0.202, 0.137),
                 ((0, hy, hz - 0.33), 0.204, 0.138)], "dirt", "hips", segs=10)
    # Jagged hanging shreds
    for ang, drop, col in ((15, 0.12, dress_col), (45, 0.08, "dirt"), (80, 0.14, dress_col),
                           (120, 0.09, "blood_dark"), (160, 0.13, dress_col), (200, 0.10, "dirt"),
                           (245, 0.14, dress_col), (290, 0.09, "dirt"), (335, 0.12, dress_col)):
        a = math.radians(ang)
        x, y = 0.198 * math.cos(a), hy + 0.133 * math.sin(a)
        top = hz - 0.31
        mb.add_loft([((x, y, top), 0.024, 0.014), ((x * 1.02, y * 1.02, top - drop), 0.004, 0.004)],
                    col, "hips", segs=5)
    # Ragged sleeve shreds
    for s in (1, -1):
        mb.add_loft([((s * 0.24, hy, hz + 0.44), 0.045, 0.045),
                     ((s * 0.25, hy, hz + 0.36), 0.048, 0.048),
                     ((s * 0.255, hy, hz + 0.28), 0.020, 0.010)], dress_col,
                    "upper_arm_" + ("R" if s > 0 else "L"), segs=6)


def _screamer_hair(mb, look, lm):
    """Long stringy matted hair streaming down the back and sides of the head."""
    h, top, f = lm["head_z"], lm["head_top"], lm["fy"]
    span = top - h
    mb.add_loft([((0, f - 0.045, h + 0.70 * span), 0.098, 0.075),
                 ((0, f - 0.060, h + 0.10 * span), 0.092, 0.060),
                 ((0, f - 0.070, h - 0.12 * span), 0.080, 0.040)], look["hair"], "head", segs=10)
    for s in (1, -1):
        mb.add_loft([((s * 0.085, f + 0.015, h + 0.75 * span), 0.014, 0.016),
                     ((s * 0.092, f + 0.028, h + 0.05 * span), 0.008, 0.010),
                     ((s * 0.095, f + 0.010, h - 0.10 * span), 0.004, 0.005)], look["hair"], "head", segs=5)


def _crawler_stumps(mb, look, lm):
    """Deletes feet and lower shins, adding severed bloody stumps with protruding bone."""
    # Delete all foot/toe vertices and lower shin vertices
    # Bone indices in humanoid.BONE_NAMES:
    # 15: shin_L, 16: foot_L, 17: toe_L
    # 19: shin_R, 20: foot_R, 21: toe_R
    del_verts = []
    for v in mb.bm.verts:
        b = v[mb.bone_layer]
        if b in (16, 17, 20, 21):
            del_verts.append(v)
        elif b in (15, 19) and v.co.z < 0.44:
            del_verts.append(v)
    bmesh.ops.delete(mb.bm, geom=del_verts, context="VERTS")

    for sign, side in ((1, "R"), (-1, "L")):
        x = sign * 0.10
        bone = "thigh_" + side
        # Torn trouser cuff
        mb.add_loft([((x, 0.004, 0.47), 0.058, 0.060), ((x, 0.004, 0.43), 0.055, 0.057)],
                    look["pants"], bone, segs=8)
        # Bloody fleshy stump cap
        mb.add_ellipsoid((x, 0.004, 0.43), (0.046, 0.048, 0.022), "blood_dark", bone, segs=8, rings=4)
        mb.add_ellipsoid((x, 0.004, 0.41), (0.032, 0.034, 0.016), "red_paint", bone, segs=6, rings=3)
        # Protruding broken bone shaft
        mb.add_loft([((x, 0.004, 0.43), 0.018, 0.016), ((x, 0.004, 0.38), 0.015, 0.013),
                     ((x + sign * 0.003, 0.006, 0.34), 0.008, 0.006)], "rock_light", bone, segs=5)
        mb.add_box((x - 0.005, 0.002, 0.335), (x + 0.005, 0.008, 0.350), "rock_light", bone)


def _crawler_torn_hem(mb, look, lm):
    """Scraped, caked-dirt shirt shreds and belly scrapes from dragging along the ground."""
    sz, hy = lm["spine_z"], lm["hy"]
    for ang, drop in ((30, 0.08), (75, 0.10), (120, 0.07), (165, 0.09),
                      (210, 0.08), (255, 0.11), (300, 0.08), (345, 0.10)):
        a = math.radians(ang)
        x, y = 0.154 * math.cos(a), hy + 0.093 * math.sin(a)
        top = sz - 0.010
        mb.add_loft([((x, y, top), 0.022, 0.014), ((x * 1.02, y * 1.02, top - drop), 0.004, 0.004)],
                    look["shirt"], "spine", segs=5)
    # Scrapes on belly and chest
    mb.add_box((-0.12, lm["sy"] + 0.088, sz - 0.02), (0.12, lm["sy"] + 0.106, sz + 0.08), "dirt", "spine")
    mb.add_box((-0.09, lm["cy"] + 0.098, lm["chest_z"] + 0.02),
               (0.09, lm["cy"] + 0.114, lm["chest_z"] + 0.14), "blood_dark", "chest")


def _frozen_frost(mb, lm, arm_points):
    """Six jagged ice/snow shards on the shoulders and head, plus a frosted winter collar."""
    h, top, f = lm["head_z"], lm["head_top"], lm["fy"]
    span = top - h

    # 2 frost shards emerging from the crown/temples of the head
    head_shards = (
        (Vector((0.058, f + 0.015, h + 0.80 * span)), Vector((0.042, 0.022, 0.072)), "ice_blue"),
        (Vector((-0.052, f - 0.022, h + 0.85 * span)), Vector((-0.036, -0.026, 0.064)), "snow_white"),
    )
    for base, tip, col in head_shards:
        mb.add_loft([(base, 0.020, 0.016), (base + tip * 0.55, 0.013, 0.010), (base + tip, 0.002, 0.002)],
                    col, "head", segs=4)

    # 4 frost shards (2 per shoulder) riding on the shoulder bones
    for sign, side, col_a, col_b in ((1, "R", "ice_blue", "snow_white"), (-1, "L", "snow_white", "ice_blue")):
        shoulder, _, _, _ = arm_points(sign, lm["hy"], lm["hips_z"] + 0.52)
        bone = "shoulder_" + side
        b1 = shoulder + Vector((-sign * 0.015, 0.008, 0.042))
        t1 = Vector((sign * 0.048, -0.012, 0.088))
        mb.add_loft([(b1, 0.024, 0.020), (b1 + t1 * 0.55, 0.015, 0.013), (b1 + t1, 0.002, 0.002)],
                    col_a, bone, segs=4)
        b2 = shoulder + Vector((-sign * 0.048, -0.020, 0.046))
        t2 = Vector((sign * 0.024, -0.030, 0.062))
        mb.add_loft([(b2, 0.018, 0.015), (b2 + t2 * 0.55, 0.011, 0.009), (b2 + t2, 0.002, 0.002)],
                    col_b, bone, segs=4)

    # Frosted winter jacket collar around the neck
    mb.add_loft([((0.0, lm["ny"], lm["neck_z"] - 0.010), 0.115, 0.092),
                 ((0.0, lm["ny"], lm["neck_z"] + 0.032), 0.105, 0.084)],
                "snow_shadow", "chest", segs=10)


def _husk_ribs(mb, lm):
    """Four thin dark horizontal rib stripes across the bare desiccated chest."""
    cz, pc = lm["chest_z"], (lm["sy"] + lm["cy"]) / 2 + 0.006
    for k, half_w in enumerate((0.095, 0.108, 0.114, 0.106)):
        z = cz + 0.022 + k * 0.034
        for s in (-1, 1):
            x0, x1 = sorted((s * 0.016, s * half_w))
            mb.add_box((x0, pc + 0.084, z), (x1, pc + 0.104, z + 0.010), "bark_dark", "chest")


def _husk_torn_clothes(mb, look, lm):
    """Torn khaki waistband shreds and a single shredded shoulder rag leaving the chest bare."""
    sz, hy = lm["spine_z"], lm["hy"]
    cloth = look["pants"]
    mb.add_loft([((0.0, hy, sz - 0.030), 0.156, 0.095),
                 ((0.0, hy, sz + 0.020), 0.154, 0.093)], cloth, "spine", segs=10)
    for ang, drop, col in ((20, 0.09, cloth), (65, 0.06, "dry_brush"), (110, 0.10, cloth),
                           (155, 0.07, cloth), (205, 0.09, "dry_brush"), (255, 0.08, cloth),
                           (305, 0.10, cloth), (345, 0.06, "dry_brush")):
        a = math.radians(ang)
        x, y = 0.155 * math.cos(a), hy + 0.094 * math.sin(a)
        top = sz - 0.022
        mb.add_loft([((x, y, top), 0.022, 0.012), ((x * 1.02, y * 1.02, top - drop), 0.003, 0.003)],
                    col, "hips", segs=5)
    # Shredded cloth strip draped over the left shoulder, clear of the bare ribbed chest
    cz, pc = lm["chest_z"], (lm["sy"] + lm["cy"]) / 2 + 0.006
    mb.add_loft([((-0.115, pc + 0.096, cz + 0.145), 0.024, 0.010),
                 ((-0.110, lm["cy"] + 0.008, lm["neck_z"] + 0.012), 0.024, 0.010),
                 ((-0.115, pc - 0.096, cz + 0.110), 0.022, 0.010)], cloth, "chest", segs=6)


def _bloater_belly(mb, lm):
    """Big swollen green-yellow belly ellipsoid on the spine bone, dotted with 4 swamp-mud boils."""
    belly_z = (lm["spine_z"] + lm["chest_z"]) * 0.5 - 0.01
    belly_y = lm["sy"] + 0.085
    rx, ry, rz = 0.175, 0.145, 0.195
    mb.add_ellipsoid((0.0, belly_y, belly_z), (rx, ry, rz), "crop_green", "spine", segs=12, rings=8)

    for theta, phi, b_rad in (
        (0.45, 0.28, (0.028, 0.022, 0.032)),
        (-0.52, -0.15, (0.032, 0.024, 0.036)),
        (0.12, -0.38, (0.024, 0.020, 0.028)),
        (-0.22, 0.42, (0.022, 0.018, 0.026)),
    ):
        bx = rx * math.cos(phi) * math.sin(theta)
        by = belly_y + ry * math.cos(phi) * math.cos(theta)
        bz = belly_z + rz * math.sin(phi)
        mb.add_ellipsoid((bx, by, bz), b_rad, "swamp_mud", "spine", segs=8, rings=4)


def _bloater_ragged_shoulders(mb, lm, arm_points):
    """Tattered shirt remnant covering only the upper chest and shoulders above the bare belly."""
    cz, pc = lm["chest_z"], (lm["sy"] + lm["cy"]) / 2 + 0.006
    cloth = "shirt_zombie"
    mb.add_loft([
        ((0.0, lm["ny"], lm["neck_z"] + 0.015), 0.105, 0.085),
        ((0.0, lm["cy"], cz + 0.19), 0.195, 0.118),
        ((0.0, pc, cz + 0.09), 0.190, 0.115),
    ], cloth, "chest", segs=12)

    for sign, side in ((1, "R"), (-1, "L")):
        shoulder, elbow, _, _ = arm_points(sign, lm["hy"], lm["hips_z"] + 0.52)
        p0 = shoulder + Vector((-sign * 0.03, 0.0, 0.005))
        p1 = shoulder.lerp(elbow, 0.34)
        mb.add_loft([(p0, 0.064, 0.064), (p1, 0.056, 0.056)], cloth, "upper_arm_" + side, segs=8)

    for ang, drop, col in ((20, 0.06, cloth), (65, 0.045, "dirt"), (115, 0.055, cloth),
                           (165, 0.05, cloth), (215, 0.065, "dirt"), (270, 0.07, cloth),
                           (320, 0.05, cloth)):
        a = math.radians(ang)
        x, y = 0.188 * math.cos(a), pc + 0.114 * math.sin(a)
        top = cz + 0.095
        mb.add_loft([((x, y, top), 0.024, 0.012), ((x * 1.02, y * 1.02, top - drop), 0.003, 0.003)],
                    col, "chest", segs=4)
