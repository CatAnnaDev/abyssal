# Abyssal

A fully autonomous, watch-only roguelike for the terminal, written in Rust.

An AI hero descends alone into an endless procedurally generated abyss: it explores, fights, loots, levels up, learns talents, trades with rare merchants, dodges telegraphed boss attacks, dies, and starts again — all on its own. You don't play. You watch.

![Abyssal in action](assets/demo.gif)

## Screenshots

Exploration — the hero reveals rooms and corridors with a torch-lit field of view, minimap and live journal on the right.

![Exploration](assets/01-explore.png)

Combat — floating damage, elemental hits, kill combos. Here the hero springs a mimic (yes, even Frieren falls for them).

![Combat](assets/02-combat.png)

Boss telegraph — floor bosses wind up their attacks over several turns, flashing the danger tiles in red so the hero can dodge the incoming charge.

![Boss telegraph](assets/03-boss.png)

Merchant — a rare trader appears with rolled stock; viewers can vote on the purchase when Twitch mode is on, and the simulation pauses during the deal.

![Merchant](assets/04-merchant.png)

Death — a cause-coherent death card with score, best scores, and a fittingly dramatic last word before a new soul descends.

![Death screen](assets/05-death.png)

## Run

```sh
cargo run --release
```

A pre-game menu lets you pick a class, a playstyle (completionist / fighter / rusher), a difficulty, and a starting boon — or just let it roll random and watch.

## Controls

The game plays itself. Input is optional:

| Key | Action |
| --- | --- |
| `space` | pause / resume |
| `+` / `-` | faster / slower (lent → ultra) |
| `m` / `1` `2` `3` | cycle / set the hero's mindset |
| `a` | mute / unmute sound |
| `g` | toggle sprite view (half-block pixel-art) / classic glyph map |
| `s` / `l` | save / load |
| `n` | new run |
| `q` / `esc` | save & quit |

Progress autoloads on launch (`abyssal.save.json`).

## Features

- Procedural dungeons with rooms, corridors, LOS field-of-view and a discovery meter
- Branching descents: at every stairway the AI weighs 2–3 paths by mindset (and heads for a Rest branch when wounded), each leading to a biome and a room type (treasure / challenge / warren / rest)
- Five biomes (Caverns, Catacombs, Frostvault, Emberdepths, Abyss), each with its own palette, lighting tint, biased fauna, ambient hazard, a themed champion mini-boss, and musical key
- Two render modes (toggle with `g`): the classic colored-glyph map, or a half-block pixel-art "sprite" view — a hero-centred camera with procedural per-archetype sprites, distinct item/feature icons, an idle bob, particles and floating damage, all drawn in any truecolor terminal with no asset files (zoom with `z`)
- Four hero classes (Warrior / Rogue / Mage / Paladin) with distinct weapons, armor, crits, cleave and bolts, plus a class active ability the AI triggers on cooldown (Warrior charge, Rogue blink-strike, Mage ice nova, Paladin smite-heal)
- Elemental synergies: shatter frozen foes for bonus damage, lightning that arcs to a nearby enemy, and poison that spreads between adjacent monsters
- A bestiary/codex (toggle `k`) listing discovered monsters with their element, behavior and depth
- Elemental system (fire / ice / poison / lightning) with resistances, weaknesses and on-hit effects
- Loot rarity and affixes, rings, amulets, scrolls, and class-restricted equipment
- Set bonuses (matching affixes across gear slots grant scaling ATK/DEF/crit) and a rare Forge feature that spends gold to upgrade a gear piece (and can enchant toward completing a set)
- A live panel showing the full loadout — weapon/armor with bonus & affix, ring, amulet, set bonus, potions, scrolls by type, and talents
- Run mutators (Sanguinaire, Cupidite, Fragile, Pullulement, Champions) rolled per run that twist spawns, scaling and rewards
- A persistent lifetime profile (runs, deaths, deepest floor, best score, total kills) shown on the menu, with milestones that unlock permanent starting bonuses (+ATK, +HP, extra potions, a starting talent)
- Monster behaviors beyond melee: ranged casters, healers that mend wounded allies, skirmishers that flee when near death, summoners, exploding kamikazes, and bosses that enrage below half health
- Temporary buffs (rage, ward, regen) with HUD icons, gamble shrines (risk/reward) and blessing fountains
- Familiars come in three flavors — striker, mender (heals you) and guardian — and level up as the hero does
- Level-up talents, persistent relics unlocked through achievements
- Unique floor bosses and a final boss with phase-based movesets — a rotation of telegraphed charge / volley / summon / slam / eruption attacks, lingering hazard tiles the hero must avoid, and an enrage at half health
- Ranged casters that wind up and can be sidestepped
- Curse altars, shrines, fountains, mimic chests, familiars and per-floor events
- Rare merchants, an endless abyss mode with a scoreboard, a bestiary and death cards
- Procedural 8/16-bit chiptune, all generated at runtime with no audio files: synthesized square/triangle/noise SFX for hits, crits, kills, loot, level-ups, boss tells and death, plus an **adaptive** chill-pop backing track — a base groove (seventh-chord progression, soft bass, pad and drums) that layers in a driving arpeggio when enemies are near and a tense theme when a boss is alive, crossfading smoothly with the action
- Optional Twitch chat integration: viewers vote on the hero's mindset and merchant purchases (fully configurable, off by default)

## Config

`abyssal.config.json` is created on first run. It controls the optional Twitch integration and the audio: `sound_enabled`, `ambient_enabled` (the music track), and the `master_volume` / `ambient_volume` levels (0.0–2.0). Sound can also be muted in-game with `a`.
