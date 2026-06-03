use crate::rng::Rng;
use serde::{Deserialize, Serialize};

pub type Color = (u8, u8, u8);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum WeaponClass {
    Light,
    Heavy,
    Staff,
    Fist,
    Bow,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ArmorClass {
    Cloth,
    Leather,
    Plate,
    Mail,
}

impl WeaponClass {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            WeaponClass::Light => "leger",
            WeaponClass::Heavy => "lourd",
            WeaponClass::Staff => "magique",
            WeaponClass::Fist => "martial",
            WeaponClass::Bow => "a distance",
        }
    }
}

impl ArmorClass {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            ArmorClass::Cloth => "tissu",
            ArmorClass::Leather => "cuir",
            ArmorClass::Plate => "plaque",
            ArmorClass::Mail => "mailles",
        }
    }
}

const LIGHT_WEAPONS: &[(&str, i32)] = &[("dague", 2), ("stylet", 4), ("kriss", 6), ("lames jumelles", 9), ("croc d'ombre", 13), ("faux de l'abime", 16)];
const HEAVY_WEAPONS: &[(&str, i32)] = &[("epee courte", 3), ("hache", 5), ("masse d'armes", 7), ("epee large", 10), ("fleau ardent", 14), ("colosse titanesque", 17)];
const STAFF_WEAPONS: &[(&str, i32)] = &[("baton", 2), ("baton runique", 4), ("sceptre", 7), ("baton de givre", 10), ("baton du chaos", 14), ("baton primordial", 17)];
const FIST_WEAPONS: &[(&str, i32)] = &[("poings de fer", 2), ("griffes", 4), ("gantelets clouttes", 7), ("poings d'acier", 10), ("griffes du dragon", 14), ("poings celestes", 16)];
const BOW_WEAPONS: &[(&str, i32)] = &[("arc court", 2), ("arc long", 5), ("arbalete", 7), ("arc composite", 10), ("arc du crepuscule", 14), ("arc des etoiles", 16)];
const CLOTH_ARMORS: &[(&str, i32)] = &[("tunique", 1), ("robe", 3), ("robe runique", 5), ("manteau arcanique", 8), ("manteau du vide", 11)];
const LEATHER_ARMORS: &[(&str, i32)] = &[("armure de cuir", 2), ("cuir cloute", 4), ("cuir renforce", 6), ("cape d'ombre", 9), ("cuir du traqueur", 12)];
const PLATE_ARMORS: &[(&str, i32)] = &[("cotte de mailles", 2), ("plastron", 4), ("armure de plates", 6), ("egide drakonienne", 9), ("plates du titan", 13)];
const MAIL_ARMORS: &[(&str, i32)] = &[("haubergeon", 2), ("cotte renforcee", 4), ("harnois", 7), ("mailles sacrees", 10), ("egide du gardien", 13)];

fn weapons_for(c: WeaponClass) -> &'static [(&'static str, i32)] {
    match c {
        WeaponClass::Light => LIGHT_WEAPONS,
        WeaponClass::Heavy => HEAVY_WEAPONS,
        WeaponClass::Staff => STAFF_WEAPONS,
        WeaponClass::Fist => FIST_WEAPONS,
        WeaponClass::Bow => BOW_WEAPONS,
    }
}

