"""swamp_log_mossy: half-sunk fallen swamp log with moss patches and snapped stubs (<= 200 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "swamp_log_mossy", "props", 200
RADIUS_A = 0.26
RADIUS_B = 0.21
LENGTH = 2.80
YAW = 0.25
Z_CENTER = 0.05


def _along(t, lat=0.0, z=0.0):
    """World coordinate at distance `t` along the log axis and lateral offset `lat`."""
    ca, sa = math.cos(YAW), math.sin(YAW)
    return (t * ca - lat * sa, t * sa + lat * ca, z)


def _log(segments):
    """Log centered on Z from -LENGTH/2 to +LENGTH/2 with damp swamp_mud cut ends."""
    log = common.make_frustum(RADIUS_A, RADIUS_B, -LENGTH / 2.0, LENGTH / 2.0,
                              segments=segments, color="swamp_bark")
    log.normal_update()
    common.paint_faces(log, [f for f in log.faces if abs(f.normal.z) > 0.9], "swamp_mud")
    common.transform(log, rot=(0.0, math.pi / 2.0, YAW), loc=(0.0, 0.0, Z_CENTER))
    return log


def build(model):
    # Half-sunk main log body (28 tris)
    model.add(_log(8))

    # Damp mud/bark ridge collar around the thicker section (28 tris)
    collar = common.make_frustum(RADIUS_A + 0.015, RADIUS_A + 0.010, -0.78, -0.68,
                                 segments=8, color="swamp_mud")
    common.transform(collar, rot=(0.0, math.pi / 2.0, YAW), loc=(0.0, 0.0, Z_CENTER))
    model.add(collar)

    # 2 snapped branch stubs (2 * 16 = 32 tris)
    model.add(veg.limb(_along(-0.35, 0.08, 0.16), _along(-0.18, 0.36, 0.44),
                       0.075, 0.040, "swamp_bark", segs=5))
    model.add(veg.limb(_along(0.65, -0.06, 0.14), _along(0.82, -0.28, 0.34),
                       0.060, 0.030, "swamp_bark", segs=5))

    # Flat and lumpy swamp_moss patches draped along the top (24 + 40 = 64 tris)
    pad_a = common.make_box((0.95, 0.22, 0.045), color="swamp_moss")
    common.transform(pad_a, rot=(0.0, 0.0, YAW), loc=_along(-0.55, 0.0, 0.28))
    model.add(pad_a)

    pad_b = common.make_box((0.75, 0.18, 0.040), color="swamp_moss")
    common.transform(pad_b, rot=(0.08, 0.0, YAW), loc=_along(0.50, 0.04, 0.25))
    model.add(pad_b)

    model.add(common.make_icosphere(
        subdiv=1, jitter=0.18, seed=31, radius=(0.32, 0.20, 0.09),
        loc=_along(-0.15, -0.04, 0.28), rot=(0.0, 0.0, YAW), color="swamp_moss",
    ))
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.18, seed=32, radius=(0.28, 0.18, 0.08),
        loc=_along(0.88, 0.02, 0.24), rot=(0.0, 0.0, YAW), color="swamp_moss",
    ))

    # Low mud banks hugging the sunk flanks (2 * 12 = 24 tris)
    bank_l = common.make_box((2.10, 0.20, 0.12), color="swamp_mud")
    common.transform(bank_l, rot=(0.0, 0.0, YAW), loc=_along(-0.05, 0.24, -0.01))
    model.add(bank_l)

    bank_r = common.make_box((1.90, 0.20, 0.11), color="swamp_mud")
    common.transform(bank_r, rot=(0.0, 0.0, YAW), loc=_along(0.10, -0.23, -0.015))
    model.add(bank_r)

    model.add_collider("log", _log(6))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
