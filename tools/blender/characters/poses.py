"""Pose tables for humanoid character animation clips. This is the index: build_characters.py reads CLIPS and ARM_DOWN_OFFSET from here.

Clip groups, in export order (each group lives in its own module):
- poses_locomotion.py: idle (2.0 s, loop), walk (1.0 s, loop), run (0.7 s, loop)
- poses_actions.py: die (1.2 s, no loop), jump, crouch
- poses_combat.py: melee_1h, melee_2h, pistol_aim, pistol_fire, rifle_aim, rifle_fire, reload, hit (player)
- poses_zombie.py: zombie_shamble, zombie_attack, and the V11-06 Phase-3 zombie clips (zombie_stagger, zombie_lurch, zombie_crawl, zombie_bang, zombie_eat, zombie_scream, zombie_runner_sprint) (zombies only)
- poses_camp.py: camp life (sit, sit_ground, talk, listen, wave, carry, chop): player and NPCs only
- poses_air.py: jump arc and landings (air_takeoff, air_rise, air_fall, land_soft, land_hard, land_roll) (player only)
- poses_shared.py: ARM_DOWN_OFFSET and the aim poses shared by several clips

Upper-body clips leave the legs unkeyed (build_characters holds them at rest).
Keyframes are keyed by time in seconds.
Bone entries can be:
- (rx, ry, rz) in degrees
- {"rot": (rx, ry, rz), "loc": (dx, dy, dz)} where loc is in meters, on the bone's local axes
  (for the hips: local X = world X, local Y = world up, local Z = world -Y)
"""
from poses_actions import CLIPS as _ACTIONS
from poses_air import CLIPS as _AIR
from poses_car import CLIPS as _CAR
from poses_climb import CLIPS as _CLIMB
from poses_aim import CLIPS as _AIM
from poses_camp import CLIPS as _CAMP
from poses_combat import CLIPS as _COMBAT
from poses_combat import PLAYER_ONLY_CLIPS as _PLAYER_ONLY
from poses_death import CLIPS as _DEATH
from poses_hurt import CLIPS as _HURT
from poses_locomotion import CLIPS as _LOCOMOTION
from poses_move import CLIPS as _MOVE
from poses_posture import CLIPS as _POSTURE
from poses_shared import ARM_DOWN_OFFSET  # noqa: F401  (re-exported for build_characters)
from poses_stance import CLIPS as _STANCE
from poses_ladder import CLIPS as _LADDER
from poses_swim import CLIPS as _SWIM
from poses_use import CLIPS as _USE
from poses_use_v2 import CLIPS as _USE_V2  # A12: improved drink clip
from poses_wound import CLIPS as _WOUND
from poses_melee_v2 import CLIPS as _MELEE_V2  # A13a: combo swings
from poses_guard import CLIPS as _GUARD  # A13b: shove and block
from poses_zombie import CLIPS as _ZOMBIE
from poses_zombie_v2 import CLIPS as _ZOMBIE_V2  # A17: directional death clips
from poses_polish import PLAYER_ONLY_CLIPS as _POLISH  # R28-02: polished overrides
from poses_revive import CLIPS as _REVIVE  # R30-02: kneeling revive

CLIPS = {**_LOCOMOTION, **_ACTIONS, **_COMBAT, **_STANCE, **_AIM, **_MOVE, **_ZOMBIE, **_ZOMBIE_V2, **_CAMP, **_REVIVE}
# Clips only the player gets (V12-04 gun clips + A06 air/landing + A07 posture + F1-F5 clip library for A09-A16 + A13 melee + A16 deaths). Stances/aim/move are shared with NPCs (R27-03). Looked up through ALL_CLIPS, never CLIPS alone.
PLAYER_ONLY_CLIPS = {**_PLAYER_ONLY, **_AIR, **_POSTURE, **_SWIM, **_LADDER,
    **_CLIMB, **_CAR, **_USE, **_USE_V2, **_HURT, **_DEATH, **_WOUND, **_MELEE_V2, **_GUARD, **_POLISH}
ALL_CLIPS = {**CLIPS, **PLAYER_ONLY_CLIPS}