fn armors_for(c: ArmorClass) -> &'static [(&'static str, i32)] {
    match c {
        ArmorClass::Cloth => CLOTH_ARMORS,
        ArmorClass::Leather => LEATHER_ARMORS,
        ArmorClass::Plate => PLATE_ARMORS,
        ArmorClass::Mail => MAIL_ARMORS,
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum HeroClass {
    Warrior,
    Rogue,
    Mage,
    Paladin,
    Necromancer,
    Ranger,
    Berserker,
    Elementalist,
    Monk,
    Druid,
    Templar,
    Warlock,
    Shaman,
    Valkyrie,
    Spellblade,
    Sentinel,
    Reaper,
    Spectre,
    Maelstrom,
    Lich,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ability {
    Charge,
    Blink,
    Nova,
    Smite,
    Raise,
    Volley,
    Furie,
    Vortex,
    Possess,
    Phase,
}

impl Ability {
    pub fn label(self) -> &'static str {
        match self {
            Ability::Charge => "charge",
            Ability::Blink => "assaut-eclair",
            Ability::Nova => "nova elementaire",
            Ability::Smite => "chatiment-soin",
            Ability::Raise => "levee des morts",
            Ability::Volley => "volee de fleches",
            Ability::Furie => "furie tournoyante",
            Ability::Vortex => "vortex (attire tout)",
            Ability::Possess => "possession",
            Ability::Phase => "phase (tp murs)",
        }
    }
}

pub struct ClassDef {
    pub class: HeroClass,
    pub label: &'static str,
    pub crit: f32,
    pub cleave: i32,
    pub bolt: i32,
    pub bleeds: bool,
    pub raises: bool,
    pub weapon: WeaponClass,
    pub armor: ArmorClass,
    pub d_hp: i32,
    pub d_might: i32,
    pub d_guard: i32,
    pub ability: Ability,
}

impl ClassDef {
    pub fn describe(&self) -> String {
        format!(
            "arme {}, armure {} · +{}PV +{}ATQ +{}DEF · crit {}% · {}",
            self.weapon.label(),
            self.armor.label(),
            self.d_hp,
            self.d_might,
            self.d_guard,
            (self.crit * 100.0) as i32,
            self.ability.label()
        )
    }
}

pub const CLASSES: &[ClassDef] = &[
    ClassDef { class: HeroClass::Warrior,      label: "Guerrier",      crit: 0.10, cleave: 2,   bolt: 999, bleeds: false, raises: false, weapon: WeaponClass::Heavy, armor: ArmorClass::Plate,   d_hp: 14, d_might: 0, d_guard: 2, ability: Ability::Charge },
    ClassDef { class: HeroClass::Rogue,        label: "Voleur",        crit: 0.28, cleave: 999, bolt: 6,   bleeds: true,  raises: false, weapon: WeaponClass::Light, armor: ArmorClass::Leather, d_hp: 0,  d_might: 3, d_guard: 0, ability: Ability::Blink },
    ClassDef { class: HeroClass::Mage,         label: "Mage",          crit: 0.14, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Staff, armor: ArmorClass::Cloth,   d_hp: 2,  d_might: 6, d_guard: 0, ability: Ability::Nova },
    ClassDef { class: HeroClass::Paladin,      label: "Paladin",       crit: 0.12, cleave: 3,   bolt: 999, bleeds: false, raises: false, weapon: WeaponClass::Heavy, armor: ArmorClass::Plate,   d_hp: 20, d_might: 1, d_guard: 3, ability: Ability::Smite },
    ClassDef { class: HeroClass::Necromancer,  label: "Necromancien",  crit: 0.13, cleave: 999, bolt: 1,   bleeds: false, raises: true,  weapon: WeaponClass::Staff, armor: ArmorClass::Cloth,   d_hp: 8,  d_might: 3, d_guard: 0, ability: Ability::Raise },
    ClassDef { class: HeroClass::Ranger,       label: "Rodeur",        crit: 0.20, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Bow,   armor: ArmorClass::Leather, d_hp: 4,  d_might: 3, d_guard: 0, ability: Ability::Volley },
    ClassDef { class: HeroClass::Berserker,    label: "Berserker",     crit: 0.18, cleave: 2,   bolt: 999, bleeds: true,  raises: false, weapon: WeaponClass::Heavy, armor: ArmorClass::Leather, d_hp: 8,  d_might: 5, d_guard: 0, ability: Ability::Furie },
    ClassDef { class: HeroClass::Elementalist, label: "Elementaliste", crit: 0.15, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Staff, armor: ArmorClass::Cloth,   d_hp: 0,  d_might: 5, d_guard: 0, ability: Ability::Nova },
    ClassDef { class: HeroClass::Monk,         label: "Moine",         crit: 0.26, cleave: 2,   bolt: 999, bleeds: true,  raises: false, weapon: WeaponClass::Fist,  armor: ArmorClass::Leather, d_hp: 10, d_might: 4, d_guard: 1, ability: Ability::Blink },
    ClassDef { class: HeroClass::Druid,        label: "Druide",        crit: 0.14, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Staff, armor: ArmorClass::Leather, d_hp: 10, d_might: 4, d_guard: 0, ability: Ability::Smite },
    ClassDef { class: HeroClass::Templar,      label: "Templier",      crit: 0.12, cleave: 3,   bolt: 999, bleeds: false, raises: false, weapon: WeaponClass::Heavy, armor: ArmorClass::Mail,    d_hp: 18, d_might: 1, d_guard: 3, ability: Ability::Charge },
    ClassDef { class: HeroClass::Warlock,      label: "Occultiste",    crit: 0.16, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Staff, armor: ArmorClass::Cloth,   d_hp: 4,  d_might: 6, d_guard: 0, ability: Ability::Nova },
    ClassDef { class: HeroClass::Shaman,       label: "Chaman",        crit: 0.15, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Staff, armor: ArmorClass::Leather, d_hp: 8,  d_might: 4, d_guard: 1, ability: Ability::Smite },
    ClassDef { class: HeroClass::Valkyrie,     label: "Valkyrie",      crit: 0.14, cleave: 2,   bolt: 999, bleeds: false, raises: false, weapon: WeaponClass::Heavy, armor: ArmorClass::Mail,    d_hp: 16, d_might: 2, d_guard: 2, ability: Ability::Charge },
    ClassDef { class: HeroClass::Spellblade,   label: "Lame-Sort",     crit: 0.20, cleave: 999, bolt: 1,   bleeds: true,  raises: false, weapon: WeaponClass::Light, armor: ArmorClass::Cloth,   d_hp: 4,  d_might: 5, d_guard: 0, ability: Ability::Blink },
    ClassDef { class: HeroClass::Sentinel,     label: "Sentinelle",    crit: 0.16, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Bow,   armor: ArmorClass::Mail,    d_hp: 12, d_might: 3, d_guard: 2, ability: Ability::Volley },
    ClassDef { class: HeroClass::Reaper,       label: "Faucheur",      crit: 0.22, cleave: 2,   bolt: 999, bleeds: true,  raises: false, weapon: WeaponClass::Light, armor: ArmorClass::Leather, d_hp: 6,  d_might: 5, d_guard: 0, ability: Ability::Furie },
    ClassDef { class: HeroClass::Spectre,      label: "Spectre",       crit: 0.20, cleave: 999, bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Staff, armor: ArmorClass::Cloth,   d_hp: 4,  d_might: 5, d_guard: 0, ability: Ability::Phase },
    ClassDef { class: HeroClass::Maelstrom,    label: "Maelstrom",     crit: 0.16, cleave: 3,   bolt: 1,   bleeds: false, raises: false, weapon: WeaponClass::Staff, armor: ArmorClass::Leather, d_hp: 8,  d_might: 5, d_guard: 0, ability: Ability::Vortex },
    ClassDef { class: HeroClass::Lich,         label: "Liche",         crit: 0.14, cleave: 999, bolt: 1,   bleeds: false, raises: true,  weapon: WeaponClass::Staff, armor: ArmorClass::Cloth,   d_hp: 8,  d_might: 4, d_guard: 0, ability: Ability::Possess },
];

impl HeroClass {
    pub const ALL: [HeroClass; 20] = [
        HeroClass::Warrior,
        HeroClass::Rogue,
        HeroClass::Mage,
        HeroClass::Paladin,
        HeroClass::Necromancer,
        HeroClass::Ranger,
        HeroClass::Berserker,
        HeroClass::Elementalist,
        HeroClass::Monk,
        HeroClass::Druid,
        HeroClass::Templar,
        HeroClass::Warlock,
        HeroClass::Shaman,
        HeroClass::Valkyrie,
        HeroClass::Spellblade,
        HeroClass::Sentinel,
        HeroClass::Reaper,
        HeroClass::Spectre,
        HeroClass::Maelstrom,
        HeroClass::Lich,
    ];

    pub fn def(self) -> &'static ClassDef {
        &CLASSES[self as usize]
    }

    pub fn label(self) -> &'static str {
        self.def().label
    }

    pub fn pick(rng: &mut Rng) -> HeroClass {
        HeroClass::ALL[rng.below(HeroClass::ALL.len())]
    }

    pub fn crit_chance(self) -> f32 {
        self.def().crit
    }

    pub fn cleave_level(self) -> i32 {
        self.def().cleave
    }

    pub fn bolt_level(self) -> i32 {
        self.def().bolt
    }

    pub fn bleeds(self) -> bool {
        self.def().bleeds
    }

    pub fn raises_dead(self) -> bool {
        self.def().raises
    }

    pub fn weapon_class(self) -> WeaponClass {
        self.def().weapon
    }

    pub fn armor_class(self) -> ArmorClass {
        self.def().armor
    }

    pub fn ability(self) -> Ability {
        self.def().ability
    }

    pub fn apply(self, h: &mut Hero) {
        let d = self.def();
        h.max_hp += d.d_hp;
        h.might += d.d_might;
        h.guard += d.d_guard;
        h.weapon = weapons_for(d.weapon)[0].0.into();
        h.armor = armors_for(d.armor)[0].0.into();
        h.hp = h.max_hp;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ScrollKind {
    Fireball,
    Teleport,
    Freeze,
    Lightning,
}

impl ScrollKind {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            ScrollKind::Fireball => "boule de feu",
            ScrollKind::Teleport => "teleportation",
            ScrollKind::Freeze => "gel de zone",
            ScrollKind::Lightning => "chaine d'eclairs",
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
    Eclaireur,
    Acier,
}

impl Talent {
    pub const ALL: [Talent; 8] = [
        Talent::Berserk,
        Talent::Sangsue,
        Talent::Colosse,
        Talent::Bourreau,
        Talent::Arcaniste,
        Talent::Regen,
        Talent::Eclaireur,
        Talent::Acier,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Talent::Berserk => "Berserk (+crit)",
            Talent::Sangsue => "Sangsue (vol de vie)",
            Talent::Colosse => "Colosse (+PV max)",
            Talent::Bourreau => "Bourreau (cleave)",
            Talent::Arcaniste => "Arcaniste (eclair)",
            Talent::Regen => "Regeneration",
            Talent::Eclaireur => "Eclaireur (+vision)",
            Talent::Acier => "Peau d'Acier (-degats)",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Relic {
    Vampire,
    Spectral,
    Storm,
    Ember,
    Colossus,
    Undying,
    Frenzy,
    Greed,
}

impl Relic {
    pub const ALL: [Relic; 8] = [Relic::Vampire, Relic::Spectral, Relic::Storm, Relic::Ember, Relic::Colossus, Relic::Undying, Relic::Frenzy, Relic::Greed];

    pub fn label(self) -> &'static str {
        match self {
            Relic::Vampire => "Coeur Vampirique",
            Relic::Spectral => "Voile Spectral",
            Relic::Storm => "Orbe Fulgurant",
            Relic::Ember => "Braise Eternelle",
            Relic::Colossus => "Talisman du Colosse",
            Relic::Undying => "Pacte Mort-vivant",
            Relic::Frenzy => "Sang Bouillant",
            Relic::Greed => "Bourse Sans Fond",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Relic::Vampire => "vampire",
            Relic::Spectral => "spectral",
            Relic::Storm => "fulgurant",
            Relic::Ember => "braise",
            Relic::Colossus => "colosse",
            Relic::Undying => "mort-vivant",
            Relic::Frenzy => "furie",
            Relic::Greed => "cupidite",
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
    Bleed,
    Sunder,
}

impl Affix {
    pub const SET_POOL: [Affix; 10] = [
        Affix::Fire,
        Affix::Frost,
        Affix::Venom,
        Affix::Shock,
        Affix::Lifesteal,
        Affix::Keen,
        Affix::Regen,
        Affix::Thorns,
        Affix::Bleed,
        Affix::Sunder,
    ];

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
            Affix::Bleed => "saignant",
            Affix::Sunder => "brise-garde",
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
    #[serde(default)]
    pub relics: Vec<Relic>,
}

impl Hero {
    pub fn has_relic(&self, r: Relic) -> bool {
        self.relics.contains(&r)
    }
}

impl Hero {
    pub fn fresh(x: i32, y: i32) -> Self {
        Hero {
            x,
            y,
            hp: 42,
            max_hp: 42,
            might: 5,
            guard: 2,
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
            relics: Vec::new(),
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
        for a in Affix::SET_POOL {
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
        for a in Affix::SET_POOL {
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

    pub fn armor_element(&self) -> Element {
        let e = self.armor_affix.element();
        if e != Element::Physical {
            e
        } else {
            self.amulet.element()
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
    #[serde(default)]
    pub bleed: i32,
    #[serde(default)]
    pub owner: String,
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
    Lost,
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
pub struct Ally {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub atk: i32,
    pub ttl: i32,
    pub glyph: char,
    pub color: Color,
    #[serde(default)]
    pub max_hp: i32,
    #[serde(default)]
    pub companion: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: u8,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub kills: i32,
}

pub const ALLY_GUARD: u8 = 0;
pub const ALLY_HUNTER: u8 = 1;
pub const ALLY_MEDIC: u8 = 2;

const LOST_NAMES: &[(&str, char, u8)] = &[
    ("Garde Aldric", 'G', ALLY_GUARD),
    ("Soeur Mirel", 'M', ALLY_MEDIC),
    ("Traqueur Vael", 'V', ALLY_HUNTER),
    ("Lame Orin", 'O', ALLY_GUARD),
    ("Sentinelle Kael", 'K', ALLY_GUARD),
    ("Erudit Brann", 'B', ALLY_MEDIC),
    ("Chasseresse Lys", 'L', ALLY_HUNTER),
    ("Vagabond Dorn", 'D', ALLY_HUNTER),
];

pub fn ally_role_label(role: u8) -> &'static str {
    match role {
        ALLY_HUNTER => "archer",
        ALLY_MEDIC => "guerisseur",
        _ => "garde",
    }
}

impl Ally {
    pub fn raised(floor: i32, x: i32, y: i32, src: &Monster) -> Ally {
        let depth = floor.max(1);
        let hp = (src.max_hp / 2).max(3);
        Ally {
            x,
            y,
            hp,
            max_hp: hp,
            atk: (src.atk * 2 / 3 + depth / 2).max(2),
            ttl: 30,
            glyph: '\u{2625}',
            color: (180, 200, 175),
            companion: false,
            name: String::new(),
            role: ALLY_GUARD,
            level: 1,
            kills: 0,
        }
    }

    pub fn skeleton(floor: i32, x: i32, y: i32) -> Ally {
        let depth = floor.max(1);
        let hp = 10 + depth * 2;
        Ally {
            x,
            y,
            hp,
            max_hp: hp,
            atk: 5 + depth,
            ttl: 40,
            glyph: '\u{2625}',
            color: (205, 210, 195),
            companion: false,
            name: String::new(),
            role: ALLY_GUARD,
            level: 1,
            kills: 0,
        }
    }

    pub fn companion(floor: i32, x: i32, y: i32, rng: &mut Rng) -> Ally {
        let depth = floor.max(1);
        let (name, glyph, role) = LOST_NAMES[rng.below(LOST_NAMES.len())];
        let (hp, atk, color) = match role {
            ALLY_HUNTER => (34 + depth * 4, 11 + depth, (210, 200, 140)),
            ALLY_MEDIC => (38 + depth * 4, 6 + depth, (180, 235, 200)),
            _ => (56 + depth * 6, 8 + depth, (255, 224, 150)),
        };
        Ally {
            x,
            y,
            hp,
            max_hp: hp,
            atk,
            ttl: i32::MAX,
            glyph,
            color,
            companion: true,
            name: name.to_string(),
            role,
            level: 1,
            kills: 0,
        }
    }

    pub fn level_up(&mut self) {
        self.level += 1;
        self.max_hp += 6 + self.max_hp / 12;
        self.hp = self.max_hp;
        self.atk += 2;
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
    MonsterKind { glyph: 'b', color: (140, 130, 160), name: "chauve-souris", hp: 5,  atk: 3,  def: 0, xp: 3,  gold: 0,  min_floor: 1,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'v', color: (120, 190, 80),  name: "vipere",  hp: 6,  atk: 5,  def: 0, xp: 5,  gold: 4,  min_floor: 2,  ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'j', color: (150, 210, 150), name: "gelee",   hp: 16, atk: 4,  def: 2, xp: 7,  gold: 4,  min_floor: 2,  ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'f', color: (200, 200, 120), name: "farfadet", hp: 7,  atk: 6,  def: 1, xp: 7,  gold: 9,  min_floor: 2,  ranged: true,  element: Element::Lightning },
    MonsterKind { glyph: 'W', color: (150, 140, 120), name: "worg",    hp: 16, atk: 9,  def: 1, xp: 12, gold: 10, min_floor: 3,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'c', color: (180, 90, 160),  name: "cultiste", hp: 14, atk: 7,  def: 1, xp: 12, gold: 16, min_floor: 4,  ranged: true,  element: Element::Poison },
    MonsterKind { glyph: 'G', color: (140, 170, 120), name: "goule",   hp: 22, atk: 10, def: 2, xp: 15, gold: 14, min_floor: 4,  ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'i', color: (230, 120, 90),  name: "diablotin", hp: 13, atk: 8, def: 1, xp: 14, gold: 20, min_floor: 5,  ranged: true,  element: Element::Fire },
    MonsterKind { glyph: 'm', color: (120, 200, 160), name: "mante",   hp: 18, atk: 12, def: 2, xp: 17, gold: 14, min_floor: 5,  ranged: false, element: Element::Ice },
    MonsterKind { glyph: 'P', color: (150, 150, 160), name: "golem",   hp: 44, atk: 11, def: 6, xp: 26, gold: 26, min_floor: 6,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'N', color: (200, 120, 230), name: "necromant", hp: 26, atk: 12, def: 2, xp: 30, gold: 40, min_floor: 7, ranged: false, element: Element::Lightning },
    MonsterKind { glyph: 'n', color: (90, 200, 180),  name: "naga",    hp: 32, atk: 13, def: 3, xp: 28, gold: 32, min_floor: 7,  ranged: true,  element: Element::Poison },
    MonsterKind { glyph: 'e', color: (255, 170, 80),  name: "elementaire", hp: 38, atk: 16, def: 3, xp: 40, gold: 42, min_floor: 8, ranged: false, element: Element::Fire },
    MonsterKind { glyph: 'M', color: (180, 220, 210), name: "meduse",  hp: 30, atk: 14, def: 3, xp: 36, gold: 38, min_floor: 8,  ranged: false, element: Element::Ice },
    MonsterKind { glyph: 'A', color: (240, 230, 180), name: "ange dechu", hp: 50, atk: 18, def: 4, xp: 52, gold: 64, min_floor: 9, ranged: true, element: Element::Lightning },
    MonsterKind { glyph: 'B', color: (110, 90, 80),   name: "behemoth", hp: 75, atk: 20, def: 6, xp: 66, gold: 84, min_floor: 11, ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'x', color: (170, 110, 200), name: "aberration", hp: 62, atk: 19, def: 4, xp: 62, gold: 72, min_floor: 12, ranged: false, element: Element::Lightning },
    MonsterKind { glyph: 'Q', color: (210, 150, 90),  name: "chimere", hp: 88, atk: 23, def: 6, xp: 84, gold: 130, min_floor: 13, ranged: false, element: Element::Fire },
    MonsterKind { glyph: 'L', color: (170, 150, 120), name: "limier",  hp: 13, atk: 9,  def: 1, xp: 10, gold: 8,  min_floor: 3,  ranged: false, element: Element::Physical },
    MonsterKind { glyph: 'u', color: (240, 230, 130), name: "luciole", hp: 8,  atk: 7,  def: 0, xp: 8,  gold: 6,  min_floor: 3,  ranged: true,  element: Element::Lightning },
    MonsterKind { glyph: 'p', color: (160, 110, 190), name: "araignee", hp: 14, atk: 7, def: 1, xp: 10, gold: 9,  min_floor: 4,  ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'C', color: (110, 160, 175), name: "crabe-roc", hp: 36, atk: 9, def: 7, xp: 22, gold: 18, min_floor: 5,  ranged: false, element: Element::Ice },
    MonsterKind { glyph: 'H', color: (200, 170, 200), name: "harpie",  hp: 20, atk: 11, def: 1, xp: 18, gold: 18, min_floor: 6,  ranged: true,  element: Element::Physical },
    MonsterKind { glyph: 'R', color: (160, 180, 200), name: "revenant", hp: 44, atk: 16, def: 3, xp: 40, gold: 32, min_floor: 9,  ranged: false, element: Element::Ice },
    MonsterKind { glyph: 'y', color: (220, 110, 70),  name: "wyverne", hp: 58, atk: 19, def: 4, xp: 54, gold: 60, min_floor: 10, ranged: false, element: Element::Fire },
    MonsterKind { glyph: 'E', color: (250, 230, 120), name: "foudroyeur", hp: 40, atk: 17, def: 2, xp: 42, gold: 40, min_floor: 9, ranged: true, element: Element::Lightning },
    MonsterKind { glyph: 'I', color: (180, 130, 200), name: "essaim", hp: 18, atk: 9, def: 1, xp: 13, gold: 8, min_floor: 5, ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'F', color: (190, 210, 230), name: "fantome", hp: 22, atk: 12, def: 1, xp: 20, gold: 14, min_floor: 6, ranged: false, element: Element::Ice },
    MonsterKind { glyph: 'V', color: (200, 80, 110),  name: "vampire", hp: 50, atk: 18, def: 4, xp: 50, gold: 60, min_floor: 11, ranged: false, element: Element::Physical },
    MonsterKind { glyph: 't', color: (150, 170, 110), name: "tisseuse", hp: 24, atk: 11, def: 2, xp: 18, gold: 14, min_floor: 6, ranged: false, element: Element::Poison },
    MonsterKind { glyph: 'q', color: (235, 130, 90),  name: "quasit", hp: 16, atk: 12, def: 1, xp: 16, gold: 18, min_floor: 7, ranged: true, element: Element::Fire },
    MonsterKind { glyph: 'U', color: (150, 200, 230), name: "ondine", hp: 30, atk: 13, def: 2, xp: 28, gold: 24, min_floor: 8, ranged: true, element: Element::Ice },
];

pub fn bestiary() -> Vec<(char, Color, &'static str, &'static str, i32, &'static str)> {
    BESTIARY
        .iter()
        .map(|k| {
            let behavior = match k.glyph {
                'z' => "kamikaze",
                'S' => "invocateur",
                'h' => "soigneur",
                'r' => "fuyard",
                _ if k.ranged => "distance",
                _ => "melee",
            };
            (k.glyph, k.color, k.name, k.element.label(), k.min_floor, behavior)
        })
        .collect()
}

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
            flees: matches!(kind.glyph, 'r' | 'a' | 'b'),
            heals: matches!(kind.glyph, 'h' | 'c'),
            bomber: kind.glyph == 'z',
            summoner: kind.glyph == 'S',
            enraged: false,
            bleed: 0,
            owner: String::new(),
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
        self.hp = self.hp * 5 / 2 + 20;
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
        let hp = 80 + floor * 18;
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
            bleed: 0,
            owner: String::new(),
        }
    }

    pub fn final_boss(floor: i32, x: i32, y: i32) -> Monster {
        let names = ["Seigneur de l'Abime", "Avatar du Chaos", "Tyran Eternel"];
        let name = names[((floor / 25 - 1).max(0) as usize) % names.len()];
        let hp = 360 + floor * 36;
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
            bleed: 0,
            owner: String::new(),
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
            hp: 18 + depth * 3,
            max_hp: 18 + depth * 3,
            atk: 6 + depth,
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
            bleed: 0,
            owner: String::new(),
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
    ("Colosse de Forge", '\u{2126}', (235, 150, 80)),
    ("Mere-Spore", '\u{2126}', (130, 225, 140)),
    ("Tisseur du Vide", '\u{2126}', (170, 120, 220)),
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
    AncientEye,
    Hourglass,
    Chalice,
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

const WEAPON_AFFIXES: &[Affix] = &[Affix::Fire, Affix::Frost, Affix::Venom, Affix::Shock, Affix::Lifesteal, Affix::Keen, Affix::Bleed, Affix::Sunder];
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
        if floor >= 4 && rng.chance(0.02) {
            return match rng.below(3) {
                0 => Item { x, y, glyph: '\u{2609}', color: (255, 236, 150), kind: ItemKind::AncientEye },
                1 => Item { x, y, glyph: '\u{2604}', color: (150, 210, 235), kind: ItemKind::Hourglass },
                _ => Item { x, y, glyph: '\u{2624}', color: (130, 235, 155), kind: ItemKind::Chalice },
            };
        }
        let r = rng.unit();
        if r < 0.32 {
            let amount = rng.between(3, 12) + floor * 2;
            Item { x, y, glyph: '$', color: (235, 205, 60), kind: ItemKind::Gold(amount) }
        } else if r < 0.52 {
            Item { x, y, glyph: '!', color: (230, 90, 150), kind: ItemKind::Potion }
        } else if r < 0.62 {
            let kind = match rng.below(4) {
                0 => ScrollKind::Fireball,
                1 => ScrollKind::Teleport,
                2 => ScrollKind::Freeze,
                _ => ScrollKind::Lightning,
            };
            Item { x, y, glyph: '?', color: (235, 235, 170), kind: ItemKind::Scroll(kind) }
        } else if r < 0.76 {
            let wc = match rng.below(5) {
                0 => WeaponClass::Light,
                1 => WeaponClass::Heavy,
                2 => WeaponClass::Staff,
                3 => WeaponClass::Fist,
                _ => WeaponClass::Bow,
            };
            let table = weapons_for(wc);
            let tier = ((floor / 2).min(table.len() as i32 - 1)).max(0) as usize;
            let pick = (tier + rng.below(2)).min(table.len() - 1);
            let (base, bonus0) = table[pick];
            let (color, rbonus, affix) = roll_rarity(rng, floor, WEAPON_AFFIXES);
            let name = if affix != Affix::None { format!("{} {}", base, affix.label()) } else { base.to_string() };
            Item { x, y, glyph: '/', color, kind: ItemKind::Weapon(bonus0 + rbonus, name, affix, wc) }
        } else if r < 0.88 {
            let ac = match rng.below(4) {
                0 => ArmorClass::Cloth,
                1 => ArmorClass::Leather,
                2 => ArmorClass::Plate,
                _ => ArmorClass::Mail,
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
