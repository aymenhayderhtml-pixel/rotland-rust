# Rotland: Game Design (living document)

*Rotland is the working title and can change.*

Status: **brainstorm → being locked**. Anything marked ❓ is an open question (see `DECISIONS.md`).

## Pitch
You are one survivor in a world taken over by zombies. Explore a huge open world generated from chunks, with forests, ruined cities and more. Fight with guns and swords, hunt, farm, cook and level up your skills. Trade with **NPC survivor settlements** that run a living economy. Travel on foot, on horseback, by car, by motorbike or in a small boat.

- **Mode:** single player, offline.
- **Camera:** third person (like Genshin) **and** first person. A key switches between them. Aiming guns works in both.
- **Look:** low-poly and stylized. It runs on integrated graphics and is easy to make with AI and Blender.

## Core loop
Explore → fight or avoid zombies → loot, hunt and harvest → cook, craft and farm → trade at a settlement → upgrade gear, skills and vehicles → explore further and more dangerous areas.

## World
- **Chunk-based procedural generation** from a seed (the same seed always gives the same world).
- Biomes (first pass):
  | Biome | Feel | Resources | Danger |
  |---|---|---|---|
  | Forest | trees, rivers, animals | wood, game animals, herbs | low |
  | Plains / farmland | open fields, farms | crops, seeds, horses | low–medium |
  | City | ruined buildings, roads, cars | guns, ammo, cars, fuel, clothes | high |
  | Lake / coast | water, docks | fish, boats | low–medium |
  | More later (swamp, mountains, military base…) | | | |
- **Day/night cycle.** At night zombies are **faster and there are more of them**, and a special **night-only zombie type** appears. Light sources and settlements matter at night.
- Roads link the cities and settlements, so the car is useful.

## Player stats
- Health, **Hunger**, **Thirst**, Stamina. You get water from rivers and lakes (boil it to make it safe), bottles and wells.
- **Skills** that gain XP by doing:
  | Skill | XP from | Effect |
  |---|---|---|
  | Combat | killing zombies | damage, weapon handling |
  | Hunting | killing and skinning animals | more meat and hide, quieter steps |
  | Cooking | cooking recipes | better food buffs, fewer burnt meals |
  | Agility | running, climbing, dodging | stamina, speed |
  | Farming | planting and harvesting | faster growth, bigger yield |

## Combat
- **Melee:** swords, machetes, bats. Uses stamina.
- **Guns:** pistols, rifles, shotguns. Use ammo, and the noise attracts zombies.
- Zombie types (first pass): walker, runner, tank, and a **night stalker** that only comes out at night. More come later.

## Farming
Plant seeds in soil. Crops grow over in-game days and need water. Harvest them for food and seeds.

## Cooking
Use a campfire or stove with recipes. Cooked food restores more hunger and can give buffs.

## Transportation
| Vehicle | Where | Needs |
|---|---|---|
| Horse | anywhere off-road | food. **Tame a wild one** (plains) **or buy one** at a settlement |
| Car | roads best | fuel, repairs |
| Motorbike | fast, agile | fuel |
| Small boat | water | fuel or rowing |

## Economy (NPC survivors)
- **Settlements** are safe zones with NPC traders.
- Currency: **dollars ($)**.
- Shops sell weapons, food, maps, clothes, armor, vehicles, seeds and ammo.
- **Bank:** keep dollars and items safe in a settlement. They are **not lost when you die** (see Death).
- **Living prices:** each settlement has supply and demand. Selling a lot of meat lowers its price, and prices differ between settlements, so trading routes pay.
- Maps can be bought to reveal parts of the world.

## Pet dog
A companion dog you can find, adopt or buy, then **feed** and **level up**.
- It follows you and **attacks zombies** or helps **hunt** animals.
- It **barks to warn** you about nearby zombies and can **sniff out loot**.
- It can be hurt. Heal it with food or bandages. If it dies, it is **gone forever** (hardcore). You have to find or buy a new dog.

## Death
**Hardcore drop:** when you die you **lose all carried items** (inventory and equipped gear), and you respawn at the last settlement visited.
You **keep** your skills XP and anything stored in the **bank**, so the bank really matters.

## Out of scope (for now)
Online multiplayer, base building, story or quests (maybe later).
