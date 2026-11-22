# Rotland sounds

The game has no sounds right now (the owner removed them on 2026-11-22 to add their own later).

To add a sound: put a `.wav` or `.ogg` file in `assets/audio/sfx/` (or `assets/audio/ambient/` for loops), then put its name in the
`"files"` list of that sound in `data/sounds.json`, for example
`"footstep_grass": { "files": ["res://assets/audio/sfx/my_step.wav"], "volumeDb": -20, ... }`.
`volumeDb` sets how loud it is (0 = full, -20 = quiet). No rebuild needed: just start the game again.
The old computer-made sounds can be made again with `python3 tools/audio/synth.py` (they are also in git history).
