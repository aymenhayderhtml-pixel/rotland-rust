"""Stylized heroic low-poly humanoid body for Rotland.

Every vertex belongs rigidly to one of the 22 bones in docs/RIG_CONTRACT.md.
Deltoid shoulder caps live on upper_arm_L/R so they rotate cleanly with the arms
(eliminating floating shoulder pads), and all accessories (watch, knee straps,
holster, cargo pocket, boots, beard, hair) sit mathematically flush to the body.

Facing +Y in Blender (-Z in Godot), right is +X, up is +Z, feet on z = 0.
"""
import math

from mathutils import Vector

import zombie_details


def _lerp(a, b, t):
    return a + (b - a) * t


def build_body(mb, look, hunched, arm_points):
    """Adds the complete stylized humanoid body to MeshBuilder `mb`."""
    hips_z = 0.90 if not hunched else 0.88
    lm = {
        "hips_z": hips_z,
        "spine_z": hips_z + 0.14,
        "chest_z": hips_z + 0.32,
        "neck_z": hips_z + 0.56,
        "head_z": hips_z + 0.64,
        "head_top": 1.80 if not hunched else 1.77,
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
    """Chunky rugged survival boots with flush lugged soles (no heel/toe overhang)."""
    shoes = look["shoes"]
    for sign, foot, toe in ((1, "foot_R", "toe_R"), (-1, "foot_L", "toe_L")):
        x = sign * 0.10
        # Flush lugged rubber sole: z = 0.000..0.020, matching exact heel (-0.060) to toe (+0.188)
        mb.add_box((x - 0.046, -0.052, 0.0), (x + 0.046, 0.100, 0.018), "tire_black", foot)
        mb.add_box((x - 0.044, 0.098, 0.0), (x + 0.044, 0.184, 0.016), "tire_black", toe)
        # Main leather boot upper (heel to ball)
        mb.add_loft(
            [
                ((x, -0.052, 0.052), 0.046, 0.036),
                ((x, -0.022, 0.058), 0.050, 0.044),
                ((x, 0.035, 0.056), 0.050, 0.042),
                ((x, 0.102, 0.046), 0.049, 0.032),
            ],
            shoes,
            foot,
            segs=8,
            axis=(0, 1, 0),
        )
        # Padded boot ankle collar wrapping cleanly into lower shin
        mb.add_loft(
            [
                ((x, -0.010, 0.058), 0.050, 0.054),
                ((x, -0.006, 0.108), 0.051, 0.053),
            ],
            shoes,
            foot,
            segs=8,
            axis=(0, 0, 1),
        )
        # Reinforced toe cap (ball to toe tip)
        mb.add_loft(
            [
                ((x, 0.098, 0.044), 0.049, 0.030),
                ((x, 0.150, 0.036), 0.045, 0.024),
                ((x, 0.188, 0.028), 0.032, 0.016),
            ],
            shoes,
            toe,
            segs=8,
            axis=(0, 1, 0),
        )
        # Clean flush laces across the boot instep
        for ly, lz in ((0.032, 0.096), (0.058, 0.088), (0.082, 0.079)):
            mb.add_box((x - 0.022, ly - 0.005, lz), (x + 0.022, ly + 0.005, lz + 0.006), "rock_light", foot)


def _legs(mb, look, lm):
    """Athletic stylized legs with snug knee pads, flush cargo pocket, and drop-leg holster."""
    pants, blood = look["pants"], look.get("blood")
    hz = lm["hips_z"]
    for sign, side in ((1, "R"), (-1, "L")):
        x = sign * 0.10
        # Fuller athletic thigh tucked cleanly into pelvis
        mb.add_loft(
            [
                ((x, 0.0, hz + 0.04), 0.078, 0.080),
                ((x, 0.004, 0.78), 0.075, 0.077),
                ((x, 0.004, 0.64), 0.066, 0.068),
                ((x, 0.002, 0.53), 0.057, 0.059),
                ((x, 0.0, 0.48), 0.054, 0.056),
            ],
            pants,
            "thigh_" + side,
            segs=8,
        )
        # Knee joint sphere on shin so bends never crack
        mb.add_ellipsoid((x, 0.004, 0.50), (0.056, 0.058, 0.050), pants, "shin_" + side, segs=8, rings=4)
        # Athletic calf tapering into cuffed pant hem over boot collar
        mb.add_loft(
            [
                ((x, 0.0, 0.50), 0.055, 0.057),
                ((x, -0.007, 0.39), 0.059, 0.064),
                ((x, -0.002, 0.24), 0.049, 0.052),
                ((x, 0.0, 0.115), 0.048, 0.050),
                ((x, 0.0, 0.075), 0.044, 0.046),
            ],
            pants,
            "shin_" + side,
            segs=8,
        )
        # Crisp pant hem cuff resting over the boot collar
        mb.add_loft(
            [
                ((x, 0.0, 0.150), 0.053, 0.055),
                ((x, 0.0, 0.098), 0.055, 0.057),
            ],
            pants,
            "shin_" + side,
            segs=8,
        )
        # Snug 360-degree knee-pad wrap + flush curved front armor pad (zero poking sticks!)
        if look.get("knee_pads"):
            mb.add_loft(
                [
                    ((x, 0.0, 0.478), 0.059, 0.061),
                    ((x, 0.0, 0.514), 0.059, 0.061),
                ],
                "gun_black",
                "shin_" + side,
                segs=8,
            )
            mb.add_box((x - 0.038, 0.044, 0.462), (x + 0.038, 0.068, 0.538), "tire_black", "shin_" + side)

        if blood and sign == 1:
            mb.add_box((x - 0.035, 0.046, 0.52), (x + 0.035, 0.066, 0.58), blood, "thigh_" + side)

    # Flush cargo pocket embedded cleanly into outer left thigh (outer edge x ~ -0.172)
    if look.get("cargo_pockets"):
        mb.add_box((-0.182, -0.040, 0.635), (-0.150, 0.042, 0.765), pants, "thigh_L")
        mb.add_box((-0.184, -0.042, 0.742), (-0.152, 0.044, 0.770), "pants_dark", "thigh_L")

    # Snug drop-leg tactical pistol holster on outer right thigh (outer edge x ~ +0.172)
    if look.get("holster"):
        # 360-degree snug thigh strap (zero protruding sticks!)
        mb.add_loft(
            [
                ((0.10, 0.004, 0.705), 0.075, 0.077),
                ((0.10, 0.004, 0.740), 0.076, 0.078),
            ],
            "tire_black",
            "thigh_R",
            segs=8,
        )
        # Holster pouch embedded snugly against right thigh
        mb.add_box((0.152, -0.034, 0.635), (0.196, 0.038, 0.775), "leather", "thigh_R")
        # Pistol grip butt tucked cleanly inside holster top
        mb.add_box((0.162, -0.038, 0.770), (0.188, 0.008, 0.825), "gun_black", "thigh_R")


def _torso(mb, look, lm):
    """Athletic V-taper torso with solid open-jacket styling, utility belt, and collar."""
    shirt, pants, skin, blood = look["shirt"], look["pants"], look["skin"], look.get("blood")
    jacket = look.get("jacket")
    outer = jacket if jacket else shirt
    hz, sz, cz, nz = lm["hips_z"], lm["spine_z"], lm["chest_z"], lm["neck_z"]
    hy, sy, cy, ny, fy = lm["hy"], lm["sy"], lm["cy"], lm["ny"], lm["fy"]

    # Pelvis and hips
    mb.add_loft(
        [
            ((0, hy - 0.004, hz - 0.075), 0.122, 0.080),
            ((0, hy, hz - 0.025), 0.156, 0.096),
            ((0, hy, hz + 0.055), 0.164, 0.100),
            ((0, hy, sz + 0.012), 0.162, 0.098),
        ],
        pants,
        "hips",
        segs=10,
    )
    # Sturdy leather utility belt + metallic buckle
    mb.add_loft(
        [
            ((0, hy, sz - 0.044), 0.169, 0.104),
            ((0, hy, sz + 0.004), 0.169, 0.104),
        ],
        "leather",
        "hips",
        segs=10,
    )
    mb.add_box((-0.028, hy + 0.098, sz - 0.038), (0.028, hy + 0.112, sz - 0.002), "chrome", "hips")

    # Compact flush hip utility pouches for survivor / player
    if look.get("chest_rig") or look.get("holster"):
        mb.add_box((-0.155, hy + 0.052, sz - 0.058), (-0.095, hy + 0.106, sz + 0.006), "canvas_khaki", "hips")
        mb.add_box((0.095, hy + 0.052, sz - 0.058), (0.155, hy + 0.106, sz + 0.006), "canvas_khaki", "hips")

    # Midriff (spine): full outer jacket/shirt volume so waist never looks pinched!
    my = (hy + sy) / 2
    mb.add_loft(
        [
            ((0, hy, sz - 0.020), 0.168, 0.103),
            ((0, my, sz + 0.085), 0.166, 0.102),
            ((0, sy, cz + 0.010), 0.176, 0.108),
        ],
        outer,
        "spine",
        segs=10,
    )

    # Athletic V-taper chest sloping smoothly into shoulders and neck
    pc = (sy + cy) / 2 + 0.005
    mb.add_loft(
        [
            ((0, sy, cz - 0.005), 0.175, 0.107),
            ((0, pc, cz + 0.115), 0.194, 0.116),
            ((0, cy, nz - 0.045), 0.200, 0.108),
            ((0, cy - 0.004, nz + 0.005), 0.130, 0.080),
        ],
        outer,
        "chest",
        segs=10,
    )

    if jacket:
        # Crisp open-jacket center placket showing the inner shirt cleanly down the front
        mb.add_box((-0.044, sy + 0.092, sz - 0.018), (0.044, sy + 0.111, cz + 0.008), shirt, "spine")
        mb.add_box((-0.046, pc + 0.096, cz - 0.002), (0.046, pc + 0.120, nz - 0.028), shirt, "chest")
        # Clean raised jacket lapel edges flanking the open center
        for sx in (-1, 1):
            x0, x1 = (0.044, 0.070) if sx > 0 else (-0.070, -0.044)
            mb.add_box((x0, sy + 0.095, sz - 0.018), (x1, sy + 0.115, cz + 0.008), jacket, "spine")
            mb.add_box((x0, pc + 0.100, cz - 0.002), (x1, pc + 0.124, nz - 0.032), jacket, "chest")
        # Buttoned chest flap pockets flush on left & right pecs
        for sx in (-1, 1):
            px0, px1 = (0.082, 0.145) if sx > 0 else (-0.145, -0.082)
            mb.add_box((px0, pc + 0.096, cz + 0.075), (px1, pc + 0.120, cz + 0.145), jacket, "chest")
            mb.add_box((px0 - 0.002, pc + 0.102, cz + 0.128), (px1 + 0.002, pc + 0.124, cz + 0.148), "leather", "chest")
        # Turned-up structured jacket collar framing the neck cleanly
        mb.add_loft(
            [
                ((0, cy - 0.004, nz - 0.022), 0.084, 0.078),
                ((0, cy - 0.006, nz + 0.034), 0.076, 0.072),
            ],
            jacket,
            "chest",
            segs=10,
        )
    else:
        # Standard shirt collar & single breast pocket
        mb.add_loft(
            [
                ((0, cy, nz - 0.025), 0.076, 0.072),
                ((0, cy + 0.002, nz + 0.026), 0.068, 0.064),
            ],
            outer,
            "chest",
            segs=8,
        )
        mb.add_box((-0.140, pc + 0.092, cz + 0.070), (-0.062, pc + 0.116, cz + 0.145), outer, "chest")

    # Compact shoulder radio mounted cleanly on left upper chest (zero torso clipping!)
    if look.get("chest_rig"):
        mb.add_box((-0.148, pc + 0.092, cz + 0.145), (-0.098, pc + 0.130, cz + 0.215), "gun_black", "chest")
        mb.add_box((-0.114, pc + 0.106, cz + 0.212), (-0.104, pc + 0.116, cz + 0.285), "tire_black", "chest")
        mb.add_box((-0.138, pc + 0.126, cz + 0.188), (-0.126, pc + 0.134, cz + 0.200), "carrot_orange", "chest")

    if blood:
        mb.add_box((0.02, sy + 0.082, sz + 0.03), (0.12, sy + 0.108, sz + 0.12), blood, "spine")
        mb.add_box((-0.14, pc + 0.094, cz + 0.08), (-0.02, pc + 0.120, nz - 0.07), blood, "chest")

    # Strong, well-proportioned athletic neck (nested cleanly into collar and jaw)
    mb.add_loft(
        [
            ((0, ny, nz - 0.035), 0.062, 0.058),
            ((0, (ny + fy) / 2, lm["head_z"] + 0.045), 0.056, 0.054),
        ],
        skin,
        "neck",
        segs=8,
    )


# Heroic stylized head profile over span = 0.24 m (chin at head_z = 1.54, skull crown at 1.78)
_HEAD = [
    (0.00, 0.044, 0.048, 0.024),  # defined chin
    (0.14, 0.072, 0.076, 0.015),  # crisp jawline
    (0.34, 0.088, 0.092, 0.005),  # cheekbones
    (0.54, 0.094, 0.098, 0.000),  # brow / eye level
    (0.76, 0.092, 0.098, -0.004), # forehead / temples
    (0.92, 0.074, 0.082, -0.008), # upper crown
    (1.00, 0.038, 0.044, -0.010), # top crown
]

# Conformal lower-jaw beard shell hugging exact chin/jaw curve (+4 mm proud, zero floating tape!)
_BEARD_JAW = [
    (-0.012, 0.047, 0.050, 0.026),
    (0.11, 0.074, 0.077, 0.017),
    (0.18, 0.082, 0.084, 0.010),
]


def _head(mb, look, lm, zombie):
    skin, hair = look["skin"], look["hair"]
    h, f = lm["head_z"], lm["fy"]
    span = min(0.242, lm["head_top"] - h)

    def prof(rows, extra=0.0):
        return [((0, f + yc, h + fr * span), rx + extra, ry + extra) for fr, rx, ry, yc in rows]

    mb.add_loft(prof(_HEAD), skin, "head", segs=10)

    # Subtle sculpted ears
    for s in (1, -1):
        mb.add_ellipsoid((s * 0.092, f - 0.004, h + 0.116), (0.013, 0.020, 0.028), skin, "head", segs=6, rings=3)

    # Subtle brow overhang ridge casting crisp low-poly shade over eyes
    mb.add_box((-0.068, f + 0.076, h + 0.136), (0.068, f + 0.098, h + 0.150), skin, "head")

    # Flush stylized eyes tucked neatly under the brow ridge (zero bug-eyes!)
    eye = "white" if not zombie else "rock_light"
    pupil = "hair_dark" if not zombie else "blood_dark"
    brow = hair if not zombie else "bark_dark"
    for s in (1, -1):
        cx = s * 0.036
        # Flat-flush sclera & iris seated under the 0.098 brow overhang
        mb.add_box((cx - 0.018, f + 0.082, h + 0.118), (cx + 0.018, f + 0.094, h + 0.132), eye, "head")
        mb.add_box((cx - 0.007, f + 0.089, h + 0.120), (cx + 0.007, f + 0.096, h + 0.131), pupil, "head")
        # Expressive brow bar
        mb.add_box((cx - 0.022, f + 0.084, h + 0.139), (cx + 0.022, f + 0.101, h + 0.149), brow, "head")

    # Sharp faceted low-poly nose bridge & tip
    mb.add_loft(
        [
            ((0, f + 0.094, h + 0.136), 0.010, 0.009),
            ((0, f + 0.112, h + 0.096), 0.016, 0.013),
            ((0, f + 0.098, h + 0.086), 0.014, 0.009),
        ],
        skin,
        "head",
        segs=5,
    )

    # Clean subtle mouth (zombies draw their open mouth/teeth in zombie_details.py)
    if not zombie:
        mb.add_box((-0.022, f + 0.084, h + 0.056), (0.022, f + 0.094, h + 0.063), "bark_dark", "head")

    # Optional jaw-hugging short beard (conformal shell + subtle chin stubble)
    beard_col = look.get("beard")
    if beard_col and not zombie:
        mb.add_loft(prof(_BEARD_JAW), beard_col, "head", segs=10)

    _hair(mb, look, h, span, f, zombie)


def _hair(mb, look, h, span, f, zombie):
    """Modular low-poly hairstyles replacing the receding bald cap."""
    hair = look["hair"]
    style = look.get("hair_style", "short_swept")

    if style == "none":
        return

    if style == "helmet_under":
        # Close-cropped hair tucked cleanly underneath helmets/caps/beanies
        cap = [
            (0.62, 0.093, 0.099, -0.008),
            (0.84, 0.086, 0.092, -0.010),
            (0.98, 0.052, 0.058, -0.010),
        ]
        mb.add_loft([((0, f + yc, h + fr * span), rx, ry) for fr, rx, ry, yc in cap], hair, "head", segs=8)
        return

    # Full natural-hairline base shell (starts cleanly at forehead brow line fr=0.69, covers back down to nape!)
    back_crown = [
        (0.32, 0.084, 0.068, -0.036),  # nape of neck in back
        (0.56, 0.096, 0.086, -0.022),  # behind ears
        (0.72, 0.098, 0.104, -0.006),  # full temple & forehead hairline!
        (0.88, 0.093, 0.099, -0.005),
        (1.01, 0.068, 0.075, -0.005),
        (1.065, 0.036, 0.040, -0.005),
    ]
    mb.add_loft([((0, f + yc, h + fr * span), rx, ry) for fr, rx, ry, yc in back_crown], hair, "head", segs=10)

    # Clean temple sideburns framing the sides of the face
    for s in (1, -1):
        mb.add_box(
            (s * 0.086 - 0.012, f + 0.004, h + 0.115),
            (s * 0.086 + 0.012, f + 0.044, h + 0.180),
            hair,
            "head",
        )

    if style == "short_swept":
        # Iconic rugged survivor swept hair volume / front quiff locks (zero bald forehead!)
        mb.add_box((-0.074, f + 0.008, h + 0.172), (0.074, f + 0.096, h + 0.212), hair, "head", rot=(-0.10, 0.0, 0.05))
        mb.add_box((-0.048, f + 0.018, h + 0.194), (0.078, f + 0.098, h + 0.236), hair, "head", rot=(-0.14, -0.08, 0.10))
        mb.add_box((-0.080, f - 0.025, h + 0.192), (0.052, f + 0.076, h + 0.246), hair, "head", rot=(-0.06, 0.10, -0.05))
    elif style == "slick":
        mb.add_box((-0.074, f - 0.015, h + 0.178), (0.074, f + 0.096, h + 0.226), hair, "head", rot=(-0.08, 0.0, 0.0))
    elif style == "long_ponytail":
        # Framing side bangs + full layered hair flowing down back + ponytail
        for s in (1, -1):
            mb.add_box((s * 0.078 - 0.016, f + 0.045, h + 0.135), (s * 0.078 + 0.016, f + 0.095, h + 0.205), hair, "head")
        mb.add_loft(
            [
                ((0, f - 0.058, h + 0.165), 0.072, 0.048),
                ((0, f - 0.082, h + 0.065), 0.056, 0.040),
                ((0, f - 0.092, h - 0.035), 0.036, 0.028),
            ],
            hair,
            "head",
            segs=8,
        )
    elif style == "wild" and not zombie:
        mb.add_box((-0.080, f + 0.010, h + 0.175), (0.080, f + 0.104, h + 0.232), hair, "head", rot=(-0.18, 0.08, 0.12))


# Curled fingers across palm: (offset along +Y, length)
_FINGERS = [(0.024, 0.062), (0.008, 0.070), (-0.008, 0.065), (-0.024, 0.052)]


def _arms(mb, look, lm, hunched, arm_points):
    """Seamless shoulders (deltoids on upper_arm!), athletic arms, snug watch, fingerless gloves."""
    skin = look["skin"]
    outer = look.get("jacket") or look["shirt"]
    sh_y = 0.0 if not hunched else 0.08
    sh_z = lm["hips_z"] + 0.52

    for sign, side in ((1, "R"), (-1, "L")):
        shoulder, elbow, wrist, tip = arm_points(sign, sh_y, sh_z)
        sleeve = skin if look.get("bare_arm") == side else outer
        a = (tip - wrist).normalized()
        back = Vector((sign * 0.5, 0.0, 0.866))
        across = Vector((0.0, 1.0, 0.0))

        # Sleek tucked clavicle bridge on shoulder_L/R (NO detached giant shoulder balloon!)
        clav = Vector((sign * 0.125, sh_y, sh_z + 0.004))
        mb.add_loft([(clav, 0.056, 0.056), (shoulder, 0.050, 0.050)], sleeve, "shoulder_" + side, segs=8)

        # Rounded deltoid shoulder cap ON upper_arm_L/R -> rotates cleanly WITH arm!
        mb.add_ellipsoid(shoulder, (0.058, 0.060, 0.056), sleeve, "upper_arm_" + side, segs=8, rings=4)

        # Athletic upper arm with clean elbow overlap
        mb.add_loft(
            [
                (shoulder, 0.056, 0.056),
                (_lerp(shoulder, elbow, 0.48), 0.054, 0.056),
                (elbow, 0.047, 0.047),
                (_lerp(elbow, wrist, 0.04), 0.045, 0.045),
            ],
            sleeve,
            "upper_arm_" + side,
            segs=8,
        )
        if sleeve != skin and not look.get("long_sleeves"):
            # Crisp rolled sleeve cuff just above elbow
            mb.add_loft(
                [
                    (_lerp(shoulder, elbow, 0.78), 0.053, 0.053),
                    (_lerp(shoulder, elbow, 0.96), 0.052, 0.052),
                ],
                sleeve,
                "upper_arm_" + side,
                segs=8,
            )

        fore_col = sleeve if (look.get("long_sleeves") and sleeve != skin) else skin
        # Seamless elbow joint sphere on forearm
        mb.add_ellipsoid(elbow, (0.047, 0.047, 0.047), fore_col, "forearm_" + side, segs=8, rings=4)
        # Muscular tapered forearm
        mb.add_loft(
            [
                (_lerp(elbow, shoulder, 0.03), 0.045, 0.045),
                (elbow, 0.045, 0.045),
                (_lerp(elbow, wrist, 0.30), 0.048, 0.046),
                (wrist, 0.035, 0.033),
            ],
            fore_col,
            "forearm_" + side,
            segs=8,
        )
        if look.get("long_sleeves") and sleeve != skin:
            mb.add_loft(
                [
                    (_lerp(elbow, wrist, 0.86), 0.040, 0.038),
                    (_lerp(elbow, wrist, 0.98), 0.039, 0.037),
                ],
                sleeve,
                "forearm_" + side,
                segs=8,
            )

        # Skin-tight wristwatch on LEFT wrist only (exact forearm radius at 0.84 is 0.037 -> watch 0.040!)
        if look.get("watch") and side == "L":
            w0 = _lerp(elbow, wrist, 0.80)
            w1 = _lerp(elbow, wrist, 0.89)
            mb.add_loft([(w0, 0.040, 0.038), (w1, 0.039, 0.037)], "tire_black", "forearm_L", segs=8)
            wmid = _lerp(w0, w1, 0.5) + back * 0.036
            mb.add_ellipsoid(wmid, (0.012, 0.014, 0.008), "chrome", "forearm_L", segs=6, rings=3)

        _hand(mb, skin, look.get("gloves"), "hand_" + side, side, wrist, a, back, across, not look.get("zombie"))


def _hand(mb, skin, glove, bone, side, wrist, a, back, across, include_fingers=True):
    """Sculpted low-poly hand with optional fingerless tactical glove."""
    palm_col = glove if glove else skin
    mb.add_loft(
        [
            (wrist, 0.033, 0.020),
            (wrist + a * 0.032, 0.042, 0.022),
            (wrist + a * 0.078, 0.040, 0.018),
        ],
        palm_col,
        bone,
        segs=6,
    )
    f_bones = ["fingers_1_" + side, "fingers_2_" + side, "fingers_2_" + side] if include_fingers else bone
    t_bones = ["thumb_1_" + side, "thumb_2_" + side, "thumb_2_" + side] if include_fingers else bone
    base_c = wrist + a * 0.072
    for dy, length in _FINGERS:
        base = base_c + across * dy
        mid = base + a * (length * 0.52) - back * 0.004
        tip = base + a * length - back * 0.014
        cols = [palm_col, skin] if glove else skin
        mb.add_loft([(base, 0.0095, 0.0085), (mid, 0.009, 0.008), (tip, 0.007, 0.0065)], cols, f_bones, segs=5)
    t0 = wrist + a * 0.022 + across * 0.028 - back * 0.006
    t1 = wrist + a * 0.064 + across * 0.046 - back * 0.012
    t_mid = (t0 + t1) * 0.5
    mb.add_loft([(t0, 0.012, 0.011), (t_mid, 0.0108, 0.010), (t1, 0.0095, 0.009)], skin, t_bones, segs=5)
