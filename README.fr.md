# Abyssal

*Read this in [English](README.md).*

> **TL;DR** — *Abyssal* est un roguelike **entièrement autonome**, à **regarder** plutôt qu'à jouer, écrit en Rust et **sans aucun fichier d'asset** : chaque carte, sprite, son et note est généré au runtime. Une héroïne IA descend seule un abîme infini généré procéduralement : elle explore, combat, ramasse, monte de niveau, meurt et recommence. Tu ne joues pas, tu regardes.
>
> **Ce que ça intègre :** 11 biomes avec génération propre à chacun · 20 classes avec aptitudes actives, équipement, reliques & talents · combat élémentaire contre ~45 types de monstres et des boss à phases télégraphiées · effets cinématiques avec hit-stop · deux modes de rendu (glyphes colorés ou sprites pixel-art procéduraux) · héroïnes vivantes (noms, traits, pensées, épitaphes, némésis inter-runs & tombes de fantômes) · méta-progression (ascension/NG+, hauts faits, défi du jour + classement local) · chiptune généré au runtime avec musique adaptative · intégration **Twitch** anonyme optionnelle (votes état d'esprit & marchand, mobs nommés par les viewers, paris sur la mort, commandes chaos) et un overlay **OBS** auto-rafraîchi.

Un roguelike **entièrement autonome** pour le terminal, à regarder plutôt qu'à jouer, écrit en Rust.

Une héroïne IA descend seule dans un abîme infini généré procéduralement : elle explore, combat, ramasse, monte de niveau, apprend des talents, marchande, esquive les attaques télégraphiées des boss, meurt, et recommence — toute seule. Tu ne joues pas. Tu regardes.

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

Un grand menu d'avant-partie permet de configurer la run sur dix axes — classe, état d'esprit (complétionniste / combattant / rusher / pilleur / prudent / traqueur), difficulté, trait de départ, variante (Normal ou Boss Rush), mutateurs (aléatoire / aucun / garanti), familier de départ, vitesse de simulation, affichage (glyphes ou sprites) et son — chacun avec une description de ce qu'il apporte ou retire. Ou laisse tout par défaut / au hasard et regarde.

## Contrôles

Le jeu se joue tout seul. Les entrées sont optionnelles :

| Touche | Action |
| --- | --- |
| `espace` | pause / reprise |
| `+` / `-` | plus vite / moins vite (lent → ultra) |
| `m` / `1` `2` `3` | changer / fixer l'état d'esprit de l'héroïne |
| `a` | couper / réactiver le son |
| `o` | ouvrir le menu d'options en jeu (met le jeu en pause) |
| `g` | basculer vue sprite (pixel-art demi-blocs) / carte glyphes classique |
| `z` | changer le zoom de la vue sprite |
| `k` | ouvrir / fermer le bestiaire |
| `h` | ouvrir / fermer le Hall des Âmes (héroïnes passées + némésis actives) |
| `d` | lancer le **défi du jour** (même donjon pour tout le monde ce jour-là) |
| `b` | (debug) faire apparaître un marchand de test |
| `s` / `l` | sauvegarder / charger |
| `n` | nouvelle partie |
| `q` / `echap` | sauvegarder & quitter |

La progression se charge automatiquement au lancement (`abyssal.save.json`).

## Fonctionnalités

### Monde & exploration
- **Génération de monde v2** — chaque biome a son propre algorithme : grottes en automate cellulaire, cryptes BSP denses, grandes halles, hybrides salles+grottes et labyrinthes à boucles, avec salles rondes, halls à piliers, connectivité garantie et escalier placé loin pour le rythme.
- **Onze biomes** (Cavernes, Catacombes, Glacier, Tréfonds, Abîme, Jardins Fongiques, Forge en Ruine, Sanctuaire Englouti, Ruche d'Obsidienne, Caldeira, Galerie Cristalline) — chacun avec palette, teinte, faune, danger ambiant, champion, couche de décor au sol et tonalité musicale.
- **Descentes ramifiées** — à chaque escalier l'IA pèse 2–3 voies (trésor / défi / nuée / repos), et fonce vers une salle de repos si elle est blessée.
- **Salles Faille** — étages « monde parallèle » violets, gorgés d'élites et de butin, relique garantie à l'entrée et un « Gardien de la Faille » surpuissant.
- **Features & événements** — autels de malédiction, sanctuaires, fontaines, coffres mimic, une Forge rare, sanctuaires de pari, et événements d'étage.
- **Deux modes de rendu** (`g`) — glyphes colorés classiques, ou une caméra sprite pixel-art en demi-blocs avec des sprites procéduraux 8×8 détaillés (contours + ombrage), particules et dégâts flottants, et une large plage de zoom (du dézoom large au gros plan, touche `z`).

### Héroïne, classes & équipement
- **Vingt classes** (Guerrière, Voleuse, Mage, Paladine, Nécromancienne, Rôdeuse, Berserker, Élémentaliste, Moine, Druidesse, Templière, Occultiste, Chamane, Valkyrie, Lame-Sort, Sentinelle, Faucheuse, Spectre, Maelström, Liche) — chacune avec arme/armure, crit, cleave/bolts et une capacité en cooldown.
- **Capacités actives** — charge, assaut-éclair, nova de glace/élémentaire, châtiment-soin, levée des morts, volée de flèches, furie, plus les **spectrales** : Vortex (attire à toi tous les monstres proches), Possession (transforme un monstre en allié) et Phase (téléportation à travers les murs).
- **Équipement** — 5 familles d'armes (légère / lourde / magique / poings / arcs) et 4 d'armures (tissu / cuir / plaque / mailles), 5–6 paliers chacune ; anneaux, amulettes, parchemins, rareté et affixes, bonus de set, et un panneau d'équipement en direct.
- **Reliques** — drops uniques (vol de vie, esquive spectrale, chaîne d'éclairs, coups enflammés, +PV max, levée des morts, furie à bas PV, cupidité) et reliques antiques à usage unique (Œil Antique révèle tout l'étage à travers les murs, Sablier fige les non-boss, Calice de Vie soigne et donne +PV max).
- **Huit talents** de montée de niveau — crit berserk, vol de vie, +PV max, cleave, chaîne d'éclairs, régén, éclaireur (+vision), peau d'acier (−dégâts subis).

### Combat
- **Combat cinématique** — un hit-stop fige brièvement l'action sur critiques, kills et coups de boss pour qu'on voie les effets même à haute vitesse ; boss et élites coriaces pour des affrontements plus longs et épiques.
- **Profondeur** — procs d'arme (saignement, brise-garde) en plus des affixes élémentaires, saignement universel sur critique et exécution des ennemis à bas PV.
- **Système élémentaire** (feu / glace / poison / foudre) — faiblesses offensives, effets au contact, résistances d'armure, et synergies (shatter des gelés, foudre qui rebondit, poison qui se propage).
- **~45 monstres** aux comportements variés (lanceurs, soigneurs, fuyards, invocateurs, kamikazes, boss qui s'enragent) et un codex (`k`) ; boss d'étage et boss final à movesets télégraphiés par phases.

### Alliés
- **Compagnons humains perdus** — survivants rares qui vous rejoignent avec un rôle (garde / archère / guérisseur), vous suivent d'étage en étage, montent en niveau et combattent (jusqu'à deux).
- **Familiers** — fauve, esprit (te soigne) ou golem, qui montent en niveau avec l'héroïne.
- **Alliés invoqués** — la Nécromancienne/la Liche relèvent ou asservissent les ennemis en combattants temporaires.

### Modes & méta
- **Six états d'esprit** — complétionniste, combattant, rusher, pilleur, prudent, traqueur — chacun changeant le comportement de l'héroïne.
- **Variante Boss Rush** — étages 1–9 pour se stuffer, puis l'étage 10 devient une arène sans fin : un boss plus fort surgit dès qu'un meurt (pas de descente, pas de sauvegarde), un compteur de vagues pilotant difficulté et score.
- **Mutateurs de run** (Sanguinaire, Cupidité, Fragile, Pullulement, Champions, Titans, Soif de Sang, Frénésie) qui modifient spawns, scaling et récompenses.
- **Ascension / NG+** et un profil persistant à vie avec des jalons qui débloquent des bonus de départ permanents.

### Audio
- **Chiptune 8/16-bit procédural**, généré au runtime sans fichier audio : SFX synthétisés, plus une musique chill-pop **adaptative** qui réagit en direct — le **tempo monte progressivement quand les ennemis approchent** et un arpège entraînant se fond dedans, culminant en combat et contre les boss, le tout en jouant sur le tempo et le layering plutôt qu'en coupant les pistes.

### Twitch (optionnel)
- Les viewers votent l'état d'esprit de l'héroïne et les achats au marchand, affichés dans un **panneau** en direct (canal, barres de votes, top du chat, fil des actions récentes).
- **Les pseudos des viewers apparaissent sur les mobs** — un chatteur actif « adopte » un monstre dont le nom flotte au-dessus de lui sur la carte.
- Le **marchand affiche un appel clair « VOTEZ MAINTENANT »** avec un décompte pendant la fenêtre de vote.
- Toutes les options Twitch sont réglables dans le menu d'options en jeu.

## Builds possibles

Un « build », c'est la combinaison de tout ce que tu peux configurer puis faire évoluer pendant une run. En comptant les axes indépendants :

| Axe | Options |
| --- | --- |
| Classe | 20 |
| État d'esprit (mode) | 6 |
| Difficulté | 4 |
| Trait de départ | 4 |
| Variante (Normal / Boss Rush) | 2 |
| Affixe d'arme (aucun inclus) | 9 |
| Affixe d'armure (aucun inclus) | 3 |
| Affixe d'anneau (aucun inclus) | 9 |
| Affixe d'amulette (aucun inclus) | 3 |
| Combinaisons de talents (8 talents, 1 de chaque max) | 2⁸ = 256 |
| Combinaisons de reliques (8 reliques) | 2⁸ = 256 |

Configuration d'avant-partie seule : 17 × 6 × 4 × 4 × 2 = **3 264** départs possibles.
Identité de build en run (affixes × talents × reliques) : 9 × 3 × 9 × 3 × 256 × 256 ≈ **47,8 millions**.

Le tout multiplié donne environ **156 milliards** de builds théoriques (≈ 1,56 × 10¹¹) — et c'est *avant* de compter les 5 familles d'armes × 6 paliers, 4 familles d'armures × 5 paliers, 3 types de familiers et 3 rôles de compagnons, qui font exploser la variété réelle bien plus haut.

## Config

`abyssal.config.json` est créé au premier lancement. Champs :

- `sound_enabled` / `ambient_enabled` — SFX et piste musicale activés/désactivés
- `master_volume` / `ambient_volume` — niveaux SFX et musique, 0.0–2.0 (réglables aussi dans le menu d'options en jeu `o`)
- `music_preset` — style de musique : 0 = Auto (par biome), ou un preset fixe (Chill / Énergique / Sombre / Rétro 8-bit / Mystique)
- `pathfinder` — algorithme de navigation de l'héroïne : 0 BFS, 1 A*, 2 Dijkstra (pondéré, évite le danger), 3 Greedy, 4 Diagonale (8 directions), 5 JPS (jump point search)
- `twitch_enabled`, `twitch_channel`, `vote_window_secs`, `allow_style_vote`, `allow_speed_vote`, `allow_merchant_vote`, `allow_chaos_vote` — l'intégration Twitch optionnelle
- `allow_bet_vote` — laisse les viewers `!bet <etage>` pronostiquer la profondeur de mort de l'héroïne
- `obs_overlay` — si activé, écrit un `abyssal.obs.html` auto-rafraîchi (fond transparent, grosses polices) ~4×/s ; à ajouter comme **source Navigateur** OBS (fichier local) pour un overlay de stream propre à côté de la capture du terminal. La carte montre les stats de l'héroïne, sa pensée du moment, les derniers événements, et une ligne Twitch (canal, pool de paris, top chat, dernier résultat de pronostic)

## Sauvegardes & fichiers

- `abyssal.save.json` — la run en cours ; chargée au lancement, écrite à la sauvegarde (`s`) et à la sortie. Les runs Boss Rush ne sont jamais sauvegardées — c'est tout ou rien, quitter abandonne la run et on ne peut pas la continuer
- `abyssal.profile.json` — le profil persistant à vie (runs, morts, étage/score max, kills, palier d'ascension) qui pilote les déblocages méta
- `abyssal.config.json` — la config ci-dessus

Les trois sont à côté du binaire et ignorés par git.

## Intégration Twitch

Avec `twitch_enabled` activé, le jeu se connecte anonymement (lecture seule, sans token) au chat de `twitch_channel` et les viewers influencent la run :

- `!1` / `!2` / `!3` — votent l'état d'esprit de l'héroïne (complétionniste / combattant / rusher)
- `!arme` / `!armure` / `!potion` / `!soin` / `!reroll` / `!purge` — votent l'achat quand un marchand est présent
- `!faster` / `!slower` — ajustent la vitesse (si `allow_speed_vote`)
- `!bless` / `!curse` — bénissent ou maudissent l'héroïne (petit buff / debuff aléatoire, cooldown partagé ; si `allow_chaos_vote`)
- `!name <x>` — rebaptisent l'héroïne (si `allow_chaos_vote`)
- `!bet <etage>` — pronostiquent l'étage où l'héroïne meurt ; à la mort, les plus proches gagnent, annoncés dans le panneau Twitch et l'overlay OBS (si `allow_bet_vote`)

Les votes sont comptés sur `vote_window_secs` ; chaque viewer compte une fois par fenêtre. Les commandes chaos (`bless`/`curse`/`name`) sont limitées en fréquence pour éviter le spam. Le pool de pronostics est remis à zéro au début de chaque run.

## Comment ça marche

Tout est généré et rendu au runtime — aucun fichier d'art, d'audio ou de données.

- `map.rs` — génération de monde v2 : algorithmes par biome (grottes en automate cellulaire, salles façon BSP, grandes halles, hybrides salles+grottes, labyrinthes à boucles), flood-fill de connectivité, salles rondes, piliers, FOV en ligne de mire (Bresenham), mesure de découverte
- `ai.rs` — pathfinding BFS (`step_toward`, `nearest_goal`)
- `entity.rs` — héroïne, classes, monstres (bestiaire), objets, affixes, reliques, talents, familiers/alliés, éléments
- `game.rs` — la simulation : ordre des tours, l'IA de l'héroïne par priorités (esquive → soin → capacité → éclair → parchemin → attaque → chasse/butin/feature/marchand/exploration/descente), combat, biomes, ramifications, mutateurs, boss
- `render.rs` — rendu ANSI truecolor manuel : tuiles éclairées avec falloff de torche + teinte par biome, le panneau encadré, le rendu sprite en demi-blocs, les overlays
- `fx.rs` — texte flottant, particules, projectiles, secousses d'écran, combos, transitions
- `audio.rs` — un mini synthé chiptune (carré/triangle/sinus/bruit + ADSR) qui alimente `rodio` ; SFX et musique adaptative en couches calculés en échantillons bruts
- `profile.rs` / `config.rs` / `twitch.rs` / `rng.rs` — persistance, config, lecteur IRC Twitch anonyme, PRNG xorshift

## Étendre le jeu

Le contenu est piloté par des données : classes, biomes, difficultés et sons vivent chacun dans une table unique, donc en ajouter un se fait en quelques lignes au même endroit. Voir [ARCHITECTURE.md](ARCHITECTURE.md) pour le pas-à-pas.

## Licence / crédits

Projet personnel de [CatAnnaDev](https://github.com/CatAnnaDev). Fait en Rust avec `crossterm`, `rodio` et `serde`.
