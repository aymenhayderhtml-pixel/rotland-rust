# Rust Parity Audit (Godot vs Rust)

Audit updated for task **R10-11** comparing the original Godot C# implementation (`scripts/`) with the Rust port as of R9, 2026-11-18.

Godot paths (scripts/, scenes/, project.godot) refer to the frozen Godot repo (github.com/aymenhayderhtml-pixel/rotland, on the owner's machine at ../zombi servival).

Every feature is marked:
- **done**: logic exists and is actively reached from `rotland/src/main.rs`, `play.rs`, or `camp.rs`.
- **logic only**: logic exists in a library crate (`game`, `world`, `save`, etc.) but is not called or reached in the running application `rotland`.
- **missing**: no logic or implementation exists in the Rust workspace.
- **new (not in Godot)**: requested on GitHub by the owner, not present in the original Godot game.

---

## Parity Matrix

### Player

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Player | Kinematic movement (walk, sprint, crouch, jump, slope limit, step climbing) | `scripts/Player/PlayerController.cs`, `scripts/Player/StepClimber.cs` | **done** | `game/src/player.rs:Player::update`, called from `rotland/src/main.rs:830` | Full kinematic movement with Rapier3D `CharacterMover`, coyote time, jump buffering, and ceiling check when standing up. |
| Player | Stamina pool (sprint & jump stamina drain, delay, regen) | `scripts/Survival/Stamina.cs`, `scripts/Player/PlayerStaminaUse.cs` | **done** | `game/src/player.rs:Stamina::update`, `try_sprint`, `try_jump`, called from `rotland/src/main.rs:830,931` | Stamina costs (sprint 15/s, jump 8) and 0.8s recovery delay match Godot `survival.json`. |
| Player | Survival needs (hunger and thirst drain, starvation damage, well-fed regen) | `scripts/Survival/NeedStat.cs`, `scripts/Survival/WellFedRegen.cs`, `scripts/Survival/SurvivalConfig.cs` | **done** | `game/src/survival/need.rs:NeedStat::update`, `game/src/survival/well_fed.rs:WellFedRegen::update`, `rotland/src/play.rs:PlayState::needs_tick`, called from `rotland/src/main.rs:931` | Continuous drain based on sprint/idle; low needs slow stamina regen; starvation damage and well-fed healing active. |
| Player | Camera rig (1st/3rd person toggle with V, mouse look orbit, shoulder offset, recoil kick, zoom) | `scripts/Player/CameraRig.cs` | **done** | `game/src/camera_rig.rs:CameraRig::update`, called from `rotland/src/main.rs:768,867` | Orbit, spring-arm terrain collision pull-in, 3rd-person shoulder offset (+0.55m X, +0.2m Y), V key mode toggle. |
| Player | Keyboard look (arrow keys turning camera) | `scripts/Player/KeyboardLook.cs`, `scripts/Player/CameraRig.cs` | **missing** | None | Godot provides arrow-key camera panning; Rust input system (`game/src/input.rs`) only routes mouse delta. |
| Player | Hotbar (6 slots, 1-6 keys equip weapons, use items, toggle flashlight) | `scripts/Player/Hotbar.cs` | **done** | `game/src/items/hotbar.rs:Hotbar::press`, `rotland/src/play.rs:PlayState::hotbar_keys`, called from `rotland/src/main.rs:763` | Keys 1-6 equip/unequip weapons, consume food/drink/medical, or toggle flashlight. |
| Player | Interaction system (E key raycast, prompts, interactables) | `scripts/Player/PlayerInteractor.cs`, `scripts/Items/IInteractable.cs` | **done** | `rotland/src/crates.rs:interact`, `interact_prompt`, `rotland/src/play.rs:PlayState::interact_tick`, called from `rotland/src/main.rs:1078` | Forward raycast detects crates, items, farm beds, campfire, dog, vehicles, NPCs; triggers E interaction. |
| Player | Death & respawn (death delay, respawn at camp settlement, needs reset to 75) | `scripts/Player/PlayerDeathHandler.cs` | **done** | `rotland/src/main.rs:849-864`, `rotland/src/play.rs:PlayState::needs_respawn` | Upon 0 HP, player freezes in death; after delay, respawns at camp spawn marker, clears ragdoll pose, resets needs to 75. |
| Player | Death penalty (inventory drop & wallet loss scaled by difficulty) | `scripts/Player/DeathPenalty.cs`, `scripts/Difficulty/DeathRule.cs` | **done** | `game/src/difficulty/death_rule.rs:DeathRule::apply_to_inventory_with_seed`, `rotland/src/camp.rs:Camp::on_death`, called from `rotland/src/main.rs:955` | Wipes carried wallet dollars; drops all, half, or keeps inventory according to world `DeathRule`. |
| Player | Ladder climbing (E grab, W/S climb, Space/E let go, top step-off onto deck, weapons blocked) | `scripts/Player/LadderClimb.cs`, `scripts/World/LadderRail.cs` | **done** | `game/src/ladders/tick.rs:climb_tick`, called from `rotland/src/main.rs:823` | E attaches to ladder; W/S moves along rungs; Space lets go; steps off at top deck; blocks weapons while climbing. |
| Player | Player 3D visual & locomotion animation (rigged mesh, locomotion blend, crouch, swing actions) | `scripts/Characters/HumanoidVisual.cs`, `scripts/Characters/HumanoidAnimTree.cs`, `scripts/Player/PlayerAnimator.cs` | **done** | `humanoid/src/animator.rs:HumanoidAnimator::update`, `rotland/src/player_visual.rs:PlayerVisual::pose`, called from `rotland/src/main.rs:1496-1509` | Poses 22-joint skeleton (`human_player.glb`), blends idle/walk/run by horizontal speed, crouch pose, upper-body actions. |
| Player | Attached visual backpack mesh (`backpack_small.glb` / `backpack_large.glb` attached to chest bone) | `scripts/Player/PlayerBackpack.cs` | **missing** | None | Godot renders visual backpack model attached to upper back bone; Rust player model currently has no backpack attachment. |
| Player | Lifetime stats tracking (kills by type, headshots, deaths, distance traveled) | `scripts/Player/PlayerStats.cs` | **done** | `game/src/profile_stats/mod.rs:ProfileStats`, `rotland/src/play.rs:59`, `rotland/src/main.rs:1453-1461,2049-2053`, `rotland/src/session.rs:118-121` | ProfileStats tracks kills by type, headshots, deaths, distance traveled, and days survived; persisted in save `stats` system and rendered in Profile tab. |

---

### Combat

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Combat | Health & damage system (hit points, damage types, invulnerability frames, death signals) | `scripts/Combat/Health.cs`, `scripts/Combat/DamageInfo.cs` | **done** | `game/src/health.rs:Health::take_damage`, `rotland/src/combat.rs:CombatState::damage_player`, called from `rotland/src/main.rs:930` | Tracks HP for player and entities; applies damage types, invulnerability window, and death triggers. |
| Combat | Melee combat (swing cooldown, stamina cost, windup, arc hit detection, knockback, stagger) | `scripts/Combat/MeleeHandler.cs`, `scripts/Combat/WeaponDef.cs` | **done** | `game/src/melee.rs:MeleeSwing::update`, `rotland/src/combat.rs:CombatState::update`, called from `rotland/src/main.rs:835-845` | Left-click swings melee weapons (bat, knife, axe, katana, crowbar); checks front arc; damages and pushes zombies and wildlife. |
| Combat | Hitscan gun firing (spread hip/aim, shotgun pellets, magazine capacity, reloading, ammo consumption) | `scripts/Combat/GunHandler.cs` | **done** | `game/src/guns_fire.rs:fire_gun`, `rotland/src/shooting.rs:Shooting::update_targets`, `rotland/src/play.rs:PlayState::shoot_tick`, called from `rotland/src/main.rs:913-925` | Left-click fires equipped gun (pistol, shotgun, rifle, ak47, smg); R reloads from carried ammo reserve; raycast damage against targets. |
| Combat | Automatic fire (continuous firing while attack held for AK-47, SMG) | `scripts/Combat/WeaponHolder.cs` | **done** | `rotland/src/shooting.rs:Shooting::update_targets` (checks `gun.auto`), called from `rotland/src/main.rs:913` | Fully automatic weapon cycling while attack input is held. |
| Combat | Gun bash (swinging empty gun as melee weapon when out of magazine and reserve ammo) | `scripts/Combat/GunBash.cs` | **done** | `rotland/src/combat.rs:Combat::bash`, `rotland/src/main.rs:1201-1204` | Left-click attack when gun magazine and carried ammo are empty executes gun bash using `gun.bash_damage` (pistol 15, ak47 30), 10 stamina, and front melee arc hit detection. |
| Combat | Gun recoil climb & muzzle flash (camera kick climb, muzzle flash light, first-person gun kick) | `scripts/Combat/GunEffects.cs`, `scripts/Combat/GunHandler.cs` | **missing** | None | No muzzle flash sprite/light or recoil pitch climb implemented in renderer or camera. |
| Combat | Rifle scope FOV zoom (narrowed 25° FOV when aiming scoped rifle) | `scripts/Combat/WeaponScope.cs` | **done** | `rotland/src/shooting.rs:aim_fov`, called from `rotland/src/main.rs:1099` | Right-click aim with rifle dynamically narrows `camera.fov_y_deg` to 25.0°. |
| Combat | Scope overlay reticle (black ring vignette with sniper reticle on screen) | `scripts/UI/ScopeOverlay.cs` | **missing** | None | Godot renders fullscreen scope overlay texture; Rust only modifies camera FOV without 2D reticle overlay. |
| Combat | 3rd person weapon in hand (weapon model attached to right hand socket with grip offset) | `scripts/Combat/WeaponMounts.cs`, `scripts/Combat/WeaponVisuals.cs` | **done** | `humanoid/src/weapon.rs:weapon_world_matrix_named`, `rotland/src/player_visual.rs:PlayerVisual::pose`, called from `rotland/src/main.rs:1881-1882` | Instances weapon model (bat, pistol, rifle, etc.) attached to `hand_R` joint with correct grip orientation. |
| Combat | 1st person weapon view model (weapon model rendered in view under camera) | `scripts/Combat/WeaponMounts.cs:FirstPersonMount` | **done** | `rotland/src/fp_weapon.rs:fp_weapon_matrix_with_physics`, `rotland/src/main.rs:1884-1890` | In first-person mode, held weapon model is rendered in front of camera with sway, kick recoil offset, and physics raycast wall proximity pull-back. |
| Combat | AK-47 magazine swap submesh hide (hiding `mag` node during reload window) | `scripts/Combat/WeaponVisuals.cs:SetMagHidden`, `scripts/Combat/GunHandler.cs:MagHidden` | **logic only** | `game/src/weapons.rs:WeaponDef.mag_swap` | `mag_swap` timing intervals are parsed from `weapons.json`, but renderer cannot hide individual sub-mesh nodes. |
| Combat | Expanded weapon arsenal (crossbow, hunting rifle, revolver, grenades, etc.) (GitHub #5) | None | **missing** | `data/weapons.json` | **new (not in Godot)**. Owner requested expanded weapon arsenal on GitHub (#5); currently 9 weapons are implemented (5 melee, 5 firearms in `data/weapons.json`). |

---

### Zombies

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Zombies | Zombie types & definitions (walker, runner, bloater from `zombies.json`) | `scripts/Enemies/ZombieDef.cs`, `scripts/Enemies/ZombieRegistry.cs` | **done** | `game/src/zombie/defs.rs:ZombieDefs::from_repo`, `rotland/src/zombies.rs:Zombies::new` | Loads all zombie types, health, speeds, damage, and behaviour constants. |
| Zombies | State machine & core AI (Idle, Wander, Investigate, Chase, Attack, Dead) | `scripts/Enemies/ZombieAI.cs`, `scripts/Enemies/ZombieAttack.cs` | **done** | `game/src/zombie/brain.rs:ZombieBrain::update`, `rotland/src/zombies.rs:Zombies::update`, called from `rotland/src/main.rs:1497` | Wanders, detects visible player, switches to Chase, lunges into Attack within range, damages player. |
| Zombies | Flow field horde navigation (global grid guiding chasing horde around obstacles) | `scripts/Enemies/ZombieNavigation.cs`, `scripts/Enemies/ZombieSteering.cs` | **done** | `game/src/flowfield.rs:FlowField::rebuild`, `rotland/src/zombies.rs:Zombies::update` | High-performance 64x64 flow field rebuilt every 0.5s around player guides chasing horde smoothly around static obstacles. |
| Zombies | Dynamic zombie spawner (distance rings, player view cone, night runner mix, despawning) | `scripts/Enemies/ZombieSpawner.cs`, `scripts/Enemies/SpawnConfig.cs` | **done** | `game/src/spawner.rs:Spawner::update`, `rotland/src/play.rs:PlayState::spawn_tick`, called from `rotland/src/main.rs:1418` | Spawns zombies in front/around player; switches to night mix with runners; despawns distant zombies. |
| Zombies | 3D visual models & animations (mesh variants `zombie_01..03`, gaits, attack lunge, hit flinch, death) | `scripts/Enemies/ZombieVisual.cs` | **done** | `rotland/src/zombies.rs:Zombies::new`, `Zombies::pose`, called from `rotland/src/main.rs:1897-1904` | Instanced rendering for 3 models; varied walk gaits (shamble, lurch); synchronized attack, flinch, and death poses. |
| Zombies | Zombie scream behavior (sighting triggers scream, alerting nearby zombies via noise) | `scripts/Enemies/ZombieAI.Actions.cs:SpotPlayer` | **logic only** | `game/src/zombie/brain_extras.rs`, `game/src/zombie/state.rs:ZombieState::Scream` | Scream state logic exists in `game` crate, but `rotland/src/zombies.rs` does not handle scream outputs or emit noise events. |
| Zombies | Corpse eating behavior (zombie kneels and eats corpses or during idle) | `scripts/Enemies/ZombieAI.Actions.cs`, `scripts/Enemies/ZombieCorpses.cs` | **logic only** | `game/src/zombie/corpses.rs:ZombieCorpses`, `game/src/zombie/brain_extras.rs` | Corpse eating logic exists, but `rotland/src/zombies.rs:213` passes `corpses: &[]` every frame. |
| Zombies | Crawler zombies from leg hits (hits below 0.6m break legs, shrink capsule, force crawl pose) | `scripts/Enemies/ZombieWounds.cs`, `scripts/Enemies/ZombieCrawl.cs` | **logic only** | `game/src/zombie/brain_extras.rs` | Brain supports crawling state, but `rotland/src/zombies.rs:215` hardcodes `leg_hit: false`. |
| Zombies | Herd chase alert (chasing zombie alerts nearby idle/wander zombies within 8m) | `scripts/Enemies/ZombieHerd.cs` | **logic only** | `game/src/zombie/herd.rs:check_herd` | Herd alert logic exists, but `rotland/src/zombies.rs:212` passes `nearby_zombies: &[]` every frame. |
| Zombies | Night horde shared wander target (night group wandering together) | `scripts/Enemies/ZombieHorde.cs` | **logic only** | `game/src/zombie/herd.rs:ZombieHorde`, `game/src/zombie/brain_extras.rs:set_horde` | `ZombieHorde` struct exists in `game` crate, but is never assigned to zombie brains in `rotland`. |
| Zombies | Banging on hideout doors/walls (banging when player is on ladder or roof) | `scripts/Enemies/ZombieHideouts.cs` | **logic only** | `game/src/zombie/hideouts.rs`, `game/src/zombie/state.rs:ZombieState::Bang` | Hideout logic exists, but `rotland/src/zombies.rs:214` hardcodes `target_unreachable: false`. |
| Zombies | Safe zone protection (player in camp safe zone sealed off from zombie targeting) | `scripts/Enemies/ZombiePlayerLink.cs`, `scripts/Settlement/SettlementRegistry.cs:SealsOff` | **logic only** | `game/src/spawner.rs:in_no_spawn_zone` | `spawner` skips spawning inside camp safe zone, but `main.rs:1497` passes `self.player` to `zombies.update` even while player is inside camp. |
| Zombies | POI building zombies & awakeness (spawn markers waking zombies on approach, cleared status) | `scripts/World/PoiManager.cs` | **done** | `game/src/poi_zombies/manager.rs:PoiZombies`, called from `rotland/src/main.rs:1501-1545` | Zombies spawn at building `spawn_*` markers when player approaches within wake distance (35m), capped at 16, despawn when distant; clearing all zombies marks building cleared and saved in `worldstate.visited_pois`. |

---

### World

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| World | Terrain height & biome generation (FastNoiseLite 3D terrain, plains/forest biomes, camp flattening) | `scripts/World/WorldGenerator.cs` | **done** | `world/src/generator.rs:WorldGenerator::build_chunk`, called from `rotland/src/terrain.rs`, `rotland/src/main.rs:364` | Golden-dump verified exact terrain heights, biomes, and camp pad flattening matching Godot. |
| World | Ground painting (vertex color blending for grass, dry grass, leaf litter, dirt paths, camp dirt, rock slopes) | `scripts/World/GroundPaint.cs` | **done** | `world/src/ground_paint.rs:paint`, called from `rotland/src/terrain.rs` | Multi-noise vertex tinting matching Godot step 6 ground paint rules. |
| World | Chunk streaming & collision (streaming 64m chunks around player, heightfield colliders) | `scripts/World/ChunkManager.cs`, `scripts/World/Chunk.cs` | **done** | `rotland/src/streaming.rs:ChunkStreamer::update`, `physics/src/world.rs:PhysicsWorld::add_heightfield` | Streams chunks dynamically; registers and unregisters Rapier3D heightfield colliders. |
| World | Procedural prop scattering & LODs (trees, rocks, bushes, props by biome, LOD0..LOD2, instancing) | `scripts/World/ScatterPlacer.cs`, `scripts/World/ChunkProps.cs`, `scripts/World/ScatterConfig.cs` | **done** | `world/src/scatter.rs:scatter_chunk`, `rotland/src/props.rs`, `rotland/src/streaming.rs` | Deterministic seeded scattering per chunk, GPU instancing, distance LODs. |
| World | Day / night clock & lighting (24h clock, sun/moon arcs, sky/ambient/fog color transitions) | `scripts/World/TimeOfDay.cs`, `scripts/World/DayNightVisuals.cs`, `scripts/World/DayNightPalette.cs` | **done** | `world/src/time.rs:Clock::advance`, `world/src/daynight.rs`, `rotland/src/light.rs`, called from `rotland/src/main.rs:1367` | Running clock; sun/moon direction and color; sky/ambient/fog lighting shader updates. |
| World | Loot crate placement & rolling (ground crates, POI crates, seeded deterministic loot tables) | `scripts/World/LootPlacer.cs`, `scripts/World/LootRoller.cs`, `scripts/World/LootCrate.cs` | **done** | `world/src/loot.rs:chunk_crates`, `game/src/loot/roller.rs:LootRoller::roll`, `rotland/src/crates.rs:interact` | Crates spawn on terrain and POI markers, roll items from `loot_tables.json`, persist opened state. |
| World | POI buildings generation (cabin, house_small, gas_station, shed, barn, watchtower on step-2 grid) | `scripts/World/PoiPlacer.cs`, `scripts/World/PoiBuilder.cs`, `scripts/World/PoiConfig.cs` | **done** | `world/src/poi.rs:PoiPlacer::place`, `rotland/src/streaming.rs:ChunkStreamer` | Places buildings on deterministic step-2 grid, clears vegetation, instances building mesh and colliders. |
| World | Tree felling (chopping trees with axe 4 times, felling tree, dropping wood items) | `scripts/World/TreeFelling.cs`, `scripts/World/TreeStrike.cs` | **done** | `game/src/trees/felling.rs:TreeFelling`, `rotland/src/main.rs:1043-1075,1654-1655`, `rotland/src/session.rs:389,568` | Striking trees 4 times with an axe fells the tree with a tipping animation, yields 3-5 wood dropped items, displays a toast, and persists felled trees in `worldstate.fallen_trees`. |
| World | POI occlusion culling boxes (BoxOccluder3D for building walls and camp palisades) | `scripts/World/PoiOccluders.cs` | **missing** | None | Custom engine relies on frustum culling on worker threads; box occluders are not implemented. |
| World | Unloaded chunk item persistence (saving dropped items across chunk unloads and sessions) | `scripts/World/WorldState.cs`, `scripts/World/DroppedItem.cs` | **logic only** | `game/src/loot/world_items.rs:WorldItems`, `rotland/src/play.rs:37` | Dropped items persist in memory during active session via `WorldItems`, but are not serialized into save files. |

---

### Camp

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Camp | Camp generation & terrain flattening (deterministic location, palisade walls, gates, interior flattening) | `scripts/Settlement/SettlementPlacer.cs`, `scripts/Settlement/SettlementGround.cs`, `scripts/Settlement/SettlementSite.cs` | **done** | `world/src/settlement.rs:find_settlement`, called from `rotland/src/main.rs:364` | Generates walled settlement with gate; flattens ground pad; loads settlement mesh and colliders. |
| Camp | Camp NPCs at posts (trader, banker, breeder, guards, survivors with idle poses & phases) | `scripts/Settlement/SettlementNpc.cs`, `scripts/Settlement/SettlementBuilder.cs` | **done** | `rotland/src/camp_npcs.rs:CampNpcs::new`, `CampNpcs::pose`, called from `rotland/src/main.rs:1894` | Spawns NPCs at marker positions with role models, holding idle/aim poses with individual phase offsets. |
| Camp | Camp Life Director & routine residents (residents walking, sitting at fire, chopping, carrying) | `scripts/Settlement/CampLifeDirector.cs`, `scripts/Settlement/CampResident.cs`, `scripts/Settlement/CampPlanner.cs` | **done** | `game/src/camp/director.rs:CampLifeDirector::tick`, `rotland/src/camp_npcs.rs:CampNpcs::update`, called from `rotland/src/main.rs:1472` | 6 residents execute daily routines (work, sitting by fire, wandering, sleeping at night). |
| Camp | Campfire burning, relighting & cooking (placeable kit fires, 5h burn time, relighting, cooking station) | `scripts/World/Campfire.cs`, `scripts/World/CampfirePlacer.cs`, `scripts/World/CampfireVisuals.cs` | **done** | `game/src/campfire/manager.rs:CampfireManager`, `rotland/src/campfire_mesh.rs`, `rotland/src/camp.rs:125`, called from `rotland/src/main.rs:1462,1908-1929` | Permanent camp fire + placed kit fires; advances burn hours; relightable; provides campfire cooking station within 4m. |
| Camp | Guard shooting combat (guards on platforms spotting zombies within 40m and firing rifle for 60 dmg) | `scripts/Settlement/SettlementGuard.cs`, `scripts/Settlement/GuardRifle.cs` | **done** | `game/src/camp_guards/guards.rs:CampGuards`, `rotland/src/main.rs:1561-1590` | Guard NPCs on platforms scan for zombies within 40m, check line of sight with physics raycasts, turn to face target, fire rifles with gunshot audio, and damage zombies with unclaimed kills (`z.hit_unclaimed`). |
| Camp | Camp speech bubbles (billboard dialogue bubbles appearing over chatting residents within 12m) | `scripts/Settlement/CampChatBubble.cs`, `scripts/Settlement/CampChat.cs` | **done** | `rotland/src/bubbles.rs:draw_bubbles`, `rotland/src/main.rs:1957-1968` | Speech bubbles with tails and rounded frames are projected from 3D head positions of chatting camp residents within 12m and drawn via egui with fade-out alpha. |
| Camp | Settlement safe zone tracking (`SettlementWatcher.cs` tracking visits, respawn camp selection) | `scripts/Settlement/SettlementWatcher.cs` | **logic only** | `game/src/camp/watcher.rs:SettlementWatcher` | Struct exists in `game` crate, but `main.rs` never invokes it (respawn always uses camp index 0). |
| Camp | Guild / faction system (reputation, guild hall, ranks, guild contracts) (GitHub #6) | None | **missing** | None | **new (not in Godot)**. Owner requested guild/faction system on GitHub (#6); neither Godot nor Rust has guild mechanics. |

---

### Economy

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Economy | Carried wallet (starting $20, adding dollars, spending, wiped on death) | `scripts/Economy/Wallet.cs` | **done** | `game/src/economy/wallet.rs:Wallet`, `rotland/src/camp.rs:Camp.wallet`, called from `rotland/src/main.rs:954` | Tracks carried cash; integrated with shops, banking, bounties, loot crates, and death penalty. |
| Economy | Trader stock, buying & selling (buy with markup, sell bag items, unequip guns/magazines) | `scripts/Economy/TraderStock.cs`, `scripts/Economy/TradeRules.cs`, `scripts/UI/TradeScreen.cs` | **done** | `game/src/economy/trader.rs:TraderStock`, `game/src/economy/rules.rs:buy`, `sell`, `rotland/src/camp_ui/draw_shop.rs`, `rotland/src/camp.rs:Camp::apply` | E on trader opens shop screen; buy/sell transactions work; trader restocks on day rollover. |
| Economy | Bank vault (deposit/withdraw 40-slot inventory and dollars, survives death) | `scripts/Economy/Bank.cs`, `scripts/UI/BankScreen.cs` | **done** | `game/src/economy/bank.rs:Bank`, `rotland/src/camp_ui/draw_bank.rs`, `rotland/src/camp.rs:Camp::apply` | E on banker opens bank; transfers items and cash ($10, $100, all); bank contents survive death. |
| Economy | Bounty board (daily & weekly bounty generation, board interaction in camp) | `scripts/Bounties/BountyBoard.cs`, `scripts/Bounties/BountyManager.cs`, `scripts/UI/BountyScreen.cs` | **done** | `game/src/bounties/manager.rs:BountyManager`, `rotland/src/camp_ui/draw_bounties.rs`, `rotland/src/camp.rs:Camp::apply` | E on bounty board opens UI; accept, abandon, and turn in completed bounties for dollars and skill XP. |
| Economy | Bounty progress tracking (KillZombies, KillAtNight, LootCrates, DeliverItems) | `scripts/Bounties/BountyManager.cs` | **done** | `game/src/bounties/progress.rs`, `rotland/src/play.rs:326`, `rotland/src/main.rs:1461` | Zombie kills and crate lootings automatically advance bounty progress; deliveries check bag inventory on turn-in. |
| Economy | Clear POI bounties (tracking whether building zombies at target POI have been eliminated) | `scripts/Bounties/BountyManager.cs`, `scripts/World/PoiManager.cs:IsCleared` | **done** | `game/src/bounties/progress.rs:on_poi_cleared`, `rotland/src/main.rs:1541` | When all building zombies at a target POI are eliminated, `bounties.on_poi_cleared` is dispatched in `main.rs`, completing Clear POI contracts. |

---

### Animals

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Animals | Wild animal species (deer, deer buck with antlers, rabbit, boar with `data/animals.json` stats) | `scripts/Animals/AnimalConfig.cs`, `scripts/Animals/AnimalSpeciesData.cs` | **done** | `game/src/animals/config.rs:AnimalConfigData`, `rotland/src/wildlife.rs:Wildlife::new` | Loads all 4 animal species and models (`deer`, `deer_buck`, `rabbit`, `boar`). |
| Animals | Wild animal AI & senses (Calm, Alert, Flee, Charge, Attack, Dead; crouch stealth, gunshot fright) | `scripts/Animals/AnimalAI.cs`, `scripts/Animals/AnimalSenses.cs`, `scripts/Animals/AnimalHerd.cs` | **done** | `game/src/animals/ai.rs:AnimalBrain`, `rotland/src/wildlife.rs:Wildlife::update`, called from `rotland/src/main.rs:1591-1640` | Animals graze; flee from running player or gunshots; boars charge; herd members alert nearby animals. |
| Animals | Animal combat (boar gore attacks on player and zombies; player melee and gunfire damaging animals) | `scripts/Animals/AnimalCombat.cs` | **done** | `game/src/animals/combat.rs:AnimalCombat`, `rotland/src/wildlife.rs:PreyTarget`, called from `rotland/src/main.rs:1006,1190,1618` | Boar charges damage player; player bullets and melee swings damage animals. |
| Animals | Animal butchering & yields (holding E on dead animal corpse for 3s to harvest meat, hide, antlers) | `scripts/Animals/AnimalButcher.cs`, `scripts/Animals/AnimalLoot.cs`, `scripts/Animals/AnimalHunting.cs` | **done** | `game/src/animals/butcher.rs:AnimalButcher`, `rotland/src/wildlife.rs:Wildlife::corpse_in_front`, called from `rotland/src/main.rs:1315-1335` | E on corpse butchers it over time; harvests meat, hide, antlers; grants hunting skill XP; faster with knife. |
| Animals | Dynamic animal spawning & herd clustering (spawning herds in biomes, despawning far away) | `scripts/Animals/AnimalSpawner.cs`, `scripts/Animals/AnimalPool.cs` | **done** | `game/src/animals/spawner.rs:pick_spawn_spot`, `game/src/animals/manager.rs:Animals::update`, `rotland/src/wildlife.rs` | Herds spawn in appropriate biomes outside camp no-spawn zone; despawn when > 150m from player. |

---

### Farming

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Farming | Farming config & crops (wheat, potato, carrot crop definitions, grow days, seed/produce items) | `scripts/Farming/FarmConfig.cs`, `scripts/Farming/CropDef.cs` | **done** | `game/src/farming/config.rs:FarmConfigData`, `rotland/src/camp.rs:Camp.farm` | Loads all crops and farming numbers from `data/farming.json`. |
| Farming | Tilling soil with hoe (ground raycast, slope check, creating tilled farm plot) | `scripts/Farming/FarmManager.Actions.cs` | **done** | `game/src/farming/actions.rs:till_soil`, `rotland/src/camp.rs:Camp::farm_interact`, called from `rotland/src/main.rs:1280` | Looking at flat ground with hoe prompts "[E] Till soil"; creates new farm bed. |
| Farming | Planting, watering & harvesting (planting seeds, watering with watering can, harvesting ripe crops + XP) | `scripts/Farming/FarmManager.Actions.cs`, `scripts/Farming/FarmRules.cs` | **done** | `game/src/farming/actions.rs`, `rotland/src/camp.rs:Camp::farm_interact`, called from `rotland/src/main.rs:1280` | E plants seeds, waters bed (can loses 1 charge), harvests ripe crop into inventory with farming skill XP. |
| Farming | Crop growth & drying/death cycle (growth over game hours when watered, drying pause, death after 72 dry hours) | `scripts/Farming/FarmManager.cs`, `scripts/Farming/FarmRules.cs` | **done** | `game/src/farming/manager.rs:FarmManager::sync_clock`, called from `rotland/src/main.rs:1484` | Crops grow while watered; pause and dry out without water; die after 72 dry hours. |
| Farming | Zombie crop trampling (zombies walking over field plots roll trample chance to destroy crop) | `scripts/Farming/FarmManager.Trample.cs` | **done** | `game/src/farming/trample.rs:trample_step`, called from `rotland/src/main.rs:1487` | Zombies walking across field beds can trample crops; camp beds remain immune. |
| Farming | Camp starter farm plots (4 pre-placed starter farm plots in camp safe zone) | `scripts/Farming/FarmManager.cs:EnsureCampPlots` | **done** | `game/src/farming/manager.rs:ensure_camp_plots`, called from `rotland/src/main.rs:474` | Initializes 4 starter beds inside camp on world creation. |
| Farming | Watering can refill at water (using water bottles in bag to refill empty watering can charges) | `scripts/Farming/FarmManager.cs:RefillCan` | **logic only** | `game/src/farming/actions.rs:refill_can` | Refill logic exists in `game` crate, but is not exposed or triggered in `rotland`. |

---

### Dogs

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Dogs | Puppy purchase & breeding (breeder NPC selling puppy for $150) | `scripts/Dogs/DogBreeder.cs` | **done** | `game/src/dogs/breeder.rs`, `game/src/dogs/manager.rs:buy_from_breeder`, called from `rotland/src/main.rs:1349` | E on breeder NPC in camp buys puppy for $150; spawns follower dog. |
| Dogs | Dog commands & movement (Follow, Stay with G key, following player, ground snapping, tree avoidance) | `scripts/Dogs/DogManager.cs`, `scripts/Dogs/DogMotion.cs`, `scripts/Dogs/DogSteering.cs` | **done** | `game/src/dogs/manager.rs:toggle_stay`, `game/src/dogs/motion.rs`, called from `rotland/src/main.rs:887,1729-1770` | G key toggles between Follow and Stay; dog navigates terrain smoothly around trees. |
| Dogs | Shift+G Go Home command (sending dog back to camp safe zone) | `scripts/Dogs/DogManager.cs:GoHome` | **logic only** | `game/src/dogs/manager.rs:go_home`, `game/src/dogs/dog_update.rs:149` | Godot allows Shift+G to send the dog home to camp; `DogManager::go_home` and `DogMode::Home` exist in `game` crate, but `main.rs` only handles G key toggle stay. |
| Dogs | Dog combat & defense (barking at zombies within 20m, attacking chasing zombies within 10m for 15 dmg) | `scripts/Dogs/DogCombat.cs` | **done** | `game/src/dogs/combat.rs:DogCombat::update`, called from `rotland/src/main.rs:1729-1770` | Dog barks at nearby zombies; bites chasing zombies; takes damage when attacked. |
| Dogs | Dog feeding & healing (feeding meat from bag to restore 30 HP) | `scripts/Dogs/Dog.cs:FeedFromBag`, `scripts/Dogs/DogFood.cs` | **done** | `game/src/dogs/food.rs`, `rotland/src/hub_ui/draw_dog.rs:Feed button`, called from `rotland/src/main.rs:2074` | Feed button in DogMenu consumes raw or cooked meat to heal dog +30 HP. |
| Dogs | Dog Menu & Dog Pack storage screen (E on dog opens menu; 4-slot 10kg pack storage) | `scripts/Dogs/DogMenu.cs`, `scripts/Dogs/DogPackScreen.cs` | **done** | `rotland/src/hub_ui/draw_dog.rs:draw_dog_menu`, `draw_dog_pack_screen`, called from `rotland/src/main.rs:1290-1300` | E on dog opens menu; allows transferring items between player bag and dog pack. |
| Dogs | Dog HUD widget (dog name, HP bar, low HP warning on main gameplay HUD) | `scripts/Dogs/DogHud.cs` | **missing** | None in `rotland/src/hud.rs` | Godot displays dog status strip above needs bars on HUD; Rust HUD only displays player stats. |
| Dogs | Dog naming dialog (popup to enter custom dog name on puppy purchase) | `scripts/Dogs/DogNameDialog.cs` | **logic only** | `rotland/src/hub_ui/draw_dog.rs:draw_dog_name_dialog`, `rotland/src/hub_ui/screen.rs:HubScreen::DogNameDialog` | Naming popup UI and validation exist in `hub_ui`, but `main.rs:1349` immediately purchases puppy with default name "Rex" without opening dialog. |
| Dogs | Stray dogs at POI lots (spawning strays at cabin/house lots, fleeing when rushed, taming with meat) | `scripts/Dogs/Stray.cs`, `scripts/Dogs/StrayDogSpawner.cs` | **done** | `rotland/src/poi_links.rs:build_dog_poi_sites`, `rotland/src/main.rs:1734-1762` | Loaded house_small and cabin POIs roll 5% chance to spawn sitting stray dog that flees when player sprints close and can be tamed with meat. |

---

### Vehicles

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Vehicles | Car driving physics (WASD throttle/steering, Space handbrake, spring suspension, speed) | `scripts/Vehicles/Car.cs`, `scripts/Vehicles/CarDrive.cs` | **done** | `game/src/vehicles/car.rs:Car::update`, called from `rotland/src/main.rs:915-940,1687` | Fully drivable hatchback with raycast suspension, forward/reverse throttle, steering, handbrake. |
| Vehicles | Car entry / exit (E key to enter or exit, speed exit block > 15 km/h, clear exit spot selection) | `scripts/Vehicles/CarDriver.cs` | **done** | `game/src/vehicles/manager.rs:CarManager::enter`, `exit`, called from `rotland/src/main.rs:910,1270` | E enters vehicle; E exits if speed <= 15 km/h; finds safe ground position beside car. |
| Vehicles | Car fuel consumption & refueling (fuel burn while driving, F key refuels from `fuel_can` item) | `scripts/Vehicles/CarFuel.cs`, `scripts/Vehicles/CarPoint.cs` | **done** | `game/src/vehicles/fuel.rs:CarFuel::burn`, called from `rotland/src/main.rs:1274` | Fuel drains with throttle and engine idle; F consumes `fuel_can` from inventory (+10L). |
| Vehicles | Car condition & damage (car HP damaged in crashes, running over zombies, wrecking at 0 HP) | `scripts/Vehicles/CarHealth.cs` | **done** | `game/src/vehicles/health.rs:CarHealth`, called from `rotland/src/main.rs:1687,1711` | Car takes damage from collisions and hitting entities; engine stops at 0 HP. |
| Vehicles | Running over zombies & animals (ramming targets at > 15 km/h deals speed * 1.8 damage) | `scripts/Vehicles/CarRunOver.cs` | **done** | `game/src/vehicles/run_over.rs:run_over_step`, called from `rotland/src/main.rs:1694-1706` | Ramming zombies or wildlife at speed knocks them down and inflicts lethal damage. |
| Vehicles | Car chase camera (camera following car at higher elevation, pulled in by terrain) | `scripts/Vehicles/CarCamera.cs` | **done** | `rotland/src/main.rs:1075-1088` | Smooth chase camera positions above and behind vehicle, adjusting height to clear hills. |
| Vehicles | Car dashboard gauge HUD (km/h speed, fuel gauge, condition HP bar) | `scripts/Vehicles/CarGauge.cs` | **done** | `game/src/vehicles/gauge.rs:format_gauge`, `rotland/src/hud.rs:draw_car_gauge`, called from `rotland/src/main.rs:1976` | Overlay displays vehicle speed, fuel level, and condition while driving. |
| Vehicles | Car horn & engine noise (H key horn, engine sound, noise alerting zombies) | `scripts/Vehicles/CarDriver.cs:Honk`, `scripts/Vehicles/CarSound.cs` | **done** | `rotland/src/main.rs:920,1708-1710` | H sounds car horn; emits 60m noise event alerting nearby zombies. |
| Vehicles | Car trunk storage screen (20 slots, 150 kg storage screen accessed via E at car trunk) | `scripts/Vehicles/Car.cs:Trunk`, `scripts/UI/TrunkScreen.cs` | **done** | `rotland/src/trunk_play.rs`, `rotland/src/main.rs:1242-1250,2077-2090` | Pressing E at back of parked car opens 20-slot trunk inventory screen, allows transferring items within weight limits, auto-closes on walk away. |
| Vehicles | Parked car solid colliders (parked cars blocking player, zombies, and other vehicles in physics) | `scripts/Vehicles/Car.cs` | **done** | `rotland/src/car_solid.rs:ParkedCarSolids`, `rotland/src/main.rs:1688-1689` | Parked cars register static Rapier3D cuboid colliders tagged with car hash, blocking entities while skipped by vehicle's own suspension rays. |
| Vehicles | Car headlights projection (headlights illuminating road with spotlights at night) | `scripts/Vehicles/CarLights.cs` | **done** | `rotland/src/car_lights.rs:headlight_spots`, `rotland/src/main.rs:1930-1941` | Renderer supports 3 spot lights; turning car lights on (L key) projects left and right headlight spot beams onto the road ahead. |
| Vehicles | POI car spawning (35% chance to spawn hatchback at gas station & house_small POIs) | `scripts/Vehicles/CarSpawnRules.cs`, `scripts/Vehicles/CarManager.cs` | **logic only** | `game/src/vehicles/spawn_rules.rs:CarSpawnRules` | Only the starter camp car is spawned in `rotland`; POI car spawning at world buildings is not called. |

---

### UI

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| UI | In-game HUD (health, stamina, hunger, thirst, clock, ammo, crosshair, messages) | `scripts/UI/HUD.cs`, `scripts/UI/FpsLabel.cs`, `scripts/UI/ClockLabel.cs`, `scripts/UI/Crosshair.cs` | **done** | `rotland/src/hud.rs:draw`, called from `rotland/src/main.rs:1970` | Complete HUD overlay with vitals bars, crosshair, ammo counter, clock, toasts, and FPS. |
| UI | Hotbar bar (6 slots on bottom center showing items, stack count, active selection, flashlight ON) | `scripts/UI/HotbarBar.cs` | **done** | `rotland/src/hud.rs:draw_hotbar`, called from `rotland/src/main.rs:1970` | Renders 6 hotbar slots with selection border and stack numbers. |
| UI | Safe zone HUD label ("Safe zone" banner displayed when inside camp safe zone) | `scripts/UI/SafeZoneLabel.cs` | **missing** | None | No "Safe zone" label rendered when inside camp perimeter. |
| UI | Key help bar (bottom right hotkey bar: "[Tab] Bag [L] Skills [J] Bounties [M] Map [E] Use [Esc] Menu") | `scripts/UI/KeyBar.cs` | **missing** | None | Godot displays quick hotkey reminder strip; missing from Rust HUD. |
| UI | Compass strip (top-center heading strip with cardinal/ordinal ticks) | `scripts/UI/Compass.cs`, `scripts/UI/CompassMath.cs` | **done** | `rotland/src/hud.rs:draw_compass`, called from `rotland/src/main.rs:1970` | Displays sliding 600px compass strip with degree ticks and N/NE/E/SE/S/SW/W/NW labels. |
| UI | Compass camp & waypoint markers (displaying camp distance/direction and user waypoint on compass) | `scripts/UI/Compass.cs:HasCamp`, `scripts/UI/Compass.cs:Waypoint` | **missing** | None in `rotland/src/hud.rs` | `draw_compass` only draws direction ticks; camp direction and waypoint diamond are not drawn on compass. |
| UI | Level up banner & XP toasts (large level up banner, stacking XP gain toasts on right) | `scripts/UI/LevelUpBanner.cs`, `scripts/UI/XpToastStack.cs` | **done** | `rotland/src/hub_ui/draw_overlays.rs:draw_overlays`, called from `rotland/src/main.rs:2018` | Large animated level-up banner and stacking XP toasts (+25 Combat XP). |
| UI | Player hub - Bag tab (24-slot inventory grid, item detail panel, use/equip/drop/split buttons) | `scripts/UI/InventoryScreen.cs`, `scripts/UI/InventoryLayout.cs`, `scripts/UI/InventoryDetailsPanel.cs` | **done** | `rotland/src/hub_ui/draw_bag.rs`, `draw_bag_details.rs`, called from `rotland/src/main.rs:2040` | Tab key opens Bag; full slot grid, item inspection, equip, drop, use, and stack splitting. |
| UI | Player hub - Crafting list (recipes list under backpack, required materials, craft button, cancel) | `scripts/UI/CraftingPanel.cs`, `scripts/UI/CraftingRecipeRow.cs` | **done** | `rotland/src/hub_ui/draw_crafting.rs`, `rotland/src/camp_ui/draw_crafting.rs`, `rotland/src/camp.rs:Camp::apply` | Renders recipes; checks materials; progresses craft timers; checks nearby lit campfire for campfire recipes. |
| UI | Player hub - Skills tab (list of all skills, current levels, and progress to next level) | `scripts/UI/SkillsPanel.cs` | **done** | `rotland/src/hub_ui/draw_skills.rs:draw_skills`, called from `rotland/src/main.rs:2040` | L key opens Skills tab; shows level, current XP, and progress bars for all skills. |
| UI | Player hub - Bounties tab (read-only view of active and offered bounties with progress) | `scripts/UI/HubBountiesPage.cs` | **done** | `rotland/src/hub_ui/draw_bounties.rs:draw_hub_bounties`, called from `rotland/src/main.rs:2040` | J key opens Bounties tab; shows active bounty progress and available daily/weekly contracts. |
| UI | Player hub - Map tab (explored fog of war, player arrow, camp marker, right-click waypoint, zoom/pan) | `scripts/UI/HubMapPage.cs`, `scripts/UI/MapView.cs`, `scripts/UI/MapTiles.cs`, `scripts/UI/MapMarkers.cs` | **done** | `rotland/src/map_ui/draw.rs:draw_map`, `draw_icons.rs`, called from `rotland/src/main.rs:2070` | M key opens Map; reveals fog around player; right-click sets waypoint; zoom in/out and drag-pan. |
| UI | Player hub - Profile tab (lifetime stats, character portrait) | `scripts/UI/HubProfilePage.cs`, `scripts/UI/HubPortrait.cs` | **done** | `rotland/src/hub_ui/draw_profile.rs:draw_profile`, called from `rotland/src/main.rs:2040` | P key opens Profile; displays stats (days, playtime, kills, headshots, distance, deaths) and difficulty info. |
| UI | Camp shop & banking screens (dedicated trader buy/sell and bank deposit/withdraw screens) | `scripts/UI/ShopScreen.cs`, `scripts/UI/TradeScreen.cs`, `scripts/UI/BankScreen.cs` | **done** | `rotland/src/camp_ui/draw_shop.rs`, `draw_bank.rs`, called from `rotland/src/main.rs:2000` | E on NPCs opens dedicated trader shop and bank vault screens. |
| UI | New player tutorial hints (tip boxes on left side of screen triggered by player actions) | `scripts/UI/HintSystem.cs`, `scripts/UI/HintPanel.cs` | **done** | `rotland/src/hud.rs:draw_hint`, `game/src/hints/system.rs:HintSystem::tick`, called from `rotland/src/main.rs:1425,1973` | Tutorial tips queue and render on screen based on game triggers. |
| UI | Main Menu & Pause Menu (ROTLAND title screen, Esc pause menu with Save/Quit) | `scripts/UI/MainMenu.cs`, `scripts/UI/PauseMenu.cs` | **done** | `rotland/src/menus/menus_draw.rs:draw_main_menu`, `draw_pause_menu`, called from `rotland/src/main.rs:1960` | Full main menu (`--menu`) and in-game pause menu with Save & Quit. |
| UI | Save slot picker (3 slot cards showing seed, day, playtime, filled/empty/damaged states) | `scripts/UI/SlotPicker.cs` | **done** | `rotland/src/menus/menus_draw/menus_draw_slots.rs:draw_slot_picker`, called from `rotland/src/main.rs:1960` | Displays 3 save slots, lets player continue, start new game, or delete with confirmation. |
| UI | New game difficulty selection screen (choosing preset, sliders for zombie amount/loot/damage/death) | `scripts/UI/DifficultyScreen.cs` | **missing** | None | New Game button starts world immediately with default preset; difficulty setup screen before world start is missing. |
| UI | Controls rebinding panel (UI panel to rebind physical action keys and resolve conflicts) | `scripts/UI/ControlsPanel.cs` | **done** | `rotland/src/menus/menus_draw/menus_draw_settings.rs:161-190`, `rotland/src/main.rs:2006` | Settings menu includes scrollable Controls section letting player rebind movement, actions, hub panels, and hotbar keys with active rebind detection. |
| UI | HUD Mini-map (radar/mini-map corner display showing player surroundings and markers during play) (GitHub #7) | None | **missing** | None | **new (not in Godot)**. Owner requested HUD mini-map on GitHub (#7); currently only full-screen map tab exists (M key). |
| UI | Profile character portrait (3D player avatar rendering / customizable portraits) (GitHub #8) | `scripts/UI/HubPortrait.cs` | **logic only** | `rotland/src/hub_ui/draw_profile.rs:42` | **new (not in Godot)** feature requested on GitHub #8. Currently draws a 2D geometric silhouette placeholder instead of an interactive 3D model preview or selectable portrait. |

---

### Audio

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Audio | Sound library & definition loading (loading `data/sounds.json` with variant audio streams) | `scripts/Audio/SoundLibrary.cs`, `scripts/Audio/SoundDef.cs` | **done** | `audio/src/library.rs:SoundLibrary::from_repo`, `audio/src/lib.rs:Audio::new` | Loads all WAV audio files, volume, and distance attenuation definitions from `data/sounds.json`. |
| Audio | Audio mixer & volume channels (Master, SFX, and Ambient volume channels) | `scripts/Audio/AudioBuses.cs` | **done** | `audio/src/lib.rs:Audio::set_volumes`, called from `rotland/src/main.rs:1380` | Master, SFX, and Ambient volumes dynamically control Kira audio sub-tracks. |
| Audio | Day / night ambient loop mixer (crossfading `day_loop.wav` and `night_loop.wav` with clock) | `scripts/Audio/AmbienceMixer.cs` | **done** | `audio/src/lib.rs:Audio::tick`, called from `rotland/src/main.rs:1382` | Seamlessly fades between daytime birds/wind and nighttime crickets/eerie loop with clock hour. |
| Audio | 3D spatial voices & listener tracking (audio sources panned and attenuated based on distance) | `scripts/Audio/AudioManager.cs`, `scripts/Audio/VoicePool.cs` | **done** | `audio/src/lib.rs:Audio::play_at`, `set_listener`, called from `rotland/src/main.rs:1378` | World sounds pan and attenuate relative to camera position and orientation. |
| Audio | Player footstep sounds (timed footsteps while walking/sprinting on ground) | `scripts/Audio/Footsteps.cs` | **done** | `rotland/src/main.rs:1385-1395` | Plays `footstep_grass` every 0.5s walking, 0.33s sprinting when moving on floor. |
| Audio | Weapon sound effects (gunshot, reload, melee swing, melee hit sounds) | `scripts/Audio/SoundLibrary.cs:WeaponSound` | **done** | `rotland/src/shooting.rs:Shooting::update_targets`, `rotland/src/main.rs:1008-1014,1198` | Plays shot, reload, dry fire, swing, and hit sounds. |
| Audio | Zombie vocalizations (periodic groans every 6-14s within 25m, attack groans) | `scripts/Audio/ZombieVoices.cs` | **done** | `game/src/zombie_voices/system.rs:ZombieVoices`, `rotland/src/main.rs:1547-1560` | Zombies within 25m periodically play 3D spatial `zombie_groan` sounds, and attacking zombies emit `zombie_attack` shrieks. |
| Audio | UI click sounds (playing `ui_click` on button clicks) | `scripts/Audio/UiClickSounds.cs` | **missing** | None | egui button clicks are silent. |

---

### Saves

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Saves | Save system architecture & atomic file writes (temp file write then replace) | `scripts/Core/SaveSystem.cs`, `scripts/Core/SaveFile.cs` | **done** | `save/src/file.rs:write_atomic`, `rotland/src/session.rs`, called from `rotland/src/main.rs:275` | Atomic JSON serialization prevents corrupted saves; full compatibility with Godot save format. |
| Saves | 3 Save slots management (`slot1.json`, `slot2.json`, `slot3.json` summary, deletion) | `scripts/Core/SaveSlots.cs` | **done** | `save/src/slots.rs:Slots`, `rotland/src/session.rs:summaries`, `rotland/src/menus/menus_draw/menus_draw_slots.rs` | Manages 3 slots; extracts metadata; deletes slots cleanly. |
| Saves | Autosave and save hotkeys (F5 save, F9 load, F8 new game, autosave interval) | `scripts/Core/SaveSystem.cs` | **done** | `rotland/src/main.rs:247-256,715-730` | F5 saves immediately; F9 reloads active slot; saves on pause quit. |
| Saves | World, Player & Time persistence (`world`, `player`, `time` save systems) | `scripts/Player/PlayerSave.cs`, `scripts/World/WorldRoot.cs`, `scripts/World/TimeOfDay.cs` | **done** | `save/src/systems/world.rs`, `player.rs`, `time.rs`, `rotland/src/session.rs:capture`, `restore` | Preserves seed, player position, camera pitch/yaw, health, day, and hour. |
| Saves | Inventory & Loot persistence (`inventory`, `worldstate.looted` save systems) | `scripts/Items/InventorySave.cs`, `scripts/World/WorldState.cs` | **done** | `save/src/systems/inventory.rs`, `worldstate.rs`, `rotland/src/session.rs:capture`, `restore_play` | Full roundtrip of carried item stacks and looted crate IDs. |
| Saves | Economy, Banking & Bounties persistence (`wallet`, `bank`, `trader`, `bounties` save systems) | `scripts/Economy/Wallet.cs`, `scripts/Economy/Bank.cs`, `scripts/Economy/TraderStock.cs`, `scripts/Bounties/BountyManager.cs` | **done** | `save/src/systems/wallet.rs`, `rotland/src/camp.rs:Camp::save_into`, `load_from` | Preserves dollars in wallet and bank, bank item inventory, trader stock, and active bounties. |
| Saves | Survival needs & Map exploration persistence (`need_hunger`, `need_thirst`, `map` save systems) | `scripts/Survival/NeedStat.cs`, `scripts/World/WorldMap.cs`, `scripts/World/MapExplore.cs` | **done** | `rotland/src/session.rs:122-124`, `game/src/map/explore.rs:to_dictionary` | Stores hunger/thirst levels and 64-bit chunk bitmasks for revealed map fog. |
| Saves | Farming & Campfires persistence (`farm`, `worldstate.campfires` save systems) | `scripts/Farming/FarmManager.Save.cs`, `scripts/World/StoredCampfire.cs` | **done** | `game/src/farming/save.rs`, `game/src/campfire/save.rs`, `rotland/src/camp.rs:205-207` | Preserves farm plots, crop growth, dry hours, and remaining campfire burn hours. |
| Saves | Vehicles & Dog persistence (`cars`, `dog` save systems) | `scripts/Vehicles/CarManager.cs`, `scripts/Dogs/DogManager.cs` | **done** | `game/src/vehicles/save.rs`, `game/src/dogs/save.rs`, `rotland/src/main.rs:293-296,443-446` | Stores car positions, fuel, condition, and dog mode, health, name, and growth. |
| Saves | Weapons equipped & magazine ammo persistence (`weapons` save system) | `scripts/Combat/WeaponHolder.cs` | **missing** | None | Loaded magazine rounds and equipped weapon selection are not saved (start bat re-equipped on load). |
| Saves | Settlement visit state persistence (`settlements` save system saving visited camps & respawn camp) | `scripts/Settlement/SettlementWatcher.cs` | **logic only** | `game/src/camp/watcher.rs:SettlementWatcher::to_dictionary` | Logic exists in `game` crate, but `main.rs` never saves or loads the `settlements` system. |

---

### Settings

| Area | Feature | Godot file(s) | Status | Rust evidence | Notes |
|---|---|---|---|---|---|
| Settings | Mouse sensitivity & Invert Y settings (adjustable in settings menu, applied to camera control) | `scripts/Core/GameSettings.cs` | **done** | `rotland/src/settings.rs:Settings`, `rotland/src/main.rs:855,857` | Mouse sensitivity multiplier and invert Y toggle are active during gameplay. |
| Settings | Audio volume controls (Master, Music/Ambience, SFX sliders applied to audio engine) | `scripts/Core/GameSettings.cs`, `scripts/Audio/AudioBuses.cs` | **done** | `rotland/src/settings.rs:Settings`, `audio/src/lib.rs:Audio::set_volumes`, `rotland/src/main.rs:1380` | Sliders in settings menu dynamically adjust Kira audio bus levels. |
| Settings | Show FPS overlay setting (toggle in settings menu controlling FPS label visibility) | `scripts/Core/GameSettings.cs`, `scripts/UI/FpsLabel.cs` | **done** | `rotland/src/settings.rs:Settings`, `rotland/src/main.rs:1968` | Toggles debug FPS counter on HUD. |
| Settings | Field of view (FOV) setting (slider in settings menu adjusting camera base FOV) | `scripts/Core/GameSettings.cs`, `scripts/Player/CameraRig.cs` | **done** | `rotland/src/display.rs:apply_fov`, `rotland/src/main.rs:1094` | FOV setting (60°-100°) is applied to gameplay camera via `display::apply_fov`, proportionally scaling base camera FOV and aim FOV. |
| Settings | Render scale setting (slider in settings menu adjusting internal rendering resolution) | `docs/RUST_PLAN.md` (engine design) | **done** | `rotland/src/display.rs:display_request`, `engine/src/app.rs:Game::display`, `rotland/src/main.rs:2151-2152` | Internal rendering target resolution dynamically scales via `DisplayRequest` (0.5 to 1.0) applied each frame. |
| Settings | V-Sync toggle (checkbox in settings menu toggling vertical sync) | `scripts/Core/GameSettings.cs` | **done** | `rotland/src/display.rs:display_request`, `engine/src/app.rs:Game::display`, `rotland/src/main.rs:2151-2152` | Presentation mode is dynamically updated via `DisplayRequest` when changed in Settings. |
| Settings | Graphics presets (Low, Medium, High controlling shadow blur, shadow distance, chunk load radius) | `scripts/Core/GraphicsPresets.cs`, `scripts/World/GraphicsApplier.cs` | **logic only** | `rotland/src/settings.rs:Settings.graphics_preset`, `rotland/src/menus/menus_draw/menus_draw_settings.rs` | Dropdown exists in settings menu, but presets are not applied to renderer shadow parameters or chunk streaming distance. |
| Settings | In-game key rebinding & conflicts resolution (custom key bindings saved to settings) | `scripts/Core/KeyBindings.cs`, `scripts/UI/ControlsPanel.cs` | **done** | `rotland/src/settings.rs:Settings.key_bindings`, `game/src/input.rs:InputBindings`, `rotland/src/main.rs:837` | Key bindings persisted in `settings.json`; active `KeyMap` in `main.rs` dynamically updates movement, interaction (E), reload (R), whistle (G), hub tabs (Tab/L/J/P/M), and hotbar slots (1-6). |

---

## Top 15 Missing / Logic-Only Features

Ranked in order of player impact (how much a player would notice the absence during gameplay), along with implementation size estimates:

1. **Zombie special behaviors (crawler from leg hits, herd alert, scream, corpse eating)** (`scripts/Enemies/ZombieAI.Actions.cs`, `ZombieWounds.cs`, `ZombieHerd.cs`)
   - *Status*: **logic only**
   - *Player impact*: Combat feels repetitive and flat: shooting zombies in the legs does not turn them into crawlers, sighting players never triggers alarm screams to summon nearby horde members, and idle zombies ignore corpses.
   - *Size*: **medium** (pass `leg_hit` from hit height < 0.6m, feed live zombie positions to `nearby_zombies`, route scream alerts to spawner/noises).

2. **Dog HUD health bar & low health indicator** (`scripts/Dogs/DogHud.cs`)
   - *Status*: **missing**
   - *Player impact*: The player has no way to monitor their companion dog's health or know when it is under attack or low on HP without manually opening the dog interaction menu.
   - *Size*: **small** (render compact dog name and health bar widget on HUD above vital bars when a dog is owned).

3. **Compass camp & waypoint markers** (`scripts/UI/Compass.cs`)
   - *Status*: **missing**
   - *Player impact*: The compass only shows cardinal degree ticks; finding camp or user-set waypoints requires constantly stopping and pressing M to view the fullscreen map.
   - *Size*: **small** (calculate bearing in `rotland/src/hud.rs:draw_compass` using `CompassMath`, render camp icon with distance and diamond waypoint marker).

4. **Scope overlay reticle on rifle zoom** (`scripts/UI/ScopeOverlay.cs`)
   - *Status*: **missing**
   - *Player impact*: Aiming with the scoped rifle zooms the camera FOV but lacks the black circular mask vignette and sniper crosshair overlay, making precision long-range aiming feel unpolished.
   - *Size*: **small** (render fullscreen black circular vignette with crosshair reticle in egui while scoped aim is active).

5. **Gun recoil climb & muzzle flash** (`scripts/Combat/GunEffects.cs`, `scripts/Combat/GunHandler.cs`)
   - *Status*: **missing**
   - *Player impact*: Firing weapons lacks tactile kick and nocturnal visual feedback; shots produce no recoil pitch climb on the camera and no muzzle flash light cone in the darkness.
   - *Size*: **medium** (apply pitch climb to camera rig in `CameraRig`, bind dynamic point light or muzzle sprite in renderer).

6. **Key help bar on HUD** (`scripts/UI/KeyBar.cs`)
   - *Status*: **missing**
   - *Player impact*: New and returning players frequently forget hotkeys for Bag (Tab), Skills (L), Bounties (J), Map (M), Whistle (G), and vehicle controls.
   - *Size*: **small** (draw subtle hotkey reminder bar in the bottom right corner of the HUD).

7. **Weapons equipped & magazine ammo persistence in saves** (`scripts/Combat/WeaponHolder.cs`)
   - *Status*: **missing**
   - *Player impact*: Reloading a save always re-equips the starter baseball bat and forgets loaded magazine counts, forcing players to re-equip and reload their guns after every load.
   - *Size*: **small** (serialize active weapon slot and current magazine ammo counts into the save file `weapons` system).

8. **Safe zone protection from zombies** (`scripts/Enemies/ZombiePlayerLink.cs`, `scripts/Settlement/SettlementRegistry.cs:SealsOff`)
   - *Status*: **logic only**
   - *Player impact*: Zombies chase the player deep into the camp safe zone and attack because `main.rs` passes `self.player` as target even while inside camp boundaries.
   - *Size*: **small** (pass `target: None` to `zombies.update` when player position is inside camp safe radius).

9. **Attached visual backpack model on player character** (`scripts/Player/PlayerBackpack.cs`)
   - *Status*: **missing**
   - *Player impact*: In 3rd person, player back looks bare despite carrying 24 inventory slots; no visual progression for bag upgrades (`backpack_small.glb` / `backpack_large.glb`).
   - *Size*: **medium** (attach backpack model to player chest/spine joint in humanoid renderer).

10. **HUD Mini-map** (GitHub #7)
    - *Status*: **missing**, **new (not in Godot)**
    - *Player impact*: High owner demand on GitHub; provides real-time navigational awareness of nearby POIs, terrain, and threats in the corner of the screen without interrupting gameplay.
    - *Size*: **medium** (render circular or square minimap texture in HUD corner showing terrain and markers).

11. **Dog naming dialog on purchase** (`scripts/Dogs/DogNameDialog.cs`)
    - *Status*: **logic only**
    - *Player impact*: Purchasing a puppy immediately names it "Rex" without giving the player a chance to choose a name, even though the UI dialog exists in `hub_ui`.
    - *Size*: **small** (open existing `HubScreen::DogNameDialog` upon purchase in `main.rs` instead of instant purchase).

12. **New game difficulty selection screen** (`scripts/UI/DifficultyScreen.cs`)
    - *Status*: **missing**
    - *Player impact*: Players cannot select difficulty presets (Easy, Normal, Hard, Permadeath) or tune damage/loot multipliers before starting a new run.
    - *Size*: **medium** (add difficulty screen before creating world save in slot picker).

13. **POI car spawning** (`scripts/Vehicles/CarSpawnRules.cs`, `scripts/Vehicles/CarManager.cs`)
    - *Status*: **logic only**
    - *Player impact*: Exploring the world only reveals the starter camp car; gas stations and houses never spawn drivable cars.
    - *Size*: **small** (evaluate `CarSpawnRules` on chunk POIs to spawn hatchbacks at world buildings).

14. **Graphics presets applied to engine** (`scripts/Core/GraphicsPresets.cs`, `scripts/World/GraphicsApplier.cs`)
    - *Status*: **logic only**
    - *Player impact*: Low/Medium/High preset dropdown in Settings changes nothing about shadow distance, shadow resolution, or chunk load radius.
    - *Size*: **small** (map preset choice to shadow distance and chunk streamer radius).

15. **Watering can refill at water** (`scripts/Farming/FarmManager.cs:RefillCan`)
    - *Status*: **logic only**
    - *Player impact*: Once the watering can runs out of charges, farming halts unless the player crafts or buys a new can, because refilling with water bottles is not hooked up.
    - *Size*: **small** (hook up `refill_can` when pressing E or using watering can with water bottles in bag).
