"""Simple icon meshes for the non-weapon items in data/items.json.

Each builder returns a list of bmeshes (built in build coordinates, 1 unit = 1 m, z up).
Weapons do not live here: render_icons.py uses their weapon model scripts instead.
"""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import warm_clothes_meshes  # noqa: E402


def _cyl(r0, r1, z0, z1, color, seg=10):
    return common.make_frustum(r0, r1, z0, z1, segments=seg, color=color)


def _box(size, loc, color, rot=(0.0, 0.0, 0.0)):
    bm = common.make_box(size, color=color)
    common.transform(bm, rot=rot, loc=loc)
    return bm


def _blob(subdiv, jitter, seed, radius, loc, color):
    return common.make_icosphere(subdiv=subdiv, jitter=jitter, seed=seed, radius=radius, loc=loc, color=color)


def _bottle(body, cap):
    return [_cyl(0.04, 0.04, 0.0, 0.16, body),
            _cyl(0.04, 0.018, 0.16, 0.19, body),
            _cyl(0.018, 0.018, 0.19, 0.21, body),
            _cyl(0.02, 0.02, 0.21, 0.245, cap)]


def canned_beans():
    return [_cyl(0.06, 0.06, 0.0, 0.12, "metal"),
            _cyl(0.061, 0.061, 0.04, 0.08, "brass"),
            _cyl(0.055, 0.055, 0.12, 0.125, "rock_light")]


def apple():
    return [_blob(2, 0.08, 1, (0.06, 0.06, 0.055), (0.0, 0.0, 0.06), "red_paint"),
            _cyl(0.003, 0.003, 0.10, 0.14, "bark_dark"),
            _box((0.05, 0.02, 0.02), (0.03, 0.0, 0.13), "leaf_mid", rot=(0.0, 0.0, 0.4))]


def raw_meat():
    return [_blob(2, 0.25, 3, (0.09, 0.06, 0.04), (0.0, 0.0, 0.04), "red_paint"),
            _blob(1, 0.2, 4, (0.03, 0.02, 0.012), (0.02, 0.01, 0.07), "white")]


def cooked_meat():
    return [_blob(2, 0.25, 5, (0.09, 0.06, 0.04), (0.0, 0.0, 0.04), "bark_light"),
            _blob(1, 0.2, 6, (0.05, 0.04, 0.02), (0.0, 0.0, 0.07), "wood_crate")]


def water_bottle():
    return _bottle("gown_blue", "water_blue")


def dirty_water():
    return _bottle("dirt", "bark_dark")


def soda():
    return [_cyl(0.035, 0.035, 0.0, 0.12, "soda_purple"),
            _cyl(0.036, 0.036, 0.05, 0.075, "white"),
            _cyl(0.03, 0.03, 0.12, 0.125, "rock_light")]


def bandage():
    return [_cyl(0.06, 0.06, 0.0, 0.04, "white", seg=12),
            _box((0.10, 0.02, 0.005), (0.0, 0.0, 0.0425), "red_paint"),
            _box((0.02, 0.10, 0.005), (0.0, 0.0, 0.0425), "red_paint")]


def medkit():
    return [_box((0.20, 0.12, 0.14), (0.0, 0.0, 0.07), "white"),
            _box((0.12, 0.006, 0.04), (0.0, -0.061, 0.07), "red_paint"),
            _box((0.04, 0.006, 0.12), (0.0, -0.061, 0.07), "red_paint"),
            _box((0.06, 0.03, 0.02), (0.0, 0.0, 0.155), "metal")]


def pistol_ammo():
    return [_box((0.16, 0.10, 0.10), (0.0, 0.0, 0.05), "brass"),
            _box((0.165, 0.035, 0.02), (0.0, 0.0, 0.10), "gun_black")]


def wood():
    return [_box((0.36, 0.10, 0.04), (0.0, 0.0, 0.02), "wood_crate"),
            _box((0.36, 0.10, 0.04), (0.01, 0.0, 0.06), "bark_light"),
            _box((0.36, 0.10, 0.04), (0.0, 0.0, 0.10), "wood_crate")]


