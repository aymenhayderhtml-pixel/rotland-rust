# Rotland

**Version 1.0**

Rotland is a solo, offline, 3D open-world zombie survival game. You start in a walled camp. From there you go out to loot houses and crates, trade with the camp trader, and fight your way back before night falls. Battle zombies with guns, a crossbow and melee weapons. Hunt, fish, farm and cook to stay fed. Keep warm in the cold, drive a car, and bring along a companion dog. The game has its own 3D engine, written in Rust. It is built to run well on laptops with integrated graphics.

![Rotland forest](docs/screenshots/forest.jpg)

## Features

**World**
- A green home land around the camp, with four other regions beyond it: desert, snow forest, swamp and mountains.
- Four towns to visit: Dusthollow, Frostmere, Blackwater and Ridgeback Station. Each town has its own loot, zombies and gun shop.
- Roads, long rivers with bridges, and trailheads that lead into the wild.
- Weather that changes by region: rain, snow and sandstorms.
- A warmth meter. Cold slows you down and wears you out. Warm clothes, scarves and hand warmers help.

**Survival**
- Food, water and warmth. Keep all three up or your health drops.
- Hunting: track wild animals and butcher them for meat and hide.
- Fishing at lakes and rivers.
- Farming: plant seeds in farm beds, water them, and harvest the crops.
- Cooking on a campfire or stove.

**Combat**
- Guns: pistol, revolver, shotgun, rifle, hunting rifle, AK-47 and more.
- Melee weapons such as the bat, with combos, shoves and blocks.
- A crossbow with bolts you can pick up again.
- Grenades with a visible throw arc. Walls block the blast.
- Gun feel: kick, recoil, muzzle flash, and a scope in first person.
- Eight kinds of zombies, from fast runners and tough brutes to swamp bloaters that release toxic clouds, and frozen zombies in the snow.

**Weapon shop**
- A gunsmith in the camp, and a gun shop in each town with its own stock.
- You pay with scrap metal.
- Buy, sell, repair and upgrade guns.
- Guns wear down over time and can jam. Repair them to keep them working.
- Attachments for guns, such as scopes and suppressors.

**Camp**
- Upgrade the camp in tiers. Upgrades unlock the gunsmith tiers and more.
- A trader who buys and sells goods.
- Guards in watchtowers, and survivors you can rescue from houses and bring back to camp.

**Car and dog**
- Drive a car with throttle, steering, handbrake, horn and headlights. It needs fuel.
- A companion dog follows you, attacks zombies and helps you hunt. You can tell it to stay or follow, and feed it.

**Skills**
- Combat, hunting, cooking, agility and farming all grow the more you use them.

## Screenshots

### The world

| World map | Desert |
|---|---|
| ![World map](docs/screenshots/world-map.jpg) | ![Desert](docs/screenshots/desert.jpg) |
| The home land in the middle, with the other regions around it. | Desert, with cacti and red rock. |
| **Snow forest** | **Swamp** |
| ![Snow forest](docs/screenshots/snow-forest.jpg) | ![Swamp](docs/screenshots/swamp.jpg) |
| Snowy pines in the cold north. | Wet ground and tall trees in the swamp. |
| **Mountains** | **River and bridge** |
| ![Mountains](docs/screenshots/mountains.jpg) | ![River and bridge](docs/screenshots/river-bridge.jpg) |
| Rocky slopes with snow on top. | A frozen river with a bridge to cross. |

### The four towns

| Dusthollow (desert) | Frostmere (snow) |
|---|---|
| ![Dusthollow](docs/screenshots/town-dusthollow.jpg) | ![Frostmere](docs/screenshots/town-frostmere.jpg) |
| Sandy streets and small buildings. | Snow-covered houses and a snowy road. |
| **Blackwater (swamp)** | **Ridgeback Station (mountains)** |
| ![Blackwater](docs/screenshots/town-blackwater.jpg) | ![Ridgeback Station](docs/screenshots/town-ridgeback.jpg) |
| A town with boardwalks in the swamp. | Rock and ruins at the foot of the mountains. |

### Camp

| Camp in daylight | Camp at dusk |
|---|---|
| ![Camp in daylight](docs/screenshots/camp-day.jpg) | ![Camp at dusk](docs/screenshots/camp-dusk.jpg) |
| The safe camp behind its fence. | The camp as night comes in. |
| **Camp gunsmith** | |
| ![Camp gunsmith](docs/screenshots/gunsmith.jpg) | |
| The gunsmith shop: buy guns and parts with scrap metal. | |

