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