def _bent_sheet(length, width, thick, height, center, yaw, flat, flange):
    """A bent plate: a flat part plus an upright flange along one long edge (sheet metal, not a slab)."""
    cx, cy, cz = center
    flat_bm = _box((length, width, thick), (cx, cy, cz + thick / 2), flat, rot=(0.0, 0.0, yaw))
    # The flange sits on the flat part's +y edge, turned with the sheet
    off_x = -math.sin(yaw) * width / 2
    off_y = math.cos(yaw) * width / 2
    flange_bm = _box((length, thick, height), (cx + off_x, cy + off_y, cz + height / 2), flange,
                     rot=(0.0, 0.0, yaw))
    return [flat_bm, flange_bm]


def _pipe(radius, length, loc, color, yaw=0.0):
    """A short pipe lying along x, turned by yaw around z."""
    bm = common.make_frustum(radius, radius, -length / 2, length / 2, segments=8, color=color)
    common.transform(bm, rot=(0.0, math.pi / 2, yaw), loc=loc)
    return bm


def _hex(radius, z0, z1, xy, color):
    """Hex nut or bolt head (6 sides reads as a nut at icon size)."""
    return common.make_frustum(radius, radius, z0, z1, segments=6, center=xy, color=color)


def scrap_metal():
    # Rust (orange-brown and red) on the sheets, grey steel on the flanges, pipe, nut and bolt
    return [*_bent_sheet(0.24, 0.14, 0.024, 0.08, (-0.02, -0.02, 0.0), 0.35, "wood_crate", "rock_mid"),
            *_bent_sheet(0.18, 0.13, 0.024, 0.07, (0.07, 0.04, 0.06), -0.7, "red_paint", "metal"),
            _pipe(0.03, 0.22, (-0.04, -0.12, 0.03), "metal", yaw=0.15),
            _hex(0.035, 0.0, 0.03, (0.12, -0.08), "metal"),
            _hex(0.012, 0.0, 0.07, (0.11, 0.10), "rock_light"),
            _hex(0.025, 0.07, 0.09, (0.11, 0.10), "rock_light")]


def cloth():
    return [_box((0.22, 0.16, 0.03), (0.0, 0.0, 0.015), "white"),
            _box((0.16, 0.12, 0.03), (0.02, 0.0, 0.045), "white", rot=(0.0, 0.0, -0.15)),
            _box((0.16, 0.01, 0.005), (0.02, 0.0, 0.062), "rock_light", rot=(0.0, 0.0, -0.15))]


def seeds_wheat():
    return [_blob(2, 0.12, 9, (0.09, 0.07, 0.09), (0.0, 0.0, 0.09), "wood_crate"),
            _cyl(0.025, 0.025, 0.13, 0.19, "bark_light"),
            _box((0.07, 0.01, 0.05), (0.0, -0.07, 0.09), "brass")]


def flashlight():
    return [_cyl(0.025, 0.025, 0.0, 0.16, "gun_black"),
            _cyl(0.026, 0.026, 0.08, 0.10, "brass"),
            _cyl(0.032, 0.032, 0.16, 0.21, "metal"),
            _cyl(0.028, 0.028, 0.21, 0.215, "white")]


def shells_12g():
    # A red shell box with a brass band at the top, like a box of 12 gauge shells
    return [_box((0.12, 0.08, 0.06), (0.0, 0.0, 0.03), "red_paint"),
            _box((0.125, 0.082, 0.015), (0.0, 0.0, 0.065), "brass")]


def rifle_rounds():
    # A dark rifle ammo box with a brass strip, so it reads apart from the shells
    return [_box((0.16, 0.10, 0.07), (0.0, 0.0, 0.035), "gun_black"),
            _box((0.165, 0.102, 0.015), (0.0, 0.0, 0.075), "brass")]


def ammo_762():
    # An olive 7.62 mm ammo box with a brass strip, so it reads apart from the rifle rounds
    return [_box((0.16, 0.10, 0.07), (0.0, 0.0, 0.035), "cloth_olive"),
            _box((0.165, 0.102, 0.015), (0.0, 0.0, 0.075), "brass")]


def fuel_can():
    return [_box((0.22, 0.14, 0.30), (0.0, 0.0, 0.15), "red_paint"),
            _box((0.10, 0.03, 0.04), (0.0, 0.0, 0.33), "metal"),
            _cyl(0.012, 0.012, 0.30, 0.36, "metal")]


