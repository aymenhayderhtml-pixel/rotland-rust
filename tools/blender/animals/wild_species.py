"""Wild animal data (V12-01): deer (doe), deer_buck (antlers), rabbit, boar.

Bone names and layout follow docs/ANIMAL_CONTRACT.md (the dog's bone list). Only the sizes differ.
Metres, Blender axes: +Y is forward (-Z in Godot), +Z is up. Each species lists its bones (name:
(parent, head, tail)), its legs and its mesh parts. A part is rigid: 100% on one bone.
Parts: ("seg", bone, p0, p1, width, depth, colour) runs along a bone; ("box", bone, size, loc, colour).
"""

LEG_NAMES = ["FL", "FR", "BL", "BR"]

# Shared body bones for the quadruped rig (same names as the dog)
_BODY_BONES = ["root", "hips", "spine", "chest", "neck", "head", "jaw", "ear_L", "ear_R", "tail_1", "tail_2"]


def _deer_bones():
    return {
        "root": (None, (0.0, 0.0, 0.0), (0.0, 0.0, 0.10)),
        "hips": ("root", (0.0, -0.42, 0.98), (0.0, -0.26, 0.98)),
        "spine": ("hips", (0.0, -0.26, 0.98), (0.0, 0.02, 1.00)),
        "chest": ("spine", (0.0, 0.02, 1.00), (0.0, 0.18, 1.00)),
        "neck": ("chest", (0.0, 0.18, 1.00), (0.0, 0.36, 1.26)),
        "head": ("neck", (0.0, 0.36, 1.26), (0.0, 0.58, 1.18)),
        "jaw": ("head", (0.0, 0.44, 1.20), (0.0, 0.58, 1.14)),
        "ear_L": ("head", (-0.06, 0.34, 1.30), (-0.20, 0.30, 1.36)),
        "ear_R": ("head", (0.06, 0.34, 1.30), (0.20, 0.30, 1.36)),
        "tail_1": ("hips", (0.0, -0.62, 0.96), (0.0, -0.70, 0.92)),
        "tail_2": ("tail_1", (0.0, -0.70, 0.92), (0.0, -0.74, 0.90)),
    }


def _deer_parts(coat="deer_fur", pale="deer_pale", dark="bark_dark"):
    return [
        # Round rump, barrel and chest, each on its own bone
        ("ell", "hips", (0.15, 0.22, 0.18), (0.0, -0.42, 0.95), coat),
        ("ell", "spine", (0.14, 0.24, 0.17), (0.0, -0.18, 0.94), coat),
        ("ell", "chest", (0.15, 0.20, 0.20), (0.0, 0.04, 0.94), coat),
        ("ell", "hips", (0.11, 0.03, 0.12), (0.0, -0.63, 0.96), pale),  # white rump patch
        ("ell", "spine", (0.10, 0.26, 0.05), (0.0, -0.16, 0.79), pale),  # pale belly
        ("tube", "neck", [((0.0, 0.14, 0.98), 0.09, 0.10), ((0.0, 0.27, 1.13), 0.065, 0.075),
                          ((0.0, 0.37, 1.27), 0.055, 0.065)], coat),
        # Long head: rounded skull tapering to a slim muzzle, dark nose, eyes
        ("tube", "head", [((0.0, 0.33, 1.28), 0.06, 0.065), ((0.0, 0.42, 1.27), 0.07, 0.075),
                          ((0.0, 0.55, 1.20), 0.045, 0.05), ((0.0, 0.66, 1.16), 0.03, 0.035)], coat),
        ("ell", "head", (0.025, 0.02, 0.022), (0.0, 0.665, 1.165), dark),
        ("ell", "head", (0.016, 0.016, 0.018), (0.062, 0.45, 1.29), dark),
        ("ell", "head", (0.016, 0.016, 0.018), (-0.062, 0.45, 1.29), dark),
        ("ell", "head", (0.04, 0.10, 0.02), (0.0, 0.52, 1.15), pale),  # pale chin and throat
        ("seg", "jaw", (0.0, 0.44, 1.20), (0.0, 0.60, 1.14), 0.07, 0.06, coat),
        # Big leaf-shaped ears
        ("tube", "ear_L", [((-0.06, 0.34, 1.31), 0.02, 0.01), ((-0.13, 0.32, 1.34), 0.045, 0.012),
                           ((-0.21, 0.30, 1.37), 0.006, 0.004)], coat),
        ("tube", "ear_R", [((0.06, 0.34, 1.31), 0.02, 0.01), ((0.13, 0.32, 1.34), 0.045, 0.012),
                           ((0.21, 0.30, 1.37), 0.006, 0.004)], coat),
        ("seg", "tail_1", (0.0, -0.60, 0.98), (0.0, -0.70, 0.93), 0.08, 0.06, coat),
        ("seg", "tail_2", (0.0, -0.69, 0.93), (0.0, -0.75, 0.90), 0.06, 0.04, pale),
    ]


