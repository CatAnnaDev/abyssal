use crate::rng::Rng;
use serde::{Deserialize, Serialize};

pub type Color = (u8, u8, u8);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum WeaponClass {
    Light,
    Heavy,
    Staff,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ArmorClass {
    Cloth,
    Leather,
    Plate,
}

impl WeaponClass {
    pub fn label(self) -> &'static str {
        match self {
            WeaponClass::Light => "leger",
            WeaponClass::Heavy => "lourd",
            WeaponClass::Staff => "magique",
        }
    }
}

impl ArmorClass {
    pub fn label(self) -> &'static str {
        match self {
            ArmorClass::Cloth => "tissu",
            ArmorClass::Leather => "cuir",
            ArmorClass::Plate => "plaque",
        }
    }
}

const LIGHT_WEAPONS: &[(&str, i32)] = &[("dague", 2), ("stylet", 4), ("kriss", 6), ("lames jumelles", 9), ("croc d'ombre", 13)];
const HEAVY_WEAPONS: &[(&str, i32)] = &[("epee courte", 3), ("hache", 5), ("masse d'armes", 7), ("epee large", 10), ("fleau ardent", 14)];
const STAFF_WEAPONS: &[(&str, i32)] = &[("baton", 2), ("baton runique", 4), ("sceptre", 7), ("baton de givre", 10), ("baton du chaos", 14)];
const CLOTH_ARMORS: &[(&str, i32)] = &[("tunique", 1), ("robe", 3), ("robe runique", 5), ("manteau arcanique", 8)];
const LEATHER_ARMORS: &[(&str, i32)] = &[("armure de cuir", 2), ("cuir cloute", 4), ("cuir renforce", 6), ("cape d'ombre", 9)];
const PLATE_ARMORS: &[(&str, i32)] = &[("cotte de mailles", 2), ("plastron", 4), ("armure de plates", 6), ("egide drakonienne", 9)];

fn weapons_for(c: WeaponClass) -> &'static [(&'static str, i32)] {
    match c {
        WeaponClass::Light => LIGHT_WEAPONS,
        WeaponClass::Heavy => HEAVY_WEAPONS,
        WeaponClass::Staff => STAFF_WEAPONS,
    }
}

