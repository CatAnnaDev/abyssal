# Architecture & ajout de contenu

Le contenu du jeu est piloté par des **tables de données** : ajouter une classe, un
biome, une difficulté ou un son se fait en quelques lignes, à un seul endroit, sans
toucher à une dizaine de `match`.

## Modules

- `entity.rs` — héros, classes, monstres (bestiaire), objets, affixes, reliques, talents, compagnons, éléments. Contient les tables `CLASSES` et `BESTIARY`.
- `game.rs` — la simulation : ordre des tours, IA du héros, combat, biomes, ramifications, mutateurs, boss. Contient les tables `BIOMES` et `DIFFICULTIES`.
- `map.rs` — génération de monde (algorithmes par style), FOV, décor.
- `render.rs` — rendu ANSI/sprite. Lit les données des biomes via `Biome::palette()`.
- `audio.rs` — synthé chiptune ; enum `Sound` + `render()` + registre `Sound::ALL`.
- `ai.rs` / `fx.rs` / `profile.rs` / `config.rs` / `twitch.rs` / `rng.rs` — pathfinding, effets, persistance, config, chat, PRNG.

## Ajouter une classe

1. Ajoute une variante à `enum HeroClass` (`entity.rs`) **à la fin** (l'ordre doit correspondre à la table).
2. Ajoute la variante à `HeroClass::ALL`.
3. Ajoute une ligne à la table `CLASSES` (même ordre) : label, crit, cleave, bolt, bleeds, raises, arme, armure, bonus PV/ATQ/DEF, et une `Ability`.
4. Si tu veux une nouvelle capacité active : ajoute une variante à `enum Ability`, un bras dans `act_ability()` (`game.rs`) et la fonction `ability_xxx`.

Le menu (`main.rs`) lit `CLASSES` automatiquement : rien d'autre à faire.

## Ajouter un biome

1. Ajoute une variante à `enum Biome` (`game.rs`) **à la fin**.
2. Ajoute une ligne à la table `BIOMES` (même ordre) : label, teinte, élément, `map_style` (0-4, voir `map.rs`), `music_style` (0-4, voir `STYLES` dans `audio.rs`), faune, lore, champion, poids d'apparition (`weight_peak`/`weight_center`/`weight_min`), palette, particule d'ambiance.

`roll_biome`, le rendu, la musique et les particules suivent automatiquement.

## Ajouter un mode (état d'esprit)

1. Ajoute une variante à `enum Playstyle` (`game.rs`) et à `Playstyle::ALL`.
2. Ajoute son `label`, un bras dans le dispatch de `hero_turn` pointant vers une `fn turn_xxx` (compose les helpers `act_hunt`/`act_loot`/`act_feature`/`act_merchant`/`act_explore`/`act_to_stairs`), et un bras dans `room_appeal`.
3. Ajoute `("Nom", Playstyle::Xxx)` à `M_MODES` (`main.rs`).

## Ajouter une difficulté

Ajoute une ligne `(label, multiplicateur)` à `DIFFICULTIES` (`game.rs`). Le menu la propose aussitôt.

## Ajouter un son

1. Ajoute une variante à `enum Sound` (`audio.rs`).
2. Ajoute son bras dans `render()` (le programme de synthèse).
3. Ajoute `("nom", Sound::Xxx)` à `Sound::ALL` (utilisé par l'aperçu/dump).

## Ajouter un style de carte

Ajoute un bras au `match style` de `Map::generate_styled` (`map.rs`) et, au besoin, une
palette de décor dans `scatter_decor`.

## Déterminisme

Le PRNG (`rng.rs`) est déterministe : une même graine reproduit une run identique. Garde
l'**ordre** des appels au RNG inchangé lors d'un refactor (l'ordre des entrées dans `BIOMES`
influence `roll_biome`, etc.). Le test `long_autoplay_is_stable` vérifie 40 000 tours sans
panique ; lance-le après toute modification de gameplay.
