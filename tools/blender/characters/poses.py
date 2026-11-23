"""Pose tables for humanoid character animation clips. This is the index: build_characters.py reads CLIPS and ARM_DOWN_OFFSET from here.

Clip groups, in export order (each group lives in its own module):
- poses_locomotion.py: idle (2.0 s, loop), walk (1.0 s, loop), run (0.7 s, loop)
- poses_actions.py: die (1.2 s, no loop), jump, crouch
- poses_combat.py: melee_1h, melee_2h, pistol_aim, pistol_fire, rifle_aim, rifle_fire, reload, hit (player)
- poses_zombie.py: zombie_shamble, zombie_attack, and the V11-06 Phase-3 zombie clips (zombie_stagger, zombie_lurch, zombie_crawl, zombie_bang, zombie_eat, zombie_scream, zombie_runner_sprint) (zombies only)
- poses_camp.py: camp life (sit, sit_ground, talk, listen, wave, carry, chop): player and NPCs only
- poses_shared.py: ARM_DOWN_OFFSET and the aim poses shared by several clips

Upper-body clips leave the legs unkeyed (build_characters holds them at rest).
Keyframes are keyed by time in seconds.
Bone entries can be:
- (rx, ry, rz) in degrees
- {"rot": (rx, ry, rz), "loc": (dx, dy, dz)} where loc is in meters, on the bone's local axes
  (for the hips: local X = world X, local Y = world up, local Z = world -Y)
"""
from poses_actions import CLIPS as _ACTIONS
from poses_camp import CLIPS as _CAMP
from poses_combat import CLIPS as _COMBAT
from poses_combat import PLAYER_ONLY_CLIPS as _PLAYER_ONLY
from poses_locomotion import CLIPS as _LOCOMOTION
from poses_shared import ARM_DOWN_OFFSET  # noqa: F401  (re-exported for build_characters)
from poses_stance import CLIPS as _STANCE
from poses_zombie import CLIPS as _ZOMBIE

CLIPS = {**_LOCOMOTION, **_ACTIONS, **_COMBAT, **_ZOMBIE, **_CAMP}
# Clips only the player gets (V12-04 gun clips + A02 weapon stances). Looked up through ALL_CLIPS, never CLIPS alone.
PLAYER_ONLY_CLIPS = {**_PLAYER_ONLY, **_STANCE}
ALL_CLIPS = {**CLIPS, **PLAYER_ONLY_CLIPS}
