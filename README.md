# Abyssal

*Read this in [Français](README.fr.md).*

A fully autonomous, watch-only roguelike for the terminal, written in Rust.

An AI hero descends alone into an endless procedurally generated abyss: it explores, fights, loots, levels up, learns talents, trades with rare merchants, dodges telegraphed boss attacks, dies, and starts again — all on its own. You don't play. You watch.

![Abyssal in action](assets/demo.gif)

## Screenshots

Exploration — a torch-lit field of view across a biome (note the palette, the full equipment panel and the active run mutator in the top bar).

![Exploration](assets/01-explore.png)

Combat — floating damage, elemental hits and kill combos against biome-flavored fauna.

![Combat](assets/02-combat.png)

Boss movesets — bosses cycle telegraphed attacks (here a charge) and flash the danger tiles red; deeper moves leave lingering hazards the hero must avoid.

![Boss telegraph](assets/03-boss.png)

Merchant — a rare trader appears with rolled stock; viewers can vote on the purchase when Twitch mode is on, and the simulation pauses during the deal.

![Merchant](assets/04-merchant.png)

Death — a cause-coherent death card with score, best scores, and a fittingly dramatic last word before a new soul descends.

![Death screen](assets/05-death.png)

Sprite mode — toggle `g` for a half-block pixel-art camera with procedural sprites (zoom with `z`); everything is still drawn in the terminal, no asset files.

![Sprite mode](assets/06-sprite.png)

Bestiary — toggle `k` for a codex of discovered monsters with their element, behavior and depth.

![Bestiary](assets/07-codex.png)

## Run

```sh
cargo run --release
```

A roomy pre-game menu lets you configure the run across ten axes — class, playstyle (completionist / fighter / rusher / looter / cautious / hunter), difficulty, starting boon, variant (Normal or Boss Rush), mutators (random / none / guaranteed), starting familiar, simulation speed, display (glyphs or sprites) and sound — each with an inline hint. Or just leave it all on the defaults / random and watch.

## Controls

The game plays itself. Input is optional:

| Key | Action |
| --- | --- |
| `space` | pause / resume |
| `+` / `-` | faster / slower (lent → ultra) |
| `m` / `1` `2` `3` | cycle / set the hero's mindset |
| `a` | mute / unmute sound |
| `g` | toggle sprite view (half-block pixel-art) / classic glyph map |
| `z` | cycle sprite-view zoom |
| `k` | open / close the bestiary codex |
| `b` | (debug) spawn a test merchant |
| `s` / `l` | save / load |
| `n` | new run |
| `q` / `esc` | save & quit |

Progress autoloads on launch (`abyssal.save.json`).

## Features

