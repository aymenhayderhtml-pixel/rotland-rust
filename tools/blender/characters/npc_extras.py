"""Extra role gear for Rotland's 7 camp NPC looks (trader, banker, guard, survivor,
villager, mechanic, scout).

All coordinates match standing landmarks in humanoid_body.py:
hips_z=0.90, spine_z=1.04, chest_z=1.22, neck_z=1.46, head_z=1.54, head_top=1.80.
Every vertex sits rigidly on one bone (docs/RIG_CONTRACT.md) and uses Sheet A/B palette colors.
"""
from mathutils import Vector


def _cap(mb, crown_col, brim_col):
    """Snug outdoor field/trucker cap resting cleanly on the skull."""
    mb.add_loft(
        [
            ((0, -0.004, 1.710), 0.100, 0.106),
            ((0, -0.004, 1.770), 0.098, 0.104),
            ((0, -0.006, 1.812), 0.076, 0.084),
            ((0, -0.006, 1.828), 0.040, 0.044),
        ],
        crown_col,
        "head",
        segs=10,
    )
    # Angled front visor brim
    mb.add_box((-0.076, 0.078, 1.708), (0.076, 0.150, 1.724), brim_col, "head", rot=(-0.10, 0.0, 0.0))


def _trader(mb):
    """Quartermaster merchant: canvas apron, weathered cap, full grey beard, leather hip satchel."""
    # Canvas apron hugging front of hips & spine + waist tie band
    mb.add_box((-0.125, 0.090, 0.88), (0.125, 0.114, 1.04), "canvas_khaki", "hips")
    mb.add_box((-0.130, 0.092, 1.04), (0.130, 0.116, 1.25), "canvas_khaki", "spine")
    mb.add_box((-0.095, 0.104, 1.24), (0.095, 0.122, 1.38), "canvas_khaki", "chest")
    mb.add_box((-0.068, 0.108, 1.06), (0.068, 0.124, 1.16), "leather", "spine")
    _cap(mb, "horse_dark", "horse_dark")
    # Full sculpted grey beard, moustache & sideburns
    mb.add_loft(
        [
            ((0.0, 0.030, 1.505), 0.056, 0.052),
            ((0.0, 0.026, 1.545), 0.080, 0.074),
            ((0.0, 0.016, 1.588), 0.090, 0.084),
        ],
        "rock_light",
        "head",
        segs=10,
    )
    mb.add_box((-0.036, 0.088, 1.594), (0.036, 0.105, 1.609), "rock_light", "head")
    # Leather merchant satchel on left hip
    mb.add_box((-0.195, -0.050, 0.85), (-0.154, 0.052, 1.01), "leather", "hips")
    mb.add_box((-0.200, -0.014, 0.92), (-0.188, 0.014, 0.95), "brass", "hips")


def _banker(mb):
    """Distinguished settlement banker: crimson tie, spectacles, ledger pouch."""
    mb.add_box((-0.016, 0.112, 1.22), (0.016, 0.125, 1.43), "car_red", "chest")
    mb.add_box((-0.022, 0.106, 1.06), (0.022, 0.119, 1.23), "car_red", "spine")
    # Slim dark spectacles with tinted glass lenses sitting flush on nose bridge
    for s in (1, -1):
        cx = s * 0.036
        mb.add_box((cx - 0.023, 0.096, 1.654), (cx + 0.023, 0.106, 1.678), "gun_black", "head")
        mb.add_box((cx - 0.017, 0.101, 1.659), (cx + 0.017, 0.108, 1.673), "glass_tint", "head")
    mb.add_box((-0.014, 0.100, 1.664), (0.014, 0.107, 1.671), "gun_black", "head")
    # Leather ledger case on right hip
    mb.add_box((0.154, -0.052, 0.86), (0.196, 0.052, 1.02), "leather", "hips")
    mb.add_box((0.190, -0.014, 0.94), (0.201, 0.014, 0.97), "brass", "hips")


def _guard(mb, arm_points=None):
    """Armored camp defender: plate carrier vest, mag pouches, tactical helmet, arm armor pads."""
    mb.add_box((-0.165, 0.094, 1.22), (0.165, 0.126, 1.43), "tire_black", "chest")
    mb.add_box((-0.158, -0.124, 1.22), (0.158, -0.090, 1.43), "tire_black", "chest")
    for mx in (-0.105, 0.0, 0.105):
        mb.add_box((mx - 0.036, 0.118, 1.23), (mx + 0.036, 0.146, 1.33), "canvas_khaki", "chest")
    mb.add_loft(
        [
            ((0, -0.008, 1.682), 0.108, 0.116),
            ((0, -0.008, 1.745), 0.110, 0.118),
            ((0, -0.010, 1.798), 0.092, 0.100),
            ((0, -0.010, 1.832), 0.050, 0.056),
        ],
        "gun_black",
        "head",
        segs=10,
    )
    mb.add_loft(
        [((0, -0.005, 1.676), 0.117, 0.126), ((0, -0.005, 1.694), 0.117, 0.126)],
        "metal",
        "head",
        segs=10,
    )
    mb.add_box((-0.022, 0.108, 1.730), (0.022, 0.126, 1.770), "rock_dark", "head")
    for s in (1, -1):
        mb.add_box((s * 0.088 - 0.006, 0.002, 1.565), (s * 0.088 + 0.006, 0.016, 1.685), "tire_black", "head")
    if arm_points is not None:
        for sign, side in ((1, "R"), (-1, "L")):
            shoulder, elbow, _, _ = arm_points(sign, 0.0, 1.42)
            p_mid = shoulder.lerp(elbow, 0.28)
            mb.add_ellipsoid(p_mid, (0.064, 0.066, 0.062), "gun_black", "upper_arm_" + side, segs=8, rings=4)


