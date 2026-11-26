"""attachments: 9 weapon attachment models (40-400 tris each), dark metal palette, origin at mount point.

Build coordinates: +Y is forward (barrel direction, -Z in engine), +Z is up (+Y in engine), 1 unit = 1 m.
Origin (0, 0, 0) is the mount point so each attachment sits flush on the gun's mount node.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, cyl_y, loft_y, rbox  # noqa: E402

CATEGORY = "weapons"
BUDGET = 400
GRIP = (0.0, 0.0, 0.0)


class _IconCollector:
    """Collects bmeshes from a build_fn(model) call for icon rendering."""

    def __init__(self):
        self.bms = []

    def add(self, bm):
        self.bms.append(bm)


def build_red_dot(model):
    # Rail clamp base, open reflex hood, tinted lens and glowing red dot center
    model.add(rbox((0.022, 0.056, 0.010), (0.0, 0.0, 0.004), "gun_black", 0.25))
    model.add(box((0.006, 0.012, 0.008), (0.013, 0.0, 0.004), "metal"))
    model.add(box((0.020, 0.044, 0.007), (0.0, 0.0, 0.0125), "rock_dark"))
    model.add(box((0.004, 0.018, 0.022), (-0.011, 0.008, 0.026), "gun_black"))
    model.add(box((0.004, 0.018, 0.022), (0.011, 0.008, 0.026), "gun_black"))
    model.add(box((0.026, 0.018, 0.004), (0.0, 0.008, 0.038), "gun_black"))
    model.add(box((0.018, 0.002, 0.018), (0.0, 0.010, 0.026), "water_blue"))
    model.add(box((0.004, 0.003, 0.004), (0.0, 0.010, 0.026), "red_paint"))


def build_scope_2x(model):
    # Compact 2x prism optic: rail mount, riser strut, flared body, lenses and fiber rib
    model.add(rbox((0.022, 0.072, 0.012), (0.0, 0.0, 0.005), "gun_black", 0.25))
    model.add(box((0.016, 0.044, 0.016), (0.0, 0.0, 0.018), "rock_dark"))
    model.add(loft_y([(-0.065, 0.017), (-0.035, 0.017), (-0.020, 0.013),
                      (0.020, 0.013), (0.040, 0.020), (0.065, 0.020)],
                     "gun_black", z=0.038, segments=10))
    model.add(cyl_y(0.016, 0.016, 0.064, 0.067, "water_blue", z=0.038, segments=8))
    model.add(cyl_y(0.014, 0.014, -0.067, -0.064, "water_blue", z=0.038, segments=8))
    model.add(box((0.008, 0.040, 0.007), (0.0, 0.010, 0.056), "red_paint"))
    model.add(rbox((0.014, 0.014, 0.010), (0.018, 0.0, 0.038), "metal", 0.3))


def build_scope_4x(model):
    # Long-range 4x rifle scope: rail base, two ring posts, sunshade bell, lenses and turrets
    model.add(rbox((0.022, 0.130, 0.010), (0.0, 0.0, 0.004), "gun_black", 0.25))
    for y in (-0.045, 0.045):
        model.add(box((0.018, 0.014, 0.044), (0.0, y, 0.024), "rock_dark"))
    model.add(loft_y([(-0.125, 0.024), (-0.085, 0.024), (-0.060, 0.017),
                      (0.065, 0.017), (0.105, 0.027), (0.165, 0.027)],
                     "gun_black", z=0.048, segments=10))
    model.add(cyl_y(0.028, 0.028, 0.145, 0.162, "metal", z=0.048, segments=10))
    model.add(cyl_y(0.022, 0.022, 0.164, 0.167, "water_blue", z=0.048, segments=8))
    model.add(cyl_y(0.020, 0.020, -0.127, -0.124, "water_blue", z=0.048, segments=8))
    model.add(rbox((0.016, 0.016, 0.024), (0.0, 0.0, 0.072), "metal", 0.3))
    model.add(rbox((0.018, 0.016, 0.016), (0.024, 0.0, 0.048), "brass", 0.3))


def build_suppressor(model):
    # Thread collar, dark baffle cylinder, grip band and muzzle endcap extending along +Y
    model.add(cyl_y(0.013, 0.015, -0.005, 0.020, "metal", segments=10))
    model.add(cyl_y(0.018, 0.018, 0.020, 0.175, "gun_black", segments=10))
    model.add(cyl_y(0.0195, 0.0195, 0.065, 0.095, "rock_dark", segments=10))
    model.add(cyl_y(0.016, 0.014, 0.175, 0.185, "metal", segments=10))
    model.add(cyl_y(0.007, 0.007, 0.184, 0.186, "gun_black", segments=6))


def build_ext_mag(model):
    # Extended vertical magazine sleeve, steel body, basepad and brass side witness strip
    model.add(rbox((0.036, 0.058, 0.057), (0.0, 0.0, -0.0165), "gun_black", 0.2))
    model.add(rbox((0.034, 0.054, 0.053), (0.0, 0.0, -0.0685), "metal", 0.2))
    model.add(rbox((0.038, 0.060, 0.014), (0.0, 0.0, -0.100), "rock_dark", 0.25))
    model.add(box((0.037, 0.010, 0.040), (0.0, -0.012, -0.065), "brass"))


def build_drum_mag(model):
    # High-capacity drum magazine with upper feed tower and rear winding key
    model.add(rbox((0.038, 0.054, 0.080), (0.0, 0.0, 0.055), "gun_black", 0.2))
    model.add(cyl_y(0.080, 0.080, -0.036, 0.036, "gun_black", z=-0.005, segments=12))
    model.add(cyl_y(0.055, 0.055, -0.041, 0.041, "metal", z=-0.005, segments=10))
    model.add(cyl_y(0.022, 0.022, -0.046, 0.046, "rock_dark", z=-0.005, segments=8))
    model.add(box((0.050, 0.008, 0.012), (0.0, -0.046, -0.005), "brass"))


def build_foregrip(model):
    # Vertical foregrip with rail clamp, ribbed shaft and bottom cap extending along -Z
    model.add(rbox((0.028, 0.044, 0.014), (0.0, 0.0, -0.005), "metal", 0.25))
    model.add(common.make_frustum(0.018, 0.020, -0.024, -0.010, segments=8, color="rock_dark"))
    model.add(common.make_frustum(0.015, 0.017, -0.092, -0.024, segments=8, color="gun_black"))
    model.add(common.make_frustum(0.013, 0.016, -0.100, -0.092, segments=8, color="metal"))
    model.add(common.make_frustum(0.0175, 0.0175, -0.048, -0.040, segments=8, color="rock_dark"))
    model.add(common.make_frustum(0.017, 0.017, -0.072, -0.064, segments=8, color="rock_dark"))


def build_stock_pad(model):
    # Steel backing plate and ribbed rubber recoil pad extending along -Y
    model.add(rbox((0.046, 0.016, 0.128), (0.0, -0.004, 0.0), "metal", 0.25))
    model.add(rbox((0.048, 0.030, 0.132), (0.0, -0.027, 0.0), "gun_black", 0.3))
    for z in (-0.035, 0.0, 0.035):
        model.add(box((0.050, 0.012, 0.018), (0.0, -0.032, z), "rock_dark"))


def build_bolt_quiver(model):
    # Side-mounted crossbow quiver (+X) with broadhead hood, clip and 3 racked bolts
    model.add(box((0.034, 0.024, 0.016), (0.015, 0.0, 0.0), "metal"))
    model.add(rbox((0.014, 0.20, 0.020), (0.032, -0.01, 0.0), "gun_black", 0.2))
    model.add(rbox((0.026, 0.046, 0.058), (0.042, 0.092, 0.0), "rock_dark", 0.2))
    model.add(box((0.024, 0.016, 0.054), (0.042, -0.080, 0.0), "gun_black"))
    for bz in (-0.018, 0.0, 0.018):
        model.add(cyl_y(0.0035, 0.0035, -0.14, 0.08, "gun_black", x=0.045, z=bz, segments=6))
        model.add(box((0.003, 0.030, 0.012), (0.045, -0.120, bz), "red_paint"))
        model.add(box((0.006, 0.008, 0.006), (0.045, -0.144, bz), "brass"))


BUILD_FNS = {
    "scope_2x": build_scope_2x,
    "scope_4x": build_scope_4x,
    "red_dot": build_red_dot,
    "suppressor": build_suppressor,
    "ext_mag": build_ext_mag,
    "drum_mag": build_drum_mag,
    "foregrip": build_foregrip,
    "stock_pad": build_stock_pad,
    "bolt_quiver": build_bolt_quiver,
}
ATTACHMENT_IDS = tuple(BUILD_FNS.keys())


def icon_meshes(att_id):
    """Returns bmeshes for `att_id`, for use by tools/blender/icons/item_meshes.py."""
    collector = _IconCollector()
    BUILD_FNS[att_id](collector)
    return collector.bms


ICON_BUILDERS = {att_id: (lambda k=att_id: icon_meshes(k)) for att_id in ATTACHMENT_IDS}
BUILDERS = ICON_BUILDERS


def build_one(att_id):
    return common.run_model(att_id, CATEGORY, BUDGET, BUILD_FNS[att_id], grip=GRIP)


def main():
    failures = [att_id for att_id in ATTACHMENT_IDS if not build_one(att_id)]
    if failures:
        print("ATTACHMENTS BUILD FAILED:", ", ".join(failures))
        return 1
    print("ATTACHMENTS BUILD OK:", len(ATTACHMENT_IDS), "attachments within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
