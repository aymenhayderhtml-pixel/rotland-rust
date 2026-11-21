"""base_spikes: 2 m wide row of sharpened wooden defensive stakes angled outward (<= 400 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "base_spikes", "building_parts", 400

# Dimensions fit 2m cell: 2.0m wide along X, stakes angle forward towards -Y
WIDTH = 2.00
NUM_STAKES = 6


def _make_pointed_stake(length, thickness, color_shaft="bark_dark", color_tip="wood_crate"):
    """Makes a stake pointing along +Z, with a sharpened 4-sided pyramid tip at the top."""
    shaft_len = length - 0.14
    shaft = common.make_box((thickness, thickness, shaft_len),
                            loc=(0.0, 0.0, shaft_len / 2),
                            color=color_shaft)

    # 4-sided pyramid tip at the top
    tip = common.make_frustum(thickness * math.sqrt(0.5), 0.0, shaft_len, length,
                              segments=4, rot_z=math.radians(45), color=color_tip)
    return common.combine([shaft, tip])


def build(model):
    parts = []

    # 1. Heavy horizontal timber base beam along X (-0.98 to +0.98, at Y = +0.18, Z = 0.06)
    beam_y = 0.18
    beam_z = 0.06
    parts.append(common.make_box((WIDTH - 0.04, 0.14, 0.12),
                                 loc=(0.0, beam_y, beam_z),
                                 color="bark_dark"))

    # 2. Four rear wooden anchor braces / kickstands angled backward (+Y) into ground
    brace_xs = (-0.75, -0.25, 0.25, 0.75)
    for bx in brace_xs:
        brace_len = 0.34
        brace = common.make_box((0.07, 0.07, brace_len), color="bark_dark")
        # Tilt backward (+Y) at ~45 degrees
        common.transform(brace, rot=(math.radians(45), 0.0, 0.0),
                         loc=(bx, beam_y + 0.14, beam_z + 0.01))
        parts.append(brace)

    # 3. Row of 6 sharpened stakes angled outward forward (-Y) and upward (+Z)
    # Jitter arrays for organic survivor defense appearance
    len_jitters = (0.0, 0.04, -0.03, 0.02, -0.02, 0.03)
    pitch_jitters = (0.0, -2.0, 3.0, -1.0, 2.0, -1.0)
    colors_shaft = ("bark_dark", "bark_light", "bark_dark", "bark_dark", "bark_light", "bark_dark")

    x_start = -0.80
    x_step = 1.60 / (NUM_STAKES - 1)  # 0.32m between stakes

    for i in range(NUM_STAKES):
        sx = x_start + i * x_step
        stake_len = 0.78 + len_jitters[i]
        stake_thick = 0.08
        pitch = math.radians(44.0 + pitch_jitters[i])

        stake = _make_pointed_stake(stake_len, stake_thick,
                                    color_shaft=colors_shaft[i],
                                    color_tip="wood_crate")

        # Rotate around X by -pitch so it leans forward towards -Y, then place on base beam
        common.transform(stake, rot=(-pitch, 0.0, 0.0),
                         loc=(sx, beam_y - 0.02, beam_z + 0.02))
        parts.append(stake)

    # 4. Horizontal binding rail / crossbar running across all stakes (at Y = -0.10, Z = 0.33)
    rail_y = -0.10
    rail_z = 0.33
    parts.append(common.make_box((WIDTH - 0.08, 0.06, 0.06),
                                 loc=(0.0, rail_y, rail_z),
                                 color="bark_light"))

    # 5. Lashed wire / strap bindings at each stake-to-rail intersection
    for i in range(NUM_STAKES):
        sx = x_start + i * x_step
        parts.append(common.make_box((0.09, 0.085, 0.07),
                                     loc=(sx, rail_y, rail_z),
                                     color="metal"))

    model.add(common.combine(parts))

    # Convex collision box for the spike barricade
    model.add_collider("spikes", common.make_box((WIDTH, 0.85, 0.68),
                                                 loc=(0.0, -0.05, 0.34),
                                                 color="bark_dark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