def campfire_kit():
    # A ring of grey stones with two crossed logs in the middle, the kit that makes a campfire
    stones = [_blob(1, 0.2, 20 + i, (0.04, 0.035, 0.03),
                    (0.09 * math.cos(i * math.pi / 4), 0.09 * math.sin(i * math.pi / 4), 0.025), "rock_mid")
              for i in range(8)]
    return [*stones,
            _box((0.22, 0.045, 0.045), (0.0, 0.0, 0.05), "bark_light", rot=(0.0, 0.0, 0.35)),
            _box((0.045, 0.22, 0.045), (0.0, 0.0, 0.085), "wood_crate", rot=(0.0, 0.0, -0.35))]


def hide():
    # A folded deer hide: a flat pelt in two layers (leather under, fur on top), so it reads as skin
    return [_box((0.24, 0.18, 0.025), (0.0, 0.0, 0.0125), "leather"),
            _box((0.22, 0.16, 0.012), (0.0, 0.0, 0.03), "deer_fur"),
            _box((0.06, 0.05, 0.014), (0.08, 0.05, 0.04), "deer_pale")]


def antler():
    # One antler: a main beam with two tines, the thing a buck sheds
    return [_box((0.03, 0.03, 0.22), (0.0, 0.0, 0.11), "antler", rot=(0.0, 0.0, 0.12)),
            _box((0.025, 0.025, 0.10), (0.04, 0.0, 0.15), "antler", rot=(0.0, 0.6, 0.0)),
            _box((0.025, 0.025, 0.08), (-0.03, 0.0, 0.19), "antler", rot=(0.0, -0.5, 0.0))]


# Farming (V12-03): the tools reuse their farming model parts, the crops and food are simple shapes.
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "farming"))
import farming_parts  # noqa: E402
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "weapons"))
import arrow as arrow_weapon  # noqa: E402


class _Bag:
    """Stands in for common.Model in the farming scripts: keeps the bmeshes they add."""

    def __init__(self):
        self.bms = []

    def add(self, bm):
        self.bms.append(bm)

    def add_collider(self, label, bm):
        pass

    def add_empty(self, label, loc):
        pass


def _farm_part(build):
    bag = _Bag()
    build(bag)
    return bag.bms


def hoe():
    return _farm_part(farming_parts.hoe)


def watering_can():
    return _farm_part(farming_parts.watering_can)


def seeds_potato():
    return [_blob(2, 0.12, 22, (0.07, 0.06, 0.06), (0.0, 0.0, 0.06), "potato_brown"),
            _box((0.02, 0.02, 0.07), (0.0, 0.0, 0.14), "sprout_green")]


def seeds_carrot():
    return [_box((0.12, 0.015, 0.16), (0.0, 0.0, 0.08), "carrot_orange"),
            _box((0.05, 0.015, 0.05), (0.0, 0.0, 0.17), "crop_green")]


def wheat():
    stalks = []
    for x in (-0.05, 0.0, 0.05):
        stalks.append(_box((0.02, 0.02, 0.28), (x, 0.0, 0.14), "crop_green"))
        stalks.append(_box((0.05, 0.03, 0.10), (x, 0.0, 0.31), "wheat_gold"))
    return stalks


def potato():
    return [_blob(2, 0.15, 21, (0.10, 0.08, 0.06), (0.0, 0.0, 0.06), "potato_brown"),
            _box((0.02, 0.02, 0.01), (0.04, 0.03, 0.11), "bark_dark")]


def carrot():
    return [common.make_frustum(0.0, 0.05, 0.0, 0.30, segments=6, color="carrot_orange"),
            _box((0.14, 0.02, 0.12), (0.0, 0.0, 0.36), "crop_green"),
            _box((0.02, 0.14, 0.10), (0.0, 0.0, 0.34), "crop_green")]


def bread():
    return [_blob(2, 0.08, 23, (0.12, 0.08, 0.06), (0.0, 0.0, 0.06), "wheat_gold"),
            _box((0.06, 0.03, 0.01), (0.0, 0.0, 0.11), "bark_light")]


