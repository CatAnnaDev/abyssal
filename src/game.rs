use crate::ai::{nearest_goal, step_toward};
use crate::audio::Sound;
use crate::entity::{Affix, Ally, Color, Element, Feature, FeatureKind, Hero, HeroClass, Item, ItemKind, Merchant, Monster, Pet, PetKind, Relic, ScrollKind, Talent};
use crate::fx::{Fx, Particle};
use crate::map::{Map, Tile};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

pub const SAVE_PATH: &str = "abyssal.save.json";

const FOV_RADIUS: i32 = 8;
const AGGRO: i32 = 8;
const LOG_CAP: usize = 64;
const DEATH_HOLD: i32 = 240;
const SHOP_HOLD: i32 = 16;

#[derive(Serialize, Deserialize)]
pub enum Phase {
    Playing,
    Dead(i32),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum FloorEvent {
    Calm,
    Fog,
    Inferno,
    Treasure,
}

impl FloorEvent {
    pub fn label(self) -> &'static str {
        match self {
            FloorEvent::Calm => "",
            FloorEvent::Fog => "\u{2601} brouillard",
            FloorEvent::Inferno => "\u{2668} etage en feu",
            FloorEvent::Treasure => "\u{25a4} salle au tresor",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Caverns,
    Catacombs,
    Frostvault,
    Emberdepths,
    Abyss,
}

impl Biome {
    pub fn label(self) -> &'static str {
        match self {
            Biome::Caverns => "Cavernes",
            Biome::Catacombs => "Catacombes",
            Biome::Frostvault => "Glacier",
            Biome::Emberdepths => "Tref-fonds",
            Biome::Abyss => "Abime",
        }
    }

    pub fn tint(self) -> (f32, f32, f32) {
        match self {
            Biome::Caverns => (1.05, 1.0, 0.9),
            Biome::Catacombs => (0.86, 1.05, 0.9),
            Biome::Frostvault => (0.82, 0.96, 1.22),
            Biome::Emberdepths => (1.22, 0.84, 0.66),
            Biome::Abyss => (1.04, 0.8, 1.2),
        }
    }

    pub fn element(self) -> Option<Element> {
        match self {
            Biome::Caverns => None,
            Biome::Catacombs => Some(Element::Poison),
            Biome::Frostvault => Some(Element::Ice),
            Biome::Emberdepths => Some(Element::Fire),
            Biome::Abyss => Some(Element::Lightning),
        }
    }

    pub fn style_id(self) -> i32 {
        match self {
            Biome::Caverns => 0,
            Biome::Catacombs => 1,
            Biome::Frostvault => 2,
            Biome::Emberdepths => 3,
            Biome::Abyss => 4,
        }
    }

    pub fn fauna(self) -> &'static [char] {
        match self {
            Biome::Caverns => &['r', 'g', 'k', 'o'],
            Biome::Catacombs => &['k', 's', 'h', 'T'],
            Biome::Frostvault => &['s', 'a', 'k', 'g'],
            Biome::Emberdepths => &['w', 'D', 'o', 'a'],
            Biome::Abyss => &['D', 'Y', 'T', 'w'],
        }
    }

