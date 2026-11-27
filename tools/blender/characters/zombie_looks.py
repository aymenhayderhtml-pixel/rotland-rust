"""R42 zombie looks: soldier, police, medic, firefighter.

Same rig, bones and size range as `zombie_01` (see humanoid.py / docs/RIG_CONTRACT.md): zombie skin,
blood and torn cloth. Each look is a recipe in build_characters.py (`zombie_soldier`, ...) plus the
role gear below, installed into the `extras` hook of npc_extras.py by `register()`.

Build only these four (Blender, no window):
  flock /tmp/rotland-blender.lock blender -b --python-expr \
    "import sys; sys.path.insert(0,'tools/blender/characters'); import zombie_looks; zombie_looks.build_all()"

Standing landmarks used below (humanoid_body.py):
- not hunched: hips_z=0.90, spine_z=1.04, chest_z=1.22, neck_z=1.46, head_z=1.54, head_top=1.80
  (all y offsets 0, chest front y = (sy+cy)/2+0.005 + 0.116)
- hunched (soldier): hips_z=0.88, spine_z=1.02, chest_z=1.20, neck_z=1.44, head_z=1.52, head_top=1.77,
  hy=0.02, sy=0.05, cy=0.10, ny=0.14, fy=0.17
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS_DIR = os.path.abspath(os.path.join(HERE, ".."))
sys.path.insert(0, TOOLS_DIR)
sys.path.insert(0, HERE)

import build_characters  # noqa: E402
import humanoid  # noqa: E402
import npc_extras  # noqa: E402

LOOK_NAMES = ["zombie_soldier", "zombie_police", "zombie_medic", "zombie_firefighter"]


def _box(mb, col, bone, x0, y0, z0, x1, y1, z1):
    mb.add_box((x0, y0, z0), (x1, y1, z1), col, bone)


def _band(mb, col, bone, z0, z1, r_side, r_up, y=0.0, x=0.0, segs=10):
    """Ring band around a limb/torso axis (a short loft between two sections)."""
    mb.add_loft([((x, y, z0), r_side, r_up), ((x, y, z1), r_side, r_up)], col, bone, segs=segs)


def _dome(mb, col, bone, sections, segs=10):
    """Stack of (y, z, r_side, r_up) rings: helmets and caps."""
    mb.add_loft([((0.0, y, z), rs, ru) for y, z, rs, ru in sections], col, bone, segs=segs)


def _tube(mb, col, bone, p0, r0, p1, r1, segs=6):
    """Round tube between two points (straps, tubes, cords)."""
    mb.add_loft([(p0, r0, r0), (p1, r1, r1)], col, bone, segs=segs)


def _soldier(mb):
    """Army line: olive fatigues + torn plate carrier, helmet with goggles, radio pouch."""
    pc = 0.080  # hunched chest y centre
    fy = 0.17  # hunched head y offset
    # Plate carrier: front plate, torn open on the lower right, back plate and shoulder straps
    _box(mb, "tire_black", "chest", -0.150, pc + 0.100, 1.300, 0.150, pc + 0.142, 1.420)
    _box(mb, "tire_black", "chest", -0.152, pc + 0.098, 1.200, 0.040, pc + 0.140, 1.300)
    _tube(mb, "tire_black", "chest", (0.100, pc + 0.126, 1.240), 0.030, (0.112, pc + 0.118, 1.140), 0.010)
    _tube(mb, "blood_dark", "chest", (0.052, pc + 0.128, 1.230), 0.026, (0.070, pc + 0.120, 1.150), 0.008)
    _box(mb, "tire_black", "chest", -0.140, pc - 0.152, 1.220, -0.014, pc - 0.110, 1.410)
    _box(mb, "tire_black", "chest", 0.006, pc - 0.152, 1.180, 0.140, pc - 0.110, 1.410)
    _box(mb, "canvas_khaki", "chest", -0.120, pc - 0.166, 1.300, -0.040, pc - 0.150, 1.372)
    _tube(mb, "blood_dark", "chest", (-0.132, pc - 0.138, 1.230), 0.020, (-0.104, pc - 0.130, 1.170), 0.006)
    for s in (1, -1):
        mb.add_loft([((s * 0.104, pc + 0.118, 1.400), 0.026, 0.013),
                     ((s * 0.104, 0.110, 1.470), 0.026, 0.013),
                     ((s * 0.104, pc - 0.118, 1.400), 0.026, 0.013)], "tire_black", "chest", segs=6)
    # Mag pouches on the front plate
    for mx in (-0.105, 0.0, 0.105):
        _box(mb, "canvas_khaki", "chest", mx - 0.034, pc + 0.140, 1.220, mx + 0.034, pc + 0.170, 1.320)
        _box(mb, "gun_black", "chest", mx - 0.030, pc + 0.166, 1.250, mx + 0.030, pc + 0.174, 1.262)
    # Radio in a pouch on the left of the carrier with a whip antenna
    _box(mb, "canvas_khaki", "chest", -0.152, pc + 0.132, 1.316, -0.098, pc + 0.176, 1.404)
    _box(mb, "gun_black", "chest", -0.120, pc + 0.142, 1.404, -0.110, pc + 0.154, 1.500)
    _box(mb, "carrot_orange", "chest", -0.146, pc + 0.174, 1.348, -0.136, pc + 0.182, 1.362)
    # Helmet: olive dome with a dark rim, goggles resting above the eyes
    _dome(mb, "cloth_olive", "head", [
        (fy - 0.004, 1.685, 0.116, 0.122),
        (fy - 0.004, 1.745, 0.112, 0.118),
        (fy - 0.006, 1.790, 0.070, 0.076),
        (fy - 0.006, 1.806, 0.030, 0.034),
    ])
    _band(mb, "gun_black", "head", 1.676, 1.694, 0.122, 0.128, y=fy - 0.004)
    # Goggles pushed up on the helmet front, above the eyes: strap band, lens block, brass frames
    _band(mb, "gun_black", "head", 1.690, 1.724, 0.106, 0.112, y=fy - 0.002, segs=8)
    _box(mb, "glass_tint", "head", -0.070, fy + 0.096, 1.694, 0.070, fy + 0.120, 1.720)
    _box(mb, "gun_black", "head", -0.074, fy + 0.100, 1.686, 0.074, fy + 0.118, 1.694)
    _box(mb, "gun_black", "head", -0.074, fy + 0.100, 1.720, 0.074, fy + 0.118, 1.728)
    _box(mb, "brass", "head", -0.008, fy + 0.104, 1.700, 0.008, fy + 0.118, 1.714)


def _police(mb):
    """Police station: dark-blue uniform shirt with a badge, duty belt and cap, dark trousers."""
    pc = 0.005
    # Badge patch on the left chest, above the shirt pocket
    _box(mb, "pants_dark", "chest", -0.148, 0.100, 1.348, -0.082, 0.120, 1.432)
    _box(mb, "brass", "chest", -0.142, 0.106, 1.354, -0.088, 0.128, 1.426)
    _box(mb, "chrome", "chest", -0.120, 0.122, 1.372, -0.110, 0.134, 1.410)
    # Duty belt over the shirt belt, with buckle, pouches, holster and radio
    _band(mb, "tire_black", "hips", 1.0 - 0.010, 1.052, 0.174, 0.107)
    _box(mb, "brass", "hips", -0.030, 0.104, 0.996, 0.030, 0.120, 1.036)
    for mx in (-0.078, 0.078):
        _box(mb, "gun_black", "hips", mx - 0.030, 0.100, 0.972, mx + 0.030, 0.122, 1.042)
    _box(mb, "gun_black", "hips", 0.150, -0.034, 0.820, 0.196, 0.040, 0.958)
    _box(mb, "bark_dark", "hips", 0.158, -0.012, 0.958, 0.186, 0.020, 0.998)
    _box(mb, "gun_black", "hips", -0.190, -0.030, 0.860, -0.150, 0.030, 0.986)
    _box(mb, "gun_black", "hips", -0.176, -0.010, 0.986, -0.166, 0.002, 1.048)
    # Uniform cap: blue crown, dark brim, cap badge
    _dome(mb, "shirt_blue", "head", [
        (-0.004, 1.712, 0.100, 0.106),
        (-0.004, 1.772, 0.098, 0.104),
        (-0.006, 1.814, 0.076, 0.084),
        (-0.006, 1.830, 0.040, 0.044),
    ])
    mb.add_box((-0.078, 0.078, 1.706), (0.078, 0.152, 1.724), "gun_black", "head", rot=(-0.10, 0.0, 0.0))
    _box(mb, "brass", "head", -0.016, 0.100, 1.738, 0.016, 0.114, 1.768)


def _medic(mb):
    """Hospital: green scrubs under a torn, blood-stained white coat, hanging mask, stethoscope."""
    pc = 0.005
    # Coat skirt hanging from the waist to mid-thigh (on the hips so the legs swing under it)
    mb.add_loft([((0.0, 0.0, 1.010), 0.170, 0.104),
                 ((0.0, 0.0, 0.840), 0.186, 0.116),
                 ((0.0, 0.0, 0.700), 0.196, 0.126)], "car_white", "hips", segs=12)
    # Jagged torn coat hem, one rag stained
    for x, y, drop, col in ((-0.150, 0.070, 0.075, "car_white"), (-0.070, 0.116, 0.055, "car_white"),
                            (0.010, 0.124, 0.090, "blood_dark"), (0.090, 0.110, 0.060, "car_white"),
                            (0.160, 0.058, 0.080, "car_white"), (-0.160, -0.070, 0.070, "car_white"),
                            (-0.060, -0.116, 0.065, "car_white"), (0.060, -0.116, 0.085, "car_white"),
                            (0.150, -0.076, 0.060, "blood_dark")):
        mb.add_loft([((x, y, 0.720), 0.024, 0.014), ((x * 1.02, y * 1.02, 0.720 - drop), 0.003, 0.003)],
                    col, "hips", segs=5)
    # Blood down the coat front and a torn sleeve on the right arm
    _box(mb, "blood_dark", "hips", -0.100, 0.100, 0.860, 0.010, 0.118, 1.000)
    _box(mb, "blood_dark", "chest", 0.026, pc + 0.118, 1.100, 0.120, pc + 0.130, 1.190)
    shoulder, elbow, wrist, _ = humanoid.arm_points(-1, 0.0, 1.42)
    _tube(mb, "blood_dark", "forearm_L", shoulder.lerp(elbow, 0.90), 0.046, wrist, 0.038)
    # Surgical mask hanging under the chin on its ear straps
    _box(mb, "white", "head", -0.048, 0.078, 1.432, 0.048, 0.108, 1.492)
    _box(mb, "rock_light", "head", -0.044, 0.104, 1.480, 0.044, 0.112, 1.490)
    _box(mb, "blood_dark", "head", -0.030, 0.104, 1.444, 0.024, 0.112, 1.462)
    for s in (1, -1):
        _tube(mb, "rock_light", "head", (s * 0.088, 0.006, 1.570), 0.006, (s * 0.046, 0.088, 1.494), 0.005)
    # Stethoscope: two tubes down the chest, chest piece on the right
    for s in (1, -1):
        mb.add_loft([((s * 0.062, pc + 0.104, 1.430), 0.012, 0.012),
                     ((s * 0.070, pc + 0.114, 1.330), 0.011, 0.011),
                     ((s * 0.022, pc + 0.120, 1.208), 0.010, 0.010)], "rock_dark", "chest", segs=5)
    mb.add_ellipsoid((0.022, pc + 0.126, 1.192), (0.024, 0.010, 0.024), "chrome", "chest", segs=8, rings=4)


def _firefighter(mb):
    """Fire station: tan turnout coat and trousers with reflective stripes, helmet, air tank, boots."""
    # Long turnout coat skirt from the waist to mid-thigh (on the hips, legs swing under it)
    mb.add_loft([((0.0, 0.0, 1.030), 0.172, 0.106),
                 ((0.0, 0.0, 0.890), 0.184, 0.114),
                 ((0.0, 0.0, 0.750), 0.198, 0.128)], "canvas_khaki", "hips", segs=12)
    # Torn coat hem: two rags shorter, one sooted
    for x, y, drop, col in ((-0.170, 0.060, 0.030, "canvas_khaki"), (0.150, 0.086, 0.040, "dirt"),
                            (-0.080, 0.122, 0.020, "canvas_khaki"), (0.060, 0.124, 0.045, "canvas_khaki"),
                            (0.175, -0.050, 0.035, "canvas_khaki"), (-0.070, -0.120, 0.040, "dirt")):
        mb.add_loft([((x, y, 0.760), 0.026, 0.014), ((x * 1.02, y * 1.02, 0.760 - drop), 0.003, 0.003)],
                    col, "hips", segs=5)
    # Reflective stripes: coat skirt (gold with a white core), chest band and sleeve cuffs
    _band(mb, "wheat_gold", "hips", 0.790, 0.856, 0.192, 0.132)
    _band(mb, "white", "hips", 0.812, 0.836, 0.196, 0.135)
    _band(mb, "wheat_gold", "chest", 1.262, 1.322, 0.198, 0.121)
    _band(mb, "white", "chest", 1.292, 1.320, 0.201, 0.124)
    # Trouser stripes hugging the pant cuffs above the boots
    for side, sx in (("R", 0.10), ("L", -0.10)):
        _band(mb, "wheat_gold", "shin_" + side, 0.160, 0.216, 0.051, 0.053, x=sx)
        _band(mb, "white", "shin_" + side, 0.178, 0.198, 0.053, 0.055, x=sx)
    for sign, side in ((1, "R"), (-1, "L")):
        shoulder, elbow, wrist, _ = humanoid.arm_points(sign, 0.0, 1.42)
        _tube(mb, "wheat_gold", "forearm_" + side, elbow.lerp(wrist, 0.78), 0.042,
              elbow.lerp(wrist, 0.96), 0.040)
    # Helmet: red dome with a wide rear brim and a front shield
    _dome(mb, "car_red", "head", [
        (-0.004, 1.690, 0.118, 0.124),
        (-0.004, 1.752, 0.114, 0.120),
        (-0.006, 1.798, 0.074, 0.080),
        (-0.006, 1.816, 0.032, 0.036),
    ])
    _band(mb, "car_red", "head", 1.680, 1.700, 0.152, 0.154, y=-0.032, segs=12)
    _box(mb, "wheat_gold", "head", -0.042, 0.100, 1.694, 0.042, 0.128, 1.752)
    _box(mb, "brass", "head", -0.014, 0.126, 1.710, 0.014, 0.134, 1.738)
    # Air tank on the upper back with a valve and shoulder straps
    mb.add_loft([((0.0, -0.198, 1.100), 0.072, 0.072),
                 ((0.0, -0.198, 1.400), 0.072, 0.072)], "metal", "chest", segs=10)
    _box(mb, "gun_black", "chest", -0.022, -0.220, 1.400, 0.022, -0.176, 1.446)
    _tube(mb, "gun_black", "chest", (-0.020, -0.196, 1.446), 0.010, (-0.070, -0.150, 1.500), 0.008)
    for s in (1, -1):
        mb.add_loft([((s * 0.100, 0.112, 1.352), 0.026, 0.013),
                     ((s * 0.100, 0.020, 1.452), 0.026, 0.013),
                     ((s * 0.100, -0.150, 1.352), 0.026, 0.013)], "tire_black", "chest", segs=6)


_EXTRA_KINDS = {
    "soldier": _soldier,
    "police": _police,
    "medic": _medic,
    "firefighter": _firefighter,
}


def register():
    """Installs the four role builders into the `extras` hook used by humanoid.build_humanoid."""
    npc_extras._KINDS.update(_EXTRA_KINDS)


def build_all():
    """Builds (and exports) the four zombie looks, then prints the triangle summary."""
    register()
    os.makedirs(build_characters.CHARACTERS_DIR, exist_ok=True)
    os.makedirs(build_characters.PREVIEWS_DIR, exist_ok=True)
    recipes = [v for v in build_characters.VARIANTS if v["name"] in LOOK_NAMES]
    missing = [n for n in LOOK_NAMES if n not in [v["name"] for v in recipes]]
    if missing:
        raise SystemExit(f"zombie_looks: recipes missing from build_characters.VARIANTS: {missing}")

    results = []
    for variant in recipes:
        tris, ok = build_characters.build_one_character(variant)
        results.append((variant["name"], tris, ok))

    print("\n--- ZOMBIE LOOKS BUILD SUMMARY ---")
    all_ok = all(ok for _, _, ok in results)
    for name, tris, ok in results:
        print(f"{name}: {tris} tris / {build_characters.BUDGET} budget [{'OK' if ok else 'FAILED'}]")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(build_all())
