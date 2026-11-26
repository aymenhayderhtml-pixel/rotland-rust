"""Builds ONLY R37 models (gunsmith_bench, gun_shop, 9 weapon attachments) and the 9 attachment icons.

Run: blender -b --python tools/blender/build_r37_models.py
Does not modify palette sheets A/B/C or rebuild existing models/icons.
Exits with 1 if any model or icon fails or exceeds budget.
"""
import inspect
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "props"))
sys.path.insert(0, os.path.join(HERE, "buildings"))
sys.path.insert(0, os.path.join(HERE, "weapons"))
sys.path.insert(0, os.path.join(HERE, "icons"))

import build_all  # noqa: E402
import gun_shop  # noqa: E402
import gunsmith_bench  # noqa: E402
import item_meshes  # noqa: E402
import render_icons  # noqa: E402

ATTACHMENT_IDS = [
    "scope_2x",
    "scope_4x",
    "red_dot",
    "suppressor",
    "ext_mag",
    "drum_mag",
    "foregrip",
    "stock_pad",
    "bolt_quiver",
]


def _render_attachment_icons(attachments_mod, failures):
    os.makedirs(render_icons.ICON_DIR, exist_ok=True)
    items = {}
    if os.path.exists(render_icons.ITEMS_PATH):
        with open(render_icons.ITEMS_PATH, encoding="utf-8") as f:
            items = json.load(f)

    for att_id in ATTACHMENT_IDS:
        try:
            bms = None
            if att_id in item_meshes.BUILDERS:
                bms = item_meshes.BUILDERS[att_id]()
            elif att_id in items and "weaponId" in items[att_id]:
                bms = render_icons._meshes_for(att_id, items[att_id])
            elif attachments_mod is not None:
                builder = None
                if hasattr(attachments_mod, "BUILDERS") and att_id in attachments_mod.BUILDERS:
                    builder = attachments_mod.BUILDERS[att_id]
                elif hasattr(attachments_mod, f"build_{att_id}"):
                    builder = getattr(attachments_mod, f"build_{att_id}")
                if builder is not None:
                    if len(inspect.signature(builder).parameters) == 0:
                        bms = builder()
                    else:
                        collector = render_icons._Collector()
                        builder(collector)
                        bms = collector.bms
            if not bms:
                print(f"ICON FAILED: no builder for {att_id}")
                failures.append(f"icon:{att_id}")
                continue
            render_icons._render_icon(att_id, bms)
            print("ICON", att_id)
        except Exception as err:
            print(f"ICON FAILED ({att_id}): {err}")
            failures.append(f"icon:{att_id}")


def main():
    failures = []

    if gunsmith_bench.main() != 0:
        failures.append("gunsmith_bench")
    if gun_shop.main() != 0:
        failures.append("gun_shop")

    attachments_mod = None
    try:
        import attachments as attachments_mod  # noqa: E402
        if hasattr(attachments_mod, "main"):
            if attachments_mod.main() != 0:
                failures.append("attachments")
        else:
            print("BUILD FAILED: tools/blender/weapons/attachments.py has no main()")
            failures.append("attachments")
    except ImportError as err:
        print(f"BUILD FAILED: could not import weapons/attachments.py ({err})")
        failures.append("attachments")

    _render_attachment_icons(attachments_mod, failures)
    build_all.write_contact_sheet(["gunsmith_bench", "gun_shop"] + ATTACHMENT_IDS, "contact_sheet_r37.png")

    if failures:
        print("R37 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R37 BUILD OK: gunsmith_bench, gun_shop, 9 attachments, and 9 icons")
    return 0


if __name__ == "__main__":
    sys.exit(main())