def _survivor(mb):
    """Trail survivor: ribbed crimson survival beanie + back rucksack with olive bedroll."""
    mb.add_loft(
        [((0, -0.005, 1.695), 0.104, 0.110), ((0, -0.005, 1.732), 0.104, 0.110)],
        "car_red",
        "head",
        segs=10,
    )
    mb.add_loft(
        [
            ((0, -0.006, 1.725), 0.098, 0.104),
            ((0, -0.010, 1.788), 0.086, 0.094),
            ((0, -0.016, 1.832), 0.046, 0.052),
        ],
        "red_paint",
        "head",
        segs=10,
    )
    # Rucksack split cleanly across spine & chest + rolled olive bedroll + straps
    mb.add_box((-0.135, -0.185, 1.05), (0.135, -0.096, 1.22), "bark_dark", "spine")
    mb.add_box((-0.140, -0.195, 1.22), (0.140, -0.100, 1.43), "bark_dark", "chest")
    mb.add_box((-0.115, -0.212, 1.25), (0.115, -0.185, 1.38), "canvas_khaki", "chest")
    mb.add_loft(
        [((-0.155, -0.150, 1.465), 0.038, 0.038), ((0.155, -0.150, 1.465), 0.038, 0.038)],
        "cloth_olive",
        "chest",
        segs=8,
    )
    for s in (1, -1):
        mb.add_box((s * 0.100 - 0.018, -0.112, 1.24), (s * 0.100 + 0.018, 0.116, 1.43), "leather", "chest")


def _villager(mb):
    """Camp villager: warm woven orange neck scarf + canvas hip satchel."""
    mb.add_loft(
        [((0, 0.0, 1.435), 0.094, 0.088), ((0, 0.0, 1.495), 0.086, 0.080)],
        "carrot_orange",
        "chest",
        segs=10,
    )
    mb.add_box((-0.082, 0.096, 1.30), (-0.028, 0.124, 1.45), "carrot_orange", "chest")
    mb.add_box((0.145, -0.044, 0.85), (0.182, 0.046, 0.99), "canvas_khaki", "hips")


def _mechanic(mb):
    """Camp mechanic: denim overalls bib, red/black trucker cap, heavy tool belt & wrench."""
    mb.add_box((-0.125, 0.096, 1.04), (0.125, 0.116, 1.22), "pants_denim", "spine")
    mb.add_box((-0.115, 0.104, 1.22), (0.115, 0.124, 1.37), "pants_denim", "chest")
    for s in (1, -1):
        mb.add_box((s * 0.090 - 0.016, 0.098, 1.35), (s * 0.090 + 0.016, 0.120, 1.45), "pants_denim", "chest")
        mb.add_box((s * 0.090 - 0.010, 0.118, 1.355), (s * 0.090 + 0.010, 0.128, 1.375), "brass", "chest")
    _cap(mb, "car_red", "gun_black")
    mb.add_box((0.155, -0.054, 0.88), (0.198, 0.054, 1.02), "leather", "hips")
    mb.add_box((0.188, -0.012, 0.80), (0.206, 0.016, 0.98), "chrome", "hips")


def _scout(mb):
    """Wasteland scout/ranger: wide-brim olive boonie hat, khaki neck gaiter, metal hip canteen."""
    mb.add_loft(
        [
            ((0, -0.004, 1.712), 0.102, 0.108),
            ((0, -0.004, 1.785), 0.094, 0.100),
            ((0, -0.004, 1.805), 0.068, 0.074),
        ],
        "cloth_olive",
        "head",
        segs=10,
    )
    mb.add_loft(
        [((0, -0.004, 1.710), 0.106, 0.112), ((0, -0.004, 1.730), 0.106, 0.112)],
        "leather",
        "head",
        segs=10,
    )
    mb.add_loft(
        [((0, -0.002, 1.702), 0.156, 0.168), ((0, -0.002, 1.714), 0.152, 0.164)],
        "cloth_olive",
        "head",
        segs=12,
    )
    mb.add_loft(
        [((0, 0.0, 1.440), 0.090, 0.084), ((0, 0.0, 1.500), 0.082, 0.076)],
        "canvas_khaki",
        "chest",
        segs=10,
    )
    mb.add_ellipsoid((-0.180, -0.010, 0.92), (0.036, 0.052, 0.068), "metal", "hips", segs=8, rings=4)
    mb.add_box((-0.192, -0.022, 0.98), (-0.168, 0.002, 1.008), "gun_black", "hips")


_KINDS = {
    "trader": _trader,
    "banker": _banker,
    "guard": _guard,
    "survivor": _survivor,
    "villager": _villager,
    "mechanic": _mechanic,
    "scout": _scout,
}


def add_extras(mb, kind, arm_points=None):
    """Adds role-specific gear to MeshBuilder `mb`."""
    builder = _KINDS.get(kind)
    if builder is None:
        return
    if kind == "guard":
        builder(mb, arm_points=arm_points)
    else:
        builder(mb)
