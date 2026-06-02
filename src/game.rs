use crate::ai::{nearest_goal, step_toward};
use crate::entity::{Affix, Color, Element, Feature, FeatureKind, Hero, HeroClass, Item, ItemKind, Merchant, Monster, Pet, ScrollKind, Talent};
use crate::fx::Fx;
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
    pub event: FloorEvent,
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
        Game::new_with(map_w, map_h, seed, None, Playstyle::Completionist, 1.0, "Normal".to_string(), Boon::None)
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
    ) -> Self {
        let mut rng = Rng::from_seed(seed);
        let class = start_class.unwrap_or_else(|| HeroClass::pick(&mut rng));
        let map = Map::generate(map_w, map_h, &mut rng);
        let (hx, hy) = map.spawn_point();
        let mut hero = Hero::fresh(hx, hy);
        class.apply(&mut hero);
        boon.apply(&mut hero);
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
            event: FloorEvent::Calm,
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
            fx: Fx::default(),
            hero_struck: false,
            merchant: None,
            forced_purchase: None,
            shop_timer: 0,
            shop_preview: false,
            merchant_votes: [0; 7],
            top_voters: Vec::new(),
            hud_note: String::new(),
            explore_target: None,
            map_w,
            map_h,
            rng,
        };
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
        if self.event == FloorEvent::Treasure {
            monster_count = (monster_count / 2).max(2);
            item_count += 4;
        }

        for _ in 0..monster_count {
            if floor_tiles.is_empty() {
                break;
            }
            let pick = self.rng.below(floor_tiles.len());
            let (x, y) = floor_tiles.swap_remove(pick);
            self.monsters.push(Monster::roll(self.floor, x, y, &mut self.rng));
        }
        let elite_chance = (0.08 + self.floor as f32 * 0.012).min(0.35);
        let promote: Vec<bool> = (0..self.monsters.len()).map(|_| self.rng.chance(elite_chance)).collect();
        for (i, m) in self.monsters.iter_mut().enumerate() {
            if promote[i] {
                m.promote();
            }
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
        let chests = 1 + self.rng.below(2) + if self.event == FloorEvent::Treasure { 4 } else { 0 };
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
                self.monsters.push(Monster::boss(self.floor, spot.0, spot.1));
            }
        }

        self.merchant = None;
        if self.floor >= 2 && self.rng.chance(0.4) && !floor_tiles.is_empty() {
            let pick = self.rng.below(floor_tiles.len());
            let (x, y) = floor_tiles.swap_remove(pick);
            self.merchant = Some(Merchant::roll(self.floor, x, y, &mut self.rng, self.class.weapon_class(), self.class.armor_class()));
        }

        if (self.diff_mult - 1.0).abs() > 0.01 {
            let mult = self.diff_mult;
            for m in self.monsters.iter_mut() {
                m.hp = ((m.hp as f32 * mult) as i32).max(1);
                m.max_hp = m.hp;
                m.atk = ((m.atk as f32 * mult) as i32).max(1);
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

        self.explore_target = None;
        let fr = self.fov_radius();
        self.map.compute_fov(hx, hy, fr);
        if first {
            self.push_log(format!("Vous entrez dans le donjon. Etage {}.", self.floor), WHITE);
        } else {
            self.push_log(format!("Vous descendez vers l'etage {}.", self.floor), MAGIC);
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

    pub fn update(&mut self) {
        self.hero_struck = false;
        self.fx.tick();
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
        let (px, py, patk) = match &self.pet {
            Some(p) => (p.x, p.y, p.atk),
            None => return,
        };
        if let Some(j) = self.monsters.iter().position(|m| (m.x - px).abs() + (m.y - py).abs() == 1) {
            let (dmg, crit) = resolve(patk, self.monsters[j].def, &mut self.rng, 0.1);
            self.hit_monster(j, dmg, crit, Element::Physical);
            return;
        }
        let target = self
            .monsters
            .iter()
            .filter(|m| self.map.is_visible(m.x, m.y))
            .min_by_key(|m| (m.x - px).abs() + (m.y - py).abs())
            .map(|m| (m.x, m.y));
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
        if self.hero.regen > 0 {
            self.hero.regen -= 1;
            self.hero.hp = (self.hero.hp + 2).min(self.hero.max_hp);
        }
        if (self.hero.has_affix(Affix::Regen) || self.hero.has_talent(Talent::Regen)) && self.hero.hp < self.hero.max_hp {
            self.hero.hp += 1;
        }
        if self.event == FloorEvent::Inferno && self.rng.chance(0.06) {
            self.hero.burn = self.hero.burn.max(2);
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

    fn start_new_run(&mut self) {
        self.runs += 1;
        self.floor = 1;
        let (hx, hy) = self.map.spawn_point();
        self.class = self.start_class.unwrap_or_else(|| HeroClass::pick(&mut self.rng));
        self.hero = Hero::fresh(hx, hy);
        self.class.apply(&mut self.hero);
        self.boon.apply(&mut self.hero);
        self.pet = None;
        self.apply_relics();
        self.phase = Phase::Playing;
        self.push_log(format!("--- Run #{} : {} ---", self.runs, self.class.label()), WHITE);
        self.populate_floor(true);
    }

    fn hero_turn(&mut self) {
        if self.act_dodge() {
            return;
        }
        if self.act_heal() {
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
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            let (nx, ny) = (hx + dx, hy + dy);
            if self.map.is_walkable(nx, ny)
                && self.monster_at(nx, ny).is_none()
                && !self.tile_dangerous(nx, ny)
                && !self.merchant.as_ref().is_some_and(|m| m.x == nx && m.y == ny)
            {
                self.hero.x = nx;
                self.hero.y = ny;
                let fr = self.fov_radius();
                self.map.compute_fov(nx, ny, fr);
                self.last_action = "esquive";
                self.pickup_here();
                return true;
            }
        }
        false
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
        let dmg = ((base_dmg as f32 * mult).round() as i32).max(1);
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
        if crit {
            self.fx.add_shake(3);
        }
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
        }
        if self.monsters[idx].hp <= 0 {
            let m = self.monsters.swap_remove(idx);
            self.hero.kills += 1;
            self.total_kills += 1;
            self.hero.gold += m.gold_reward;
            self.fx.bump_combo();
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
                self.fx.burst(&mut self.rng, self.hero.x, self.hero.y, (255, 225, 120), 16, '\u{2022}');
                self.fx.label(self.hero.x, self.hero.y, "NIVEAU+", (255, 225, 120));
                self.push_log(
                    format!("NIVEAU {} ! PV/ATQ/DEF augmentes, soins complets.", self.hero.level),
                    WARN,
                );
                self.grant_talent();
            }
        } else {
            self.push_log(format!("Vous touchez le {} ({} degats).", name, dmg), WHITE);
        }
    }

    fn act_heal(&mut self) -> bool {
        if self.hero.hp * 3 < self.hero.max_hp && self.hero.potions > 0 {
            self.last_action = "potion";
            self.hero.potions -= 1;
            let heal = (self.hero.max_hp / 2).max(10);
            self.hero.hp = (self.hero.hp + heal).min(self.hero.max_hp);
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
            let open: [(i32, i32); 0] = [];
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
            let open: [(i32, i32); 0] = [];
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
            let open: [(i32, i32); 0] = [];
            if let Some((dx, dy)) = step_toward(&self.map, hx, hy, &open, |x, y| x == tx && y == ty) {
                self.last_action = "autel";
                self.move_or_act(dx, dy);
                return true;
            }
        }
        false
    }

    fn act_merchant(&mut self) -> bool {
        if !self.merchant_wants_trade() {
            return false;
        }
        if let Some((mx, my)) = self.merchant.as_ref().map(|m| (m.x, m.y)) {
            if self.map.is_explored(mx, my) {
                let open: [(i32, i32); 0] = [];
                if let Some((dx, dy)) =
                    step_toward(&self.map, self.hero.x, self.hero.y, &open, |x, y| x == mx && y == my)
                {
                    self.last_action = "marchand";
                    self.move_or_act(dx, dy);
                    return true;
                }
            }
        }
        false
    }

    fn act_explore(&mut self) -> bool {
        let open: [(i32, i32); 0] = [];
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
        let open: [(i32, i32); 0] = [];
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
                self.pet = Some(Pet::new(self.floor, spot.0, spot.1));
                self.fx.burst(&mut self.rng, hx, hy, (120, 230, 180), 14, '\u{2726}');
                self.fx.label(hx, hy, "FAMILIER", (120, 230, 180));
                self.push_log("Un familier se joint a vous !".into(), (120, 230, 180));
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

    fn hero_crit(&self) -> f32 {
        self.class.crit_chance()
            + if self.hero.has_affix(Affix::Keen) { 0.12 } else { 0.0 }
            + 0.08 * self.hero.talent_count(Talent::Berserk) as f32
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

    fn grant_talent(&mut self) {
        let t = Talent::ALL[self.rng.below(Talent::ALL.len())];
        self.hero.talents.push(t);
        if t == Talent::Colosse {
            self.hero.max_hp += 12;
            self.hero.hp = self.hero.max_hp;
        }
        self.fx.label(self.hero.x, self.hero.y, "TALENT", (180, 220, 255));
        self.push_log(format!("TALENT : {}", t.label()), (180, 220, 255));
    }

    fn hero_attacks(&mut self, idx: usize) {
        let cc = self.hero_crit();
        let (dmg, crit) = resolve(self.hero.atk(), self.monsters[idx].def, &mut self.rng, cc);
        let el = self.hero.weapon_element();
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
            }
            if self.monsters[i].stun > 0 {
                self.monsters[i].stun -= 1;
                continue;
            }

            if self.monsters[i].boss {
                let dnow = (mx - self.hero.x).abs().max((my - self.hero.y).abs());
                if dnow <= 9 {
                    if self.boss_wind > 0 {
                        self.boss_wind -= 1;
                        if self.boss_wind == 0 {
                            match self.boss_pending {
                                0 if self.monsters.len() < 40 => self.summon_minions(i),
                                1 => self.boss_charge(i),
                                _ => self.boss_volley(i),
                            }
                            self.danger.clear();
                            self.monsters[i].summon_cd = 10;
                            if matches!(self.phase, Phase::Dead(_)) {
                                return;
                            }
                        }
                        continue;
                    } else if self.monsters[i].summon_cd > 0 {
                        self.monsters[i].summon_cd -= 1;
                    } else {
                        let pend = self.rng.below(4) as i32;
                        if pend == 3 {
                            self.boss_heal(i);
                            self.monsters[i].summon_cd = 10;
                        } else {
                            self.boss_pending = pend;
                            self.boss_wind = 3;
                            self.set_danger(i, pend);
                            let warn = match pend {
                                0 => "INVOCATION imminente !",
                                1 => "CHARGE imminente !",
                                _ => "SALVE imminente !",
                            };
                            self.fx.label(mx, my, "!", (255, 80, 80));
                            self.push_log(format!("Le boss prepare : {}", warn), (255, 140, 80));
                            continue;
                        }
                    }
                }
            }

            if self.monsters[i].cast_wind > 0 {
                self.monsters[i].cast_wind -= 1;
                if self.monsters[i].cast_wind == 0 {
                    let (tx, ty) = (self.monsters[i].cast_tx, self.monsters[i].cast_ty);
                    self.ranged_attack_at(i, tx, ty);
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
                self.monster_attacks(i);
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

            if self.monsters[i].ranged
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
            _ => {
                self.danger_color = (235, 140, 60);
                self.danger.push((hx, hy));
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.danger.push((hx + dx, hy + dy));
                }
            }
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
    }

    fn tile_dangerous(&self, x: i32, y: i32) -> bool {
        self.danger.iter().any(|&(a, b)| a == x && b == y)
            || self.cast_danger.iter().any(|&(a, b)| a == x && b == y)
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
            let (dmg, _) = resolve(atk, self.hero.def(), &mut self.rng, 0.1);
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
            for _ in 0..3 {
                let (dmg, _) = resolve(self.monsters[i].atk * 2 / 3, self.hero.def(), &mut self.rng, 0.05);
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
        let (dmg, crit) = resolve(self.monsters[idx].atk - 1, self.hero.def(), &mut self.rng, 0.08);
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
        let (dmg, crit) = resolve(self.monsters[idx].atk, self.hero.def(), &mut self.rng, 0.08);
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
        let score = self.floor * 1000 + self.hero.gold + self.hero.kills * 10;
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
        self.phase = Phase::Dead(DEATH_HOLD);
    }

    fn descend(&mut self) {
        if self.objective == Objective::Swift && !self.objective_done && self.floor_turns <= self.objective_target {
            self.complete_objective();
        }
        self.floor += 1;
        self.best_floor = self.best_floor.max(self.floor);
        if self.floor >= 10 {
            self.unlock("plongeur", "Plongeur - etage 10");
        }
        if self.floor >= 20 {
            self.unlock("abysses", "Maitre des abysses - etage 20");
        }
        self.populate_floor(false);
        self.fx.begin_transition(self.floor);
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
            crate::render::draw(&game, 80 + super::super_panel(), 30, false, "1x", &mut sink);
        }
        assert!(game.best_floor >= 1);
    }
}

#[cfg(test)]
fn super_panel() -> i32 {
    34
}
