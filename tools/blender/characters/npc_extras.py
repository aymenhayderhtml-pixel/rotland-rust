"""Extra parts for the NPC looks (trader, guard, survivor, villager) on top of the humanoid body.

Heights are the standing (not hunched) landmarks from humanoid_body: head 1.54 to 1.80 m,
chest front about 0.10 m forward. Each part sits on one bone, so the rigid weights stay exact
(see docs/RIG_CONTRACT.md).
"""


def _trader(mb):
    # White canvas apron over the shirt, from the belt to the chest
    mb.add_box((-0.120, 0.086, 0.92), (0.120, 0.104, 1.04), "white", "hips")
    mb.add_box((-0.125, 0.086, 1.04), (0.125, 0.104, 1.24), "white", "spine")
    # Flat cap: soft crown plus a front peak
    mb.add_loft([((0, -0.004, 1.735), 0.100, 0.108), ((0, -0.004, 1.795), 0.098, 0.106),
                 ((0, 0.0, 1.826), 0.080, 0.090)], "bark_dark", "head", segs=12)
    mb.add_box((-0.075, 0.085, 1.733), (0.075, 0.150, 1.748), "bark_dark", "head")
    # Full grey beard round the jaw, a moustache and sideburns
    mb.add_ellipsoid((0.0, 0.040, 1.582), (0.080, 0.068, 0.052), "rock_light", "head", segs=12, rings=5)
    mb.add_box((-0.034, 0.095, 1.598), (0.034, 0.110, 1.612), "rock_light", "head")
    for s in (1, -1):
        mb.add_box((s * 0.092 - 0.006, -0.02, 1.58), (s * 0.092 + 0.006, 0.05, 1.66), "rock_light", "head")


def _guard(mb):
    # Sandbag-coloured chest pouches over the black jacket
    mb.add_box((-0.150, 0.084, 1.24), (-0.025, 0.112, 1.40), "dirt", "chest")
    mb.add_box((0.025, 0.084, 1.24), (0.150, 0.112, 1.40), "dirt", "chest")
    # Round helmet over the hair, with a rim and a chin strap
    mb.add_loft([((0, -0.006, 1.690), 0.106, 0.114), ((0, -0.008, 1.750), 0.108, 0.116),
                 ((0, -0.010, 1.800), 0.090, 0.098), ((0, -0.010, 1.835), 0.048, 0.054)], "metal", "head", segs=12)
    mb.add_loft([((0, -0.004, 1.686), 0.116, 0.126), ((0, -0.004, 1.698), 0.116, 0.126)], "metal", "head", segs=12)
    for s in (1, -1):
        mb.add_box((s * 0.088 - 0.005, 0.0, 1.56), (s * 0.088 + 0.005, 0.012, 1.69), "bark_dark", "head")


def _survivor(mb):
    # Rucksack on the back: lower part on the spine, upper part on the chest, with a bedroll on top
    mb.add_box((-0.13, -0.165, 1.06), (-0.0, -0.085, 1.22), "bark_dark", "spine")
    mb.add_box((0.0, -0.165, 1.06), (0.13, -0.085, 1.22), "bark_dark", "spine")
    mb.add_box((-0.13, -0.170, 1.22), (0.13, -0.090, 1.40), "bark_dark", "chest")
    mb.add_loft([((-0.15, -0.135, 1.43), 0.035, 0.035), ((0.15, -0.135, 1.43), 0.035, 0.035)], "leaf_dark", "chest", segs=8)
    # Shoulder straps
    for s in (1, -1):
        mb.add_box((s * 0.09 - 0.018, -0.10, 1.30), (s * 0.09 + 0.018, 0.105, 1.42), "bark_dark", "chest")


def _villager(mb):
    # Long hair down the back of the head to the shoulders (on the head bone, so it stays rigid)
    mb.add_loft([((0, -0.040, 1.66), 0.094, 0.068), ((0, -0.052, 1.55), 0.090, 0.055),
                 ((0, -0.060, 1.43), 0.080, 0.040)], "bark_dark", "head", segs=10)


_KINDS = {"trader": _trader, "guard": _guard, "survivor": _survivor, "villager": _villager}


def add_extras(mb, kind):
    """Adds the NPC's extra parts to the MeshBuilder `mb`. `kind` None or unknown adds nothing."""
    builder = _KINDS.get(kind)
    if builder is not None:
        builder(mb)