def _antlers(colour="antler"):
    parts = []
    for x in (-1.0, 1.0):
        parts += [
            ("seg", "head", (0.05 * x, 0.40, 1.30), (0.10 * x, 0.34, 1.62), 0.03, 0.03, colour),  # beam
            ("seg", "head", (0.08 * x, 0.36, 1.46), (0.20 * x, 0.34, 1.52), 0.025, 0.025, colour),  # tine
            ("seg", "head", (0.10 * x, 0.35, 1.56), (0.18 * x, 0.28, 1.66), 0.025, 0.025, colour),  # tine
        ]
    return parts


def _deer():
    return {
        "style": "deer",
        "bones": _deer_bones(),
        "legs": {  # parent, x, y, top z (the joint in rest pose)
            "FL": ("chest", -0.13, 0.12, 1.00), "FR": ("chest", 0.13, 0.12, 1.00),
            "BL": ("hips", -0.13, -0.42, 0.98), "BR": ("hips", 0.13, -0.42, 0.98),
        },
        "knee_z": 0.50, "ankle_z": 0.06, "paw_len": 0.10, "paw_w": 0.07,
        "upper_w": 0.08, "lower_w": 0.065,
        "coat": "deer_fur", "hoof": "bark_dark",
        "parts": _deer_parts(),
        "budget": 2500,
    }


