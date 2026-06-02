# Abyssal

A fully autonomous, watch-only roguelike for the terminal, written in Rust.

An AI hero descends alone into an endless procedurally generated abyss: it explores, fights, loots, levels up, learns talents, trades with rare merchants, dodges telegraphed boss attacks, dies, and starts again — all on its own. You don't play. You watch.

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
| `s` / `l` | save / load |
| `n` | new run |
| `q` / `esc` | save & quit |

Progress autoloads on launch (`abyssal.save.json`).

## Features

- Procedural dungeons with rooms, corridors, LOS field-of-view and a discovery meter
- Three hero classes (Warrior / Rogue / Mage) with distinct weapons, armor, crits, cleave and bolts
- Elemental system (fire / ice / poison / lightning) with resistances, weaknesses and on-hit effects
- Loot rarity and affixes, rings, amulets, scrolls, and class-restricted equipment
- Level-up talents, persistent relics unlocked through achievements
- Unique floor bosses and a final boss, each telegraphing charge / volley / summon attacks the hero can dodge
- Ranged casters that wind up and can be sidestepped
- Curse altars, shrines, fountains, mimic chests, familiars and per-floor events
- Rare merchants, an endless abyss mode with a scoreboard, a bestiary and death cards
- Optional Twitch chat integration: viewers vote on the hero's mindset and merchant purchases (fully configurable, off by default)

## Config

`abyssal.config.json` is created on first run and controls the optional Twitch integration.
