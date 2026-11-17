"""Farming models (V12-03): the soil plot, the three crops at three stages, the watering can and the hoe.

Crops are six small plants on a 2 x 2 m bed (two boxes each, 24 tris a plant) so every stage stays
under the 200-tri budget. Origin at the ground (the top of the plot's soil), as for every prop.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

PLANT_SPOTS = [(x, y) for x in (-0.55, 0.0, 0.55) for y in (-0.45, 0.45)]
SOIL_TOP = 0.14  # height of the tilled bed; plants stand on it


def _box(size, loc, color, rot=(0.0, 0.0, 0.0)):
    bm = common.make_box(size, color=color)
    common.transform(bm, rot=rot, loc=loc)
    return bm


def soil_plot(model):
    """A 2 x 2 m tilled bed: dark soil with four ridges and a plank border. Low collider (the slab)."""
    bed = [_box((2.0, 2.0, 0.10), (0.0, 0.0, 0.05), "soil_dark")]
    for y in (-0.6, -0.2, 0.2, 0.6):
        bed.append(_box((1.84, 0.16, 0.06), (0.0, y, 0.13), "soil_dark"))
    for y in (-1.0, 1.0):
        bed.append(_box((2.1, 0.1, 0.14), (0.0, y, 0.07), "wood_crate"))
    for x in (-1.0, 1.0):
        bed.append(_box((0.1, 2.0, 0.14), (x, 0.0, 0.07), "wood_crate"))
    for bm in bed:
        model.add(bm)
    model.add_collider("slab", common.make_box((2.0, 2.0, SOIL_TOP), loc=(0.0, 0.0, SOIL_TOP / 2),
                                               color="soil_dark"))


def _stalk(size, color, z, leaf=None):
    """One plant builder: a stem box of `size` at height z, plus an optional leaf box (size, colour, z, dx)."""
    def parts(x, y):
        out = [_box(size, (x, y, SOIL_TOP + z), color)]
        if leaf is not None:
            lsize, lcolor, lz, dx = leaf
            out.append(_box(lsize, (x + dx, y, SOIL_TOP + lz), lcolor))
        return out
    return parts


CROPS = {
    "wheat": {
        "sprout": _stalk((0.02, 0.02, 0.10), "sprout_green", 0.05, ((0.06, 0.01, 0.03), "sprout_green", 0.09, 0.02)),
        "young": _stalk((0.025, 0.025, 0.30), "crop_green", 0.15, ((0.12, 0.015, 0.03), "crop_green", 0.12, 0.03)),
        "ripe": _stalk((0.025, 0.025, 0.40), "crop_green", 0.20, ((0.05, 0.04, 0.14), "wheat_gold", 0.47, 0.0)),
    },
    "potato": {
        "sprout": _stalk((0.02, 0.02, 0.08), "sprout_green", 0.04, ((0.07, 0.05, 0.03), "sprout_green", 0.09, 0.0)),
        "young": _stalk((0.03, 0.03, 0.18), "crop_green", 0.09, ((0.20, 0.18, 0.08), "crop_green", 0.22, 0.0)),
        "ripe": _stalk((0.26, 0.24, 0.16), "crop_green", 0.22, ((0.08, 0.07, 0.05), "potato_brown", 0.025, 0.0)),
    },
    "carrot": {
        "sprout": _stalk((0.015, 0.015, 0.06), "sprout_green", 0.03, ((0.06, 0.01, 0.03), "sprout_green", 0.07, 0.0)),
        "young": _stalk((0.02, 0.02, 0.16), "crop_green", 0.08, ((0.14, 0.02, 0.10), "crop_green", 0.20, 0.0)),
        "ripe": _stalk((0.18, 0.012, 0.16), "crop_green", 0.28, ((0.04, 0.04, 0.05), "carrot_orange", 0.025, 0.0)),
    },
}


def crop_builder(crop, stage):
    """Build function for one crop stage: the same six plants on the bed."""
    plant = CROPS[crop][stage]

    def build(model):
        for x, y in PLANT_SPOTS:
            for bm in plant(x, y):
                model.add(bm)
    return build


def watering_can(model):
    """A 22 cm galvanized can: round body, long spout, loop handle, rose on the spout."""
    model.add(common.make_frustum(0.09, 0.09, 0.0, 0.22, segments=8, color="metal"))
    model.add(_box((0.22, 0.025, 0.025), (0.19, 0.0, 0.10), "metal"))
    model.add(_box((0.03, 0.03, 0.22), (-0.10, 0.0, 0.14), "metal"))
    model.add(_box((0.22, 0.03, 0.03), (0.0, 0.0, 0.27), "metal"))
    model.add(_box((0.05, 0.05, 0.03), (0.30, 0.0, 0.12), "rock_light"))


def hoe(model):
    """A 1 m hoe: long wooden handle, flat steel blade at the bottom (blade sits on the ground)."""
    model.add(_box((0.04, 0.04, 0.94), (0.0, 0.0, 0.53), "wood_crate"))
    model.add(_box((0.16, 0.03, 0.06), (0.05, 0.0, 0.03), "metal"))
    model.add(_box((0.05, 0.05, 0.10), (0.0, 0.0, 0.06), "bark_dark"))
