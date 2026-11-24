"""Improved drinking clip overriding the older drink pose in poses_use.py.

Sign rules:
- spine/chest/head X negative = bend forward, positive = lean back
- upper_arm X positive = arm forward/up (0 = hanging)
- forearm X positive = elbow bends
Upper body bones only.
"""

CLIPS = {
    "drink": {
        "duration": 1.5,
        "loop": False,
        "bones": {
            "spine": [
                (0.0, (0.0, 0.0, 0.0)),
                (0.5, (2.0, 0.0, 0.0)),
                (1.0, (2.0, 0.0, 0.0)),
                (1.5, (0.0, 0.0, 0.0)),
            ],
            "chest": [
                (0.0, (0.0, 0.0, 0.0)),
                (0.35, (2.0, 0.0, 0.0)),
                (0.55, (5.0, 0.0, 0.0)),
                (1.0, (5.0, 0.0, 0.0)),
                (1.25, (2.0, 0.0, 0.0)),
                (1.5, (0.0, 0.0, 0.0)),
            ],
            "head": [
                (0.0, (0.0, 0.0, 0.0)),
                (0.35, (8.0, 0.0, 0.0)),
                (0.55, (22.0, 0.0, 0.0)),
                (0.8, (24.0, 0.0, 0.0)),
                (1.0, (21.0, 0.0, 0.0)),
                (1.25, (6.0, 0.0, 0.0)),
                (1.5, (0.0, 0.0, 0.0)),
            ],
            "upper_arm_R": [
                (0.0, (0.0, 0.0, 0.0)),
                (0.35, (85.0, 0.0, -10.0)),
                (0.55, (110.0, 0.0, -14.0)),
                (1.0, (112.0, 0.0, -14.0)),
                (1.25, (65.0, 0.0, -8.0)),
                (1.5, (0.0, 0.0, 0.0)),
            ],
            "forearm_R": [
                (0.0, (0.0, 0.0, 0.0)),
                (0.35, (95.0, 0.0, 12.0)),
                (0.55, (122.0, 0.0, 18.0)),
                (1.0, (120.0, 0.0, 18.0)),
                (1.25, (70.0, 0.0, 10.0)),
                (1.5, (0.0, 0.0, 0.0)),
            ],
            "upper_arm_L": [
                (0.0, (0.0, 0.0, 0.0)),
                (0.75, (4.0, 0.0, 2.0)),
                (1.5, (0.0, 0.0, 0.0)),
            ],
            "forearm_L": [
                (0.0, (0.0, 0.0, 0.0)),
                (0.75, (8.0, 0.0, 0.0)),
                (1.5, (0.0, 0.0, 0.0)),
            ],
        },
    },
}


def _to_frames(clip):
    """Claude fix: Neon wrote per-bone key lists; build_characters reads {"length", "loop", "frames": {t: {bone: rot}}}.
    Every key time gets every bone, linearly interpolated between that bone's own keys."""
    bones = clip["bones"]
    times = sorted({t for keys in bones.values() for t, _ in keys})

    def sample(keys, t):
        if t <= keys[0][0]:
            return keys[0][1]
        for (t0, a), (t1, b) in zip(keys, keys[1:]):
            if t0 <= t <= t1:
                f = (t - t0) / (t1 - t0) if t1 > t0 else 0.0
                return tuple(x + (y - x) * f for x, y in zip(a, b))
        return keys[-1][1]

    frames = {t: {bone: sample(keys, t) for bone, keys in bones.items()} for t in times}
    return {"length": clip["duration"], "loop": clip["loop"], "frames": frames}


CLIPS = {name: _to_frames(clip) for name, clip in CLIPS.items()}
