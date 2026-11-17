# Rotland sound preview (v1, synthesized)

Every sound is made by `python3 tools/audio/synth.py`. Nothing is recorded or downloaded.
To hear them all in one go, run `tools/audio/play_all.sh` from the project folder.
All effects are normalized to about -3 dB peak. Volumes in the game are set in `data/sounds.json`.

## Guns
- **pistol_shot_1, pistol_shot_2**: a short, dry pistol crack with a little bass and a small room tail (0.45 s).
- **shotgun_shot**: a deeper, wider boom with a longer tail (0.7 s).
- **rifle_shot**: a sharp crack with a long tail and a faint metallic ring (1.1 s).
- **dry_fire**: a small empty click when the trigger is pulled on an empty gun (0.12 s).
- **reload**: three metal clicks, like a magazine going in and out (1.0 s).

## Melee and hits
- **swing_light**: a quick whoosh for the knife and machete (0.42 s).
- **swing_heavy**: a deeper, slower whoosh for the bat and axe (0.65 s).
- **hit_flesh_1, hit_flesh_2**: a soft, meaty thud when a swing lands on a zombie (0.3 s each).
- **hit_wood**: a dull knock, like an axe on wood (0.25 s). Used for axe swings that hit nothing for now.

## Footsteps and zombies
- **footstep_grass_1 to _4**: four soft steps on grass, with a small rustle (0.28 s each).
- **zombie_groan_1 to _3**: three low, falling growls, about 1.6 s each.
- **zombie_attack**: an angry, rasping snarl when a zombie starts to attack (0.9 s).

## Interface and player
- **pickup**: a soft two-note blip when an item is picked up (0.3 s).
- **ui_click**: a tiny click for buttons (0.07 s).
- **craft_done**: a bright, bell-like chime when crafting finishes (1.1 s).
- **level_up**: a short rising arpeggio when a skill levels up (1.2 s).
- **player_hurt**: a short grunt when the player takes damage (0.45 s).

## Ambience (loops, 20 s each, join seamlessly)
- **day_loop**: gentle wind with sparse, soft bird chirps.
- **night_loop**: darker, deeper wind with a few crickets.