    pub fn lore(self) -> &'static str {
        match self {
            Biome::Caverns => "L'air sent la terre humide et le minerai.",
            Biome::Catacombs => "Des ossements craquent sous vos pas.",
            Biome::Frostvault => "Un froid mordant fige votre souffle.",
            Biome::Emberdepths => "La chaleur fait onduler l'air, le sol gronde.",
            Biome::Abyss => "Le vide murmure des choses sans nom.",
        }
    }

    pub fn champion(self) -> (char, &'static str, Element) {
        match self {
            Biome::Caverns => ('o', "Roi-Gobelin des Cavernes", Element::Physical),
            Biome::Catacombs => ('s', "Liche des Catacombes", Element::Poison),
            Biome::Frostvault => ('s', "Seigneur de Givre", Element::Ice),
            Biome::Emberdepths => ('w', "Archimage de Braise", Element::Fire),
            Biome::Abyss => ('D', "Heraut de l'Abime", Element::Lightning),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RoomKind {
    Standard,
    Treasure,
    Challenge,
    Rest,
    Warren,
}

impl RoomKind {
    pub fn label(self) -> &'static str {
        match self {
            RoomKind::Standard => "salle",
            RoomKind::Treasure => "tresor",
            RoomKind::Challenge => "defi",
            RoomKind::Rest => "repos",
            RoomKind::Warren => "nuee",
        }
    }
}

fn default_biome() -> Biome {
    Biome::Caverns
}

fn default_room() -> RoomKind {
    RoomKind::Standard
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MerchantPick {
    Weapon,
    Armor,
    Potion,
    Heal,
    Reroll,
    Cleanse,
    Skip,
}

impl MerchantPick {
    pub fn index(self) -> usize {
        match self {
            MerchantPick::Weapon => 0,
            MerchantPick::Armor => 1,
            MerchantPick::Potion => 2,
            MerchantPick::Heal => 3,
            MerchantPick::Reroll => 4,
            MerchantPick::Cleanse => 5,
            MerchantPick::Skip => 6,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MerchantPick::Weapon => "arme",
            MerchantPick::Armor => "armure",
            MerchantPick::Potion => "potion",
            MerchantPick::Heal => "soin",
            MerchantPick::Reroll => "reroll",
            MerchantPick::Cleanse => "purge",
            MerchantPick::Skip => "rien",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Objective {
    None,
    KillElites,
    ClearFloor,
    Swift,
}

impl Objective {
    pub fn desc(self, target: i32) -> String {
        match self {
            Objective::None => String::new(),
            Objective::KillElites => "tuer toutes les elites".to_string(),
            Objective::ClearFloor => "nettoyer l'etage".to_string(),
            Objective::Swift => format!("escalier en < {} tours", target),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Boon {
    None,
    Tough,
    Sharp,
    Rich,
}

impl Boon {
    pub fn label(self) -> &'static str {
        match self {
            Boon::None => "aucun",
            Boon::Tough => "Robuste (+PV)",
            Boon::Sharp => "Affute (+ATQ)",
            Boon::Rich => "Riche (+or/pot)",
        }
    }

    fn apply(self, h: &mut Hero) {
        match self {
            Boon::None => {}
            Boon::Tough => {
                h.max_hp += 15;
                h.hp = h.max_hp;
            }
            Boon::Sharp => h.might += 3,
            Boon::Rich => {
                h.gold += 80;
                h.potions += 2;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Playstyle {
    Completionist,
    Combatant,
    Rusher,
}

impl Playstyle {
    pub fn label(self) -> &'static str {
        match self {
            Playstyle::Completionist => "completionniste",
            Playstyle::Combatant => "combattant",
            Playstyle::Rusher => "rusher",
        }
    }

    pub fn next(self) -> Playstyle {
        match self {
            Playstyle::Completionist => Playstyle::Combatant,
            Playstyle::Combatant => Playstyle::Rusher,
            Playstyle::Rusher => Playstyle::Completionist,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutator {
    Sanguinaire,
    Cupidite,
    Fragile,
    Pullulement,
    Champions,
}

impl Mutator {
    pub const ALL: [Mutator; 5] = [Mutator::Sanguinaire, Mutator::Cupidite, Mutator::Fragile, Mutator::Pullulement, Mutator::Champions];

    pub fn label(self) -> &'static str {
        match self {
            Mutator::Sanguinaire => "Sanguinaire",
            Mutator::Cupidite => "Cupidite",
            Mutator::Fragile => "Fragile",
            Mutator::Pullulement => "Pullulement",
            Mutator::Champions => "Champions",
        }
    }

    fn count_mult(self) -> f32 {
        match self {
            Mutator::Pullulement => 1.6,
            _ => 1.0,
        }
    }

    fn hp_mult(self) -> f32 {
        match self {
            Mutator::Cupidite => 1.25,
            Mutator::Pullulement => 0.7,
            _ => 1.0,
        }
    }

    fn atk_mult(self) -> f32 {
        match self {
            Mutator::Sanguinaire => 1.25,
            _ => 1.0,
        }
    }

    fn gold_mult(self) -> f32 {
        match self {
            Mutator::Cupidite => 2.0,
            Mutator::Sanguinaire => 1.5,
            _ => 1.0,
        }
    }

    fn elite_add(self) -> f32 {
        match self {
            Mutator::Champions => 0.25,
            _ => 0.0,
        }
    }

    fn apply_hero(self, h: &mut Hero) {
        if self == Mutator::Fragile {
            h.might += 6;
            h.max_hp = (h.max_hp * 8 / 10).max(10);
            h.hp = h.max_hp;
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LogLine {
    pub text: String,
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub map: Map,
    pub hero: Hero,
    pub monsters: Vec<Monster>,
    pub items: Vec<Item>,
    pub log: Vec<LogLine>,
    pub floor: i32,
    pub phase: Phase,
    pub runs: i32,
    pub best_floor: i32,
    pub best_gold: i32,
    pub features: Vec<Feature>,
    pub pet: Option<Pet>,
    #[serde(default)]
    pub allies: Vec<Ally>,
    pub event: FloorEvent,
    #[serde(default = "default_biome")]
    pub biome: Biome,
    #[serde(default = "default_room")]
    pub room_kind: RoomKind,
    pub total_kills: i32,
    pub unlocked: Vec<String>,
    pub discovered: Vec<String>,
    pub high_scores: Vec<i32>,
    pub last_score: i32,
    pub objective: Objective,
    pub objective_done: bool,
    pub objective_target: i32,
    pub floor_turns: i32,
    pub start_class: Option<HeroClass>,
    pub diff_mult: f32,
    pub diff_label: String,
    pub boon: Boon,
    #[serde(default)]
    pub mutators: Vec<Mutator>,
    #[serde(default)]
    pub ascension: i32,
    #[serde(default)]
    meta_hp: i32,
    #[serde(default)]
    meta_might: i32,
    #[serde(default)]
    meta_pot: i32,
    #[serde(default)]
    meta_talent: bool,
    pub last_cause: String,
    pub death_quip: String,
    #[serde(skip)]
    pub last_action: &'static str,
    pub style: Playstyle,
    pub class: HeroClass,
    #[serde(skip)]
    pub flashes: Vec<(i32, i32, Color, i32)>,
    #[serde(skip)]
    pub danger: Vec<(i32, i32)>,
    #[serde(skip)]
    pub danger_color: Color,
    #[serde(skip)]
    pub cast_danger: Vec<(i32, i32)>,
    #[serde(skip)]
    pub boss_wind: i32,
    #[serde(skip)]
    pub boss_pending: i32,
    #[serde(skip)]
    pub boss_move: i32,
    #[serde(skip)]
    pub hazard: Vec<(i32, i32, i32)>,
    #[serde(skip)]
    pub fx: Fx,
    #[serde(skip)]
    pub hero_struck: bool,
    pub merchant: Option<Merchant>,
    #[serde(skip)]
    pub forced_purchase: Option<MerchantPick>,
    #[serde(skip)]
    pub shop_timer: i32,
    #[serde(skip)]
    pub shop_preview: bool,
    #[serde(skip)]
    pub merchant_votes: [u32; 7],
    #[serde(skip)]
    pub top_voters: Vec<(String, u32)>,
    #[serde(skip)]
    pub hud_note: String,
    #[serde(skip)]
    pub sfx: Vec<Sound>,
    #[serde(skip)]
    pub low_hp_pulse: f32,
    #[serde(skip)]
    pub anim_t: u32,
    #[serde(skip)]
    pub lunge: (i32, i32, i32),
    #[serde(skip)]
    chaining: bool,
    #[serde(skip)]
    pub show_codex: bool,
    #[serde(skip)]
    prev_tile: (i32, i32),
    #[serde(skip)]
    turn_start_tile: (i32, i32),
    #[serde(skip)]
    pursue_merchant: bool,
    #[serde(skip)]
    explore_target: Option<(i32, i32)>,
    map_w: i32,
    map_h: i32,
    rng: Rng,
}

const WHITE: Color = (220, 220, 220);
const DIM: Color = (140, 140, 150);
const GOOD: Color = (120, 220, 120);
const WARN: Color = (235, 200, 70);
const BAD: Color = (235, 90, 80);
const GOLD: Color = (235, 205, 60);
const MAGIC: Color = (160, 150, 240);

impl Game {
    pub fn new(map_w: i32, map_h: i32, seed: u64) -> Self {
        Game::new_with(map_w, map_h, seed, None, Playstyle::Completionist, 1.0, "Normal".to_string(), Boon::None, (0, 0, 0, false, 0))
    }

    pub fn new_with(
        map_w: i32,
        map_h: i32,
        seed: u64,
        start_class: Option<HeroClass>,
        style: Playstyle,
        diff_mult: f32,
        diff_label: String,
        boon: Boon,
        meta: (i32, i32, i32, bool, i32),
    ) -> Self {
        let mut rng = Rng::from_seed(seed);
        let class = start_class.unwrap_or_else(|| HeroClass::pick(&mut rng));
        let map = Map::generate(map_w, map_h, &mut rng);
        let (hx, hy) = map.spawn_point();
        let mut hero = Hero::fresh(hx, hy);
        class.apply(&mut hero);
        boon.apply(&mut hero);
        hero.max_hp += meta.0;
        hero.might += meta.1;
        hero.potions += meta.2;
        hero.hp = hero.max_hp;
        if meta.3 {
            let t = Talent::ALL[rng.below(Talent::ALL.len())];
            hero.talents.push(t);
        }
        let mut game = Game {
            map,
            hero,
            monsters: Vec::new(),
            items: Vec::new(),
            log: Vec::new(),
            floor: 1,
            phase: Phase::Playing,
            runs: 1,
            best_floor: 1,
            best_gold: 0,
            features: Vec::new(),
            pet: None,
            allies: Vec::new(),
            event: FloorEvent::Calm,
            biome: Biome::Caverns,
            room_kind: RoomKind::Standard,
            total_kills: 0,
            unlocked: Vec::new(),
            discovered: Vec::new(),
            high_scores: Vec::new(),
            last_score: 0,
            objective: Objective::None,
            objective_done: false,
            objective_target: 0,
            floor_turns: 0,
            start_class,
            diff_mult,
            diff_label,
            boon,
            mutators: Vec::new(),
            ascension: meta.4,
            meta_hp: meta.0,
            meta_might: meta.1,
            meta_pot: meta.2,
            meta_talent: meta.3,
            last_cause: String::new(),
            death_quip: String::new(),
            last_action: "spawn",
            style,
            class,
            flashes: Vec::new(),
            danger: Vec::new(),
            danger_color: (0, 0, 0),
            cast_danger: Vec::new(),
            boss_wind: 0,
            boss_pending: 0,
            boss_move: 0,
            hazard: Vec::new(),
            fx: Fx::default(),
            hero_struck: false,
            merchant: None,
            forced_purchase: None,
            shop_timer: 0,
            shop_preview: false,
            merchant_votes: [0; 7],
            top_voters: Vec::new(),
            hud_note: String::new(),
            sfx: Vec::new(),
            low_hp_pulse: 0.0,
            anim_t: 0,
            lunge: (0, 0, 0),
            chaining: false,
            show_codex: false,
            prev_tile: (-1, -1),
            turn_start_tile: (-1, -1),
            pursue_merchant: false,
            explore_target: None,
            map_w,
            map_h,
            rng,
        };
        game.roll_mutators();
        game.populate_floor(true);
        game
    }

    fn populate_floor(&mut self, first: bool) {
        let map = Map::generate(self.map_w, self.map_h, &mut self.rng);
        let (hx, hy) = map.spawn_point();
        self.map = map;
        self.hero.x = hx;
        self.hero.y = hy;
        self.monsters.clear();
        self.items.clear();
        self.features.clear();

        self.event = if self.floor < 2 {
            FloorEvent::Calm
        } else {
            match self.rng.below(100) {
                0..=14 => FloorEvent::Fog,
                15..=29 => FloorEvent::Inferno,
                30..=42 => FloorEvent::Treasure,
                _ => FloorEvent::Calm,
            }
        };

        let mut floor_tiles: Vec<(i32, i32)> = Vec::new();
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                if self.map.tile(x, y) == Tile::Floor && !(x == hx && y == hy) {
                    let near_spawn = (x - hx).abs() + (y - hy).abs() < 5;
                    if !near_spawn {
                        floor_tiles.push((x, y));
                    }
                }
            }
        }

        let mut monster_count = (4 + self.floor * 2).min(28) as usize;
        let mut item_count = (3 + self.floor).min(14) as usize;
        let mut bonus_chests = 0usize;
        if self.event == FloorEvent::Treasure {
            monster_count = (monster_count / 2).max(2);
            item_count += 4;
        }
        match self.room_kind {
            RoomKind::Treasure => {
                monster_count = (monster_count / 2).max(2);
                item_count += 5;
                bonus_chests += 4;
            }
            RoomKind::Challenge => {
                monster_count += 3;
                item_count += 2;
            }
            RoomKind::Warren => {
                monster_count += 6;
            }
            RoomKind::Rest => {
                monster_count = (monster_count / 3).max(1);
            }
            RoomKind::Standard => {}
        }
        if self.floor >= 3 {
            monster_count = ((monster_count as f32 * self.mut_count_mult()) as usize).min(40);
        }
        if self.floor <= 2 {
            monster_count = monster_count.min(2 + self.floor as usize * 2);
        }

        let biome_el = self.biome.element();
        let fauna = self.biome.fauna();
        for _ in 0..monster_count {
            if floor_tiles.is_empty() {
                break;
            }
            let pick = self.rng.below(floor_tiles.len());
            let (x, y) = floor_tiles.swap_remove(pick);
            let mut m = Monster::roll_biased(self.floor, x, y, &mut self.rng, fauna);
            if let Some(el) = biome_el {
                if self.rng.chance(0.55) {
                    m.element = el;
                }
            }
            self.monsters.push(m);
        }
        let mut elite_chance = (0.08 + self.floor as f32 * 0.012).min(0.35);
        if self.room_kind == RoomKind::Challenge {
            elite_chance = (elite_chance + 0.2).min(0.6);
        } else if self.room_kind == RoomKind::Warren {
            elite_chance *= 0.4;
        }
        elite_chance = (elite_chance + self.mut_elite_add()).min(0.75);
        let promote: Vec<bool> = (0..self.monsters.len()).map(|_| self.rng.chance(elite_chance)).collect();
        for (i, m) in self.monsters.iter_mut().enumerate() {
            if promote[i] {
                m.promote();
            }
        }

        if self.floor >= 4 && self.floor % 5 != 0 && !floor_tiles.is_empty() && self.rng.chance(0.2) {
            let pick = self.rng.below(floor_tiles.len());
            let (x, y) = floor_tiles.swap_remove(pick);
            let (glyph, name, element) = self.biome.champion();
            let mut champ = Monster::specific(glyph, self.floor, x, y);
            champ.promote();
            champ.hp = (champ.hp * 9 / 5).max(1);
            champ.max_hp = champ.hp;
            champ.atk += 3 + self.floor / 4;
            champ.name = name.to_string();
            champ.element = element;
            champ.gold_reward += 25 + self.floor * 2;
            champ.xp_reward += 20 + self.floor;
            self.monsters.push(champ);
            self.push_log(format!("Un champion rode : {} !", name), (255, 150, 120));
        }

        for _ in 0..item_count {
            if floor_tiles.is_empty() {
                break;
            }
            let pick = self.rng.below(floor_tiles.len());
            let (x, y) = floor_tiles.swap_remove(pick);
            self.items.push(Item::roll(self.floor, x, y, &mut self.rng));
        }

        let place_feature = |tiles: &mut Vec<(i32, i32)>, rng: &mut Rng, kind: FeatureKind, feats: &mut Vec<Feature>| {
            if !tiles.is_empty() {
                let (x, y) = tiles.swap_remove(rng.below(tiles.len()));
                feats.push(Feature { x, y, kind });
            }
        };
        if self.rng.chance(0.35) {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Shrine, &mut self.features);
        }
        if self.rng.chance(0.4) {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Fountain, &mut self.features);
        }
        if self.floor >= 2 && self.rng.chance(0.22) {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Altar, &mut self.features);
        }
        if self.pet.is_none() && self.rng.chance(0.15) {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Familiar, &mut self.features);
        }
        if self.floor >= 4 && self.rng.chance(0.08) {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Forge, &mut self.features);
        }
        if self.floor >= 2 && self.rng.chance(0.16) {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Gamble, &mut self.features);
        }
        let chests = 1 + self.rng.below(2) + if self.event == FloorEvent::Treasure { 4 } else { 0 } + bonus_chests;
        for _ in 0..chests {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Chest, &mut self.features);
        }
        let traps = self.rng.below(4) + self.floor.min(3) as usize;
        for _ in 0..traps {
            place_feature(&mut floor_tiles, &mut self.rng, FeatureKind::Trap, &mut self.features);
        }

        if self.floor % 5 == 0 {
            let (sx, sy) = self.map.stairs;
            let spot = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1)]
                .into_iter()
                .map(|(dx, dy)| (sx + dx, sy + dy))
                .find(|&(x, y)| self.map.is_walkable(x, y) && self.monster_at(x, y).is_none())
                .unwrap_or((sx, sy));
            if self.floor % 25 == 0 {
                let b = Monster::final_boss(self.floor, spot.0, spot.1);
                let bn = b.name.clone();
                self.monsters.push(b);
                self.push_log(format!("\u{2638} BOSS FINAL : {} vous attend !", bn), (255, 80, 120));
            } else {
                let b = Monster::boss(self.floor, spot.0, spot.1);
                let bn = b.name.clone();
                self.monsters.push(b);
                self.push_log(format!("\u{2620} {} garde l'escalier...", bn), (255, 150, 90));
            }
        }

        self.merchant = None;
        self.pursue_merchant = false;
        if self.floor >= 2 && self.rng.chance(0.4) && !floor_tiles.is_empty() {
            let pick = self.rng.below(floor_tiles.len());
            let (x, y) = floor_tiles.swap_remove(pick);
            self.merchant = Some(Merchant::roll(self.floor, x, y, &mut self.rng, self.class.weapon_class(), self.class.armor_class()));
        }

        let asc = 1.0 + 0.1 * self.ascension as f32;
        let (mm_hp, mm_atk) = if self.floor >= 3 { (self.mut_hp_mult(), self.mut_atk_mult()) } else { (1.0, 1.0) };
        let hp_scale = self.diff_mult * mm_hp * asc;
        let atk_scale = self.diff_mult * mm_atk * asc;
        if (hp_scale - 1.0).abs() > 0.01 || (atk_scale - 1.0).abs() > 0.01 {
            for m in self.monsters.iter_mut() {
                m.hp = ((m.hp as f32 * hp_scale) as i32).max(1);
                m.max_hp = m.hp;
                m.atk = ((m.atk as f32 * atk_scale) as i32).max(1);
            }
        }

        if let Some(p) = self.pet.as_mut() {
            p.x = hx;
            p.y = hy;
            p.hp = p.max_hp;
        }

        self.boss_wind = 0;
        self.boss_pending = 0;
        self.danger.clear();
        self.cast_danger.clear();
        self.hazard.clear();
        self.allies.clear();
        self.floor_turns = 0;
        self.objective = Objective::None;
        self.objective_done = false;
        self.objective_target = 0;
        if self.floor >= 3 && self.rng.chance(0.5) {
            match self.rng.below(3) {
                0 => {
                    let e = self.monsters.iter().filter(|m| m.elite).count() as i32;
                    if e > 0 {
                        self.objective = Objective::KillElites;
                        self.objective_target = e;
                    }
                }
                1 => self.objective = Objective::ClearFloor,
                _ => {
                    self.objective = Objective::Swift;
                    self.objective_target = 90;
                }
            }
        }

        if first && self.class.raises_dead() {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (hx + dx, hy + dy);
                if self.map.is_walkable(nx, ny) && self.monster_at(nx, ny).is_none() {
                    self.allies.push(Ally::skeleton(self.floor, nx, ny));
                    break;
                }
            }
        }

        self.explore_target = None;
        let fr = self.fov_radius();
        self.map.compute_fov(hx, hy, fr);
        if first {
            self.push_log(format!("Vous entrez dans le donjon. Etage {} - {}.", self.floor, self.biome.label()), WHITE);
        } else {
            self.push_log(format!("Etage {} - {} ({}).", self.floor, self.biome.label(), self.room_kind.label()), MAGIC);
        }
        self.push_log(self.biome.lore().to_string(), (150, 150, 165));
        if self.room_kind == RoomKind::Rest {
            self.hero.hp = self.hero.max_hp;
            self.hero.burn = 0;
            self.hero.poison = 0;
            self.push_log("Une salle de repos : vous reprenez votre souffle.".into(), GOOD);
        }
    }

    pub fn push_log(&mut self, text: String, color: Color) {
        self.log.push(LogLine { text, color });
        if self.log.len() > LOG_CAP {
            self.log.remove(0);
        }
    }

    pub fn monster_at(&self, x: i32, y: i32) -> Option<usize> {
        self.monsters.iter().position(|m| m.x == x && m.y == y)
    }

    pub fn is_alive(&self) -> bool {
        matches!(self.phase, Phase::Playing)
    }

    pub fn hp_fraction(&self) -> f32 {
        self.hero.hp as f32 / self.hero.max_hp.max(1) as f32
    }

    pub fn music_mode(&self) -> crate::audio::MusicMode {
        use crate::audio::MusicMode;
        if matches!(self.phase, Phase::Dead(_)) {
            return MusicMode::Calm;
        }
        if self.monsters.iter().any(|m| m.boss) {
            return MusicMode::Boss;
        }
        let threat = self
            .monsters
            .iter()
            .filter(|m| self.map.is_visible(m.x, m.y) && (m.x - self.hero.x).abs().max((m.y - self.hero.y).abs()) <= 7)
            .count();
        if threat >= 1 {
            MusicMode::Combat
        } else {
            MusicMode::Calm
        }
    }

    fn spawn_ambient(&mut self) {
        let x = self.hero.x + self.rng.between(-9, 10);
        let y = self.hero.y + self.rng.between(-6, 7);
        if !self.map.is_visible(x, y) {
            return;
        }
        let (glyph, color, vy) = match self.biome {
            Biome::Caverns => ('\u{00b7}', (120, 110, 95), 0.05),
            Biome::Catacombs => ('\u{00b7}', (120, 150, 110), 0.0),
            Biome::Frostvault => ('*', (210, 225, 245), 0.12),
            Biome::Emberdepths => ('\u{2218}', (240, 150, 70), -0.10),
            Biome::Abyss => ('\u{00b7}', (175, 125, 215), 0.0),
        };
        self.fx.particles.push(Particle {
            x: x as f32,
            y: y as f32,
            vx: self.rng.range(-0.04, 0.04),
            vy,
            glyph,
            color,
            ttl: self.rng.between(8, 16),
        });
    }

    pub fn update(&mut self) {
        self.hero_struck = false;
        if self.sfx.len() > 256 {
            self.sfx.clear();
        }
        self.fx.tick();
        if matches!(self.phase, Phase::Playing) && self.rng.chance(0.3) {
            self.spawn_ambient();
        }
        self.flashes.retain_mut(|f| {
            f.3 -= 1;
            f.3 > 0
        });
        if let Phase::Dead(t) = self.phase {
            if t <= 1 {
                self.start_new_run();
            } else {
                self.phase = Phase::Dead(t - 1);
            }
            return;
        }
        if self.shop_timer > 0 {
            self.shop_timer -= 1;
            if self.shop_timer == 0 {
                self.trade();
            }
            return;
        }
        self.tick_hero_statuses();
        if matches!(self.phase, Phase::Dead(_)) {
            return;
        }
        self.hero_turn();
        self.best_gold = self.best_gold.max(self.hero.gold);
        if matches!(self.phase, Phase::Dead(_)) {
            return;
        }
        self.monster_turns();
        if matches!(self.phase, Phase::Dead(_)) {
            return;
        }
        self.pet_turn();
        self.ally_turns();
        self.floor_turns += 1;
        self.check_objective();
    }

    fn check_objective(&mut self) {
        if self.objective == Objective::None || self.objective_done {
            return;
        }
        let ok = match self.objective {
            Objective::KillElites => !self.monsters.iter().any(|m| m.elite),
            Objective::ClearFloor => self.monsters.is_empty(),
            _ => false,
        };
        if ok {
            self.complete_objective();
        }
    }

    fn complete_objective(&mut self) {
        self.objective_done = true;
        let reward = 30 + self.floor * 5;
        self.hero.gold += reward;
        self.hero.potions += 1;
        self.fx.label(self.hero.x, self.hero.y, "OBJECTIF !", (120, 230, 160));
        self.fx.burst(&mut self.rng, self.hero.x, self.hero.y, (120, 230, 160), 14, '\u{2726}');
        self.push_log(format!("OBJECTIF REUSSI ! +{} or, +1 potion.", reward), (120, 230, 160));
    }

    fn pet_turn(&mut self) {
        let (px, py, patk, pkind, plevel) = match &self.pet {
            Some(p) => (p.x, p.y, p.atk, p.kind, p.level),
            None => return,
        };
        if let Some(p) = self.pet.as_mut() {
            if p.heal_cd > 0 {
                p.heal_cd -= 1;
            }
        }
        if pkind == PetKind::Mender {
            let near_hero = (px - self.hero.x).abs().max((py - self.hero.y).abs()) <= 4;
            let hurt = self.hero.hp * 10 < self.hero.max_hp * 7;
            if near_hero && hurt && self.pet.as_ref().is_some_and(|p| p.heal_cd == 0) {
                let heal = 5 + plevel * 2;
                self.hero.hp = (self.hero.hp + heal).min(self.hero.max_hp);
                if let Some(p) = self.pet.as_mut() {
                    p.heal_cd = 5;
                }
                self.fx.burst(&mut self.rng, self.hero.x, self.hero.y, (140, 235, 170), 8, '\u{2726}');
                self.fx.label(self.hero.x, self.hero.y, "+", (140, 235, 170));
                return;
            }
        }
        if let Some(j) = self.monsters.iter().position(|m| (m.x - px).abs() + (m.y - py).abs() == 1) {
            let (dmg, crit) = resolve(patk, self.monsters[j].def, &mut self.rng, 0.1);
            self.hit_monster(j, dmg, crit, Element::Physical);
            return;
        }
        let target = if pkind == PetKind::Mender {
            None
        } else {
            self.monsters
                .iter()
                .filter(|m| self.map.is_visible(m.x, m.y))
                .min_by_key(|m| (m.x - px).abs() + (m.y - py).abs())
                .map(|m| (m.x, m.y))
        };
        let goal = target.unwrap_or((self.hero.x, self.hero.y));
        let occupied: Vec<(i32, i32)> = self
            .monsters
            .iter()
            .map(|m| (m.x, m.y))
            .chain(std::iter::once((self.hero.x, self.hero.y)))
            .collect();
        if let Some((dx, dy)) = step_toward(&self.map, px, py, &occupied, |x, y| x == goal.0 && y == goal.1) {
            let (nx, ny) = (px + dx, py + dy);
            if self.monster_at(nx, ny).is_none() && !(nx == self.hero.x && ny == self.hero.y) {
                if let Some(p) = self.pet.as_mut() {
                    p.x = nx;
                    p.y = ny;
                }
            }
        }
    }

    fn tick_hero_statuses(&mut self) {
        if self.hero.shield > 0 {
            self.hero.shield -= 1;
        }
        if self.hero.bolt_cd > 0 {
            self.hero.bolt_cd -= 1;
        }
        if self.hero.ability_cd > 0 {
            self.hero.ability_cd -= 1;
        }
        if self.hero.rage > 0 {
            self.hero.rage -= 1;
        }
        if self.hero.regen > 0 {
            self.hero.regen -= 1;
            self.hero.hp = (self.hero.hp + 2).min(self.hero.max_hp);
        }
        if (self.hero.has_affix(Affix::Regen) || self.hero.has_talent(Talent::Regen) || self.hero.has_relic(Relic::Colossus)) && self.hero.hp < self.hero.max_hp {
            self.hero.hp += 1;
        }
        if self.event == FloorEvent::Inferno && self.rng.chance(0.06) {
            self.hero.burn = self.hero.burn.max(2);
        }
        match self.biome {
            Biome::Emberdepths if self.rng.chance(0.045) => self.hero.burn = self.hero.burn.max(2),
            Biome::Catacombs if self.rng.chance(0.045) => self.hero.poison = self.hero.poison.max(2),
            _ => {}
        }
        if !self.hazard.is_empty() {
            let (hx, hy) = (self.hero.x, self.hero.y);
            let on_hazard = self.hazard.iter().any(|&(x, y, _)| x == hx && y == hy);
            if on_hazard {
                let d = 4 + self.floor;
                self.hero.hp -= d;
                self.fx.damage(hx, hy, d, true);
                self.fx.add_shake(2);
                if self.hero.hp <= 0 {
                    self.hero.hp = 0;
                    self.die("une eruption");
                }
            }
            for h in self.hazard.iter_mut() {
                h.2 -= 1;
            }
            self.hazard.retain(|&(_, _, t)| t > 0);
        }
        let mut dot = 0;
        if self.hero.burn > 0 {
            self.hero.burn -= 1;
            dot += 2;
        }
        if self.hero.poison > 0 {
            self.hero.poison -= 1;
            dot += 2;
        }
        if dot > 0 {
            self.hero.hp -= dot;
            self.fx.damage(self.hero.x, self.hero.y, dot, false);
            if self.hero.hp <= 0 {
                self.hero.hp = 0;
                self.die("ses blessures");
            }
        }
    }

    fn desperate(&self) -> bool {
        self.hero.potions == 0 && self.hero.hp * 5 < self.hero.max_hp
    }

    fn fov_radius(&self) -> i32 {
        if self.event == FloorEvent::Fog {
            4
        } else {
            FOV_RADIUS
        }
    }

    fn consume_scroll(&mut self, k: ScrollKind) -> bool {
        if let Some(i) = self.hero.scrolls.iter().position(|s| *s == k) {
            self.hero.scrolls.remove(i);
            true
        } else {
            false
        }
    }

    fn act_scroll(&mut self) -> bool {
        if self.hero.scrolls.is_empty() {
            return false;
        }
        let (hx, hy) = (self.hero.x, self.hero.y);
        let near = self
            .monsters
            .iter()
            .filter(|m| self.map.is_visible(m.x, m.y) && (m.x - hx).abs().max((m.y - hy).abs()) <= 4)
            .count();
        let adj = self.monsters.iter().filter(|m| (m.x - hx).abs() + (m.y - hy).abs() <= 2).count();

        if self.desperate() && self.consume_scroll(ScrollKind::Teleport) {
            self.cast_teleport();
            return true;
        }
        if near >= 3 && self.consume_scroll(ScrollKind::Fireball) {
            self.cast_fireball();
            return true;
        }
        if adj >= 2 && self.consume_scroll(ScrollKind::Freeze) {
            self.cast_freeze();
            return true;
        }
        false
    }

    fn cast_fireball(&mut self) {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let dmg = 10 + self.floor * 2;
        self.last_action = "boule de feu";
        self.sfx.push(Sound::Scroll);
        self.fx.burst(&mut self.rng, hx, hy, (255, 140, 50), 26, '\u{2736}');
        self.fx.label(hx, hy, "BOULE DE FEU", (255, 150, 60));
        self.fx.add_shake(5);
        let coords: Vec<(i32, i32)> = self
            .monsters
            .iter()
            .filter(|m| (m.x - hx).abs().max((m.y - hy).abs()) <= 3)
            .map(|m| (m.x, m.y))
            .collect();
        for (cx, cy) in coords {
            if let Some(j) = self.monster_at(cx, cy) {
                self.hit_monster(j, dmg, false, Element::Fire);
            }
        }
        self.push_log("Parchemin : boule de feu !".into(), (255, 150, 60));
    }

    fn cast_freeze(&mut self) {
        let (hx, hy) = (self.hero.x, self.hero.y);
        self.last_action = "gel";
        self.sfx.push(Sound::Scroll);
        for m in self.monsters.iter_mut() {
            if (m.x - hx).abs().max((m.y - hy).abs()) <= 4 {
                m.stun = m.stun.max(4);
            }
        }
        self.fx.burst(&mut self.rng, hx, hy, (140, 220, 255), 18, '\u{2744}');
        self.fx.label(hx, hy, "GEL DE ZONE", (140, 220, 255));
        self.push_log("Parchemin : gel de zone !".into(), (140, 220, 255));
    }

    fn cast_teleport(&mut self) {
        let mut best: Option<(i32, i32)> = None;
        let mut tries = 0;
        while tries < 200 {
            tries += 1;
            let x = self.rng.between(0, self.map.width);
            let y = self.rng.between(0, self.map.height);
            if self.map.is_walkable(x, y) && self.map.is_explored(x, y) && self.monster_at(x, y).is_none() {
                let nearest = self
                    .monsters
                    .iter()
                    .map(|m| (m.x - x).abs() + (m.y - y).abs())
                    .min()
                    .unwrap_or(99);
                if nearest >= 6 {
                    best = Some((x, y));
                    break;
                }
                if best.is_none() {
                    best = Some((x, y));
                }
            }
        }
        if let Some((x, y)) = best {
            self.fx.burst(&mut self.rng, self.hero.x, self.hero.y, (180, 160, 255), 12, '\u{2736}');
            self.hero.x = x;
            self.hero.y = y;
            let fr = self.fov_radius();
            self.map.compute_fov(x, y, fr);
            self.fx.burst(&mut self.rng, x, y, (180, 160, 255), 12, '\u{2736}');
            self.fx.label(x, y, "TELEPORT", (180, 160, 255));
            self.last_action = "teleport";
            self.push_log("Parchemin : teleportation !".into(), (180, 160, 255));
        }
    }

    fn mut_count_mult(&self) -> f32 {
        self.mutators.iter().map(|m| m.count_mult()).product()
    }
    fn mut_hp_mult(&self) -> f32 {
        self.mutators.iter().map(|m| m.hp_mult()).product()
    }
    fn mut_atk_mult(&self) -> f32 {
        self.mutators.iter().map(|m| m.atk_mult()).product()
    }
    fn mut_gold_mult(&self) -> f32 {
        self.mutators.iter().map(|m| m.gold_mult()).product()
    }
    fn mut_elite_add(&self) -> f32 {
        self.mutators.iter().map(|m| m.elite_add()).sum()
    }

    fn roll_mutators(&mut self) {
        self.mutators.clear();
        if !self.rng.chance(0.55) {
            return;
        }
        let mut pool: Vec<Mutator> = Mutator::ALL.to_vec();
        let k = if self.rng.chance(0.3) { 2 } else { 1 };
        for _ in 0..k {
            if pool.is_empty() {
                break;
            }
            let i = self.rng.below(pool.len());
            self.mutators.push(pool.remove(i));
        }
        let muts = self.mutators.clone();
        for m in muts {
            m.apply_hero(&mut self.hero);
        }
        let names: Vec<&str> = self.mutators.iter().map(|m| m.label()).collect();
        if !names.is_empty() {
            self.push_log(format!("Mutateurs : {}", names.join(", ")), (235, 130, 200));
        }
    }

    fn start_new_run(&mut self) {
        self.runs += 1;
        self.floor = 1;
        let (hx, hy) = self.map.spawn_point();
        self.class = self.start_class.unwrap_or_else(|| HeroClass::pick(&mut self.rng));
        self.hero = Hero::fresh(hx, hy);
        self.class.apply(&mut self.hero);
        self.boon.apply(&mut self.hero);
        self.hero.max_hp += self.meta_hp;
        self.hero.might += self.meta_might;
        self.hero.potions += self.meta_pot;
        self.hero.hp = self.hero.max_hp;
        if self.meta_talent {
            let t = Talent::ALL[self.rng.below(Talent::ALL.len())];
            self.hero.talents.push(t);
        }
        self.roll_mutators();
        self.pet = None;
        self.apply_relics();
        self.phase = Phase::Playing;
        self.push_log(format!("--- Run #{} : {} ---", self.runs, self.class.label()), WHITE);
        self.populate_floor(true);
    }

    fn hero_turn(&mut self) {
        self.prev_tile = self.turn_start_tile;
        self.turn_start_tile = (self.hero.x, self.hero.y);
        if self.act_dodge() {
            return;
        }
        if self.act_heal() {
            return;
        }
        if self.act_ability() {
            return;
        }
        if self.act_bolt() {
            return;
        }
        if self.act_scroll() {
            return;
        }
        if self.act_attack_adjacent() {
            return;
        }
        if self.desperate() {
            self.last_action = "fuite";
            if self.on_stairs() {
                self.descend();
                return;
            }
            if self.act_to_stairs() {
                return;
            }
        }
        match self.style {
            Playstyle::Completionist => self.turn_completionist(),
            Playstyle::Combatant => self.turn_combatant(),
            Playstyle::Rusher => self.turn_rusher(),
        }
    }

    fn turn_completionist(&mut self) {
        if self.act_hunt(true) {
            return;
        }
        if self.act_loot() {
            return;
        }
        if self.act_feature() {
            return;
        }
        if self.act_merchant() {
            return;
        }
        if self.act_explore() {
            return;
        }
        if self.on_stairs() {
            self.last_action = "descente";
            self.descend();
            return;
        }
        if self.act_to_stairs() {
            return;
        }
        self.last_action = "attente";
    }

    fn turn_combatant(&mut self) {
        if self.act_hunt(true) {
            return;
        }
        if !self.monsters.is_empty() && self.act_explore() {
            self.last_action = "traque";
            return;
        }
        if self.on_stairs() {
            self.last_action = "descente";
            self.descend();
            return;
        }
        if self.act_to_stairs() {
            return;
        }
        if self.act_loot() {
            return;
        }
        if self.act_explore() {
            return;
        }
        self.last_action = "attente";
    }

    fn turn_rusher(&mut self) {
        if self.on_stairs() {
            self.last_action = "descente";
            self.descend();
            return;
        }
        if self.act_to_stairs() {
            self.last_action = "rush escalier";
            return;
        }
        if self.act_explore() {
            return;
        }
        self.last_action = "attente";
    }

    fn on_stairs(&self) -> bool {
        self.map.tile(self.hero.x, self.hero.y) == Tile::StairsDown
    }

    fn act_attack_adjacent(&mut self) -> bool {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let adj: Vec<(i32, i32)> = self
            .monsters
            .iter()
            .filter(|m| (m.x - hx).abs() + (m.y - hy).abs() == 1)
            .map(|m| (m.x, m.y))
            .collect();
        if adj.is_empty() {
            return false;
        }
        if self.hero.level >= self.cleave_level() && adj.len() >= 2 {
            self.last_action = "cleave";
            self.fx.label(hx, hy, "CLEAVE", (255, 180, 90));
            self.fx.add_shake(2);
            let cc = self.hero_crit();
            for (cx, cy) in adj {
                if let Some(j) = self.monster_at(cx, cy) {
                    let (dmg, crit) = resolve(self.hero.atk(), self.monsters[j].def, &mut self.rng, cc);
                    let el = self.hero.weapon_element();
                    self.hit_monster(j, dmg, crit, el);
                }
            }
        } else {
            self.last_action = "combat";
            let idx = self
                .monsters
                .iter()
                .enumerate()
                .filter(|(_, m)| (m.x - hx).abs() + (m.y - hy).abs() == 1)
                .min_by_key(|(_, m)| m.hp)
                .map(|(i, _)| i)
                .unwrap();
            self.hero_attacks(idx);
        }
        true
    }

    fn act_dodge(&mut self) -> bool {
        if !self.hero_in_danger() {
            return false;
        }
        let (hx, hy) = (self.hero.x, self.hero.y);
        let safe = |s: &Self, nx: i32, ny: i32| {
            s.map.is_walkable(nx, ny)
                && s.monster_at(nx, ny).is_none()
                && !s.tile_dangerous(nx, ny)
                && !s.merchant.as_ref().is_some_and(|m| m.x == nx && m.y == ny)
        };
        let threat = self
            .monsters
            .iter()
            .filter(|m| self.map.is_visible(m.x, m.y))
            .min_by_key(|m| (m.x - hx).abs() + (m.y - hy).abs())
            .map(|m| (m.x, m.y));
        let mut best: Option<(i32, i32, i32)> = None;
        for allow_reverse in [false, true] {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
                let (nx, ny) = (hx + dx, hy + dy);
                if !safe(self, nx, ny) {
                    continue;
                }
                if !allow_reverse && (nx, ny) == self.prev_tile {
                    continue;
                }
                let score = match threat {
                    Some((tx, ty)) => (nx - tx).abs() + (ny - ty).abs(),
                    None => 0,
                };
                if best.map_or(true, |b| score < b.2) {
                    best = Some((nx, ny, score));
                }
            }
            if best.is_some() {
                break;
            }
        }
        if let Some((nx, ny, _)) = best {
            self.hero.x = nx;
            self.hero.y = ny;
            let fr = self.fov_radius();
            self.map.compute_fov(nx, ny, fr);
            self.last_action = "esquive";
            self.pickup_here();
            return true;
        }
        false
    }

    fn act_ability(&mut self) -> bool {
        if self.hero.ability_cd > 0 {
            return false;
        }
        match self.class {
            HeroClass::Warrior => self.ability_charge(),
            HeroClass::Rogue => self.ability_blink(),
            HeroClass::Mage => self.ability_nova(),
            HeroClass::Paladin => self.ability_smite(),
            HeroClass::Necromancer => self.ability_raise(),
        }
    }

    fn ability_raise(&mut self) -> bool {
        if self.allies.len() >= 4 {
            return false;
        }
        let (hx, hy) = (self.hero.x, self.hero.y);
        let mut spawned = 0;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            if spawned >= 2 || self.allies.len() >= 4 {
                break;
            }
            let (nx, ny) = (hx + dx, hy + dy);
            if self.map.is_walkable(nx, ny) && self.monster_at(nx, ny).is_none() && !self.allies.iter().any(|a| a.x == nx && a.y == ny) {
                self.allies.push(Ally::skeleton(self.floor, nx, ny));
                self.fx.burst(&mut self.rng, nx, ny, (200, 210, 195), 6, '\u{2736}');
                spawned += 1;
            }
        }
        if spawned == 0 {
            return false;
        }
        self.fx.label(hx, hy, "LEVEE DES MORTS", (200, 210, 195));
        self.sfx.push(Sound::Scroll);
        self.hero.ability_cd = 10;
        self.last_action = "levee";
        true
    }

    fn ally_turns(&mut self) {
        let mut i = 0;
        while i < self.allies.len() {
            self.allies[i].ttl -= 1;
            if self.allies[i].ttl <= 0 || self.allies[i].hp <= 0 {
                let (ax, ay) = (self.allies[i].x, self.allies[i].y);
                self.fx.burst(&mut self.rng, ax, ay, (120, 130, 120), 5, '\u{00b7}');
                self.allies.swap_remove(i);
                continue;
            }
            let (ax, ay, atk) = (self.allies[i].x, self.allies[i].y, self.allies[i].atk);
            if let Some(j) = self.monsters.iter().position(|m| (m.x - ax).abs() + (m.y - ay).abs() == 1) {
                let (dmg, crit) = resolve(atk, self.monsters[j].def, &mut self.rng, 0.08);
                self.hit_monster(j, dmg, crit, Element::Physical);
                i += 1;
                continue;
            }
            let target = self
                .monsters
                .iter()
                .filter(|m| self.map.is_visible(m.x, m.y))
                .min_by_key(|m| (m.x - ax).abs() + (m.y - ay).abs())
                .map(|m| (m.x, m.y));
            if let Some((tx, ty)) = target {
                let mut occ: Vec<(i32, i32)> = self.monsters.iter().map(|m| (m.x, m.y)).collect();
                occ.push((self.hero.x, self.hero.y));
                if let Some(p) = &self.pet {
                    occ.push((p.x, p.y));
                }
                for (k, a) in self.allies.iter().enumerate() {
                    if k != i {
                        occ.push((a.x, a.y));
                    }
                }
                if let Some((dx, dy)) = step_toward(&self.map, ax, ay, &occ, |x, y| x == tx && y == ty) {
                    let (nx, ny) = (ax + dx, ay + dy);
                    if self.monster_at(nx, ny).is_none() && !(nx == self.hero.x && ny == self.hero.y) {
                        self.allies[i].x = nx;
                        self.allies[i].y = ny;
                    }
                }
            }
            i += 1;
        }
    }

    fn ability_smite(&mut self) -> bool {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let adj: Vec<(i32, i32)> = self
            .monsters
            .iter()
            .filter(|m| (m.x - hx).abs().max((m.y - hy).abs()) <= 1)
            .map(|m| (m.x, m.y))
            .collect();
        let hurt = self.hero.hp * 2 < self.hero.max_hp;
        if adj.is_empty() && !hurt {
            return false;
        }
        self.hero.hp = (self.hero.hp + self.hero.max_hp / 4).min(self.hero.max_hp);
        self.hero.shield = self.hero.shield.max(14);
        self.fx.burst(&mut self.rng, hx, hy, (255, 235, 150), 22, '\u{2737}');
        self.fx.label(hx, hy, "CHATIMENT", (255, 235, 150));
        self.fx.add_shake(4);
        self.sfx.push(Sound::Crit);
        let cc = self.hero_crit();
        for (cx, cy) in adj {
            if let Some(j) = self.monster_at(cx, cy) {
                let (dmg, crit) = resolve(self.hero.atk(), self.monsters[j].def, &mut self.rng, cc);
                self.hit_monster(j, dmg, crit, Element::Physical);
            }
        }
        self.hero.ability_cd = 8;
        self.last_action = "chatiment";
        true
    }

    fn ability_charge(&mut self) -> bool {
        let (hx, hy) = (self.hero.x, self.hero.y);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for dist in 2..=3 {
                let (tx, ty) = (hx + dx * dist, hy + dy * dist);
                let Some(mi) = self.monster_at(tx, ty) else { continue };
                if self.monsters[mi].boss {
                    continue;
                }
                let mut clear = true;
                for s in 1..dist {
                    let (cx, cy) = (hx + dx * s, hy + dy * s);
                    if !self.map.is_walkable(cx, cy) || self.monster_at(cx, cy).is_some() {
                        clear = false;
                        break;
                    }
                }
                if !clear {
                    continue;
                }
                let (lx, ly) = (hx + dx * (dist - 1), hy + dy * (dist - 1));
                self.fx.projectile(hx, hy, lx, ly, '\u{00bb}', (255, 200, 120));
                self.hero.x = lx;
                self.hero.y = ly;
                let fr = self.fov_radius();
                self.map.compute_fov(lx, ly, fr);
                self.fx.label(lx, ly, "CHARGE", (255, 180, 90));
                self.fx.add_shake(4);
                self.sfx.push(Sound::Crit);
                let cc = self.hero_crit();
                let (dmg, crit) = resolve(self.hero.atk() + 4, self.monsters[mi].def, &mut self.rng, cc);
                let el = self.hero.weapon_element();
                self.hit_monster(mi, dmg, crit, el);
                self.hero.ability_cd = 7;
                self.last_action = "charge";
                return true;
            }
        }
        false
    }

    fn ability_blink(&mut self) -> bool {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let target = self
            .monsters
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                let cheb = (m.x - hx).abs().max((m.y - hy).abs());
                let man = (m.x - hx).abs() + (m.y - hy).abs();
                man > 1 && cheb <= 5 && self.map.is_visible(m.x, m.y)
            })
            .min_by_key(|(_, m)| (m.x - hx).abs() + (m.y - hy).abs())
            .map(|(i, _)| i);
        let Some(mi) = target else { return false };
        let (mx, my) = (self.monsters[mi].x, self.monsters[mi].y);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            let (lx, ly) = (mx + dx, my + dy);
            if self.map.is_walkable(lx, ly) && self.monster_at(lx, ly).is_none() && !(lx == hx && ly == hy) {
                self.fx.burst(&mut self.rng, hx, hy, (150, 120, 220), 8, '\u{2022}');
                self.hero.x = lx;
                self.hero.y = ly;
                let fr = self.fov_radius();
                self.map.compute_fov(lx, ly, fr);
                self.fx.label(lx, ly, "ASSAUT", (180, 140, 245));
                self.fx.burst(&mut self.rng, lx, ly, (180, 140, 245), 10, '\u{2736}');
                self.sfx.push(Sound::Crit);
                let (dmg, crit) = resolve(self.hero.atk() + 2, self.monsters[mi].def, &mut self.rng, 1.0);
                let el = self.hero.weapon_element();
                self.hit_monster(mi, dmg, crit, el);
                self.hero.ability_cd = 6;
                self.last_action = "assaut";
                return true;
            }
        }
        false
    }

    fn ability_nova(&mut self) -> bool {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let coords: Vec<(i32, i32)> = self
            .monsters
            .iter()
            .filter(|m| (m.x - hx).abs().max((m.y - hy).abs()) <= 2)
            .map(|m| (m.x, m.y))
            .collect();
        if coords.len() < 3 {
            return false;
        }
        let dmg = 8 + self.floor * 2;
        self.fx.burst(&mut self.rng, hx, hy, (130, 200, 255), 28, '\u{2737}');
        self.fx.label(hx, hy, "NOVA", (140, 210, 255));
        self.fx.add_shake(5);
        self.sfx.push(Sound::Scroll);
        for (cx, cy) in coords {
            if let Some(j) = self.monster_at(cx, cy) {
                self.hit_monster(j, dmg, false, Element::Ice);
            }
        }
        self.hero.ability_cd = 9;
        self.last_action = "nova";
        true
    }

    fn act_bolt(&mut self) -> bool {
        if self.hero.level < self.bolt_level() || self.hero.bolt_cd > 0 {
            return false;
        }
        let (hx, hy) = (self.hero.x, self.hero.y);
        let target = self
            .monsters
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                let d = (m.x - hx).abs().max((m.y - hy).abs());
                d >= 1 && d <= 6 && self.map.is_visible(m.x, m.y) && self.map.line_of_sight(hx, hy, m.x, m.y)
            })
            .min_by_key(|(_, m)| {
                let prio = if m.ranged { 0 } else { 1 };
                (prio, (m.x - hx).abs() + (m.y - hy).abs())
            })
            .map(|(i, _)| i);
        if let Some(idx) = target {
            let (mx, my) = (self.monsters[idx].x, self.monsters[idx].y);
            self.fx.projectile(hx, hy, mx, my, '\u{2726}', (170, 225, 255));
            self.sfx.push(Sound::Bolt);
            let cc = self.hero_crit();
            let (dmg, crit) = resolve(self.hero.atk() - 2, self.monsters[idx].def, &mut self.rng, cc);
            self.last_action = "eclair";
            self.hit_monster(idx, dmg, crit, Element::Lightning);
            self.hero.bolt_cd = if matches!(self.class, HeroClass::Mage) { 3 } else { 6 };
            return true;
        }
        false
    }

    fn hit_monster(&mut self, idx: usize, base_dmg: i32, crit: bool, element: Element) {
        let mob_el = self.monsters[idx].element;
        let mult = if element == Element::Physical {
            1.0
        } else if mob_el == element {
            0.5
        } else if mob_el == element.opposite() {
            1.6
        } else {
            1.0
        };
        let was_frozen = self.monsters[idx].stun > 0;
        let shatter = was_frozen && element != Element::Ice;
        let mut dmg = ((base_dmg as f32 * mult).round() as i32).max(1);
        if shatter {
            dmg = (dmg * 3 / 2).max(1);
            self.monsters[idx].stun = 0;
        }
        self.monsters[idx].hp -= dmg;
        let (mx, my) = (self.monsters[idx].x, self.monsters[idx].y);
        let color = self.monsters[idx].color;
        let is_boss = self.monsters[idx].boss;
        let name = self.monsters[idx].name.clone();
        self.flashes.push((mx, my, (255, 255, 255), 2));
        if element == Element::Physical {
            self.fx.damage(mx, my, dmg, crit);
        } else {
            self.fx.damage_el(mx, my, dmg, crit, element.color());
        }
        if mult > 1.2 {
            self.fx.label(mx, my, "FAIBLE!", element.color());
        }
        if shatter {
            self.fx.label(mx, my, "BRISE!", (160, 220, 255));
            self.fx.burst(&mut self.rng, mx, my, (180, 230, 255), 12, '\u{2744}');
        }
        if crit {
            self.fx.add_shake(3);
        }
        self.sfx.push(if crit { Sound::Crit } else { Sound::Hit });
        if self.hero.has_affix(Affix::Lifesteal) || self.hero.has_talent(Talent::Sangsue) {
            self.hero.hp = (self.hero.hp + (dmg / 4).max(1)).min(self.hero.max_hp);
        }
        if self.monsters[idx].hp > 0 {
            match element {
                Element::Fire | Element::Poison => {
                    self.monsters[idx].poison = self.monsters[idx].poison.max(4);
                }
                Element::Ice => {
                    self.monsters[idx].stun = self.monsters[idx].stun.max(2);
                }
                _ => {}
            }
            if self.class.bleeds() {
                self.monsters[idx].poison = self.monsters[idx].poison.max(3);
            }
            if self.hero.has_relic(Relic::Ember) {
                self.monsters[idx].poison = self.monsters[idx].poison.max(3);
            }
        }
        if self.monsters[idx].hp <= 0 {
            let m = self.monsters.swap_remove(idx);
            let raise_cap = if self.hero.has_relic(Relic::Undying) { 6 } else { 4 };
            let can_raise = self.class.raises_dead() || self.hero.has_relic(Relic::Undying);
            if can_raise && !m.boss && self.allies.len() < raise_cap && self.monster_at(mx, my).is_none() && self.rng.chance(0.4) {
                self.allies.push(Ally::raised(self.floor, mx, my, &m));
                self.fx.burst(&mut self.rng, mx, my, (170, 220, 180), 8, '\u{2736}');
                self.fx.label(mx, my, "LEVE", (170, 220, 180));
            }
            if self.hero.has_relic(Relic::Vampire) {
                self.hero.hp = (self.hero.hp + 4).min(self.hero.max_hp);
            }
            if is_boss {
                self.grant_relic();
            } else if m.elite && self.rng.chance(0.12) {
                self.grant_relic();
            }
            self.hero.kills += 1;
            self.total_kills += 1;
            self.hero.gold += (m.gold_reward as f32 * self.mut_gold_mult()) as i32;
            self.fx.bump_combo();
            self.sfx.push(if is_boss { Sound::BossHit } else { Sound::Kill });
            self.discover(&name);
            let sparks = if is_boss { 30 } else { 9 };
            self.fx.burst(&mut self.rng, mx, my, color, sparks, '\u{2736}');
            if is_boss {
                self.boss_wind = 0;
                self.danger.clear();
                self.fx.add_shake(8);
                self.fx.label(mx, my, "BOSS VAINCU", (255, 220, 90));
                self.push_log(format!("BOSS VAINCU : {} ! (+{} XP)", name, m.xp_reward), WARN);
                self.unlock("boss", "Tueur de boss");
            } else {
                self.push_log(format!("Vous terrassez le {} ! (+{} XP)", name, m.xp_reward), GOOD);
            }
            if self.total_kills == 1 {
                self.unlock("first_blood", "Premier sang");
            }
            if self.total_kills >= 100 {
                self.unlock("centurion", "Centurion - 100 elimines");
            }
            if self.hero.gain_xp(m.xp_reward) {
                self.sfx.push(Sound::LevelUp);
                self.fx.burst(&mut self.rng, self.hero.x, self.hero.y, (255, 225, 120), 16, '\u{2022}');
                self.fx.label(self.hero.x, self.hero.y, "NIVEAU+", (255, 225, 120));
                self.push_log(
                    format!("NIVEAU {} ! PV/ATQ/DEF augmentes, soins complets.", self.hero.level),
                    WARN,
                );
                self.grant_talent();
                if let Some(p) = self.pet.as_mut() {
                    p.level += 1;
                    p.max_hp += 5;
                    p.hp = p.max_hp;
                    p.atk += 2;
                }
            }
        } else {
            self.push_log(format!("Vous touchez le {} ({} degats).", name, dmg), WHITE);
        }

        let storm = self.hero.has_relic(Relic::Storm) && self.rng.chance(0.3);
        if (element == Element::Lightning || storm) && !self.chaining {
            let target = self
                .monsters
                .iter()
                .enumerate()
                .filter(|(_, m)| !(m.x == mx && m.y == my) && (m.x - mx).abs().max((m.y - my).abs()) <= 3)
                .min_by_key(|(_, m)| (m.x - mx).abs() + (m.y - my).abs())
                .map(|(i, _)| i);
            if let Some(j) = target {
                let (jx, jy) = (self.monsters[j].x, self.monsters[j].y);
                self.fx.projectile(mx, my, jx, jy, '\u{2741}', (245, 230, 90));
                let chain_dmg = (base_dmg / 2).max(1);
                self.chaining = true;
                self.hit_monster(j, chain_dmg, false, Element::Lightning);
                self.chaining = false;
            }
        }
    }

    fn act_heal(&mut self) -> bool {
        if self.hero.hp * 3 < self.hero.max_hp && self.hero.potions > 0 {
            self.last_action = "potion";
            self.hero.potions -= 1;
            let heal = (self.hero.max_hp / 2).max(10);
            self.hero.hp = (self.hero.hp + heal).min(self.hero.max_hp);
            self.sfx.push(Sound::Quaff);
            self.push_log(format!("Vous buvez une potion (+{} PV).", heal), GOOD);
            return true;
        }
        false
    }

    fn act_hunt(&mut self, track_seen: bool) -> bool {
        let target = if track_seen {
            self.nearest_seen_monster()
        } else {
            self.nearest_visible_monster()
        };
        if let Some((tx, ty)) = target {
            let open = self.blocked_tiles();
            if let Some((dx, dy)) =
                step_toward(&self.map, self.hero.x, self.hero.y, &open, |x, y| x == tx && y == ty)
            {
                self.last_action = "chasse";
                self.move_or_act(dx, dy);
                return true;
            }
        }
        false
    }

    fn act_loot(&mut self) -> bool {
        if let Some((tx, ty)) = self.nearest_seen_item() {
            let open = self.blocked_tiles();
            if let Some((dx, dy)) =
                step_toward(&self.map, self.hero.x, self.hero.y, &open, |x, y| x == tx && y == ty)
            {
                self.last_action = "butin";
                self.move_or_act(dx, dy);
                return true;
            }
        }
        false
    }

    fn act_feature(&mut self) -> bool {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let target = self
            .features
            .iter()
            .filter(|f| f.kind != FeatureKind::Trap && self.map.is_explored(f.x, f.y))
            .min_by_key(|f| (f.x - hx).abs() + (f.y - hy).abs())
            .map(|f| (f.x, f.y));
        if let Some((tx, ty)) = target {
            let open = self.blocked_tiles();
            if let Some((dx, dy)) = step_toward(&self.map, hx, hy, &open, |x, y| x == tx && y == ty) {
                self.last_action = "autel";
                self.move_or_act(dx, dy);
                return true;
            }
        }
        false
    }

    fn act_merchant(&mut self) -> bool {
        let Some((mx, my)) = self.merchant.as_ref().map(|m| (m.x, m.y)) else {
            self.pursue_merchant = false;
            return false;
        };
        if !self.pursue_merchant {
            if !self.merchant_wants_trade() {
                return false;
            }
            self.pursue_merchant = true;
        }
        if !self.map.is_explored(mx, my) {
            return false;
        }
        let open = self.blocked_tiles();
        if let Some((dx, dy)) = step_toward(&self.map, self.hero.x, self.hero.y, &open, |x, y| x == mx && y == my) {
            self.last_action = "marchand";
            self.move_or_act(dx, dy);
            return true;
        }
        self.pursue_merchant = false;
        false
    }

    fn act_explore(&mut self) -> bool {
        let open = self.blocked_tiles();
        let target_valid = match self.explore_target {
            Some((tx, ty)) => self.map.has_unexplored_neighbor(tx, ty),
            None => false,
        };
        if !target_valid {
            self.explore_target = nearest_goal(&self.map, self.hero.x, self.hero.y, &open, |x, y| {
                self.map.has_unexplored_neighbor(x, y)
            });
        }
        if let Some((tx, ty)) = self.explore_target {
            if let Some((dx, dy)) =
                step_toward(&self.map, self.hero.x, self.hero.y, &open, |x, y| x == tx && y == ty)
            {
                self.last_action = "exploration";
                self.move_or_act(dx, dy);
                return true;
            }
            self.explore_target = None;
        }
        false
    }

    fn act_to_stairs(&mut self) -> bool {
        let (sx, sy) = self.map.stairs;
        let open = self.blocked_tiles();
        if let Some((dx, dy)) = step_toward(&self.map, self.hero.x, self.hero.y, &open, |x, y| x == sx && y == sy) {
            self.last_action = "vers escalier";
            self.move_or_act(dx, dy);
            return true;
        }
        false
    }

    fn nearest_visible_monster(&self) -> Option<(i32, i32)> {
        self.monsters
            .iter()
            .filter(|m| self.map.is_visible(m.x, m.y))
            .min_by_key(|m| (m.x - self.hero.x).abs() + (m.y - self.hero.y).abs())
            .map(|m| (m.x, m.y))
    }

    fn nearest_seen_monster(&self) -> Option<(i32, i32)> {
        self.monsters
            .iter()
            .filter(|m| self.map.is_explored(m.x, m.y))
            .min_by_key(|m| (m.x - self.hero.x).abs() + (m.y - self.hero.y).abs())
            .map(|m| (m.x, m.y))
    }

    pub fn cycle_style(&mut self) {
        self.style = self.style.next();
        self.push_log(format!("Etat d'esprit : {}.", self.style.label()), (200, 170, 90));
    }

    pub fn set_style(&mut self, style: Playstyle) {
        if self.style != style {
            self.style = style;
            self.push_log(format!("Etat d'esprit : {}.", style.label()), (200, 170, 90));
        }
    }

    pub fn spawn_test_merchant(&mut self) {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let spot = [(1, 0), (-1, 0), (0, 1), (0, -1), (2, 0), (0, 2)]
            .into_iter()
            .map(|(dx, dy)| (hx + dx, hy + dy))
            .find(|&(x, y)| self.map.is_walkable(x, y) && self.monster_at(x, y).is_none());
        if let Some((x, y)) = spot {
            self.merchant = Some(Merchant::roll(self.floor.max(1), x, y, &mut self.rng, self.class.weapon_class(), self.class.armor_class()));
            self.push_log("[test] Un marchand apparait !".into(), (130, 235, 240));
        }
    }

    pub fn save(&mut self) {
        match serde_json::to_string(self) {
            Ok(json) => match std::fs::write(SAVE_PATH, json) {
                Ok(()) => self.push_log(format!("Partie sauvegardee ({}).", SAVE_PATH), (120, 220, 230)),
                Err(e) => self.push_log(format!("Echec sauvegarde : {}", e), BAD),
            },
            Err(e) => self.push_log(format!("Echec serialisation : {}", e), BAD),
        }
    }

    pub fn load() -> Option<Game> {
        let data = std::fs::read_to_string(SAVE_PATH).ok()?;
        let mut game: Game = serde_json::from_str(&data).ok()?;
        game.last_action = "charge";
        game.push_log("Partie chargee depuis la sauvegarde.".into(), (120, 220, 230));
        Some(game)
    }

    fn nearest_seen_item(&self) -> Option<(i32, i32)> {
        self.items
            .iter()
            .filter(|it| self.map.is_explored(it.x, it.y))
            .min_by_key(|it| (it.x - self.hero.x).abs() + (it.y - self.hero.y).abs())
            .map(|it| (it.x, it.y))
    }

    fn merchant_wants_trade(&self) -> bool {
        let Some(m) = &self.merchant else { return false };
        if matches!(self.forced_purchase, Some(p) if p != MerchantPick::Skip) {
            return true;
        }
        if self.hero.gold <= 0 {
            return false;
        }
        let weapon_deal = m.weapon.as_ref().is_some_and(|&(_, b, p)| b > self.hero.weapon_bonus && self.hero.gold >= p);
        let armor_deal = m.armor.as_ref().is_some_and(|&(_, b, p)| b > self.hero.armor_bonus && self.hero.gold >= p);
        let heal_deal = self.hero.hp * 10 < self.hero.max_hp * 7 && self.hero.gold >= m.heal_price;
        let potion_deal = self.hero.potions < 4 && self.hero.gold >= m.potion_price + 30;
        let cleanse_deal = (self.hero.poison > 0 || self.hero.burn > 0) && self.hero.gold >= 15 + self.floor * 2;
        weapon_deal || armor_deal || heal_deal || potion_deal || cleanse_deal
    }

    fn trade(&mut self) {
        self.pursue_merchant = false;
        let Some(m) = self.merchant.take() else { return };
        match self.forced_purchase.take() {
            Some(pick) => {
                self.push_log(format!("Le chat fait acheter : {}.", pick.label()), (180, 130, 235));
                self.buy_pick(&m, pick);
            }
            None => {
                self.push_log("Vous abordez un marchand itinerant.".into(), (120, 220, 230));
                self.auto_buy(&m);
            }
        }
        self.push_log("Le marchand remballe son etal.".into(), DIM);
    }

    fn buy_pick(&mut self, m: &Merchant, pick: MerchantPick) {
        match pick {
            MerchantPick::Weapon => {
                if let Some((name, bonus, price)) = &m.weapon {
                    if self.hero.gold >= *price {
                        self.hero.gold -= *price;
                        self.hero.weapon_bonus = self.hero.weapon_bonus.max(*bonus);
                        self.hero.weapon = name.clone();
                        self.hero.weapon_affix = Affix::None;
                        self.push_log(format!("Achat : {} ({} or, ATQ {}).", self.hero.weapon, price, self.hero.atk()), GOOD);
                        return;
                    }
                }
                self.push_log("Pas d'arme abordable ici.".into(), DIM);
            }
            MerchantPick::Armor => {
                if let Some((name, bonus, price)) = &m.armor {
                    if self.hero.gold >= *price {
                        self.hero.gold -= *price;
                        self.hero.armor_bonus = self.hero.armor_bonus.max(*bonus);
                        self.hero.armor = name.clone();
                        self.hero.armor_affix = Affix::None;
                        self.push_log(format!("Achat : {} ({} or, DEF {}).", self.hero.armor, price, self.hero.def()), GOOD);
                        return;
                    }
                }
                self.push_log("Pas d'armure abordable ici.".into(), DIM);
            }
            MerchantPick::Potion => {
                if self.hero.gold >= m.potion_price {
                    self.hero.gold -= m.potion_price;
                    self.hero.potions += 1;
                    self.push_log(format!("Achat d'une potion ({} or).", m.potion_price), MAGIC);
                } else {
                    self.push_log("Pas assez d'or pour une potion.".into(), DIM);
                }
            }
            MerchantPick::Heal => {
                if self.hero.gold >= m.heal_price {
                    self.hero.gold -= m.heal_price;
                    self.hero.hp = self.hero.max_hp;
                    self.push_log(format!("Soin complet ({} or).", m.heal_price), GOOD);
                } else {
                    self.push_log("Pas assez d'or pour un soin.".into(), DIM);
                }
            }
            MerchantPick::Reroll => {
                let fee = 20 + self.floor * 3;
                if self.hero.gold >= fee {
                    self.hero.gold -= fee;
                    self.push_log(format!("Reroll du marchand ({} or) !", fee), (120, 220, 230));
                    let fresh = Merchant::roll(self.floor, m.x, m.y, &mut self.rng, self.class.weapon_class(), self.class.armor_class());
                    self.auto_buy(&fresh);
                } else {
                    self.push_log("Pas assez d'or pour reroll.".into(), DIM);
                }
            }
            MerchantPick::Cleanse => {
                let fee = 15 + self.floor * 2;
                if self.hero.gold >= fee {
                    self.hero.gold -= fee;
                    self.hero.poison = 0;
                    self.hero.burn = 0;
                    self.hero.hp = (self.hero.hp + 15).min(self.hero.max_hp);
                    self.push_log(format!("Purification ({} or) : maux retires.", fee), GOOD);
                } else {
                    self.push_log("Pas assez d'or pour purifier.".into(), DIM);
                }
            }
            MerchantPick::Skip => {
                self.push_log("Le chat passe son tour.".into(), DIM);
            }
        }
    }

    fn auto_buy(&mut self, m: &Merchant) {
        if let Some((name, bonus, price)) = &m.weapon {
            if *bonus > self.hero.weapon_bonus && self.hero.gold >= *price {
                self.hero.gold -= *price;
                self.hero.weapon_bonus = *bonus;
                self.hero.weapon = name.clone();
                self.hero.weapon_affix = Affix::None;
                self.push_log(format!("Achat : {} pour {} or (ATQ {}).", self.hero.weapon, price, self.hero.atk()), GOOD);
            }
        }
        if let Some((name, bonus, price)) = &m.armor {
            if *bonus > self.hero.armor_bonus && self.hero.gold >= *price {
                self.hero.gold -= *price;
                self.hero.armor_bonus = *bonus;
                self.hero.armor = name.clone();
                self.hero.armor_affix = Affix::None;
                self.push_log(format!("Achat : {} pour {} or (DEF {}).", self.hero.armor, price, self.hero.def()), GOOD);
            }
        }
        if self.hero.hp * 10 < self.hero.max_hp * 7 && self.hero.gold >= m.heal_price {
            self.hero.gold -= m.heal_price;
            self.hero.hp = self.hero.max_hp;
            self.push_log(format!("Soin complet pour {} or.", m.heal_price), GOOD);
        }
        while self.hero.potions < 5 && self.hero.gold >= m.potion_price + 30 {
            self.hero.gold -= m.potion_price;
            self.hero.potions += 1;
            self.push_log(format!("Achat d'une potion pour {} or.", m.potion_price), MAGIC);
        }
        let cleanse_fee = 15 + self.floor * 2;
        if (self.hero.poison > 0 || self.hero.burn > 0) && self.hero.gold >= cleanse_fee {
            self.hero.gold -= cleanse_fee;
            self.hero.poison = 0;
            self.hero.burn = 0;
            self.push_log(format!("Purification pour {} or.", cleanse_fee), GOOD);
        }
    }

    fn move_or_act(&mut self, dx: i32, dy: i32) {
        let nx = self.hero.x + dx;
        let ny = self.hero.y + dy;
        if let Some(i) = self.monster_at(nx, ny) {
            self.hero_attacks(i);
            return;
        }
        if self.merchant.as_ref().is_some_and(|m| m.x == nx && m.y == ny) {
            if self.shop_timer == 0 {
                self.shop_timer = SHOP_HOLD;
                self.sfx.push(Sound::Trade);
                self.fx.label(self.hero.x, self.hero.y, "MARCHAND", (130, 235, 240));
                self.push_log("Vous abordez le marchand...".into(), (120, 220, 230));
            }
            return;
        }
        if self.map.is_walkable(nx, ny) {
            self.hero.x = nx;
            self.hero.y = ny;
            let fr = self.fov_radius();
            self.map.compute_fov(nx, ny, fr);
            self.pickup_here();
        }
    }

    fn pickup_here(&mut self) {
        let hx = self.hero.x;
        let hy = self.hero.y;
        if let Some(i) = self.items.iter().position(|it| it.x == hx && it.y == hy) {
            let item = self.items.swap_remove(i);
            let rarity = item.color;
            match item.kind {
                ItemKind::Gold(amount) => {
                    self.hero.gold += amount;
                    self.sfx.push(Sound::Gold);
                    self.push_log(format!("Vous ramassez {} pieces d'or.", amount), GOLD);
                }
                ItemKind::Potion => {
                    self.hero.potions += 1;
                    self.push_log("Vous trouvez une potion de soin.".into(), MAGIC);
                }
                ItemKind::Weapon(bonus, name, affix, wclass) => {
                    if wclass != self.class.weapon_class() {
                        self.hero.gold += bonus + 4;
                        self.push_log(format!("Vous revendez : {} ({}).", name, wclass.label()), DIM);
                    } else if bonus > self.hero.weapon_bonus
                        || (bonus == self.hero.weapon_bonus && affix != Affix::None)
                    {
                        self.hero.weapon_bonus = bonus;
                        self.hero.weapon = name;
                        self.hero.weapon_affix = affix;
                        self.sfx.push(Sound::Item);
                        self.push_log(format!("Vous equipez : {} (ATQ {}).", self.hero.weapon, self.hero.atk()), rarity);
                    } else {
                        self.hero.gold += bonus;
                        self.push_log(format!("Vous revendez : {}.", name), DIM);
                    }
                }
                ItemKind::Armor(bonus, name, affix, aclass) => {
                    if aclass != self.class.armor_class() {
                        self.hero.gold += bonus + 4;
                        self.push_log(format!("Vous revendez : {} ({}).", name, aclass.label()), DIM);
                    } else if bonus > self.hero.armor_bonus
                        || (bonus == self.hero.armor_bonus && affix != Affix::None)
                    {
                        self.hero.armor_bonus = bonus;
                        self.hero.armor = name;
                        self.hero.armor_affix = affix;
                        self.sfx.push(Sound::Item);
                        self.push_log(format!("Vous equipez : {} (DEF {}).", self.hero.armor, self.hero.def()), rarity);
                    } else {
                        self.hero.gold += bonus;
                        self.push_log(format!("Vous revendez : {}.", name), DIM);
                    }
                }
                ItemKind::Ring(bonus, affix) => {
                    if bonus > self.hero.ring_bonus || (bonus == self.hero.ring_bonus && affix != Affix::None) {
                        self.hero.ring_bonus = bonus;
                        self.hero.ring = affix;
                        self.push_log(format!("Anneau equipe (+{} ATQ, {}).", bonus, affix.label()), rarity);
                    } else {
                        self.hero.gold += 8;
                        self.push_log("Vous revendez un anneau.".into(), DIM);
                    }
                }
                ItemKind::Amulet(bonus, affix) => {
                    if bonus > self.hero.amulet_bonus || (bonus == self.hero.amulet_bonus && affix != Affix::None) {
                        self.hero.amulet_bonus = bonus;
                        self.hero.amulet = affix;
                        self.push_log(format!("Amulette equipee (+{} DEF, {}).", bonus, affix.label()), rarity);
                    } else {
                        self.hero.gold += 8;
                        self.push_log("Vous revendez une amulette.".into(), DIM);
                    }
                }
                ItemKind::Scroll(kind) => {
                    self.hero.scrolls.push(kind);
                    self.sfx.push(Sound::Item);
                    self.push_log(format!("Parchemin ramasse : {}.", kind.label()), (235, 235, 170));
                }
            }
            if rarity == (255, 170, 60) {
                self.unlock("legende", "Legende - objet legendaire");
            }
        }
        self.trigger_feature();
    }

    fn trigger_feature(&mut self) {
        let (hx, hy) = (self.hero.x, self.hero.y);
        let Some(fi) = self.features.iter().position(|f| f.x == hx && f.y == hy) else {
            return;
        };
        let kind = self.features[fi].kind;
        self.features.swap_remove(fi);
        match kind {
            FeatureKind::Shrine => {
                match self.rng.below(3) {
                    0 => {
                        self.hero.might += 2;
                        self.push_log("Sanctuaire : +2 FORCE.".into(), (200, 170, 255));
                    }
                    1 => {
                        self.hero.guard += 2;
                        self.push_log("Sanctuaire : +2 GARDE.".into(), (200, 170, 255));
                    }
                    _ => {
                        self.hero.max_hp += 12;
                        self.hero.hp += 12;
                        self.push_log("Sanctuaire : +12 PV max.".into(), (200, 170, 255));
                    }
                }
                if self.rng.chance(0.3) {
                    self.hero.rage = self.hero.rage.max(16);
                    self.push_log("Sanctuaire : ferveur (RAGE) !".into(), (235, 120, 90));
                }
                self.fx.burst(&mut self.rng, hx, hy, (200, 170, 255), 14, '\u{2727}');
                self.fx.label(hx, hy, "BENEDICTION", (200, 170, 255));
            }
            FeatureKind::Fountain => {
                self.hero.hp = self.hero.max_hp;
                self.hero.burn = 0;
                self.hero.poison = 0;
                self.push_log("Fontaine : soins complets.".into(), (110, 200, 230));
                self.fx.burst(&mut self.rng, hx, hy, (110, 200, 230), 12, '\u{2248}');
                self.fx.label(hx, hy, "SOIN", (110, 200, 230));
            }
            FeatureKind::Chest => {
                let mimic_spot = if self.rng.chance(0.28) {
                    [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1)]
                        .into_iter()
                        .map(|(dx, dy)| (hx + dx, hy + dy))
                        .find(|&(x, y)| self.map.is_walkable(x, y) && self.monster_at(x, y).is_none())
                } else {
                    None
                };
                if let Some((mxp, myp)) = mimic_spot {
                    self.monsters.push(Monster::mimic(self.floor, mxp, myp));
                    let bite = 5 + self.floor;
                    self.hero.hp -= bite;
                    self.fx.add_shake(6);
                    self.fx.damage(hx, hy, bite, true);
                    self.fx.burst(&mut self.rng, mxp, myp, (235, 150, 80), 12, '\u{2736}');
                    self.fx.label(hx, hy, "MIMIC !", (235, 150, 80));
                    self.push_log("C'est un MIMIC ! Meme Frieren se fait avoir...".into(), BAD);
                    if self.hero.hp <= 0 {
                        self.hero.hp = 0;
                        self.die("un mimic (comme Frieren)");
                    }
                } else {
                    let loot = 25 + self.floor * 6;
                    self.hero.gold += loot;
                    self.hero.potions += 1;
                    self.items.push(Item::roll(self.floor + 2, hx, hy, &mut self.rng));
                    self.push_log(format!("Coffre ! +{} or, +1 potion, butin.", loot), GOLD);
                    self.fx.burst(&mut self.rng, hx, hy, (255, 210, 90), 16, '\u{2736}');
                    self.fx.label(hx, hy, "TRESOR", (255, 210, 90));
                }
            }
            FeatureKind::Altar => {
                self.fx.burst(&mut self.rng, hx, hy, (205, 90, 225), 16, '\u{2726}');
                self.fx.label(hx, hy, "PACTE", (215, 110, 235));
                match self.rng.below(4) {
                    0 => {
                        self.hero.might += 6;
                        self.hero.max_hp = (self.hero.max_hp - 12).max(10);
                        self.hero.hp = self.hero.hp.min(self.hero.max_hp);
                        self.push_log("Pacte de Force : +6 ATQ, -12 PV max.".into(), (215, 110, 235));
                    }
                    1 => {
                        self.hero.guard += 5;
                        self.hero.poison = self.hero.poison.max(8);
                        self.push_log("Pacte de Garde : +5 DEF mais le sang bout (poison).".into(), (215, 110, 235));
                    }
                    2 => {
                        self.hero.weapon_affix = Affix::Lifesteal;
                        self.hero.max_hp = (self.hero.max_hp - 10).max(10);
                        self.hero.hp = self.hero.hp.min(self.hero.max_hp);
                        self.push_log("Pacte de Sang : arme vampirique, -10 PV max.".into(), (215, 110, 235));
                    }
                    _ => {
                        self.hero.gold *= 2;
                        let pen = self.hero.max_hp / 5;
                        self.hero.max_hp = (self.hero.max_hp - pen).max(10);
                        self.hero.hp = self.hero.hp.min(self.hero.max_hp);
                        self.push_log("Pacte d'Or : or x2, -20% PV max.".into(), (215, 110, 235));
                    }
                }
            }
            FeatureKind::Familiar => {
                let spot = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .into_iter()
                    .map(|(dx, dy)| (hx + dx, hy + dy))
                    .find(|&(x, y)| self.map.is_walkable(x, y))
                    .unwrap_or((hx, hy));
                let pet = Pet::new(self.floor, spot.0, spot.1, &mut self.rng);
                let pname = pet.name.clone();
                self.pet = Some(pet);
                self.fx.burst(&mut self.rng, hx, hy, (120, 230, 180), 14, '\u{2726}');
                self.fx.label(hx, hy, "FAMILIER", (120, 230, 180));
                self.push_log(format!("Un familier ({}) se joint a vous !", pname), (120, 230, 180));
            }
            FeatureKind::Trap => {
                let dmg = 4 + self.floor * 2;
                self.hero.hp -= dmg;
                self.fx.damage(hx, hy, dmg, true);
                self.fx.add_shake(5);
                self.fx.burst(&mut self.rng, hx, hy, (220, 90, 70), 10, '\u{2716}');
                self.push_log(format!("PIEGE ! Vous subissez {} degats.", dmg), BAD);
                if self.hero.hp <= 0 {
                    self.hero.hp = 0;
                    self.die("un piege");
                }
            }
            FeatureKind::Gamble => {
                self.fx.burst(&mut self.rng, hx, hy, (235, 200, 120), 14, '\u{2737}');
                self.fx.label(hx, hy, "PARI", (235, 210, 130));
                if self.rng.chance(0.62) {
                    match self.rng.below(5) {
                        0 => {
                            let g = 30 + self.floor * 6;
                            self.hero.gold += g;
                            self.push_log(format!("Pari gagne : +{} or !", g), GOLD);
                        }
                        1 => {
                            self.hero.rage = self.hero.rage.max(20);
                            self.push_log("Pari gagne : RAGE (+ATQ) !".into(), (235, 120, 90));
                        }
                        2 => {
                            self.hero.shield = self.hero.shield.max(18);
                            self.push_log("Pari gagne : bouclier !".into(), (150, 200, 240));
                        }
                        3 => {
                            self.hero.regen = self.hero.regen.max(20);
                            self.push_log("Pari gagne : regeneration !".into(), (140, 230, 150));
                        }
                        _ => {
                            self.hero.hp = self.hero.max_hp;
                            self.push_log("Pari gagne : soins complets !".into(), GOOD);
                        }
                    }
                } else {
                    match self.rng.below(3) {
                        0 => {
                            let d = 5 + self.floor;
                            self.hero.hp -= d;
                            self.fx.damage(hx, hy, d, true);
                            self.push_log(format!("Pari perdu : {} degats !", d), BAD);
                            if self.hero.hp <= 0 {
                                self.hero.hp = 0;
                                self.die("un pari foireux");
                            }
                        }
                        1 => {
                            self.hero.poison = self.hero.poison.max(6);
                            self.push_log("Pari perdu : empoisonne !".into(), BAD);
                        }
                        _ => {
                            let loss = (self.hero.gold / 3).min(120);
                            self.hero.gold -= loss;
                            self.push_log(format!("Pari perdu : -{} or.", loss), DIM);
                        }
                    }
                }
            }
            FeatureKind::Forge => {
                let cost = 25 + self.floor * 5;
                if self.hero.gold < cost {
                    self.push_log("La forge rare est froide : pas assez d'or.".into(), DIM);
                    return;
                }
                self.hero.gold -= cost;
                self.fx.burst(&mut self.rng, hx, hy, (255, 170, 70), 18, '\u{2737}');
                self.fx.label(hx, hy, "FORGE", (255, 170, 70));
                self.fx.add_shake(4);
                let amt = 2 + self.floor / 6;
                if self.hero.weapon_bonus <= self.hero.armor_bonus {
                    self.hero.weapon_bonus += amt;
                    self.push_log(format!("Forge rare ({} or) : {} ameliore (+{} ATQ).", cost, self.hero.weapon, amt), (255, 200, 110));
                } else {
                    self.hero.armor_bonus += amt;
                    self.push_log(format!("Forge rare ({} or) : {} ameliore (+{} DEF).", cost, self.hero.armor, amt), (255, 200, 110));
                }
                if self.hero.weapon_affix == Affix::None || self.hero.armor_affix == Affix::None {
                    let affixes = [Affix::Fire, Affix::Frost, Affix::Venom, Affix::Shock, Affix::Lifesteal, Affix::Keen, Affix::Regen, Affix::Thorns];
                    let existing = [self.hero.weapon_affix, self.hero.armor_affix, self.hero.ring, self.hero.amulet]
                        .into_iter()
                        .find(|&a| a != Affix::None);
                    let pick = existing.unwrap_or(affixes[self.rng.below(affixes.len())]);
                    if self.hero.weapon_affix == Affix::None {
                        self.hero.weapon_affix = pick;
                        self.push_log(format!("La forge insuffle : arme {}.", pick.label()), (255, 190, 90));
                    } else {
                        self.hero.armor_affix = pick;
                        self.push_log(format!("La forge insuffle : armure {}.", pick.label()), (255, 190, 90));
                    }
                    if let Some(a) = self.hero.set_affix() {
                        self.push_log(format!("SET actif : {} x{} !", a.label(), self.hero.set_bonus()), (255, 215, 120));
                    }
                }
            }
        }
    }

    fn discover(&mut self, name: &str) {
        if !self.discovered.iter().any(|n| n == name) {
            self.discovered.push(name.to_string());
        }
    }

    fn apply_relics(&mut self) {
        let relics = self.unlocked.clone();
        for id in &relics {
            match id.as_str() {
                "first_blood" => self.hero.potions += 1,
                "boss" => self.hero.max_hp += 8,
                "plongeur" => self.hero.might += 2,
                "centurion" => self.hero.guard += 2,
                "abysses" => self.hero.max_hp += 12,
                "legende" => self.hero.might += 2,
                _ => {}
            }
        }
        self.hero.hp = self.hero.max_hp;
        if !relics.is_empty() {
            self.push_log(format!("{} relique(s) active(s).", relics.len()), (200, 170, 90));
        }
    }

    fn unlock(&mut self, id: &str, label: &str) {
        if !self.unlocked.iter().any(|u| u == id) {
            self.unlocked.push(id.to_string());
            self.push_log(format!("SUCCES : {}", label), (255, 215, 120));
            self.fx.label(self.hero.x, self.hero.y, "SUCCES", (255, 215, 120));
        }
    }

    fn hero_def_mult(&self, attacker: Element) -> f32 {
        if attacker == Element::Physical {
            return 1.0;
        }
        let armor = self.hero.armor_element();
        if armor == Element::Physical {
            1.0
        } else if armor == attacker {
            0.7
        } else if armor == attacker.opposite() {
            1.3
        } else {
            1.0
        }
    }

    fn hero_crit(&self) -> f32 {
        self.class.crit_chance()
            + if self.hero.has_affix(Affix::Keen) { 0.12 } else { 0.0 }
            + 0.08 * self.hero.talent_count(Talent::Berserk) as f32
            + 0.04 * self.hero.set_bonus() as f32
    }

    fn cleave_level(&self) -> i32 {
        if self.hero.has_talent(Talent::Bourreau) {
            2
        } else {
            self.class.cleave_level()
        }
    }

    fn bolt_level(&self) -> i32 {
        if self.hero.has_talent(Talent::Arcaniste) {
            self.class.bolt_level().min(4)
        } else {
            self.class.bolt_level()
        }
    }

    fn grant_relic(&mut self) {
        let available: Vec<Relic> = Relic::ALL.iter().copied().filter(|r| !self.hero.has_relic(*r)).collect();
        if available.is_empty() {
            self.hero.gold += 50;
            return;
        }
        let r = available[self.rng.below(available.len())];
        self.hero.relics.push(r);
        if r == Relic::Colossus {
            self.hero.max_hp += 20;
            self.hero.hp += 20;
        }
        self.fx.burst(&mut self.rng, self.hero.x, self.hero.y, (255, 200, 90), 20, '\u{2726}');
        self.fx.label(self.hero.x, self.hero.y, "RELIQUE", (255, 210, 110));
        self.sfx.push(Sound::LevelUp);
        self.push_log(format!("RELIQUE : {} !", r.label()), (255, 210, 120));
    }

    fn grant_talent(&mut self) {
        let available: Vec<Talent> = Talent::ALL.iter().copied().filter(|t| !self.hero.has_talent(*t)).collect();
        if available.is_empty() {
            self.hero.might += 1;
            self.hero.guard += 1;
            self.hero.max_hp += 6;
            self.hero.hp = self.hero.max_hp;
            self.fx.label(self.hero.x, self.hero.y, "MAITRISE", (200, 220, 255));
            self.push_log("Maitrise : tous les talents acquis, +PV/ATQ/DEF.".into(), (200, 220, 255));
            return;
        }
        let t = available[self.rng.below(available.len())];
        self.hero.talents.push(t);
        self.sfx.push(Sound::Talent);
        if t == Talent::Colosse {
            self.hero.max_hp += 12;
            self.hero.hp = self.hero.max_hp;
        }
        self.fx.label(self.hero.x, self.hero.y, "TALENT", (180, 220, 255));
        self.push_log(format!("TALENT : {}", t.label()), (180, 220, 255));
    }

    fn hero_attacks(&mut self, idx: usize) {
        let (mx, my) = (self.monsters[idx].x, self.monsters[idx].y);
        self.lunge = ((mx - self.hero.x).signum(), (my - self.hero.y).signum(), 3);
        let el = self.hero.weapon_element();
        let scol = if el != Element::Physical { el.color() } else { (235, 235, 245) };
        self.fx.burst(&mut self.rng, mx, my, scol, 5, '\u{2215}');
        let cc = self.hero_crit();
        let (dmg, crit) = resolve(self.hero.atk(), self.monsters[idx].def, &mut self.rng, cc);
        self.hit_monster(idx, dmg, crit, el);
    }

    fn monster_turns(&mut self) {
        self.cast_danger.clear();
        let count = self.monsters.len();
        for i in 0..count {
            if i >= self.monsters.len() {
                break;
            }
            let (mx, my) = (self.monsters[i].x, self.monsters[i].y);

            if self.monsters[i].poison > 0 {
                self.monsters[i].poison -= 1;
                if self.monsters[i].hp > 1 {
                    self.monsters[i].hp -= 2;
                    self.fx.damage(mx, my, 2, false);
                }
                if self.monsters[i].poison > 1 && self.rng.chance(0.2) {
                    if let Some(j) = self.monsters.iter().position(|m| (m.x - mx).abs() + (m.y - my).abs() == 1 && m.poison == 0) {
                        self.monsters[j].poison = 3;
                        let (jx, jy) = (self.monsters[j].x, self.monsters[j].y);
                        self.fx.burst(&mut self.rng, jx, jy, (150, 220, 90), 5, '\u{2735}');
                    }
                }
            }
            if self.monsters[i].stun > 0 {
                self.monsters[i].stun -= 1;
                continue;
            }
            if self.monsters[i].cast_cd > 0 {
                self.monsters[i].cast_cd -= 1;
            }

            if self.monsters[i].boss {
                let dnow = (mx - self.hero.x).abs().max((my - self.hero.y).abs());
                if dnow <= 9 {
                    if !self.monsters[i].enraged && self.monsters[i].hp * 2 < self.monsters[i].max_hp {
                        self.monsters[i].enraged = true;
                        self.monsters[i].atk = self.monsters[i].atk * 3 / 2;
                        self.monsters[i].summon_cd = 0;
                        self.boss_wind = 0;
                        self.danger.clear();
                        self.fx.add_shake(8);
                        self.fx.label(mx, my, "ENRAGE", (255, 80, 80));
                        self.push_log("Le boss entre en RAGE !".into(), (255, 90, 90));
                    }
                    if self.boss_wind > 0 {
                        self.boss_wind -= 1;
                        if self.boss_wind == 0 {
                            match self.boss_pending {
                                0 if self.monsters.len() < 40 => self.summon_minions(i),
                                1 => self.boss_charge(i),
                                2 => self.boss_volley(i),
                                3 => self.boss_slam(i),
                                _ => self.boss_erupt(i),
                            }
                            self.danger.clear();
                            self.monsters[i].summon_cd = 6;
                            if matches!(self.phase, Phase::Dead(_)) {
                                return;
                            }
                        }
                        continue;
                    } else if self.monsters[i].summon_cd > 0 {
                        self.monsters[i].summon_cd -= 1;
                    } else if self.monsters[i].hp * 3 < self.monsters[i].max_hp && self.rng.chance(0.3) {
                        self.boss_heal(i);
                        self.monsters[i].summon_cd = 8;
                    } else {
                        let phase2 = self.monsters[i].enraged;
                        let rotation: &[i32] = if phase2 { &[1, 3, 2, 4, 2] } else { &[2, 0, 1, 3] };
                        let pend = rotation[(self.boss_move as usize) % rotation.len()];
                        self.boss_move = self.boss_move.wrapping_add(1);
                        self.boss_pending = pend;
                        self.boss_wind = if phase2 { 2 } else { 3 };
                        self.set_danger(i, pend);
                        let warn = match pend {
                            0 => "INVOCATION",
                            1 => "CHARGE",
                            2 => "SALVE",
                            3 => "FRACAS",
                            _ => "ERUPTION",
                        };
                        self.fx.label(mx, my, "!", (255, 80, 80));
                        self.sfx.push(Sound::BossWarn);
                        self.push_log(format!("Le boss prepare : {} imminent !", warn), (255, 140, 80));
                        continue;
                    }
                }
            }

            if self.monsters[i].heals && self.monsters[i].cast_cd == 0 {
                let mut target = None;
                let mut bestd = i32::MAX;
                for j in 0..self.monsters.len() {
                    if j == i {
                        continue;
                    }
                    let mj = &self.monsters[j];
                    if mj.hp < mj.max_hp {
                        let d = (mj.x - mx).abs().max((mj.y - my).abs());
                        if d <= 4 && d < bestd {
                            bestd = d;
                            target = Some(j);
                        }
                    }
                }
                if let Some(j) = target {
                    let heal = 5 + self.floor / 2;
                    let mj = &mut self.monsters[j];
                    mj.hp = (mj.hp + heal).min(mj.max_hp);
                    let (tx, ty) = (mj.x, mj.y);
                    self.monsters[i].cast_cd = 6;
                    self.fx.burst(&mut self.rng, tx, ty, (120, 235, 180), 8, '\u{2726}');
                    self.fx.label(tx, ty, "+", (120, 235, 180));
                    continue;
                }
            }

            if self.monsters[i].cast_wind > 0 {
                self.monsters[i].cast_wind -= 1;
                if self.monsters[i].cast_wind == 0 {
                    let (tx, ty) = (self.monsters[i].cast_tx, self.monsters[i].cast_ty);
                    self.ranged_attack_at(i, tx, ty);
                    if i < self.monsters.len() {
                        self.monsters[i].cast_cd = 4;
                    }
                    if matches!(self.phase, Phase::Dead(_)) {
                        return;
                    }
                } else {
                    self.cast_danger.push((self.monsters[i].cast_tx, self.monsters[i].cast_ty));
                }
                continue;
            }

            let dist = (mx - self.hero.x).abs().max((my - self.hero.y).abs());
            let manhattan = (mx - self.hero.x).abs() + (my - self.hero.y).abs();

            if manhattan == 1 {
                self.monsters[i].aggro = true;
                if self.monsters[i].bomber {
                    self.detonate(i);
                } else {
                    self.monster_attacks(i);
                }
                if matches!(self.phase, Phase::Dead(_)) {
                    return;
                }
                continue;
            }

            if self.map.is_visible(mx, my) && dist <= AGGRO {
                self.monsters[i].aggro = true;
            }
            if !self.monsters[i].aggro {
                continue;
            }

            let hx = self.hero.x;
            let hy = self.hero.y;

            if self.monsters[i].summoner && self.monsters[i].summon_cd == 0 && self.monsters.len() < 34 {
                self.summon_from(i);
                self.monsters[i].summon_cd = 9;
                continue;
            }
            if self.monsters[i].summon_cd > 0 {
                self.monsters[i].summon_cd -= 1;
            }

            if self.monsters[i].ranged
                && self.monsters[i].cast_cd == 0
                && dist >= 2
                && dist <= 6
                && self.map.line_of_sight(mx, my, hx, hy)
            {
                self.monsters[i].cast_wind = 2;
                self.monsters[i].cast_tx = hx;
                self.monsters[i].cast_ty = hy;
                self.cast_danger.push((hx, hy));
                self.fx.label(mx, my, "!", (235, 150, 60));
                continue;
            }

            let fleeing = self.monsters[i].flees && self.monsters[i].hp * 100 < self.monsters[i].max_hp * 35;
            if fleeing {
                let mut best: Option<(i32, i32, i32)> = None;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (mx + dx, my + dy);
                    if self.map.is_walkable(nx, ny) && self.monster_at(nx, ny).is_none() && !(nx == hx && ny == hy) {
                        let d = (nx - hx).abs() + (ny - hy).abs();
                        if best.map_or(true, |b| d > b.2) {
                            best = Some((nx, ny, d));
                        }
                    }
                }
                if let Some((nx, ny, d)) = best {
                    if d > manhattan {
                        self.monsters[i].x = nx;
                        self.monsters[i].y = ny;
                        continue;
                    }
                }
            }

            let blocked: Vec<(i32, i32)> = self
                .monsters
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, m)| (m.x, m.y))
                .collect();
            if let Some((dx, dy)) = step_toward(&self.map, mx, my, &blocked, |x, y| x == hx && y == hy) {
                let tx = mx + dx;
                let ty = my + dy;
                if !(tx == hx && ty == hy) && self.monster_at(tx, ty).is_none() {
                    self.monsters[i].x = tx;
                    self.monsters[i].y = ty;
                }
            }
        }
    }

    fn set_danger(&mut self, i: usize, pend: i32) {
        let (bx, by) = (self.monsters[i].x, self.monsters[i].y);
        let (hx, hy) = (self.hero.x, self.hero.y);
        self.danger.clear();
        match pend {
            0 => {
                self.danger_color = (150, 90, 200);
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
                    let (x, y) = (bx + dx, by + dy);
                    if self.map.is_walkable(x, y) {
                        self.danger.push((x, y));
                    }
                }
            }
            1 => {
                self.danger_color = (215, 70, 60);
                self.danger.push((hx, hy));
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.danger.push((hx + dx, hy + dy));
                }
            }
            2 => {
                self.danger_color = (235, 140, 60);
                self.danger.push((hx, hy));
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.danger.push((hx + dx, hy + dy));
                }
            }
            3 => {
                self.danger_color = (235, 90, 70);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (x, y) = (bx + dx, by + dy);
                        if self.map.is_walkable(x, y) {
                            self.danger.push((x, y));
                        }
                    }
                }
            }
            _ => {
                self.danger_color = (235, 120, 40);
                self.danger.push((hx, hy));
                for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
                    let (x, y) = (hx + dx, hy + dy);
                    if self.map.is_walkable(x, y) {
                        self.danger.push((x, y));
                    }
                }
            }
        }
    }

    fn boss_slam(&mut self, i: usize) {
        let (bx, by) = (self.monsters[i].x, self.monsters[i].y);
        let color = self.monsters[i].color;
        self.fx.burst(&mut self.rng, bx, by, color, 26, '\u{2737}');
        self.fx.add_shake(7);
        if self.hero_in_danger() {
            let atk = self.monsters[i].atk * 7 / 4;
            let em = self.monsters[i].element;
            let (raw, _) = resolve(atk, self.hero.def(), &mut self.rng, 0.1);
            let dmg = ((raw as f32 * self.hero_def_mult(em)) as i32).max(1);
            self.hero.hp -= dmg;
            self.hero_struck = true;
            self.fx.damage(self.hero.x, self.hero.y, dmg, true);
            self.thorns_reflect(i);
            self.push_log(format!("FRACAS du boss ! ({} degats)", dmg), BAD);
            if self.hero.hp <= 0 {
                self.hero.hp = 0;
                let name = self.monsters[i].name.clone();
                self.die(&name);
            }
        } else {
            self.fx.label(self.hero.x, self.hero.y, "esquive!", (120, 230, 160));
            self.push_log("Le heros echappe au fracas !".into(), GOOD);
        }
    }

    fn boss_erupt(&mut self, _i: usize) {
        let tiles: Vec<(i32, i32)> = self.danger.clone();
        for (x, y) in tiles {
            self.hazard.push((x, y, 5));
            self.fx.burst(&mut self.rng, x, y, (235, 120, 40), 5, '\u{2737}');
        }
        self.fx.add_shake(4);
        self.push_log("Le sol entre en eruption !".into(), (235, 130, 60));
    }

    fn detonate(&mut self, i: usize) {
        let (mx, my) = (self.monsters[i].x, self.monsters[i].y);
        let raw = 8 + self.floor * 2;
        self.monsters.swap_remove(i);
        self.fx.burst(&mut self.rng, mx, my, (255, 140, 50), 24, '\u{2737}');
        self.fx.add_shake(6);
        self.fx.label(self.hero.x, self.hero.y, "BOOM", (255, 140, 50));
        let (dmg, _) = resolve(raw, self.hero.def(), &mut self.rng, 0.0);
        self.hero.hp -= dmg;
        self.hero_struck = true;
        self.fx.damage(self.hero.x, self.hero.y, dmg, true);
        self.push_log(format!("La bombe explose ! ({} degats)", dmg), BAD);
        if self.hero.hp <= 0 {
            self.hero.hp = 0;
            self.die("une bombe vivante");
        }
    }

    fn summon_from(&mut self, i: usize) {
        let (bx, by) = (self.monsters[i].x, self.monsters[i].y);
        let floor = (self.floor / 2).max(1);
        let mut spawned = 0;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            if spawned >= 2 {
                break;
            }
            let (nx, ny) = (bx + dx, by + dy);
            if self.map.is_walkable(nx, ny) && self.monster_at(nx, ny).is_none() && !(nx == self.hero.x && ny == self.hero.y) {
                let mut m = Monster::roll(floor, nx, ny, &mut self.rng);
                m.aggro = true;
                self.monsters.push(m);
                self.fx.burst(&mut self.rng, nx, ny, (200, 110, 230), 6, '\u{2736}');
                spawned += 1;
            }
        }
        if spawned > 0 {
            self.fx.label(bx, by, "INVOCATION", (200, 110, 230));
            self.push_log("L'invocateur appelle des sbires !".into(), (200, 120, 235));
        }
    }

    fn summon_minions(&mut self, i: usize) {
        let (bx, by) = (self.monsters[i].x, self.monsters[i].y);
        let floor = self.floor;
        let mut spawned = 0;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            if spawned >= 2 {
                break;
            }
            let (nx, ny) = (bx + dx, by + dy);
            if self.map.is_walkable(nx, ny)
                && self.monster_at(nx, ny).is_none()
                && !(nx == self.hero.x && ny == self.hero.y)
            {
                let mut m = Monster::roll(floor, nx, ny, &mut self.rng);
                m.aggro = true;
                self.monsters.push(m);
                self.fx.burst(&mut self.rng, nx, ny, (200, 120, 240), 6, '\u{2736}');
                spawned += 1;
            }
        }
        if spawned > 0 {
            self.push_log("Le boss invoque des renforts !".into(), (220, 120, 235));
            self.fx.label(bx, by, "INVOCATION", (220, 120, 235));
        }
    }

    fn hero_in_danger(&self) -> bool {
        let (hx, hy) = (self.hero.x, self.hero.y);
        self.danger.iter().any(|&(x, y)| x == hx && y == hy)
            || self.cast_danger.iter().any(|&(x, y)| x == hx && y == hy)
            || self.hazard.iter().any(|&(x, y, _)| x == hx && y == hy)
    }

    fn blocked_tiles(&self) -> Vec<(i32, i32)> {
        let mut v = self.danger.clone();
        v.extend_from_slice(&self.cast_danger);
        v.extend(self.hazard.iter().map(|&(x, y, _)| (x, y)));
        v
    }

    fn tile_dangerous(&self, x: i32, y: i32) -> bool {
        self.danger.iter().any(|&(a, b)| a == x && b == y)
            || self.cast_danger.iter().any(|&(a, b)| a == x && b == y)
            || self.hazard.iter().any(|&(a, b, _)| a == x && b == y)
    }

    fn boss_charge(&mut self, i: usize) {
        let (bx, by) = (self.monsters[i].x, self.monsters[i].y);
        let color = self.monsters[i].color;
        let target = self.danger.first().copied().unwrap_or((self.hero.x, self.hero.y));
        let land = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .map(|(dx, dy)| (target.0 + dx, target.1 + dy))
            .find(|&(x, y)| self.map.is_walkable(x, y) && self.monster_at(x, y).is_none() && !(x == self.hero.x && y == self.hero.y));
        if let Some((nx, ny)) = land {
            self.fx.projectile(bx, by, nx, ny, '\u{00bb}', color);
            self.monsters[i].x = nx;
            self.monsters[i].y = ny;
        }
        self.fx.add_shake(6);
        if self.hero_in_danger() {
            let atk = self.monsters[i].atk * 3 / 2;
            let em = self.monsters[i].element;
            let (raw, _) = resolve(atk, self.hero.def(), &mut self.rng, 0.1);
            let dmg = ((raw as f32 * self.hero_def_mult(em)) as i32).max(1);
            self.hero.hp -= dmg;
            self.hero_struck = true;
            self.fx.damage(self.hero.x, self.hero.y, dmg, true);
            self.thorns_reflect(i);
            self.push_log(format!("CHARGE du boss ! ({} degats)", dmg), BAD);
            if self.hero.hp <= 0 {
                self.hero.hp = 0;
                let name = self.monsters[i].name.clone();
                self.die(&name);
            }
        } else {
            self.fx.label(self.hero.x, self.hero.y, "esquive!", (120, 230, 160));
            self.push_log("Le heros esquive la charge !".into(), GOOD);
        }
    }

    fn boss_volley(&mut self, i: usize) {
        let (bx, by) = (self.monsters[i].x, self.monsters[i].y);
        let color = self.monsters[i].color;
        let target = self.danger.first().copied().unwrap_or((self.hero.x, self.hero.y));
        self.fx.add_shake(3);
        for _ in 0..3 {
            self.fx.projectile(bx, by, target.0, target.1, '\u{2217}', color);
        }
        if self.hero_in_danger() {
            let em = self.monsters[i].element;
            for _ in 0..3 {
                let (raw, _) = resolve(self.monsters[i].atk * 2 / 3, self.hero.def(), &mut self.rng, 0.05);
                let dmg = ((raw as f32 * self.hero_def_mult(em)) as i32).max(1);
                self.hero.hp -= dmg;
                self.hero_struck = true;
                self.fx.damage(self.hero.x, self.hero.y, dmg, false);
            }
            self.push_log("La salve du boss touche !".into(), BAD);
            if self.hero.hp <= 0 {
                self.hero.hp = 0;
                let name = self.monsters[i].name.clone();
                self.die(&name);
            }
        } else {
            self.fx.label(self.hero.x, self.hero.y, "esquive!", (120, 230, 160));
            self.push_log("Le heros evite la salve !".into(), GOOD);
        }
    }

    fn boss_heal(&mut self, i: usize) {
        let (bx, by) = (self.monsters[i].x, self.monsters[i].y);
        let heal = self.monsters[i].max_hp / 6;
        self.monsters[i].hp = (self.monsters[i].hp + heal).min(self.monsters[i].max_hp);
        self.fx.burst(&mut self.rng, bx, by, (120, 230, 120), 14, '\u{2726}');
        self.fx.label(bx, by, "SOIN", (120, 230, 120));
        self.push_log(format!("Le boss se regenere (+{}).", heal), WARN);
    }

    fn ranged_attack_at(&mut self, idx: usize, tx: i32, ty: i32) {
        let (mx, my) = (self.monsters[idx].x, self.monsters[idx].y);
        let color = self.monsters[idx].color;
        let name = self.monsters[idx].name.clone();
        let caster = self.monsters[idx].glyph == 'w';
        self.fx.projectile(mx, my, tx, ty, '\u{2217}', color);
        if self.hero.x != tx || self.hero.y != ty {
            self.fx.label(self.hero.x, self.hero.y, "esquive!", (120, 230, 160));
            self.push_log(format!("Le {} vous rate.", name), GOOD);
            return;
        }
        if self.hero.has_relic(Relic::Spectral) && self.rng.chance(0.2) {
            self.fx.label(self.hero.x, self.hero.y, "spectre!", (180, 200, 240));
            return;
        }
        let (raw, crit) = resolve(self.monsters[idx].atk - 1, self.hero.def(), &mut self.rng, 0.08);
        let dmg = ((raw as f32 * self.hero_def_mult(self.monsters[idx].element)) as i32).max(1);
        self.hero.hp -= dmg;
        self.hero_struck = true;
        self.fx.damage(self.hero.x, self.hero.y, dmg, crit);
        if caster {
            self.hero.burn = self.hero.burn.max(3);
            self.push_log(format!("Le {} vous embrase ({} degats).", name, dmg), BAD);
        } else {
            self.push_log(format!("Le {} vous tire dessus ({} degats).", name, dmg), BAD);
        }
        if self.hero.hp <= 0 {
            self.hero.hp = 0;
            self.die(&name);
        }
    }

    fn thorns_reflect(&mut self, idx: usize) {
        if !self.hero.has_affix(Affix::Thorns) || idx >= self.monsters.len() {
            return;
        }
        if self.monsters[idx].hp > 1 {
            self.monsters[idx].hp -= 3;
            let (mx, my) = (self.monsters[idx].x, self.monsters[idx].y);
            self.fx.damage(mx, my, 3, false);
        }
    }

    fn monster_attacks(&mut self, idx: usize) {
        if self.hero.has_relic(Relic::Spectral) && self.rng.chance(0.2) {
            self.fx.label(self.hero.x, self.hero.y, "spectre!", (180, 200, 240));
            return;
        }
        let (raw, crit) = resolve(self.monsters[idx].atk, self.hero.def(), &mut self.rng, 0.08);
        let dmg = ((raw as f32 * self.hero_def_mult(self.monsters[idx].element)) as i32).max(1);
        let name = self.monsters[idx].name.clone();
        self.hero.hp -= dmg;
        self.thorns_reflect(idx);
        self.hero_struck = true;
        self.flashes.push((self.hero.x, self.hero.y, (255, 70, 70), 2));
        self.fx.damage(self.hero.x, self.hero.y, dmg, crit);
        self.fx.add_shake(if crit { 6 } else { 3 });
        self.fx.combo = 0;
        if self.hero.hp <= 0 {
            self.hero.hp = 0;
            self.push_log(format!("Le {} vous porte un coup fatal ({}).", name, dmg), BAD);
            self.die(&name);
        } else {
            self.push_log(format!("Le {} vous blesse ({} degats).", name, dmg), BAD);
        }
    }

    fn die(&mut self, cause: &str) {
        self.best_floor = self.best_floor.max(self.floor);
        self.best_gold = self.best_gold.max(self.hero.gold);
        let base = self.floor * 1000 + self.hero.gold + self.hero.kills * 10;
        let score = (base as f32 * (1.0 + 0.25 * self.ascension as f32)) as i32;
        self.last_score = score;
        self.high_scores.push(score);
        self.high_scores.sort_by(|a, b| b.cmp(a));
        self.high_scores.truncate(5);
        self.last_cause = cause.to_string();
        self.death_quip = death_quip(cause, &mut self.rng);
        self.push_log(self.death_quip.clone(), (235, 180, 90));
        self.push_log(
            format!(
                "VOUS ETES MORT. Etage {}, niveau {}, {} or, {} elimines.",
                self.floor, self.hero.level, self.hero.gold, self.hero.kills
            ),
            BAD,
        );
        self.sfx.push(Sound::Death);
        self.phase = Phase::Dead(DEATH_HOLD);
    }

    fn roll_biome(&mut self) -> Biome {
        let f = self.floor;
        let weights: [(Biome, i32); 5] = [
            (Biome::Caverns, (20 - f).max(2)),
            (Biome::Catacombs, (16 - (f - 5).abs()).max(2)),
            (Biome::Frostvault, (13 - (f - 11).abs()).max(1)),
            (Biome::Emberdepths, (13 - (f - 16).abs()).max(1)),
            (Biome::Abyss, (f - 6).max(1)),
        ];
        let total: i32 = weights.iter().map(|(_, w)| w).sum();
        let mut roll = self.rng.below(total.max(1) as usize) as i32;
        for (b, w) in weights {
            roll -= w;
            if roll < 0 {
                return b;
            }
        }
        Biome::Caverns
    }

    fn roll_room(&mut self) -> RoomKind {
        match self.rng.below(100) {
            0..=38 => RoomKind::Standard,
            39..=58 => RoomKind::Treasure,
            59..=76 => RoomKind::Challenge,
            77..=89 => RoomKind::Warren,
            _ => RoomKind::Rest,
        }
    }

    fn room_appeal(&self, room: RoomKind) -> i32 {
        match self.style {
            Playstyle::Completionist => match room {
                RoomKind::Treasure => 4,
                RoomKind::Challenge => 3,
                RoomKind::Warren => 2,
                RoomKind::Standard => 1,
                RoomKind::Rest => 0,
            },
            Playstyle::Combatant => match room {
                RoomKind::Challenge => 4,
                RoomKind::Warren => 3,
                RoomKind::Treasure => 1,
                RoomKind::Standard => 1,
                RoomKind::Rest => 0,
            },
            Playstyle::Rusher => match room {
                RoomKind::Rest => 3,
                RoomKind::Standard => 2,
                RoomKind::Treasure => 1,
                RoomKind::Warren => 0,
                RoomKind::Challenge => 0,
            },
        }
    }

    fn choose_branch(&mut self) {
        let n = 2 + self.rng.below(2);
        let candidates: Vec<(Biome, RoomKind)> = (0..n).map(|_| (self.roll_biome(), self.roll_room())).collect();
        let hurt = self.hp_fraction() < 0.4;
        let mut best = 0usize;
        let mut best_score = i32::MIN;
        for (i, &(_, room)) in candidates.iter().enumerate() {
            let mut score = self.room_appeal(room) * 2;
            if hurt && room == RoomKind::Rest {
                score += 7;
            }
            if hurt && room == RoomKind::Challenge {
                score -= 3;
            }
            score += self.rng.below(2) as i32;
            if score > best_score {
                best_score = score;
                best = i;
            }
        }
        let (biome, room) = candidates[best];
        self.biome = biome;
        self.room_kind = room;
        let mut line = String::from("Voie : ");
        for (i, (b, r)) in candidates.iter().enumerate() {
            if i == best {
                let _ = std::fmt::Write::write_fmt(&mut line, format_args!("[{} {}] ", b.label(), r.label()));
            } else {
                let _ = std::fmt::Write::write_fmt(&mut line, format_args!("{} {} · ", b.label(), r.label()));
            }
        }
        self.push_log(line, (170, 205, 150));
    }

    fn descend(&mut self) {
        if self.objective == Objective::Swift && !self.objective_done && self.floor_turns <= self.objective_target {
            self.complete_objective();
        }
        self.floor += 1;
        self.choose_branch();
        self.best_floor = self.best_floor.max(self.floor);
        if self.floor >= 10 {
            self.unlock("plongeur", "Plongeur - etage 10");
        }
        if self.floor >= 20 {
            self.unlock("abysses", "Maitre des abysses - etage 20");
        }
        self.populate_floor(false);
        self.fx.begin_transition(self.floor);
        self.sfx.push(Sound::Descend);
    }
}