def _rabbit():
    return {
        "style": "rabbit",
        "bones": {
            "root": (None, (0.0, 0.0, 0.0), (0.0, 0.0, 0.04)),
            "hips": ("root", (0.0, -0.09, 0.19), (0.0, -0.04, 0.19)),
            "spine": ("hips", (0.0, -0.04, 0.19), (0.0, 0.02, 0.20)),
            "chest": ("spine", (0.0, 0.02, 0.20), (0.0, 0.08, 0.21)),
            "neck": ("chest", (0.0, 0.08, 0.21), (0.0, 0.11, 0.25)),
            "head": ("neck", (0.0, 0.11, 0.25), (0.0, 0.19, 0.27)),
            "jaw": ("head", (0.0, 0.14, 0.23), (0.0, 0.19, 0.22)),
            "ear_L": ("head", (-0.025, 0.11, 0.29), (-0.035, 0.10, 0.41)),
            "ear_R": ("head", (0.025, 0.11, 0.29), (0.035, 0.10, 0.41)),
            "tail_1": ("hips", (0.0, -0.12, 0.19), (0.0, -0.15, 0.20)),
            "tail_2": ("tail_1", (0.0, -0.15, 0.20), (0.0, -0.16, 0.21)),
        },
        "legs": {
            "FL": ("chest", -0.045, 0.06, 0.19), "FR": ("chest", 0.045, 0.06, 0.19),
            "BL": ("hips", -0.05, -0.10, 0.19), "BR": ("hips", 0.05, -0.10, 0.19),
        },
        "knee_z": 0.08, "ankle_z": 0.02, "paw_len": 0.05, "paw_w": 0.035,
        "upper_w": 0.04, "lower_w": 0.03,
        "coat": "rabbit_fur", "hoof": "rabbit_pale",
        "parts": [
            ("ell", "hips", (0.065, 0.075, 0.075), (0.0, -0.08, 0.18), "rabbit_fur"),
            ("ell", "chest", (0.055, 0.06, 0.065), (0.0, 0.04, 0.19), "rabbit_fur"),
            ("ell", "spine", (0.045, 0.09, 0.025), (0.0, -0.02, 0.12), "rabbit_pale"),  # belly
            ("ell", "head", (0.042, 0.055, 0.042), (0.0, 0.16, 0.265), "rabbit_fur"),
            ("ell", "head", (0.028, 0.03, 0.026), (0.0, 0.205, 0.25), "rabbit_pale"),  # muzzle
            ("ell", "head", (0.01, 0.006, 0.008), (0.0, 0.234, 0.255), "bark_dark"),  # nose
            ("ell", "head", (0.01, 0.01, 0.011), (0.036, 0.18, 0.275), "bark_dark"),  # eyes
            ("ell", "head", (0.01, 0.01, 0.011), (-0.036, 0.18, 0.275), "bark_dark"),
            ("seg", "jaw", (0.0, 0.14, 0.23), (0.0, 0.19, 0.22), 0.035, 0.025, "rabbit_fur"),
            ("tube", "ear_L", [((-0.022, 0.11, 0.29), 0.012, 0.006), ((-0.03, 0.105, 0.35), 0.02, 0.007),
                               ((-0.036, 0.10, 0.41), 0.006, 0.004)], "rabbit_fur"),
            ("tube", "ear_R", [((0.022, 0.11, 0.29), 0.012, 0.006), ((0.03, 0.105, 0.35), 0.02, 0.007),
                               ((0.036, 0.10, 0.41), 0.006, 0.004)], "rabbit_fur"),
            ("ell", "tail_1", (0.03, 0.03, 0.03), (0.0, -0.145, 0.20), "rabbit_pale"),
            ("ell", "tail_2", (0.015, 0.015, 0.015), (0.0, -0.165, 0.21), "rabbit_pale"),
        ],
        "budget": 1500,
    }


