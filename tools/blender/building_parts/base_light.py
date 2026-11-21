"""base_light: hanging hurricane lantern on a wooden post with light_1 marker (<= 300 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "base_light", "building_parts", 300

# Post position and arm reach
POST_X = -0.12
LANTERN_X = 0.20
POST_H = 1.75
FLAME_Z = 1.40


def build(model):
    parts = []

    # 1. Floor cross base (two crossing wooden timber feet on ground)
    parts.append(common.make_box((0.48, 0.10, 0.04),
                                 loc=(POST_X, 0.0, 0.02),
                                 color="bark_dark"))
    parts.append(common.make_box((0.10, 0.48, 0.04),
                                 loc=(POST_X, 0.0, 0.02),
                                 color="bark_dark"))

    # 2. Upright wooden post (from z=0 to z=POST_H)
    parts.append(common.make_box((0.08, 0.08, POST_H),
                                 loc=(POST_X, 0.0, POST_H / 2),
                                 color="bark_dark"))

    # 3. Horizontal wooden arm extending out to the right (+X)
    arm_len = LANTERN_X - POST_X + 0.08  # ~0.40m
    arm_center_x = POST_X + arm_len / 2 - 0.04
    arm_z = POST_H - 0.05  # 1.70m
    parts.append(common.make_box((arm_len, 0.06, 0.06),
                                 loc=(arm_center_x, 0.0, arm_z),
                                 color="bark_dark"))

    # 4. Diagonal 45-degree wooden support brace
    brace_len = 0.32
    brace = common.make_box((0.05, 0.05, brace_len), color="bark_dark")
    common.transform(brace, rot=(0.0, math.radians(45), 0.0),
                     loc=(POST_X + 0.11, 0.0, arm_z - 0.12))
    parts.append(brace)

    # 5. Metal hanging hook at end of arm
    parts.append(common.make_box((0.02, 0.02, 0.06),
                                 loc=(LANTERN_X, 0.0, arm_z - 0.04),
                                 color="metal"))

    # 6. Hanging hurricane lantern (centered at X=LANTERN_X, Y=0.0)
    # A. Wire bail / handle arching up to the hook
    bail_h = 0.10
    bail_top_z = arm_z - 0.06  # 1.64m
    # Top horizontal wire of bail
    parts.append(common.make_box((0.08, 0.015, 0.015),
                                 loc=(LANTERN_X, 0.0, bail_top_z),
                                 color="metal"))
    # Left and right wire sides of bail
    for sx in (-1, 1):
        parts.append(common.make_box((0.015, 0.015, bail_h),
                                     loc=(LANTERN_X + sx * 0.04, 0.0, bail_top_z - bail_h / 2),
                                     color="metal"))

    # B. Metal top cap / ventilator dome (6 segments)
    cap_z0 = 1.49
    cap_z1 = 1.54
    parts.append(common.make_frustum(0.06, 0.035, cap_z0, cap_z1, segments=6,
                                     center=(LANTERN_X, 0.0), color="red_paint"))
    parts.append(common.make_frustum(0.025, 0.025, cap_z1, cap_z1 + 0.025, segments=6,
                                     center=(LANTERN_X, 0.0), color="metal"))

    # C. Glass globe (6 segments)
    globe_z0 = 1.34
    parts.append(common.make_frustum(0.052, 0.062, globe_z0, cap_z0, segments=6,
                                     center=(LANTERN_X, 0.0), color="glass_tint"))

    # D. Inside flame
    parts.append(common.make_frustum(0.015, 0.0, FLAME_Z - 0.015, FLAME_Z + 0.025, segments=4,
                                     center=(LANTERN_X, 0.0), color="wheat_gold"))

    # E. Bottom fuel fount / tank base (6 segments)
    tank_z0 = 1.25
    tank_z1 = globe_z0
    parts.append(common.make_frustum(0.075, 0.055, tank_z0, tank_z1, segments=6,
                                     center=(LANTERN_X, 0.0), color="red_paint"))
    # Base rim flange
    parts.append(common.make_frustum(0.08, 0.08, tank_z0 - 0.015, tank_z0, segments=6,
                                     center=(LANTERN_X, 0.0), color="metal"))

    # F. 2 side wire guard rods
    for sx in (-1, 1):
        parts.append(common.make_box((0.012, 0.012, cap_z0 - globe_z0),
                                     loc=(LANTERN_X + sx * 0.065, 0.0, (globe_z0 + cap_z0) / 2),
                                     color="metal"))

    model.add(common.combine(parts))

    # Empty marker at the flame position for lighting in engine/Godot
    model.add_empty("light_1", (LANTERN_X, 0.0, FLAME_Z))

    # Convex collision box for the lamp post and base
    model.add_collider("post", common.make_box((0.50, 0.50, POST_H),
                                               loc=(POST_X + 0.05, 0.0, POST_H / 2),
                                               color="bark_dark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
