# Abyssal

*Read this in [English](README.md).*

Un roguelike **entièrement autonome** pour le terminal, à regarder plutôt qu'à jouer, écrit en Rust.

Un héros IA descend seul dans un abîme infini généré procéduralement : il explore, combat, ramasse, monte de niveau, apprend des talents, marchande, esquive les attaques télégraphiées des boss, meurt, et recommence — tout seul. Tu ne joues pas. Tu regardes.

![Abyssal en action](assets/demo.gif)

## Captures

Exploration — un champ de vision éclairé à la torche dans un biome (la palette, le panneau d'équipement complet et le mutateur de run en haut).

![Exploration](assets/01-explore.png)

Combat — dégâts flottants, coups élémentaires et combos contre une faune propre au biome.

![Combat](assets/02-combat.png)

Boss à movesets — les boss enchaînent des attaques télégraphiées (ici une charge) et font clignoter les cases de danger en rouge ; les attaques profondes laissent des zones persistantes à éviter.

![Télégraphe de boss](assets/03-boss.png)

Marchand — un marchand rare apparaît avec un stock tiré au sort ; les viewers peuvent voter l'achat en mode Twitch, et la simulation se met en pause pendant l'échange.

![Marchand](assets/04-merchant.png)

Mort — une carte de mort cohérente avec la cause, le score, les meilleurs scores et une dernière réplique dramatique avant qu'une nouvelle âme descende.

![Écran de mort](assets/05-death.png)

Mode sprite — touche `g` pour une caméra pixel-art en demi-blocs avec des sprites procéduraux (zoom avec `z`) ; tout est dessiné dans le terminal, sans aucun fichier d'asset.

![Mode sprite](assets/06-sprite.png)

Bestiaire — touche `k` pour un codex des monstres découverts avec leur élément, comportement et profondeur.

![Bestiaire](assets/07-codex.png)

## Lancer

```sh
cargo run --release
```

Un menu d'avant-partie permet de choisir une classe, un état d'esprit (complétionniste / combattant / rusher), une difficulté et un trait de départ — ou de tout laisser au hasard et regarder.

## Contrôles

Le jeu se joue tout seul. Les entrées sont optionnelles :

| Touche | Action |
| --- | --- |
| `espace` | pause / reprise |
| `+` / `-` | plus vite / moins vite (lent → ultra) |
| `m` / `1` `2` `3` | changer / fixer l'état d'esprit du héros |
| `a` | couper / réactiver le son |
| `g` | basculer vue sprite (pixel-art demi-blocs) / carte glyphes classique |
| `z` | changer le zoom de la vue sprite |
| `k` | ouvrir / fermer le bestiaire |
| `b` | (debug) faire apparaître un marchand de test |
| `s` / `l` | sauvegarder / charger |
| `n` | nouvelle partie |
| `q` / `echap` | sauvegarder & quitter |

La progression se charge automatiquement au lancement (`abyssal.save.json`).

## Fonctionnalités

- Donjons procéduraux (salles, couloirs), champ de vision en ligne de mire et compteur de découverte
- Descentes ramifiées : à chaque escalier l'IA pèse 2–3 voies selon son état d'esprit (et fonce vers une salle de repos si elle est blessée), chacune menant à un biome et un type de salle (trésor / défi / nuée / repos)
- Cinq biomes (Cavernes, Catacombes, Glacier, Tréfonds, Abîme), chacun avec sa palette, sa teinte d'éclairage, sa faune biaisée, un danger ambiant, un champion thématique et sa tonalité musicale
- Deux modes de rendu (touche `g`) : la carte en glyphes colorés classique, ou une vue « sprite » pixel-art en demi-blocs — caméra centrée sur le héros, sprites procéduraux par archétype, icônes d'objets distinctes, petit balancement, particules et dégâts flottants, le tout dans n'importe quel terminal truecolor sans fichier (zoom avec `z`)
- Huit classes de héros (Guerrier / Voleur / Mage / Paladin / Nécromancien / Rôdeur / Berserker / Élémentaliste), armes/armures/crit/cleave/bolts distincts, plus une capacité active déclenchée par l'IA en cooldown (charge, assaut-éclair, nova de glace, châtiment-soin, levée des morts, volée de flèches, furie tournoyante, nova élémentaire)
- Un bestiaire d'une trentaine de monstres sur toute la profondeur, chacun avec son glyphe, son élément, son comportement et un sprite procédural
- Alliés invoqués : le Nécromancien relève les monstres tués en morts-vivants temporaires qui combattent à ses côtés (système d'unités alliées réutilisable)
- Reliques uniques lâchées par les boss et champions, à effets spéciaux (vol de vie aux kills, esquive spectrale, éclairs en chaîne, coups enflammés, +PV max, levée des morts pour toute classe)
- Ambiance : phrases de lore par biome, intros de boss, et particules d'ambiance (braises, neige, brume) qui flottent dans la zone éclairée
- Synergies élémentaires : « shatter » des ennemis gelés pour des dégâts bonus, foudre qui rebondit sur un ennemi proche, poison qui se propage entre monstres adjacents
- Système élémentaire (feu / glace / poison / foudre) avec faiblesses offensives, effets au contact, et affinités défensives de l'armure (l'élément de ton équipement résiste à l'élément correspondant et est faible à son opposé)
- Codex / bestiaire (touche `k`) listant les monstres découverts avec leur élément, comportement et profondeur
- Ascension / NG+ : atteindre des étages profonds monte un palier d'ascension permanent, qui empile le scaling ennemi et un multiplicateur de score sur les runs suivantes
- Rareté de butin et affixes, anneaux, amulettes, parchemins, équipement restreint par classe
- Bonus de set (affixes identiques sur plusieurs emplacements → ATQ/DEF/crit qui scalent) et une Forge rare qui dépense de l'or pour améliorer une pièce (et peut enchanter vers un set)
- Un panneau en direct montrant tout l'équipement — arme/armure avec bonus & affixe, anneau, amulette, bonus de set, potions, parchemins par type, et talents
- Mutateurs de run (Sanguinaire, Cupidité, Fragile, Pullulement, Champions) tirés à chaque run, qui modifient spawns, scaling et récompenses
- Un profil persistant à vie (runs, morts, étage max, meilleur score, kills, ascension) affiché au menu, avec des jalons qui débloquent des bonus de départ permanents (+ATQ, +PV, potions, talent de départ)
- Comportements de monstres au-delà du corps-à-corps : lanceurs à distance, soigneurs qui rapiècent leurs alliés, fuyards quand ils sont presque morts, invocateurs, kamikazes explosifs, et boss qui s'enragent sous la moitié de leurs PV
- Buffs temporaires (rage, bouclier, régén) avec icônes, sanctuaires de pari (risque/récompense) et fontaines de bénédiction
- Familiers de trois types — fauve, esprit (te soigne) et golem — qui montent en niveau avec le héros
- Talents au niveau supérieur, reliques persistantes débloquées par succès
- Boss d'étage uniques et un boss final à movesets par phases — rotation d'attaques télégraphiées (charge / salve / invocation / fracas / éruption), zones de danger persistantes à éviter, et enrage à mi-vie
- Effet de PV bas : vignettage rouge pulsé et battement de cœur qui accélère quand la vie chute
- Autels de malédiction, sanctuaires, fontaines, coffres mimic, familiers et événements d'étage
- Marchands rares, mode abîme infini avec tableau des scores, bestiaire et cartes de mort
- Chiptune 8/16-bit procédural, entièrement généré au runtime sans fichier audio : SFX synthétisés (carré/triangle/bruit) pour les coups, critiques, kills, butin, niveaux, télégraphes de boss et mort, plus une musique de fond chill-pop **adaptative** — une base (progression d'accords de septième, basse douce, nappe et batterie) qui ajoute un arpège quand des ennemis approchent et un thème tendu quand un boss est en vie, en fondu avec l'action
- Intégration Twitch optionnelle : les viewers votent l'état d'esprit du héros et les achats au marchand (entièrement configurable, désactivée par défaut)

## Config

`abyssal.config.json` est créé au premier lancement. Champs :

- `sound_enabled` / `ambient_enabled` — SFX et piste musicale activés/désactivés
- `master_volume` / `ambient_volume` — niveaux, 0.0–2.0 (le son se coupe aussi en jeu avec `a`)
- `twitch_enabled`, `twitch_channel`, `vote_window_secs`, `allow_style_vote`, `allow_speed_vote`, `allow_merchant_vote` — l'intégration Twitch optionnelle (désactivée par défaut)

## Sauvegardes & fichiers

- `abyssal.save.json` — la run en cours ; chargée au lancement, écrite à la sauvegarde (`s`) et à la sortie
- `abyssal.profile.json` — le profil persistant à vie (runs, morts, étage/score max, kills, palier d'ascension) qui pilote les déblocages méta
- `abyssal.config.json` — la config ci-dessus

Les trois sont à côté du binaire et ignorés par git.

## Intégration Twitch

Avec `twitch_enabled` activé, le jeu se connecte anonymement (lecture seule, sans token) au chat de `twitch_channel` et les viewers influencent la run :

- `!1` / `!2` / `!3` — votent l'état d'esprit du héros (complétionniste / combattant / rusher)
- `!arme` / `!armure` / `!potion` / `!soin` / `!reroll` / `!purge` — votent l'achat quand un marchand est présent
- `!faster` / `!slower` — ajustent la vitesse (si `allow_speed_vote`)

Les votes sont comptés sur `vote_window_secs` ; chaque viewer compte une fois par fenêtre.

## Comment ça marche

Tout est généré et rendu au runtime — aucun fichier d'art, d'audio ou de données.

- `map.rs` — génération procédurale (salles + couloirs), FOV en ligne de mire (Bresenham), mesure de découverte
- `ai.rs` — pathfinding BFS (`step_toward`, `nearest_goal`)
- `entity.rs` — héros, classes, monstres (bestiaire), objets, affixes, reliques, talents, familiers/alliés, éléments
- `game.rs` — la simulation : ordre des tours, l'IA du héros par priorités (esquive → soin → capacité → éclair → parchemin → attaque → chasse/butin/feature/marchand/exploration/descente), combat, biomes, ramifications, mutateurs, boss
- `render.rs` — rendu ANSI truecolor manuel : tuiles éclairées avec falloff de torche + teinte par biome, le panneau encadré, le rendu sprite en demi-blocs, les overlays
- `fx.rs` — texte flottant, particules, projectiles, secousses d'écran, combos, transitions
- `audio.rs` — un mini synthé chiptune (carré/triangle/sinus/bruit + ADSR) qui alimente `rodio` ; SFX et musique adaptative en couches calculés en échantillons bruts
- `profile.rs` / `config.rs` / `twitch.rs` / `rng.rs` — persistance, config, lecteur IRC Twitch anonyme, PRNG xorshift

## Licence / crédits

Projet personnel de [CatAnnaDev](https://github.com/CatAnnaDev). Fait en Rust avec `crossterm`, `rodio` et `serde`.