def baked_potato():
    return [_blob(2, 0.12, 24, (0.09, 0.08, 0.07), (0.0, 0.0, 0.07), "potato_brown"),
            _box((0.07, 0.012, 0.01), (0.0, 0.0, 0.13), "rock_light")]


def vegetable_stew():
    # A metal bowl with stew, orange carrot pieces and green potato bits
    return [common.make_frustum(0.10, 0.07, 0.0, 0.08, segments=8, color="metal"),
            common.make_frustum(0.085, 0.085, 0.07, 0.085, segments=8, color="carrot_orange"),
            _box((0.03, 0.03, 0.03), (0.02, 0.02, 0.09), "crop_green"),
            _box((0.03, 0.03, 0.03), (-0.03, -0.02, 0.09), "crop_green")]


def arrow():
    bag = _Bag()
    arrow_weapon.build(bag)
    return bag.bms


def arrow_broken():
    # An arrow snapped in two: the two halves meet at the jagged fracture vertex in an angled V
    bag = _Bag()
    arrow_weapon.build(bag)
    # Front half: ferrule, blade, edge, sinew wrap (indices 0..3)
    front_bms = bag.bms[:4]
    # Broken front shaft from y = -0.260 to -0.075 with jagged splinters at the break
    front_bms.append(arrow_weapon.cyl_y(0.0042, 0.0042, -0.260, -0.075, "wood_crate", segments=8))
    front_bms.append(_box((0.003, 0.024, 0.002), (0.001, -0.268, 0.001), "wood_crate"))
    front_bms.append(_box((0.002, 0.018, 0.002), (-0.002, -0.265, -0.001), "bark_light"))

    # Rear half: feathers, bindings, nock, slit (indices 5..11; omit full shaft at index 4)
    rear_bms = bag.bms[5:]
    # Broken rear shaft from y = -0.710 to -0.440 with jagged splinters at the break
    rear_bms.append(arrow_weapon.cyl_y(0.0042, 0.0042, -0.710, -0.440, "wood_crate", segments=8))
    rear_bms.append(_box((0.003, 0.024, 0.002), (0.001, -0.432, 0.001), "wood_crate"))
    rear_bms.append(_box((0.002, 0.018, 0.002), (-0.002, -0.435, -0.001), "bark_light"))

    # Front piece: place break at origin, rotate arm up-right
    for bm in front_bms:
        common.transform(bm, loc=(0.0, 0.260, 0.0))
        common.transform(bm, rot=(0.0, 0.0, -0.40))

    # Rear piece: place break adjacent to origin, angle arm down-left
    for bm in rear_bms:
        common.transform(bm, loc=(0.018, 0.435, 0.005))
        common.transform(bm, rot=(0.0, 0.0, 0.55))

    return front_bms + rear_bms


BUILDERS = {
    "arrow": arrow,
    "arrow_broken": arrow_broken,
    "hide": hide,
    "antler": antler,
    "campfire_kit": campfire_kit,
    "fuel_can": fuel_can,
    "canned_beans": canned_beans,
    "shells_12g": shells_12g,
    "rifle_rounds": rifle_rounds,
    "ammo_762": ammo_762,
    "apple": apple,
    "raw_meat": raw_meat,
    "cooked_meat": cooked_meat,
    "water_bottle": water_bottle,
    "dirty_water": dirty_water,
    "soda": soda,
    "bandage": bandage,
    "medkit": medkit,
    "pistol_ammo": pistol_ammo,
    "wood": wood,
    "scrap_metal": scrap_metal,
    "cloth": cloth,
    "seeds_wheat": seeds_wheat,
    "flashlight": flashlight,
    "seeds_potato": seeds_potato,
    "seeds_carrot": seeds_carrot,
    "wheat": wheat,
    "potato": potato,
    "carrot": carrot,
    "hoe": hoe,
    "watering_can": watering_can,
    "bread": bread,
    "baked_potato": baked_potato,
    "vegetable_stew": vegetable_stew,
    "warm_jacket": warm_clothes_meshes.warm_jacket,
    "wool_hat": warm_clothes_meshes.wool_hat,
    "desert_scarf": warm_clothes_meshes.desert_scarf,
    "hand_warmer": warm_clothes_meshes.hand_warmer,
}
