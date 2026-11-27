"""Builds all R41 item models (12 items, 4 crates) and renders the 12 item icons.

Run standalone:
    flock /tmp/rotland-blender.lock blender -b --python tools/blender/gear/items_r41.py
Or called by tools/blender/build_r41_models.py via `items_r41.build_all()`.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS_DIR = os.path.abspath(os.path.join(HERE, ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, TOOLS_DIR)
sys.path.insert(0, os.path.join(TOOLS_DIR, "icons"))
sys.path.insert(0, os.path.join(TOOLS_DIR, "weapons"))

import army_gear  # noqa: E402
import common  # noqa: E402
import crates_r41  # noqa: E402
import meds_parts  # noqa: E402
import render_icons  # noqa: E402

ITEM_BUDGET = 1500
RIFLE_BUDGET = 3000  # <= data/budgets.json weapons cap (3000) and task cap (6000)
CRATE_BUDGET = 3000

# (name, category, budget, build_fn, grip, has_icon, view)
CRATE_VIEW = (1.0, 1.0, 0.7)  # crates have front +Y
MODELS = [
    ("army_rifle", "weapons", RIFLE_BUDGET, army_gear.build_army_rifle, army_gear.ARMY_RIFLE_GRIP, True, None),
    ("body_armor", "gear", ITEM_BUDGET, army_gear.build_body_armor, None, True, None),
    ("army_helmet", "gear", ITEM_BUDGET, army_gear.build_army_helmet, None, True, None),
    ("ammo_556", "gear", ITEM_BUDGET, army_gear.build_ammo_556, None, True, None),
    ("army_ration", "gear", ITEM_BUDGET, army_gear.build_army_ration, None, True, None),
    ("night_vision", "gear", ITEM_BUDGET, army_gear.build_night_vision, None, True, None),
    ("battery", "gear", ITEM_BUDGET, army_gear.build_battery, None, True, None),
    ("antibiotics", "gear", ITEM_BUDGET, meds_parts.build_antibiotics, None, True, None),
    ("surgery_kit", "gear", ITEM_BUDGET, meds_parts.build_surgery_kit, None, True, None),
    ("adrenaline_shot", "gear", ITEM_BUDGET, meds_parts.build_adrenaline_shot, None, True, None),
    ("gun_parts", "gear", ITEM_BUDGET, meds_parts.build_gun_parts, None, True, None),
    ("car_parts", "gear", ITEM_BUDGET, meds_parts.build_car_parts, None, True, None),
    ("army_crate", "props", CRATE_BUDGET, crates_r41.build_army_crate, None, False, CRATE_VIEW),
    ("metro_locker", "props", CRATE_BUDGET, crates_r41.build_metro_locker, None, False, CRATE_VIEW),
    ("med_cabinet", "props", CRATE_BUDGET, crates_r41.build_med_cabinet, None, False, CRATE_VIEW),
    ("camp_stash", "props", CRATE_BUDGET, crates_r41.build_camp_stash, None, False, CRATE_VIEW),
]


def _render_item_icon(item_id, build_fn):
    collector = render_icons._Collector()
    build_fn(collector)
    render_icons._render_icon(item_id, collector.bms)
    print(f"ICON {item_id} -> {os.path.join(render_icons.ICON_DIR, item_id + '.png')}")


def build_all():
    """Build all 16 R41 models and 12 icons. Returns list of (name, ok) tuples."""
    os.makedirs(render_icons.ICON_DIR, exist_ok=True)
    results = []
    for name, category, budget, build_fn, grip, has_icon, view in MODELS:
        ok = common.run_model(name, category, budget, build_fn, grip=grip, view=view)
        results.append((name, ok))
        if has_icon:
            try:
                _render_item_icon(name, build_fn)
            except Exception as err:
                print(f"ICON FAILED ({name}): {err}")
                results.append((f"icon:{name}", False))
    return results


def main():
    results = build_all()
    failed = [name for name, ok in results if not ok]
    if failed:
        print("R41 ITEMS FAILED:", ", ".join(failed))
        return 1
    print("R41 ITEMS OK: 16 models and 12 icons")
    return 0


if __name__ == "__main__":
    sys.exit(main())