- Advanced procedural world generation (v2): each biome uses its own algorithm — cellular-automata caves (Caverns), dense BSP-style crypts (Catacombs), wide great halls (Frostvault), rooms fused with organic cave blobs (Emberdepths), and a loop-heavy labyrinth (Abyss) — with round rooms, pillared halls, guaranteed full connectivity, extra loops, and stairs placed at the farthest reachable point for pacing
- Branching descents: at every stairway the AI weighs 2–3 paths by mindset (and heads for a Rest branch when wounded), each leading to a biome and a room type (treasure / challenge / warren / rest)
- Rift rooms — a rare parallel-world floor (a purple-tinted "Faille") packed with elites and loot: more monsters with a sharply higher elite rate, extra items and chests, and a guaranteed relic on entry; high risk, high reward
- Ten biomes (Caverns, Catacombs, Frostvault, Emberdepths, Abyss, Fungal Gardens, Ruined Forge, Sunken Sanctuary, Obsidian Hive, Caldera), each with its own palette, lighting tint, biased fauna, ambient hazard, a themed champion mini-boss, a cosmetic terrain-decor layer (moss, water, bones, cracks, ice, embers, void motes) and musical key
- Two render modes (toggle with `g`): the classic colored-glyph map, or a half-block pixel-art "sprite" view — a hero-centred camera with procedural per-archetype sprites, distinct item/feature icons, an idle bob, particles and floating damage, all drawn in any truecolor terminal with no asset files (zoom with `z`)
- Seventeen hero classes (Warrior / Rogue / Mage / Paladin / Necromancer / Ranger / Berserker / Elementalist / Monk / Druid / Templar / Warlock / Shaman / Valkyrie / Spellblade / Sentinel / Reaper) with distinct weapons, armor, crits, cleave and bolts, plus a class active ability the AI triggers on cooldown
- Six AI playstyles (modes) that reshape how the hero behaves: completionist (explore everything), fighter (seek combat), rusher (dive for stairs), looter (grab treasure, dodge fights), cautious (heal, disengage, retreat to stairs when threatened) and hunter (chase every monster, race boss to boss)
- Boss Rush variant (menu option): floors 1-9 are normal so you can gear up, then from floor 10 on every single floor is guarded by a boss — a relentless back-to-back gauntlet with trash kept to a minimum
- Eight talents rolled on level-up — berserk crit, lifesteal, +max HP, cleave, chain lightning, regen, scout (+vision) and steel skin (-incoming damage)
- Five weapon families (light / heavy / staff / martial fists / bows) and four armor families (cloth / leather / plate / mail), each with six (weapons) or five (armor) tiers from starter gear up to legendary endgame pieces
- Rare lost human companions: every so often a survivor is found stranded deep in the dungeon and joins you — a named, persistent ally with a role (guard / huntress / medic), distinct behavior (tanking, ranged shots, healing you), who follows across floors, levels up alongside the hero, and fights at your side until it falls (up to two at once)
- Combat depth: weapon procs (bleeding wounds, armor-sundering hits) on top of the elemental affixes, plus universal crit-bleed and low-HP execute finishers
- Rift world-bosses: the rare parallel-world floors are stalked by an over-leveled "Guardian of the Rift" on top of the elite swarm
- A bestiary of ~42 monster kinds across the depth range, each with its glyph, element, behavior and a procedural sprite
- Summoned allies: the Necromancer raises slain monsters as temporary undead that fight alongside the hero (a reusable allied-unit system)
- Unique relics dropped by bosses and champions with special effects (lifesteal on kill, ghostly dodge, chain-lightning procs, burning hits, +max HP, raise-the-dead for any class, low-HP frenzy damage, greed for extra gold/potions)
- Rare single-use ancient relics found as loot: the Ancient Eye (dispels the whole floor's fog of war at once, through walls and distance), the Hourglass (freezes every non-boss monster on the floor) and the Chalice of Life (full heal, cleanse, and +12 permanent max HP)
- Four scroll types the AI reads situationally: fireball (AoE), freeze (crowd control), teleport (escape) and chain lightning (arcs between several foes)
- Atmosphere: per-biome lore lines, boss intros, and ambient particles (embers, snow, fog) drifting through the lit area
- Elemental synergies: shatter frozen foes for bonus damage, lightning that arcs to a nearby enemy, and poison that spreads between adjacent monsters
- Elemental system (fire / ice / poison / lightning) with offensive weaknesses, on-hit effects, and defensive armor affinities (your gear's element resists matching attacks and is weak to its opposite)
- A bestiary/codex (toggle `k`) listing discovered monsters with their element, behavior and depth
- Ascension / NG+: reaching deeper floors permanently raises your ascension tier, stacking enemy scaling and a score multiplier on future runs
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

## Possible builds

A "build" is the combination of everything you can set up and grow over a run. Counting the independent axes:

| Axis | Options |
| --- | --- |
| Class | 17 |
| Playstyle (mode) | 6 |
| Difficulty | 4 |
| Starting boon | 4 |
| Variant (Normal / Boss Rush) | 2 |
| Weapon affix (incl. none) | 9 |
| Armor affix (incl. none) | 3 |
| Ring affix (incl. none) | 9 |
| Amulet affix (incl. none) | 3 |
| Talent combinations (8 talents, one of each max) | 2⁸ = 256 |
| Relic combinations (8 relics) | 2⁸ = 256 |

Pre-game setup alone: 17 × 6 × 4 × 4 × 2 = **3,264** starting configurations.
In-run build identity (affixes × talents × relics): 9 × 3 × 9 × 3 × 256 × 256 ≈ **47.8 million**.

Multiplied together that is roughly **156 billion** theoretical builds (≈ 1.56 × 10¹¹) — and that is *before* counting the 5 weapon families × 6 tiers, 4 armor families × 5 tiers, 3 familiar types and 3 companion roles, which push the real variety far higher.

## Config

`abyssal.config.json` is created on first run. Fields:

- `sound_enabled` / `ambient_enabled` — SFX and the music track on/off
- `master_volume` / `ambient_volume` — levels, 0.0–2.0 (sound also toggles in-game with `a`)
- `twitch_enabled`, `twitch_channel`, `vote_window_secs`, `allow_style_vote`, `allow_speed_vote`, `allow_merchant_vote` — the optional Twitch integration (off by default)

## Saves & files

- `abyssal.save.json` — the current run; autoloads on launch, written on save (`s`) and quit. Boss Rush runs are never saved — it is all-or-nothing, so quitting one abandons it and it can't be continued
- `abyssal.profile.json` — the persistent lifetime profile (runs, deaths, best floor/score, total kills, ascension tier) that drives meta unlocks
- `abyssal.config.json` — the config above

All three live next to the binary and are git-ignored.

## Twitch integration

With `twitch_enabled` on, the game connects anonymously (read-only, no token) to `twitch_channel`'s chat and viewers can influence the run:

- `!1` / `!2` / `!3` — vote the hero's mindset (completionist / fighter / rusher)
- `!arme` / `!armure` / `!potion` / `!soin` / `!reroll` / `!purge` — vote the merchant purchase when a trader is up
- `!faster` / `!slower` — nudge the speed (if `allow_speed_vote`)

Votes are tallied over `vote_window_secs`; each viewer counts once per window.

## How it works

Everything is generated and rendered at runtime — no art, audio, or data files.

- `map.rs` — world generation v2: per-biome algorithms (cellular-automata caves, BSP-style rooms, great halls, room+cave hybrids, loop labyrinths), connectivity flood-fill, round rooms, pillars, Bresenham line-of-sight FOV, discovery metering
- `ai.rs` — BFS pathfinding (`step_toward`, `nearest_goal`)
- `entity.rs` — hero, classes, monsters (bestiary), items, affixes, relics, talents, pets/allies, elements
- `game.rs` — the simulation: turn order, the hero's priority-based AI (dodge → heal → ability → bolt → scroll → attack → hunt/loot/feature/merchant/explore/descend), combat, biomes, branching, mutators, bosses
- `render.rs` — manual ANSI truecolor rendering: lit tiles with torch falloff + per-biome tint, the framed panel, the half-block sprite renderer, overlays
- `fx.rs` — floating text, particles, projectiles, screen shake, combos, transitions
- `audio.rs` — a tiny chiptune synth (square/triangle/sine/noise + ADSR) feeding `rodio`; SFX and the layered adaptive music are computed as raw samples
- `profile.rs` / `config.rs` / `twitch.rs` / `rng.rs` — persistence, config, anonymous Twitch IRC reader, xorshift PRNG

## Extending the game

Content is data-driven: classes, biomes, difficulties and sounds each live in a single table, so adding one is a few lines in one place. See [ARCHITECTURE.md](ARCHITECTURE.md) for the step-by-step.

## License / credits

A personal project by [CatAnnaDev](https://github.com/CatAnnaDev). Built in Rust with `crossterm`, `rodio`, and `serde`.
