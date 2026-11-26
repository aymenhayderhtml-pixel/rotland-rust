"""Lobby interior props: reception desk, elevators, mailboxes, bench, rubble, dead plant.

Run standalone: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/towers/lobby_props.py`
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, HERE)

import common  # noqa: E402
import jam_util  # noqa: E402
import lobby_props_b  # noqa: E402

BUDGET = 6000


def build_desk(model):
    """2.6 x 0.9 x 1.1 m reception counter with dead monitor, phone and papers."""
    # Main counter body, kickplate, side wings, and transaction top
    model.add(jam_util.pbox((-1.26, -0.35, 0.08), (1.26, 0.35, 0.98), "bark_light", bevel=0.015))
    model.add(jam_util.pbox((-1.24, -0.32, 0.0), (1.24, 0.32, 0.08), "soot", bevel=0.01))
    model.add(jam_util.pbox((-1.30, -0.45, 0.98), (1.30, 0.45, 1.10), "rock_light", bevel=0.018))
    # Front decorative pilasters and recessed front panel
    for x in (-1.22, -0.40, 0.40, 1.22):
        model.add(jam_util.pbox((x - 0.05, 0.34, 0.08), (x + 0.05, 0.40, 0.98), "bark_dark", bevel=0.01))
    model.add(jam_util.pbox((-1.15, 0.35, 0.14), (1.15, 0.38, 0.92), "brownstone", bevel=0.01))

    # Dead monitor on the counter (left-center)
    model.add(jam_util.pbox((-0.65, -0.10, 1.10), (-0.35, 0.12, 1.14), "gun_black", bevel=0.006))
    model.add(jam_util.pbox((-0.55, -0.04, 1.14), (-0.45, 0.04, 1.24), "gun_black", bevel=0.006))
    model.add(jam_util.pbox((-0.74, -0.12, 1.24), (-0.26, 0.06, 1.56), "gun_black", bevel=0.012))
    model.add(jam_util.pbox((-0.71, -0.13, 1.27), (-0.29, -0.11, 1.53), "soot", bevel=0.004))
    # Keyboard
    model.add(jam_util.pbox((-0.68, -0.34, 1.10), (-0.32, -0.16, 1.13), "gun_black", bevel=0.005))

    # Office phone (right-center)
    model.add(jam_util.pbox((0.35, -0.18, 1.10), (0.58, 0.04, 1.16), "white", bevel=0.008))
    model.add(jam_util.pbox((0.37, -0.16, 1.16), (0.44, 0.02, 1.21), "white", bevel=0.006))

    # Scattered papers
    for loc, rot, col in [
        ((0.02, -0.15, 1.105), (0.0, 0.0, 0.15), "white"),
        ((0.06, -0.10, 1.112), (0.0, 0.0, -0.28), "bone_white"),
        ((0.82, -0.05, 1.105), (0.0, 0.0, 0.42), "white"),
    ]:
        sheet = jam_util.pbox((-0.11, -0.14, -0.003), (0.11, 0.14, 0.003), col, bevel=0.001)
        common.transform(sheet, rot=rot, loc=loc)
        model.add(sheet)

    model.add_collider("desk", common.make_box((2.6, 0.9, 1.1), loc=(0.0, 0.0, 0.55), color="white"))


def build_elevators(model):
    """4.0 x 0.6 x 3.0 m two-car elevator bank; left door jammed 0.4 m open, dark shaft behind."""
    # Surround frame: left, center, right pilasters and top lintel
    model.add(jam_util.pbox((-2.0, -0.30, 0.0), (-1.65, 0.30, 3.0), "concrete_grey", bevel=0.015))
    model.add(jam_util.pbox((-0.35, -0.30, 0.0), (0.35, 0.30, 3.0), "concrete_grey", bevel=0.015))
    model.add(jam_util.pbox((1.65, -0.30, 0.0), (2.0, 0.30, 3.0), "concrete_grey", bevel=0.015))
    model.add(jam_util.pbox((-1.65, -0.30, 2.35), (1.65, 0.30, 3.0), "concrete_grey", bevel=0.015))

    # Dark shaft backing + thresholds (visual only, no shaft collider; split per bay to avoid pilaster overlap)
    model.add(jam_util.pbox((-1.65, -0.26, 0.0), (-0.35, -0.10, 2.35), "soot", bevel=0.005))
    model.add(jam_util.pbox((0.35, -0.26, 0.0), (1.65, -0.10, 2.35), "soot", bevel=0.005))
    model.add(jam_util.pbox((-1.65, -0.10, 0.0), (-0.35, 0.28, 0.06), "metal", bevel=0.008))
    model.add(jam_util.pbox((0.35, -0.10, 0.0), (1.65, 0.28, 0.06), "metal", bevel=0.008))

    # Left bay: left leaf jammed 0.4 m open (0.40 m gap to right leaf), right leaf closed
    model.add(jam_util.pbox((-1.64, 0.04, 0.06), (-1.38, 0.14, 2.34), "metal", bevel=0.01))
    model.add(jam_util.pbox((-0.98, 0.04, 0.06), (-0.36, 0.14, 2.34), "metal", bevel=0.01))

    # Right bay: two closed steel door leaves
    model.add(jam_util.pbox((0.36, 0.04, 0.06), (0.99, 0.14, 2.34), "metal", bevel=0.01))
    model.add(jam_util.pbox((1.01, 0.04, 0.06), (1.64, 0.14, 2.34), "metal", bevel=0.01))

    # Dead floor indicator panels above each bay
    for cx in (-1.0, 1.0):
        model.add(jam_util.pbox((cx - 0.22, 0.28, 2.52), (cx + 0.22, 0.33, 2.74), "gun_black", bevel=0.008))
        model.add(jam_util.pbox((cx - 0.16, 0.32, 2.58), (cx + 0.16, 0.34, 2.68), "soot", bevel=0.003))

    # Call button plate on center divider
    model.add(jam_util.pbox((-0.10, 0.29, 1.00), (0.10, 0.33, 1.32), "metal", bevel=0.005))
    model.add(jam_util.pbox((-0.03, 0.32, 1.18), (0.03, 0.35, 1.24), "red_paint", bevel=0.003))
    model.add(jam_util.pbox((-0.03, 0.32, 1.08), (0.03, 0.35, 1.14), "gun_black", bevel=0.003))

    # Diagonal "OUT OF ORDER" warning bar across the jammed left bay (no words)
    bar = jam_util.pbox((-0.68, -0.02, -0.045), (0.68, 0.02, 0.045), "red_paint", bevel=0.005)
    common.transform(bar, rot=(0.0, 0.28, 0.0), loc=(-1.0, 0.20, 1.35))
    model.add(bar)

    # Colliders for frame and door leaves only (shaft stays open)
    for label, lo, hi in [
        ("frame_l", (-2.0, -0.30, 0.0), (-1.65, 0.30, 3.0)),
        ("frame_c", (-0.35, -0.30, 0.0), (0.35, 0.30, 3.0)),
        ("frame_r", (1.65, -0.30, 0.0), (2.0, 0.30, 3.0)),
        ("frame_top", (-1.65, -0.30, 2.35), (1.65, 0.30, 3.0)),
        ("door_l1", (-1.64, 0.04, 0.06), (-1.38, 0.14, 2.34)),
        ("door_l2", (-0.98, 0.04, 0.06), (-0.36, 0.14, 2.34)),
        ("door_r", (0.36, 0.04, 0.06), (1.64, 0.14, 2.34)),
    ]:
        size = [hi[i] - lo[i] for i in range(3)]
        loc = [(lo[i] + hi[i]) / 2 for i in range(3)]
        model.add_collider(label, common.make_box(size, loc=loc, color="white"))


def build_mailboxes(model):
    """3.0 x 0.4 x 2.0 m wall unit with a 4 x 3 grid of metal boxes, 3 doors hanging open."""
    # Outer cabinet & base
    model.add(jam_util.pbox((-1.50, -0.20, 0.0), (1.50, 0.20, 0.40), "concrete_grey", bevel=0.015))
    model.add(jam_util.pbox((-1.50, -0.20, 0.40), (1.50, 0.10, 2.00), "concrete_grey", bevel=0.015))
    model.add(jam_util.pbox((-1.40, 0.08, 0.46), (1.40, 0.14, 1.88), "soot", bevel=0.006))

    # 4 x 3 grid of mailbox doors
    open_cells = {(0, 2), (2, 1), (3, 0)}
    col_w, row_h = 0.66, 0.44
    for col in range(4):
        for row in range(3):
            x0 = -1.32 + col * col_w + 0.02
            x1 = x0 + col_w - 0.04
            z0 = 0.50 + row * row_h + 0.02
            z1 = z0 + row_h - 0.04
            if (col, row) in open_cells:
                # Door swung open around its left edge
                w = x1 - x0
                door = jam_util.pbox((0.0, -0.015, z0), (w, 0.015, z1), "metal", bevel=0.005)
                common.transform(door, rot=(0.0, 0.0, -1.15), loc=(x0, 0.16, 0.0))
                model.add(door)
            else:
                model.add(jam_util.pbox((x0, 0.13, z0), (x1, 0.18, z1), "metal", bevel=0.006))
                # Key slot + number plate
                model.add(jam_util.pbox((x1 - 0.10, 0.18, (z0 + z1) / 2 - 0.03),
                                        (x1 - 0.05, 0.20, (z0 + z1) / 2 + 0.03), "brass", bevel=0.002))
                model.add(jam_util.pbox((x0 + 0.06, 0.18, z1 - 0.10),
                                        (x0 + 0.20, 0.195, z1 - 0.04), "white", bevel=0.002))

    model.add_collider("mailboxes", common.make_box((3.0, 0.4, 2.0), loc=(0.0, 0.0, 1.0), color="white"))


def build_all():
    front = (1.0, 1.0, 0.55)
    results = [
        ("lobby_desk", common.run_model("lobby_desk", "buildings", BUDGET, build_desk, view=front)),
        ("lobby_elevators", common.run_model("lobby_elevators", "buildings", BUDGET, build_elevators, view=front)),
        ("lobby_mailboxes", common.run_model("lobby_mailboxes", "buildings", BUDGET, build_mailboxes, view=front)),
    ]
    results.extend(lobby_props_b.build_all())
    return results


if __name__ == "__main__":
    failed = [name for name, ok in build_all() if not ok]
    if failed:
        print("LOBBY PROPS FAILED:", ", ".join(failed))
        sys.exit(1)
    print("LOBBY PROPS OK")