### Weather

| Snowfall | Sandstorm |
|---|---|
| ![Snowfall](docs/screenshots/snowfall.jpg) | ![Sandstorm](docs/screenshots/sandstorm.jpg) |
| Snow falls over the snow forest. | Sand blows across the desert. |
| **Rain at camp** | **Snow at night** |
| ![Rain at camp](docs/screenshots/rain.jpg) | ![Snow at night](docs/screenshots/snow-night.jpg) |
| Rain over the camp. | A snowy forest at night. |

### Combat

| First person with a scope | Explosion |
|---|---|
| ![First person rifle](docs/screenshots/fp-rifle-scope.jpg) | ![Explosion](docs/screenshots/explosion.jpg) |
| Aiming a rifle with a scope. | A grenade blast leaves a scorched crater. |
| **Husk zombie (desert)** | **Bloater zombie (swamp)** |
| ![Husk zombie](docs/screenshots/zombie-husk.jpg) | ![Bloater zombie](docs/screenshots/zombie-bloater.jpg) |
| A husk stands in front of you on the sand. | A bloater, a slow swamp zombie, close by. |

## Play it

### Build and run

You need Rust (edition 2024) and the `mold` linker. On Ubuntu or Debian, install mold with:

```bash
sudo apt install mold
```

Then, in the folder you cloned the game into:

```bash
# Only once on a fresh copy: bake the 3D models into assets/models.pack
cargo run --release -p baker

# Build the game the way the owner plays it
cargo build --profile play

# Start the game
target/play/rotland
```

The file `assets/models.pack` is not stored in git. That is why the first step (`baker`) is needed on a fresh clone.

Start straight on the main menu with `target/play/rotland --menu`.

### Saves

Your saves are kept in `~/.local/share/rotland-rust/saves`. To copy a save from the older Godot version of the game into this one, start with `--import-godot=N`, where `N` is the slot number from 1 to 3. The Godot save is only read, never changed.

### No sound yet

Rotland has no sound in this version. This is on purpose. Sound will be added later.

## Controls

These are the default keys. Most of them can be changed in Settings > Controls.

### On foot

| Key / Mouse | Action |
|---|---|
| W, A, S, D | Move (camera-relative) |
| Mouse | Look around |
| Shift | Sprint (forward only, uses stamina) |
| Space | Jump (uses stamina) |
| C | Crouch |
| Left click | Attack: swing a melee weapon, or shoot (hold for automatic guns) |
| Right click (hold) | Aim (zoom in; uses the scope on a rifle) |
| E | Use or interact: loot crates, talk to people, butcher animals, tend farm beds, climb ladders, get in a car |
| R | Reload |
| 1 to 6 | Hotbar slots (equip a weapon or use an item) |
| V | Switch between first and third person camera |
| Mouse wheel | Zoom the camera (third person) |
| Esc | Pause menu, or release the mouse cursor (click the window to capture it again) |

### Screens

| Key | Screen |
|---|---|
| Tab | Bag and crafting |
| L | Skills |
| J | Bounties |
| P | Profile |
| M | World map (drag to pan, wheel to zoom, right click to place a waypoint) |

### Driving

| Key | Action |
|---|---|
| W / S | Throttle; brake and reverse |
| A / D | Steer |
| Space | Handbrake |
| H | Horn |
| L | Headlights on or off |
| E | Get out |

### Companion dog

| Key | Action |
|---|---|
| G | Stay or follow |
| E (on the dog) | Open the dog menu: pack, feed, stay or follow |

## Tech

Rotland is built as a Rust workspace of small crates:

- `engine`: the 3D renderer, window and menus, built on wgpu, winit and egui.
- `physics`: collisions and movement, built on rapier3d.
- `audio`: sound playback, built on kira (not used yet, see above).
- `game`: the game rules (zombies, camp, player, world logic), kept free of graphics so it can be tested.
- `world`: terrain, biomes, roads, rivers, towns and weather.
- `humanoid`: the character rig and its animation states.
- `rotland_ui`: the heads-up display, menus, camp and shop screens.
- `save`: save files and the import of Godot saves.
- `pack`: the binary format for models and other assets.
- `baker`: turns the `.glb` models into `assets/models.pack`.
- `paths`: finds the game's data and asset folders.
- `rotland`: the main program that wires everything together.

The 3D models are made with Blender Python scripts in `tools/blender/`. The game's numbers and lists (items, recipes, zombies, loot, towns and more) live in `data/*.json`, so they can be changed without touching the Rust code.
