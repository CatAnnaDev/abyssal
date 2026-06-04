use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum Trait {
    #[default]
    Brave,
    Greedy,
    Coward,
    Reckless,
    Curious,
    Vengeful,
}

impl Trait {
    pub const ALL: [Trait; 6] = [Trait::Brave, Trait::Greedy, Trait::Coward, Trait::Reckless, Trait::Curious, Trait::Vengeful];

    pub fn pick(rng: &mut Rng) -> Trait {
        Trait::ALL[rng.below(Trait::ALL.len())]
    }

    pub fn label(self) -> &'static str {
        match self {
            Trait::Brave => "brave",
            Trait::Greedy => "cupide",
            Trait::Coward => "peureux",
            Trait::Reckless => "temeraire",
            Trait::Curious => "curieux",
            Trait::Vengeful => "rancunier",
        }
    }

    pub fn flee_threshold_fifths(self) -> i32 {
        match self {
            Trait::Coward => 2,
            Trait::Reckless => 0,
            _ => 1,
        }
    }

    pub fn heal_threshold_thirds(self) -> i32 {
        match self {
            Trait::Coward => 2,
            Trait::Reckless => 1,
            _ => 1,
        }
    }
}

const FIRST: [&str; 24] = [
    "Mira", "Veyra", "Lys", "Nyx", "Esca", "Ilka", "Selka", "Adda", "Yseult", "Anouk", "Sable", "Lyra", "Wren", "Vela", "Thessa", "Maeve", "Isolde", "Nessa", "Runa", "Kira", "Sora", "Orin", "Rhune", "Cael",
];
const SECOND: [&str; 16] = ["", "", "", "el", "is", "or", "wyn", "ax", "eth", "ra", "us", "in", "ka", "ov", "ael", "ys"];
const ORIGIN: [&str; 14] = [
    "des terres hautes",
    "du port noye",
    "de la cite morte",
    "des marais gris",
    "du dernier rempart",
    "des cendres",
    "de nulle part",
    "du clan brise",
    "des dunes",
    "du col de givre",
    "de la fosse",
    "des chapelles vides",
    "du long exil",
    "des bas-fonds",
];

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Identity {
    pub name: String,
    pub origin: String,
    pub trait_kind: Trait,
}

impl Identity {
    pub fn roll(rng: &mut Rng) -> Identity {
        let name = format!("{}{}", FIRST[rng.below(FIRST.len())], SECOND[rng.below(SECOND.len())]);
        Identity {
            name,
            origin: ORIGIN[rng.below(ORIGIN.len())].to_string(),
            trait_kind: Trait::pick(rng),
        }
    }

    pub fn title(&self) -> String {
        format!("{} {}", self.name, self.origin)
    }
}

pub fn obituary(id: &Identity, class: &str, cause: &str, floor: i32, kills: i32, level: i32, rng: &mut Rng) -> String {
    let deeds = if kills <= 0 {
        "sans avoir verse une goutte de sang".to_string()
    } else if kills < 12 {
        format!("apres {} adversaires", kills)
    } else {
        format!("au bout de {} victoires", kills)
    };
    let trait_line = match id.trait_kind {
        Trait::Brave => "Jamais ne recula.",
        Trait::Greedy => "Mourut les poches pleines.",
        Trait::Coward => "Courut, mais pas assez vite.",
        Trait::Reckless => "Ne sut jamais s'arreter.",
        Trait::Curious => "Une porte de trop.",
        Trait::Vengeful => "Emporta ses rancunes dans la tombe.",
    };
    let closer = ["L'abysse garde son nom.", "Une ame de plus pour le gouffre.", "On ne le reverra pas remonter.", "Le silence l'a repris."][rng.below(4)];
    format!("{}, {} ({}), tombe a l'etage {} au niveau {}, {}, {} {}", id.name, id.origin, class, floor, level, deeds, format_cause(cause), format!("— {} {}", trait_line, closer))
}