const QUIPS_TRAP: &[&str] = &[
    "a glisse sur un caillou. RIP.",
    "s'est cogne a un coin de mur.",
    "a marche sur le piege comme un vrai bleu.",
    "Indiana Jones n'aurait pas fait pire.",
    "le sol etait un sale menteur.",
    "mort idiote +100 (succes non debloque).",
    "le piege etait pourtant bien visible...",
    "a teste le sol. le sol a gagne.",
    "Home Alone, version donjon.",
    "victime d'un game design hostile.",
    "le tutoriel n'avait pas prevu ca.",
    "a découvert la gravite a ses depens.",
    "RIP. cause : negligence flagrante.",
    "a fait *clic*. mauvais *clic*.",
];

const QUIPS_DOT: &[&str] = &[
    "est parti en fumee, tel Ace a Marineford.",
    "a serieusement sous-estime le poison.",
    "fallait lire l'etiquette : non comestible.",
    "consume de l'interieur. tres punk.",
    "a oublie que les DoT, ca tue aussi.",
    "cuit a point. dommage.",
    "a confondu antidote et apero.",
    "Zoro aurait coupe le poison en deux.",
    "brule lentement, comme ses espoirs.",
    "la regen ? quelle regen ?",
    "intoxique. cinq etoiles, reviendrai pas.",
];