fn armors_for(c: ArmorClass) -> &'static [(&'static str, i32)] {
    match c {
        ArmorClass::Cloth => CLOTH_ARMORS,
        ArmorClass::Leather => LEATHER_ARMORS,
        ArmorClass::Plate => PLATE_ARMORS,
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum HeroClass {
    Warrior,
    Rogue,
    Mage,
}

impl HeroClass {
    pub fn label(self) -> &'static str {
        match self {
            HeroClass::Warrior => "Guerrier",
            HeroClass::Rogue => "Voleur",
            HeroClass::Mage => "Mage",
        }
    }

    pub fn pick(rng: &mut Rng) -> HeroClass {
        match rng.below(3) {
            0 => HeroClass::Warrior,
            1 => HeroClass::Rogue,
            _ => HeroClass::Mage,
        }
    }

    pub fn crit_chance(self) -> f32 {
        match self {
            HeroClass::Warrior => 0.10,
            HeroClass::Rogue => 0.28,
            HeroClass::Mage => 0.14,
        }
    }

    pub fn cleave_level(self) -> i32 {
        match self {
            HeroClass::Warrior => 2,
            _ => 999,
        }
    }

    pub fn bolt_level(self) -> i32 {
        match self {
            HeroClass::Mage => 1,
            HeroClass::Rogue => 6,
            HeroClass::Warrior => 999,
        }
    }

    pub fn bleeds(self) -> bool {
        matches!(self, HeroClass::Rogue)
    }

    pub fn weapon_class(self) -> WeaponClass {
        match self {
            HeroClass::Warrior => WeaponClass::Heavy,
            HeroClass::Rogue => WeaponClass::Light,
            HeroClass::Mage => WeaponClass::Staff,
        }
    }

    pub fn armor_class(self) -> ArmorClass {
        match self {
            HeroClass::Warrior => ArmorClass::Plate,
            HeroClass::Rogue => ArmorClass::Leather,
            HeroClass::Mage => ArmorClass::Cloth,
        }
    }

    pub fn apply(self, h: &mut Hero) {
        match self {
            HeroClass::Warrior => {
                h.max_hp += 14;
                h.guard += 2;
            }
            HeroClass::Rogue => {
                h.might += 3;
            }
            HeroClass::Mage => {
                h.might += 4;
                h.max_hp -= 6;
            }
        }
        h.weapon = weapons_for(self.weapon_class())[0].0.into();
        h.armor = armors_for(self.armor_class())[0].0.into();
        h.hp = h.max_hp;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ScrollKind {
    Fireball,
    Teleport,
    Freeze,
}

impl ScrollKind {
    pub fn label(self) -> &'static str {
        match self {
            ScrollKind::Fireball => "boule de feu",
            ScrollKind::Teleport => "teleportation",
            ScrollKind::Freeze => "gel de zone",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Talent {
    Berserk,
    Sangsue,
    Colosse,
    Bourreau,
    Arcaniste,
    Regen,
}

impl Talent {
    pub const ALL: [Talent; 6] = [
        Talent::Berserk,
        Talent::Sangsue,
        Talent::Colosse,
        Talent::Bourreau,
        Talent::Arcaniste,
        Talent::Regen,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Talent::Berserk => "Berserk (+crit)",
            Talent::Sangsue => "Sangsue (vol de vie)",
            Talent::Colosse => "Colosse (+PV max)",
            Talent::Bourreau => "Bourreau (cleave)",
            Talent::Arcaniste => "Arcaniste (eclair)",
            Talent::Regen => "Regeneration",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Physical,
    Fire,
    Ice,
    Poison,
    Lightning,
}

impl Element {
    pub fn color(self) -> Color {
        match self {
            Element::Physical => (235, 235, 235),
            Element::Fire => (255, 140, 60),
            Element::Ice => (140, 210, 255),
            Element::Poison => (150, 220, 90),
            Element::Lightning => (245, 230, 90),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Element::Physical => "physique",
            Element::Fire => "feu",
            Element::Ice => "glace",
            Element::Poison => "poison",
            Element::Lightning => "foudre",
        }
    }

    pub fn opposite(self) -> Element {
        match self {
            Element::Fire => Element::Ice,
            Element::Ice => Element::Fire,
            Element::Poison => Element::Lightning,
            Element::Lightning => Element::Poison,
            Element::Physical => Element::Physical,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Affix {
    None,
    Fire,
    Frost,
    Venom,
    Shock,
    Lifesteal,
    Keen,
    Regen,
    Thorns,
}

impl Affix {
    pub fn label(self) -> &'static str {
        match self {
            Affix::None => "",
            Affix::Fire => "ardent",
            Affix::Frost => "glacial",
            Affix::Venom => "venimeux",
            Affix::Shock => "foudroyant",
            Affix::Lifesteal => "vampirique",
            Affix::Keen => "affute",
            Affix::Regen => "vivifiant",
            Affix::Thorns => "epineux",
        }
    }

    pub fn element(self) -> Element {
        match self {
            Affix::Fire => Element::Fire,
            Affix::Frost => Element::Ice,
            Affix::Venom => Element::Poison,
            Affix::Shock => Element::Lightning,
            _ => Element::Physical,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Hero {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub might: i32,
    pub guard: i32,
    pub weapon_bonus: i32,
    pub armor_bonus: i32,
    pub level: i32,
    pub xp: i32,
    pub xp_next: i32,
    pub potions: i32,
    pub gold: i32,
    pub kills: i32,
    pub weapon: String,
    pub armor: String,
    pub burn: i32,
    pub poison: i32,
    pub regen: i32,
    pub shield: i32,
    #[serde(default)]
    pub rage: i32,
    pub bolt_cd: i32,
    pub weapon_affix: Affix,
    pub armor_affix: Affix,
    pub ring: Affix,
    pub ring_bonus: i32,
    pub amulet: Affix,
    pub amulet_bonus: i32,
    pub scrolls: Vec<ScrollKind>,
    pub talents: Vec<Talent>,
    #[serde(default)]
    pub ability_cd: i32,
}

impl Hero {
    pub fn fresh(x: i32, y: i32) -> Self {
        Hero {
            x,
            y,
            hp: 30,
            max_hp: 30,
            might: 5,
            guard: 1,
            weapon_bonus: 0,
            armor_bonus: 0,
            level: 1,
            xp: 0,
            xp_next: 12,
            potions: 1,
            gold: 0,
            kills: 0,
            weapon: "dague".into(),
            armor: "tunique".into(),
            burn: 0,
            poison: 0,
            regen: 0,
            shield: 0,
            rage: 0,
            bolt_cd: 0,
            weapon_affix: Affix::None,
            armor_affix: Affix::None,
            ring: Affix::None,
            ring_bonus: 0,
            amulet: Affix::None,
            amulet_bonus: 0,
            scrolls: Vec::new(),
            talents: Vec::new(),
            ability_cd: 0,
        }
    }

    pub fn has_talent(&self, t: Talent) -> bool {
        self.talents.contains(&t)
    }

    pub fn talent_count(&self, t: Talent) -> usize {
        self.talents.iter().filter(|&&x| x == t).count()
    }

    pub fn set_bonus(&self) -> i32 {
        let slots = [self.weapon_affix, self.armor_affix, self.ring, self.amulet];
        let mut best = 0;
        for a in [Affix::Fire, Affix::Frost, Affix::Venom, Affix::Shock, Affix::Lifesteal, Affix::Keen, Affix::Regen, Affix::Thorns] {
            let n = slots.iter().filter(|&&s| s == a).count() as i32;
            if n > best {
                best = n;
            }
        }
        if best >= 2 {
            best
        } else {
            0
        }
    }

    pub fn set_affix(&self) -> Option<Affix> {
        let slots = [self.weapon_affix, self.armor_affix, self.ring, self.amulet];
        for a in [Affix::Fire, Affix::Frost, Affix::Venom, Affix::Shock, Affix::Lifesteal, Affix::Keen, Affix::Regen, Affix::Thorns] {
            if slots.iter().filter(|&&s| s == a).count() >= 2 {
                return Some(a);
            }
        }
        None
    }

    pub fn atk(&self) -> i32 {
        self.might + self.weapon_bonus + self.ring_bonus + self.set_bonus() + if self.rage > 0 { 6 } else { 0 }
    }

    pub fn def(&self) -> i32 {
        self.guard + self.armor_bonus + self.amulet_bonus + self.set_bonus() + if self.shield > 0 { 6 } else { 0 }
    }

    pub fn has_affix(&self, a: Affix) -> bool {
        self.weapon_affix == a || self.armor_affix == a || self.ring == a || self.amulet == a
    }

    pub fn weapon_element(&self) -> Element {
        let e = self.weapon_affix.element();
        if e != Element::Physical {
            e
        } else {
            self.ring.element()
        }
    }

    pub fn gain_xp(&mut self, amount: i32) -> bool {
        self.xp += amount;
        if self.xp >= self.xp_next {
            self.xp -= self.xp_next;
            self.level += 1;
            self.xp_next = self.xp_next + self.xp_next / 2 + 6;
            self.max_hp += 8;
            self.hp = self.max_hp;
            self.might += 2;
            self.guard += 1;
            true
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Monster {
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color: Color,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub xp_reward: i32,
    pub gold_reward: i32,
    pub aggro: bool,
    pub boss: bool,
    pub elite: bool,
    pub ranged: bool,
    pub stun: i32,
    pub poison: i32,
    pub summon_cd: i32,
    pub element: Element,
    pub cast_wind: i32,
    pub cast_tx: i32,
    pub cast_ty: i32,
    #[serde(default)]
    pub cast_cd: i32,
    #[serde(default)]
    pub flees: bool,
    #[serde(default)]
    pub heals: bool,
    #[serde(default)]
    pub bomber: bool,
    #[serde(default)]
    pub summoner: bool,
    #[serde(default)]
    pub enraged: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    Shrine,
    Fountain,
    Chest,
    Trap,
    Altar,
    Familiar,
    Forge,
    Gamble,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum PetKind {
    #[default]
    Striker,
    Mender,
    Guardian,
}

impl PetKind {
    pub fn label(self) -> &'static str {
        match self {
            PetKind::Striker => "fauve",
            PetKind::Mender => "esprit",
            PetKind::Guardian => "golem",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Pet {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub name: String,
    #[serde(default)]
    pub kind: PetKind,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub heal_cd: i32,
}

impl Pet {
    pub fn new(floor: i32, x: i32, y: i32, rng: &mut Rng) -> Pet {
        let depth = floor.max(1);
        let kind = match rng.below(3) {
            0 => PetKind::Striker,
            1 => PetKind::Mender,
            _ => PetKind::Guardian,
        };
        let (hp, atk) = match kind {
            PetKind::Striker => (18 + depth * 3, 8 + depth),
            PetKind::Mender => (18 + depth * 3, 4 + depth),
            PetKind::Guardian => (32 + depth * 5, 6 + depth),
        };
        Pet {
            x,
            y,
            hp,
            max_hp: hp,
            atk,
            name: kind.label().to_string(),
            kind,
            level: 1,
            heal_cd: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Feature {
    pub x: i32,
    pub y: i32,
    pub kind: FeatureKind,
}

struct MonsterKind {
    glyph: char,
    color: Color,
    name: &'static str,
    hp: i32,
    atk: i32,
    def: i32,
    xp: i32,
    gold: i32,
    min_floor: i32,
    ranged: bool,
    element: Element,
}

const BESTIARY: &[MonsterKind] = &[
    MonsterKind { glyph: 'r', color: (150, 140, 130), name: "rat",     hp: 4,  atk: 3,  def: 0, xp: 2,  gold: 0,  min_floor: 1,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'g', color: (110, 200, 90),  name: "gobelin", hp: 8,  atk: 5,  def: 1, xp: 5,  gold: 6,  min_floor: 1,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'k', color: (120, 110, 170), name: "kobold",  hp: 10, atk: 6,  def: 1, xp: 6,  gold: 8,  min_floor: 2,  ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'a', color: (210, 180, 90),  name: "archer",  hp: 9,  atk: 7,  def: 1, xp: 8,  gold: 10, min_floor: 2,  ranged: true,  element: Element::Physical },
    MonsterKind { glyph: 'o', color: (90, 160, 70),   name: "orc",     hp: 16, atk: 8,  def: 2, xp: 11, gold: 14, min_floor: 3,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 's', color: (190, 190, 210), name: "spectre", hp: 14, atk: 10, def: 1, xp: 13, gold: 10, min_floor: 4,  ranged: false, element: Element::Ice },
    MonsterKind { glyph: 'h', color: (120, 235, 180), name: "pretre",  hp: 12, atk: 5,  def: 1, xp: 14, gold: 18, min_floor: 4,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'z', color: (240, 150, 70),  name: "bombe",   hp: 7,  atk: 4,  def: 0, xp: 9,  gold: 6,  min_floor: 3,  ranged: false, element: Element::Fire },
    MonsterKind { glyph: 'S', color: (200, 110, 230), name: "invocateur", hp: 18, atk: 6, def: 1, xp: 20, gold: 26, min_floor: 6, ranged: false, element: Element::Lightning },
    MonsterKind { glyph: 'w', color: (180, 120, 240), name: "sorcier", hp: 16, atk: 12, def: 1, xp: 16, gold: 22, min_floor: 4,  ranged: true,  element: Element::Fire },
    MonsterKind { glyph: 'O', color: (70, 130, 60),   name: "ogre",    hp: 30, atk: 12, def: 3, xp: 22, gold: 30, min_floor: 5,  ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'T', color: (90, 200, 120),  name: "troll",   hp: 42, atk: 14, def: 4, xp: 34, gold: 40, min_floor: 6,  ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'D', color: (230, 80, 60),   name: "demon",   hp: 55, atk: 18, def: 5, xp: 55, gold: 70, min_floor: 8,  ranged: false, element: Element::Fire },
    MonsterKind { glyph: 'Y', color: (240, 160, 40),  name: "dragon",  hp: 90, atk: 24, def: 7, xp: 90, gold: 140, min_floor: 10, ranged: false, element: Element::Fire },
];

impl Monster {
    fn from_kind(kind: &MonsterKind, floor: i32, x: i32, y: i32) -> Monster {
        let depth = floor.max(1);
        Monster {
            x,
            y,
            glyph: kind.glyph,
            color: kind.color,
            name: kind.name.to_string(),
            hp: kind.hp + depth * 2,
            max_hp: kind.hp + depth * 2,
            atk: kind.atk + depth * 3 / 4,
            def: kind.def + depth / 5,
            xp_reward: kind.xp + depth,
            gold_reward: kind.gold + depth,
            aggro: false,
            boss: false,
            elite: false,
            ranged: kind.ranged,
            stun: 0,
            poison: 0,
            summon_cd: 0,
            element: kind.element,
            cast_wind: 0,
            cast_tx: 0,
            cast_ty: 0,
            cast_cd: 0,
            flees: matches!(kind.glyph, 'r' | 'a'),
            heals: kind.glyph == 'h',
            bomber: kind.glyph == 'z',
            summoner: kind.glyph == 'S',
            enraged: false,
        }
    }

    fn pool(floor: i32) -> Vec<&'static MonsterKind> {
        let unlocked: Vec<&MonsterKind> = BESTIARY.iter().filter(|k| k.min_floor <= floor).collect();
        let recent: Vec<&MonsterKind> = unlocked.iter().copied().filter(|k| k.min_floor >= floor - 5).collect();
        if recent.is_empty() { unlocked } else { recent }
    }

    pub fn roll(floor: i32, x: i32, y: i32, rng: &mut Rng) -> Monster {
        let pool = Monster::pool(floor);
        let kind = pool[rng.below(pool.len())];
        Monster::from_kind(kind, floor, x, y)
    }

    pub fn roll_biased(floor: i32, x: i32, y: i32, rng: &mut Rng, prefer: &[char]) -> Monster {
        let pool = Monster::pool(floor);
        let favored: Vec<&MonsterKind> = pool.iter().copied().filter(|k| prefer.contains(&k.glyph)).collect();
        let kind = if !favored.is_empty() && rng.chance(0.6) {
            favored[rng.below(favored.len())]
        } else {
            pool[rng.below(pool.len())]
        };
        Monster::from_kind(kind, floor, x, y)
    }

    pub fn specific(glyph: char, floor: i32, x: i32, y: i32) -> Monster {
        let kind = BESTIARY.iter().find(|k| k.glyph == glyph).unwrap_or(&BESTIARY[0]);
        Monster::from_kind(kind, floor, x, y)
    }

    pub fn promote(&mut self) {
        self.elite = true;
        self.hp = self.hp * 2 + 10;
        self.max_hp = self.hp;
        self.atk += self.atk / 2 + 2;
        self.def += 1;
        self.xp_reward *= 2;
        self.gold_reward *= 2;
        self.name = format!("{} d'elite", self.name);
        self.color = (
            (self.color.0 as u16 + 60).min(255) as u8,
            (self.color.1 as u16 + 30).min(255) as u8,
            (self.color.2 as u16 + 60).min(255) as u8,
        );
    }

    pub fn boss(floor: i32, x: i32, y: i32) -> Monster {
        let tier = ((floor / 5 - 1).max(0) as usize) % BOSSES.len();
        let (name, glyph, color) = BOSSES[tier];
        let hp = 45 + floor * 10;
        Monster {
            x,
            y,
            glyph,
            color,
            name: name.to_string(),
            hp,
            max_hp: hp,
            atk: 7 + floor * 3 / 4,
            def: 2 + floor / 4,
            xp_reward: 60 + floor * 8,
            gold_reward: 80 + floor * 12,
            aggro: true,
            boss: true,
            elite: false,
            ranged: false,
            stun: 0,
            poison: 0,
            summon_cd: 6,
            element: Element::Physical,
            cast_wind: 0,
            cast_tx: 0,
            cast_ty: 0,
            cast_cd: 0,
            flees: false,
            heals: false,
            bomber: false,
            summoner: false,
            enraged: false,
        }
    }

    pub fn final_boss(floor: i32, x: i32, y: i32) -> Monster {
        let names = ["Seigneur de l'Abime", "Avatar du Chaos", "Tyran Eternel"];
        let name = names[((floor / 25 - 1).max(0) as usize) % names.len()];
        let hp = 220 + floor * 22;
        Monster {
            x,
            y,
            glyph: '\u{2638}',
            color: (255, 80, 120),
            name: name.to_string(),
            hp,
            max_hp: hp,
            atk: 15 + floor,
            def: 6 + floor / 3,
            xp_reward: 200 + floor * 10,
            gold_reward: 200 + floor * 15,
            aggro: true,
            boss: true,
            elite: false,
            ranged: false,
            stun: 0,
            poison: 0,
            summon_cd: 4,
            element: Element::Fire,
            cast_wind: 0,
            cast_tx: 0,
            cast_ty: 0,
            cast_cd: 0,
            flees: false,
            heals: false,
            bomber: false,
            summoner: false,
            enraged: false,
        }
    }

    pub fn mimic(floor: i32, x: i32, y: i32) -> Monster {
        let depth = floor.max(1);
        Monster {
            x,
            y,
            glyph: '\u{25a4}',
            color: (235, 150, 80),
            name: "mimic".to_string(),
            hp: 24 + depth * 4,
            max_hp: 24 + depth * 4,
            atk: 10 + depth,
            def: 2 + depth / 4,
            xp_reward: 25 + depth * 3,
            gold_reward: 60 + depth * 8,
            aggro: true,
            boss: false,
            elite: false,
            ranged: false,
            stun: 0,
            poison: 0,
            summon_cd: 0,
            element: Element::Physical,
            cast_wind: 0,
            cast_tx: 0,
            cast_ty: 0,
            cast_cd: 0,
            flees: false,
            heals: false,
            bomber: false,
            summoner: false,
            enraged: false,
        }
    }
}

const BOSSES: &[(&str, char, Color)] = &[
    ("Gobelin Roi", '\u{2126}', (120, 230, 110)),
    ("Liche Affamee", '\u{2126}', (180, 150, 245)),
    ("Golem de Pierre", '\u{2126}', (200, 175, 120)),
    ("Hydre des Tunnels", '\u{2126}', (90, 220, 200)),
    ("Archidemon", '\u{2126}', (240, 70, 60)),
    ("Dragon Ancien", '\u{2126}', (245, 165, 40)),
];

#[derive(Serialize, Deserialize)]
pub enum ItemKind {
    Gold(i32),
    Potion,
    Weapon(i32, String, Affix, WeaponClass),
    Armor(i32, String, Affix, ArmorClass),
    Ring(i32, Affix),
    Amulet(i32, Affix),
    Scroll(ScrollKind),
}

fn roll_rarity(rng: &mut Rng, floor: i32, pool: &[Affix]) -> (Color, i32, Affix) {
    let roll = rng.unit() + floor as f32 * 0.004;
    if roll > 0.97 {
        rarity_pick(rng, (255, 170, 60), 6, pool)
    } else if roll > 0.86 {
        rarity_pick(rng, (195, 120, 240), 4, pool)
    } else if roll > 0.62 {
        rarity_pick(rng, (100, 165, 255), 2, pool)
    } else {
        ((185, 185, 185), 0, Affix::None)
    }
}

fn rarity_pick(rng: &mut Rng, color: Color, bonus: i32, pool: &[Affix]) -> (Color, i32, Affix) {
    let affix = pool[rng.below(pool.len())];
    (color, bonus, affix)
}

const WEAPON_AFFIXES: &[Affix] = &[Affix::Fire, Affix::Frost, Affix::Venom, Affix::Shock, Affix::Lifesteal, Affix::Keen];
const ARMOR_AFFIXES: &[Affix] = &[Affix::Regen, Affix::Thorns];

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color: Color,
    pub kind: ItemKind,
}

#[derive(Serialize, Deserialize)]
pub struct Merchant {
    pub x: i32,
    pub y: i32,
    pub weapon: Option<(String, i32, i32)>,
    pub armor: Option<(String, i32, i32)>,
    pub potion_price: i32,
    pub heal_price: i32,
}

impl Merchant {
    pub fn roll(floor: i32, x: i32, y: i32, rng: &mut Rng, wc: WeaponClass, ac: ArmorClass) -> Merchant {
        let wtable = weapons_for(wc);
        let atable = armors_for(ac);
        let wtier = ((floor / 2 + 1).min(wtable.len() as i32 - 1)).max(0) as usize;
        let atier = ((floor / 3 + 1).min(atable.len() as i32 - 1)).max(0) as usize;
        let weapon = if rng.chance(0.75) {
            let (name, bonus) = wtable[wtier];
            Some((name.to_string(), bonus, bonus * 9 + floor * 4))
        } else {
            None
        };
        let armor = if rng.chance(0.65) {
            let (name, bonus) = atable[atier];
            Some((name.to_string(), bonus, bonus * 11 + floor * 4))
        } else {
            None
        };
        Merchant {
            x,
            y,
            weapon,
            armor,
            potion_price: 14 + floor * 2,
            heal_price: 20 + floor * 3,
        }
    }
}

impl Item {
    pub fn roll(floor: i32, x: i32, y: i32, rng: &mut Rng) -> Item {
        let r = rng.unit();
        if r < 0.32 {
            let amount = rng.between(3, 12) + floor * 2;
            Item { x, y, glyph: '$', color: (235, 205, 60), kind: ItemKind::Gold(amount) }
        } else if r < 0.52 {
            Item { x, y, glyph: '!', color: (230, 90, 150), kind: ItemKind::Potion }
        } else if r < 0.62 {
            let kind = match rng.below(3) {
                0 => ScrollKind::Fireball,
                1 => ScrollKind::Teleport,
                _ => ScrollKind::Freeze,
            };
            Item { x, y, glyph: '?', color: (235, 235, 170), kind: ItemKind::Scroll(kind) }
        } else if r < 0.76 {
            let wc = match rng.below(3) {
                0 => WeaponClass::Light,
                1 => WeaponClass::Heavy,
                _ => WeaponClass::Staff,
            };
            let table = weapons_for(wc);
            let tier = ((floor / 2).min(table.len() as i32 - 1)).max(0) as usize;
            let pick = (tier + rng.below(2)).min(table.len() - 1);
            let (base, bonus0) = table[pick];
            let (color, rbonus, affix) = roll_rarity(rng, floor, WEAPON_AFFIXES);
            let name = if affix != Affix::None { format!("{} {}", base, affix.label()) } else { base.to_string() };
            Item { x, y, glyph: '/', color, kind: ItemKind::Weapon(bonus0 + rbonus, name, affix, wc) }
        } else if r < 0.88 {
            let ac = match rng.below(3) {
                0 => ArmorClass::Cloth,
                1 => ArmorClass::Leather,
                _ => ArmorClass::Plate,
            };
            let table = armors_for(ac);
            let tier = ((floor / 3).min(table.len() as i32 - 1)).max(0) as usize;
            let pick = (tier + rng.below(2)).min(table.len() - 1);
            let (base, bonus0) = table[pick];
            let (color, rbonus, affix) = roll_rarity(rng, floor, ARMOR_AFFIXES);
            let name = if affix != Affix::None { format!("{} {}", base, affix.label()) } else { base.to_string() };
            Item { x, y, glyph: '[', color, kind: ItemKind::Armor(bonus0 + rbonus, name, affix, ac) }
        } else if r < 0.95 {
            let (color, rbonus, affix) = roll_rarity(rng, floor, WEAPON_AFFIXES);
            Item { x, y, glyph: '\u{2218}', color, kind: ItemKind::Ring(1 + rbonus, affix) }
        } else {
            let (color, rbonus, affix) = roll_rarity(rng, floor, ARMOR_AFFIXES);
            Item { x, y, glyph: '\u{2666}', color, kind: ItemKind::Amulet(1 + rbonus, affix) }
        }
    }
}