fn format_cause(cause: &str) -> String {
    if cause.is_empty() {
        "vaincue par l'abysse".to_string()
    } else {
        format!("vaincue par {}", cause)
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Ghost {
    pub name: String,
    pub origin: String,
    pub class: String,
    pub floor: i32,
    pub weapon: String,
    pub armor: String,
    pub gold: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct DailyResult {
    pub day: u64,
    pub code: String,
    pub best_floor: i32,
    pub best_score: i32,
    pub attempts: u32,
    pub name: String,
    pub class: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Nemesis {
    pub name: String,
    pub base: String,
    pub glyph: char,
    pub rank: i32,
    pub hero_kills: i32,
}

const EPITHET: [&str; 12] = [
    "le Balafre",
    "l'Insatiable",
    "qui-revient",
    "l'Echappe",
    "Mange-ames",
    "le Patient",
    "aux-mille-fuites",
    "le Tenace",
    "Sans-repos",
    "l'Ombre",
    "le Maudit",
    "Brise-lames",
];

pub fn nemesis_name(base: &str, rng: &mut Rng) -> String {
    format!("{} {}", base, EPITHET[rng.below(EPITHET.len())])
}

pub const FEATS: &[(&str, &str, &str)] = &[
    ("premier_sang", "Premier sang", "Tuer une premiere creature"),
    ("tueur_de_boss", "Tueuse de boss", "Abattre un boss d'etage"),
    ("tueuse_elite", "Briseuse d'elites", "Abattre un elite"),
    ("chasseur_de_nemesis", "Chasseuse de nemesis", "Regler son compte a une nemesis"),
    ("pilleur_de_tombe", "Pilleuse de tombe", "Profaner la tombe d'une ancienne"),
    ("plongeur_10", "Plongeuse", "Atteindre l'etage 10"),
    ("speleologue", "Speleologue", "Atteindre l'etage 20"),
    ("abime_30", "Sondeuse de l'abime", "Atteindre l'etage 30"),
    ("exterminateur", "Exterminatrice", "100 victimes en une descente"),
    ("rescape", "Rescapee", "Survivre sous 10% de PV"),
    ("coeur_de_labime", "Coeur de l'abime", "Tenir a 100% de corruption"),
    ("nabab", "Nabab", "Amasser 500 or"),
    ("fortune", "Fortune", "Amasser 2000 or"),
    ("collectionneur", "Collectionneuse", "Porter 4 reliques"),
    ("maitre_runes", "Maitresse des reliques", "Porter 6 reliques"),
    ("ensemble", "Panoplie", "Completer une panoplie (4 pieces)"),
    ("erudite", "Erudite", "Atteindre le niveau 20"),
    ("ame_ascendante", "Ame ascendante", "Atteindre l'ascension I"),
];

pub fn feat_name(id: &str) -> &'static str {
    FEATS.iter().find(|f| f.0 == id).map(|f| f.1).unwrap_or("haut fait")
}

pub fn corruption_label(corr: i32) -> &'static str {
    match corr {
        c if c >= 100 => "Absolue",
        c if c >= 75 => "Hurlante",
        c if c >= 50 => "Murmurante",
        c if c >= 25 => "Eveillee",
        _ => "Latente",
    }
}

pub fn corruption_tier(corr: i32) -> usize {
    match corr {
        c if c >= 100 => 4,
        c if c >= 75 => 3,
        c if c >= 50 => 2,
        c if c >= 25 => 1,
        _ => 0,
    }
}

const ABYSS_LATENT: &[&str] = &[
    "Encore une ame qui descend. Elles descendent toujours.",
    "Je t'observe depuis le premier pas.",
    "Le silence ici n'est pas vide. Il attend.",
    "Tu sens cette odeur ? C'est celle de celles d'avant.",
];
const ABYSS_AWAKE: &[&str] = &[
    "Plus bas. Toujours plus bas. Tu m'entends deja, n'est-ce pas ?",
    "Tes pas reveillent des choses qui dormaient.",
    "Chaque etage te prend un peu. Tu ne le remarques pas encore.",
    "Les murs se souviennent de ton visage.",
];
const ABYSS_WHISPER: &[&str] = &[
    "Ta presence me nourrit. Et moi, je te rends... genereuse.",
    "Sens-tu mes veines s'ouvrir ? L'or coule pour celles qui osent.",
    "Mes creatures enragent a ton approche. Elles te craignent.",
    "Tu n'es plus tout a fait toi. C'est bien. C'est ce que je voulais.",
];
const ABYSS_SCREAM: &[&str] = &[
    "TU M'APPARTIENS DESORMAIS.",
    "Chaque coup que tu portes, c'est moi qui le porte.",
    "Regarde tes mains. Sont-elles encore les tiennes ?",
    "Le fond t'appelle. Et le fond ne ment jamais.",
];
const ABYSS_ABSOLUTE: &[&str] = &[
    "Nous ne faisons plus qu'un, toi et l'abime.",
    "Il n'y a plus de retour. Il n'y en a jamais eu.",
    "Tu es mon coeur qui bat dans les profondeurs.",
];

pub fn abyss_whisper(corr: i32, rng: &mut Rng) -> &'static str {
    let pool = match corruption_tier(corr) {
        4 => ABYSS_ABSOLUTE,
        3 => ABYSS_SCREAM,
        2 => ABYSS_WHISPER,
        1 => ABYSS_AWAKE,
        _ => ABYSS_LATENT,
    };
    pool[rng.below(pool.len())]
}

const ABYSS_TIER_UP: [&str; 5] = [
    "L'abime s'agite a ton arrivee.",
    "L'abime s'eveille. Corruption Eveillee.",
    "L'abime murmure ton nom, et ses tresors saignent plus fort.",
    "L'abime hurle. Tout ici veut ta peau — et ta recompense enfle.",
    "L'abime t'a engloutie. Corruption Absolue.",
];

pub fn abyss_tier_up(tier: usize) -> &'static str {
    ABYSS_TIER_UP[tier.min(4)]
}

const BOSS_TAUNTS: &[&str] = &[
    "Tu es allee trop loin, petite flamme.",
    "Beaucoup sont venues. Toutes reposent ici.",
    "Ce couloir sera ta tombe.",
    "L'abime m'a confie ta fin.",
    "Approche. Finissons-en.",
];

pub fn boss_taunt(rng: &mut Rng) -> &'static str {
    BOSS_TAUNTS[rng.below(BOSS_TAUNTS.len())]
}

const NEMESIS_TAUNTS: &[&str] = &[
    "Tu te souviens de moi ? Moi, je n'ai jamais oublie.",
    "Tu aurais du m'achever quand tu le pouvais.",
    "Cette fois, tu ne t'echapperas pas.",
    "J'ai survecu pour cet instant precis.",
];

pub fn nemesis_taunt(rng: &mut Rng) -> &'static str {
    NEMESIS_TAUNTS[rng.below(NEMESIS_TAUNTS.len())]
}