def _boar():
    return {
        "style": "boar",
        "bones": {
            "root": (None, (0.0, 0.0, 0.0), (0.0, 0.0, 0.10)),
            "hips": ("root", (0.0, -0.36, 0.72), (0.0, -0.22, 0.73)),
            "spine": ("hips", (0.0, -0.22, 0.73), (0.0, 0.0, 0.80)),
            "chest": ("spine", (0.0, 0.0, 0.80), (0.0, 0.20, 0.80)),
            "neck": ("chest", (0.0, 0.20, 0.80), (0.0, 0.32, 0.66)),
            "head": ("neck", (0.0, 0.32, 0.66), (0.0, 0.56, 0.50)),
            "jaw": ("head", (0.0, 0.36, 0.58), (0.0, 0.52, 0.50)),
            "ear_L": ("head", (-0.07, 0.32, 0.66), (-0.12, 0.28, 0.76)),
            "ear_R": ("head", (0.07, 0.32, 0.66), (0.12, 0.28, 0.76)),
            "tail_1": ("hips", (0.0, -0.54, 0.74), (0.0, -0.60, 0.70)),
            "tail_2": ("tail_1", (0.0, -0.60, 0.70), (0.0, -0.63, 0.68)),
        },
        "legs": {
            "FL": ("chest", -0.12, 0.12, 0.80), "FR": ("chest", 0.12, 0.12, 0.80),
            "BL": ("hips", -0.12, -0.34, 0.72), "BR": ("hips", 0.12, -0.34, 0.72),
        },
        "knee_z": 0.30, "ankle_z": 0.05, "paw_len": 0.10, "paw_w": 0.10,
        "upper_w": 0.15, "lower_w": 0.14,
        "coat": "boar_hide", "hoof": "boar_bristle",
        "parts": [
            # Heavy, low-slung body: the round chest and rump reach down to about 0.38 m
            ("ell", "hips", (0.19, 0.24, 0.25), (0.0, -0.30, 0.64), "boar_hide"),
            ("ell", "spine", (0.20, 0.22, 0.27), (0.0, -0.08, 0.66), "boar_hide"),
            ("ell", "chest", (0.21, 0.22, 0.28), (0.0, 0.10, 0.66), "boar_hide"),
            # Bristle mane: a row of short tufts along the back
            *[("tube", "spine" if y < 0.0 else "chest", [((0.0, y, 0.88 + 0.03 * (y > -0.1)), 0.035, 0.03),
                                                         ((0.0, y - 0.04, 0.97 + 0.03 * (y > -0.1)), 0.006, 0.006)],
               "boar_bristle") for y in (-0.38, -0.26, -0.14, -0.02, 0.10, 0.20)],
            ("tube", "neck", [((0.0, 0.18, 0.78), 0.15, 0.16), ((0.0, 0.32, 0.68), 0.12, 0.13)], "boar_hide"),
            # Wedge head down to a flat snout disc, small eyes
            ("tube", "head", [((0.0, 0.30, 0.70), 0.12, 0.13), ((0.0, 0.43, 0.59), 0.085, 0.09),
                              ((0.0, 0.56, 0.50), 0.05, 0.05)], "boar_hide"),
            ("tube", "head", [((0.0, 0.555, 0.503), 0.055, 0.046), ((0.0, 0.59, 0.49), 0.055, 0.046)], "boar_bristle"),
            ("ell", "head", (0.014, 0.014, 0.014), (0.085, 0.42, 0.66), "boar_bristle"),
            ("ell", "head", (0.014, 0.014, 0.014), (-0.085, 0.42, 0.66), "boar_bristle"),
            ("seg", "jaw", (0.0, 0.36, 0.58), (0.0, 0.53, 0.49), 0.12, 0.09, "boar_hide"),
            # Curved tusks from the snout sides, forward then up
            ("tube", "jaw", [((0.07, 0.48, 0.50), 0.014, 0.014), ((0.10, 0.57, 0.52), 0.011, 0.011),
                             ((0.10, 0.61, 0.59), 0.004, 0.004)], "boar_tusk"),
            ("tube", "jaw", [((-0.07, 0.48, 0.50), 0.014, 0.014), ((-0.10, 0.57, 0.52), 0.011, 0.011),
                             ((-0.10, 0.61, 0.59), 0.004, 0.004)], "boar_tusk"),
            ("tube", "ear_L", [((-0.07, 0.32, 0.67), 0.03, 0.012), ((-0.12, 0.28, 0.77), 0.005, 0.004)], "boar_bristle"),
            ("tube", "ear_R", [((0.07, 0.32, 0.67), 0.03, 0.012), ((0.12, 0.28, 0.77), 0.005, 0.004)], "boar_bristle"),
            ("seg", "tail_1", (0.0, -0.52, 0.76), (0.0, -0.60, 0.70), 0.04, 0.04, "boar_hide"),
            ("seg", "tail_2", (0.0, -0.60, 0.70), (0.0, -0.64, 0.66), 0.03, 0.03, "boar_bristle"),
        ],
        "budget": 2500,
    }


def _with(base, **changes):
    out = dict(base)
    out.update(changes)
    return out


# Every model the build script makes. deer_buck is the same rig and clips with antlers added.
SPECIES = {
    "deer": _deer(),
    "deer_buck": _with(_deer(), parts=_deer_parts() + _antlers()),
    "rabbit": _rabbit(),
    "boar": _boar(),
}