const QUIPS_MONSTER: &[&str] = &[
    "« je suis devenu trop confiant. »",
    "a oublie d'esquiver. Gon est decu.",
    "Continue ? 9... 8... 7...",
    "la hype etait pourtant reelle.",
    "skill issue, disent les anciens.",
    "respawn dans une autre vie.",
    "a tank avec son visage.",
    "« c'etait quoi son cooldown deja ? »",
    "Aizen avait tout prevu. encore.",
    "no hit run : echec a l'etage actuel.",
    "a oublie de boire sa potion. classique.",
    "GG WP, dit le monstre poliment.",
    "victoire morale (la seule).",
    "a confondu courage et imprudence.",
    "« je le tenais pourtant... »",
    "meme Kirito serait mort la.",
    "a clique trop vite sur 'foncer'.",
    "leeroy jenkins serait fier.",
];

const QUIPS_MIMIC: &[&str] = &[
    "encore un mimic. meme Frieren se fait avoir.",
    "le coffre a mordu. quelle surprise.",
    "« c'etait un piege ! » — tout le monde",
    "a appris la mefiance trop tard.",
    "le loot le plus cher de sa vie.",
];

const QUIPS_BOSS: &[&str] = &[
    "vaincu par le boss. l'arc narratif s'arrete la.",
    "le boss avait une phase 2. evidemment.",
    "pas assez stuff pour ce DPS check.",
    "a vu le pattern... une fois de trop.",
    "le boss envoie ses condoleances.",
];

