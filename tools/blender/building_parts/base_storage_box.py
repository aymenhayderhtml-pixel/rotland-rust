"""base_storage_box: wooden storage chest with hinged lid child mesh (<= 300 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "base_storage_box", "building_parts", 300

# Dimensions fit 2m cell: chest is 0.90m wide, 0.60m deep, 0.50m tall
CHEST_W = 0.90
CHEST_D = 0.60
CHEST_H = 0.42
LID_W = 0.94
LID_D = 0.64
LID_H = 0.05


def build(model):
    body_parts = []

    # 1. Main wooden chest body
    body_parts.append(common.make_box((CHEST_W, CHEST_D, CHEST_H),
                                      loc=(0.0, 0.0, CHEST_H / 2),
                                      color="wood_crate"))

    # 2. Bottom wooden runners / skids (underneath chest)
    skid_h = 0.035
    for sx in (-1, 1):
        x = sx * 0.36
        body_parts.append(common.make_box((0.10, CHEST_D, skid_h),
                                          loc=(x, 0.0, skid_h / 2),
                                          color="bark_dark"))

    # 3. Four vertical iron corner angle brackets
    bracket_w = 0.04
    for sx in (-1, 1):
        for sy in (-1, 1):
            x = sx * (CHEST_W / 2 - bracket_w / 2 + 0.005)
            y = sy * (CHEST_D / 2 - bracket_w / 2 + 0.005)
            body_parts.append(common.make_box((bracket_w, bracket_w, CHEST_H),
                                              loc=(x, y, CHEST_H / 2),
                                              color="metal"))

    # 4. Horizontal metal reinforcing band around the chest perimeter
    band_z = 0.20
    body_parts.append(common.make_box((CHEST_W + 0.015, CHEST_D + 0.015, 0.035),
                                      loc=(0.0, 0.0, band_z),
                                      color="metal"))

    # 5. Front lock hasp base at front (-Y)
    hasp_y = -(CHEST_D / 2 + 0.01)
    body_parts.append(common.make_box((0.08, 0.02, 0.08),
                                      loc=(0.0, hasp_y, 0.33),
                                      color="brass"))

    # 6. Iron drop handles on left (-X) and right (+X)
    for sx in (-1, 1):
        x = sx * (CHEST_W / 2 + 0.012)
        body_parts.append(common.make_box((0.02, 0.14, 0.04),
                                          loc=(x, 0.0, 0.24),
                                          color="metal"))

    model.add(common.combine(body_parts))

    # 7. Lid: separate child mesh "lid" hinged at the back (+Y, Z = CHEST_H)
    lid_parts = []
    lid_base_z = CHEST_H + LID_H / 2
    # Main flat wooden lid board
    lid_parts.append(common.make_box((LID_W, LID_D, LID_H),
                                     loc=(0.0, 0.0, lid_base_z),
                                     color="wood_crate"))

    # Raised middle slat along the top of the lid
    lid_parts.append(common.make_box((LID_W, 0.32, 0.025),
                                     loc=(0.0, 0.0, CHEST_H + LID_H + 0.025 / 2),
                                     color="bark_light"))

    # Two metal reinforcing straps across the lid (along Y)
    for sx in (-1, 1):
        x = sx * 0.28
        lid_parts.append(common.make_box((0.045, LID_D + 0.015, 0.02),
                                         loc=(x, 0.0, CHEST_H + LID_H + 0.01),
                                         color="metal"))

    # Front clasp / latch hanging down from the lid over the hasp (-Y)
    lid_parts.append(common.make_box((0.06, 0.02, 0.08),
                                     loc=(0.0, -(LID_D / 2 + 0.015), CHEST_H + 0.02),
                                     color="metal"))

    # Two rear hinges at the back (+Y) top edge of the chest
    for sx in (-1, 1):
        x = sx * 0.28
        # Back hinge barrel connecting chest and lid
        lid_parts.append(common.make_box((0.07, 0.035, 0.04),
                                         loc=(x, CHEST_D / 2 + 0.01, CHEST_H),
                                         color="metal"))

    # Add lid as a separate child mesh node "lid"
    model.add_part("lid", common.combine(lid_parts))

    # Convex collision box for the storage chest
    model.add_collider("box", common.make_box((LID_W, LID_D, CHEST_H + LID_H + 0.03),
                                              loc=(0.0, 0.0, (CHEST_H + LID_H + 0.03) / 2),
                                              color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