fn death_quip(cause: &str, rng: &mut Rng) -> String {
    if cause.contains("mimic") {
        return QUIPS_MIMIC[rng.below(QUIPS_MIMIC.len())].to_string();
    }
    match cause {
        "un piege" => QUIPS_TRAP[rng.below(QUIPS_TRAP.len())].to_string(),
        "ses blessures" => QUIPS_DOT[rng.below(QUIPS_DOT.len())].to_string(),
        _ => {
            let is_boss = ["Gobelin Roi", "Liche", "Golem", "Hydre", "Archidemon", "Dragon"]
                .iter()
                .any(|b| cause.contains(b));
            if is_boss {
                QUIPS_BOSS[rng.below(QUIPS_BOSS.len())].to_string()
            } else if rng.chance(0.5) {
                format!("nerf le {} stp.", cause)
            } else {
                QUIPS_MONSTER[rng.below(QUIPS_MONSTER.len())].to_string()
            }
        }
    }
}

fn resolve(atk: i32, def: i32, rng: &mut Rng, crit_chance: f32) -> (i32, bool) {
    let crit = rng.chance(crit_chance);
    let mut dmg = (atk - def + rng.between(-1, 3)).max(1);
    if crit {
        dmg *= 2;
    }
    (dmg, crit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_autoplay_is_stable() {
        let mut game = Game::new(80, 30, 0xDEAD_BEEF);
        let mut sink: Vec<u8> = Vec::new();
        for _ in 0..40_000 {
            game.update();
            assert!(game.hero.hp <= game.hero.max_hp);
            assert!(game.floor >= 1 && game.runs >= 1);
            for m in &game.monsters {
                assert!(game.map.in_bounds(m.x, m.y));
            }
            assert!(game.map.in_bounds(game.hero.x, game.hero.y));
            sink.clear();
            crate::render::draw(&game, 80 + super::super_panel(), 30, false, "1x", false, 4, &mut sink);
        }
        assert!(game.best_floor >= 1);
    }
}

#[cfg(test)]
fn super_panel() -> i32 {
    34
}

