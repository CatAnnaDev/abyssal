mod ai;
mod audio;
mod config;
mod entity;
mod fx;
mod game;
mod lore;
mod map;
mod render;
mod profile;
mod rng;
mod twitch;

use config::Config;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use entity::HeroClass;
use game::{Boon, Game, MerchantPick, Playstyle};
use std::collections::HashMap;
use std::io::{self};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use twitch::ViewerCmd;

const PANEL_W: i32 = 42;
const SPEEDS: [(&str, f32); 5] = [("lent", 4.0), ("normal", 7.0), ("rapide", 12.0), ("turbo", 20.0), ("ultra", 36.0)];

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;

    let result = run(&mut stdout);

    let _ = stdout.execute(cursor::Show);
    let _ = stdout.execute(LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
    result
}

fn seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
}

fn leader<K: Copy>(votes: &HashMap<K, u32>) -> Option<K> {
    votes.iter().max_by_key(|(_, n)| **n).map(|(k, _)| *k)
}

fn twitch_hud(
    cfg: &Config,
    style_votes: &HashMap<Playstyle, u32>,
    merch_votes: &HashMap<MerchantPick, u32>,
    merchant_here: bool,
) -> String {
    let mut s = format!("TWITCH #{}", cfg.twitch_channel.trim().trim_start_matches('#'));
    if cfg.allow_style_vote {
        let c = style_votes.get(&Playstyle::Completionist).copied().unwrap_or(0);
        let b = style_votes.get(&Playstyle::Combatant).copied().unwrap_or(0);
        let r = style_votes.get(&Playstyle::Rusher).copied().unwrap_or(0);
        s.push_str(&format!(" | vote !1:{} !2:{} !3:{}", c, b, r));
    }
    if merchant_here && cfg.allow_merchant_vote {
        let w = merch_votes.get(&MerchantPick::Weapon).copied().unwrap_or(0);
        let a = merch_votes.get(&MerchantPick::Armor).copied().unwrap_or(0);
        let p = merch_votes.get(&MerchantPick::Potion).copied().unwrap_or(0);
        let h = merch_votes.get(&MerchantPick::Heal).copied().unwrap_or(0);
        s.push_str(&format!(" | shop !arme:{} !armure:{} !potion:{} !soin:{}", w, a, p, h));
    }
    s
}

struct Setup {
    class: Option<HeroClass>,
    style: Playstyle,
    diff_mult: f32,
    diff_label: String,
    boon: Boon,
    boss_rush: bool,
    mutator_pref: i32,
    start_pet: bool,
    speed_idx: usize,
    sprite: bool,
    muted: bool,
}

enum MenuResult {
    Start(Setup),
    Continue,
    Quit,
}

fn class_choices() -> Vec<(&'static str, Option<HeroClass>)> {
    let mut v: Vec<(&'static str, Option<HeroClass>)> = vec![("Aleatoire", None)];
    for c in entity::CLASSES {
        v.push((c.label, Some(c.class)));
    }
    v
}
const M_MODES: [(&str, Playstyle); 6] = [
    ("Completionniste", Playstyle::Completionist),
    ("Combattant", Playstyle::Combatant),
    ("Rusher", Playstyle::Rusher),
    ("Pilleur", Playstyle::Looter),
    ("Prudent", Playstyle::Cautious),
    ("Traqueur", Playstyle::Hunter),
];
const M_DIFFS: &[(&str, f32)] = game::DIFFICULTIES;
const M_VARIANTS: [(&str, bool); 2] = [("Normal", false), ("Boss Rush", true)];
const M_MUTATORS: [(&str, i32); 3] = [("Aleatoire", 0), ("Aucun", 1), ("Garanti", 2)];
const M_FAMILIAR: [(&str, bool); 2] = [("Aucun", false), ("Au depart", true)];
const M_DISPLAY: [(&str, bool); 2] = [("Glyphes", false), ("Sprites", true)];
const M_SOUND: [(&str, bool); 2] = [("Active", false), ("Coupe", true)];
const M_MODE_DESC: [&str; 6] = [
    "Explore tout, ramasse, combat, puis descend.",
    "Cherche le combat, traque les monstres.",
    "Fonce vers l'escalier, evite les detours.",
    "Rafle le butin et les coffres, evite les combats.",
    "Prudent : se soigne tot, se replie face au danger.",
    "Traqueur : pourchasse chaque monstre, enchaine les boss.",
];
const M_BOON_DESC: [&str; 4] = [
    "Aucun bonus de depart.",
    "Robuste : +15 PV max au depart.",
    "Affute : +3 ATQ au depart.",
    "Riche : +80 or et +2 potions au depart.",
];
const M_MUT_DESC: [&str; 3] = [
    "Mutateurs tires au hasard (~55% de chance).",
    "Aucun mutateur : run vanille, sans modificateur.",
    "Un mutateur garanti chaque run (plus de piment).",
];
fn option_desc(sel: usize, idx: &[usize]) -> String {
    match sel {
        0 => {
            if idx[0] == 0 {
                "Classe tiree au hasard a chaque lancement.".to_string()
            } else {
                entity::CLASSES[idx[0] - 1].describe()
            }
        }
        1 => M_MODE_DESC[idx[1]].to_string(),
        2 => format!("Ennemis : PV et ATQ x{} (loot/score suivent).", M_DIFFS[idx[2]].1),
        3 => M_BOON_DESC[idx[3]].to_string(),
        4 => {
            if M_VARIANTS[idx[4]].1 {
                "Boss Rush : arene sans fin des l'etage 10, sans sauvegarde.".to_string()
            } else {
                "Run normale : descente d'etage en etage.".to_string()
            }
        }
        5 => M_MUT_DESC[idx[5]].to_string(),
        6 => {
            if M_FAMILIAR[idx[6]].1 {
                "Demarre avec un familier qui monte en niveau avec vous.".to_string()
            } else {
                "Pas de familier au depart (trouvable en jeu).".to_string()
            }
        }
        7 => format!("Vitesse de simulation au lancement : {} (+/- en jeu).", SPEEDS[idx[7]].0),
        8 => {
            if M_DISPLAY[idx[8]].1 {
                "Vue sprites pixel-art en demi-blocs (touche g).".to_string()
            } else {
                "Carte en glyphes colores classiques (touche g).".to_string()
            }
        }
        _ => {
            if M_SOUND[idx[9]].1 {
                "Son coupe au lancement (touche a en jeu).".to_string()
            } else {
                "Son actif au lancement (touche a en jeu).".to_string()
            }
        }
    }
}
const M_BOONS: [(&str, Boon); 4] = [
    ("Aucun", Boon::None),
    ("Robuste", Boon::Tough),
    ("Affute", Boon::Sharp),
    ("Riche", Boon::Rich),
];

fn menu(stdout: &mut io::Stdout, cols: i32, rows: i32, has_save: bool, profile: &profile::Profile) -> io::Result<MenuResult> {
    use std::fmt::Write as _;
    use std::io::Write as _;
    let mut sel = 0i32;
    let mut idx = [0usize, 1, 1, 0, 0, 0, 0, 1, 0, 0];
    let labels = [
        "Classe",
        "Etat d'esprit",
        "Difficulte",
        "Trait de depart",
        "Variante",
        "Mutateurs",
        "Familier",
        "Vitesse",
        "Affichage",
        "Son",
    ];
    let m_classes = class_choices();
    let nrows = labels.len();
    let lens = |i: usize| -> usize {
        [
            m_classes.len(),
            M_MODES.len(),
            M_DIFFS.len(),
            M_BOONS.len(),
            M_VARIANTS.len(),
            M_MUTATORS.len(),
            M_FAMILIAR.len(),
            SPEEDS.len(),
            M_DISPLAY.len(),
            M_SOUND.len(),
        ][i]
    };
    loop {
        let mut buf = String::new();
        buf.push_str("\x1b[2J\x1b[H");
        let bw = 64i32;
        let bh = (nrows as i32) + 13;
        let ox = (cols - bw) / 2;
        let oy = ((rows - bh) / 2).max(0);
        let c = (120, 120, 150);
        let put = |buf: &mut String, x: i32, y: i32, col: (u8, u8, u8), s: &str| {
            let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m{}", y, x, col.0, col.1, col.2, s);
        };
        put(&mut buf, ox.max(1), oy, c, "\u{2554}");
        for _ in 0..bw - 2 {
            buf.push('\u{2550}');
        }
        buf.push('\u{2557}');
        for i in 1..bh - 1 {
            put(&mut buf, ox.max(1), oy + i, c, "\u{2551}");
            let _ = write!(buf, "\x1b[{};{}H\u{2551}", oy + i, ox + bw - 1);
        }
        put(&mut buf, ox.max(1), oy + bh - 1, c, "\u{255a}");
        for _ in 0..bw - 2 {
            buf.push('\u{2550}');
        }
        buf.push('\u{255d}');

        put(&mut buf, ox + 3, oy + 1, (255, 225, 130), "ABYSSAL  —  configuration de la run");
        put(&mut buf, ox + 3, oy + 2, c, &"\u{2500}".repeat((bw - 6) as usize));

        let values = [
            m_classes[idx[0]].0,
            M_MODES[idx[1]].0,
            M_DIFFS[idx[2]].0,
            M_BOONS[idx[3]].0,
            M_VARIANTS[idx[4]].0,
            M_MUTATORS[idx[5]].0,
            M_FAMILIAR[idx[6]].0,
            SPEEDS[idx[7]].0,
            M_DISPLAY[idx[8]].0,
            M_SOUND[idx[9]].0,
        ];
        for r in 0..nrows {
            let y = oy + 3 + r as i32;
            let arrow = if sel == r as i32 { "\u{25b6} " } else { "  " };
            let lab_col = if sel == r as i32 { (255, 230, 140) } else { (170, 170, 185) };
            let val_col = if sel == r as i32 { (255, 245, 200) } else { (200, 210, 225) };
            put(&mut buf, ox + 3, y, lab_col, &format!("{}{:<16}", arrow, labels[r]));
            put(&mut buf, ox + 26, y, val_col, &format!("\u{2039} {:^16} \u{203a}", values[r]));
        }

        let sep1 = oy + 3 + nrows as i32;
        put(&mut buf, ox + 3, sep1, c, &"\u{2500}".repeat((bw - 6) as usize));
        let clamp = |s: String| -> String { s.chars().take((bw - 6) as usize).collect() };
        let width = (bw - 8) as usize;
        let desc = option_desc(sel as usize, &idx);
        let dchars: Vec<char> = desc.chars().collect();
        let line1: String = dchars.iter().take(width).collect();
        let line2: String = dchars.iter().skip(width).take(width).collect();
        put(&mut buf, ox + 3, sep1 + 1, (160, 200, 230), &format!("\u{2192} {}", line1));
        if !line2.is_empty() {
            put(&mut buf, ox + 3, sep1 + 2, (140, 175, 205), &format!("  {}", line2));
        }

        if profile.runs > 0 {
            put(
                &mut buf,
                ox + 3,
                sep1 + 3,
                (170, 150, 110),
                &clamp(format!("Profil: {} runs · etage {} · score {} · asc {}", profile.runs, profile.best_floor, profile.best_score, profile.ascension)),
            );
            let perks = profile.perk_labels();
            let perks_txt = if perks.is_empty() { "aucun (atteins l'etage 4...)".to_string() } else { perks.join(", ") };
            put(&mut buf, ox + 3, sep1 + 4, (150, 200, 140), &clamp(format!("Bonus: {}", perks_txt)));
        }
        put(&mut buf, ox + 3, oy + bh - 3, c, &"\u{2500}".repeat((bw - 6) as usize));
        put(&mut buf, ox + 3, oy + bh - 2, (150, 200, 150), "fleches: choisir/changer    Entree: lancer");
        let cont = if has_save { "c: continuer la sauvegarde    q: quitter" } else { "q: quitter" };
        put(&mut buf, ox + 34, oy + bh - 2, (150, 150, 170), cont);

        buf.push_str("\x1b[0m");
        let _ = stdout.write_all(buf.as_bytes());
        let _ = stdout.flush();

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(MenuResult::Quit),
                    KeyCode::Char('c') if has_save => return Ok(MenuResult::Continue),
                    KeyCode::Up => sel = (sel + nrows as i32 - 1) % nrows as i32,
                    KeyCode::Down => sel = (sel + 1) % nrows as i32,
                    KeyCode::Left => {
                        let len = lens(sel as usize);
                        idx[sel as usize] = (idx[sel as usize] + len - 1) % len;
                    }
                    KeyCode::Right => {
                        let len = lens(sel as usize);
                        idx[sel as usize] = (idx[sel as usize] + 1) % len;
                    }
                    KeyCode::Enter => {
                        return Ok(MenuResult::Start(Setup {
                            class: m_classes[idx[0]].1,
                            style: M_MODES[idx[1]].1,
                            diff_mult: M_DIFFS[idx[2]].1,
                            diff_label: M_DIFFS[idx[2]].0.to_string(),
                            boon: M_BOONS[idx[3]].1,
                            boss_rush: M_VARIANTS[idx[4]].1,
                            mutator_pref: M_MUTATORS[idx[5]].1,
                            start_pet: M_FAMILIAR[idx[6]].1,
                            speed_idx: idx[7],
                            sprite: M_DISPLAY[idx[8]].1,
                            muted: M_SOUND[idx[9]].1,
                        }));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn config_menu(stdout: &mut io::Stdout, cols: i32, rows: i32, cfg: &mut Config, audio: &mut audio::Audio) -> io::Result<()> {
    use std::fmt::Write as _;
    use std::io::Write as _;
    let mut sel = 0i32;
    let nrows = 11i32;
    let apply = |cfg: &Config, audio: &mut audio::Audio| {
        let music = if cfg.ambient_enabled { cfg.ambient_volume } else { 0.0 };
        audio.set_levels(cfg.master_volume, music);
        audio.set_muted(!cfg.sound_enabled);
        audio.set_preset(cfg.music_preset);
    };
    let bar = |v: f32| -> String {
        let n = (v / 2.0 * 12.0).round() as i32;
        let mut s = String::new();
        for i in 0..12 {
            s.push(if i < n { '\u{2588}' } else { '\u{2591}' });
        }
        s
    };
    loop {
        let mut buf = String::new();
        buf.push_str("\x1b[2J\x1b[H");
        let bw = 60i32;
        let bh = nrows + 9;
        let ox = (cols - bw) / 2;
        let oy = ((rows - bh) / 2).max(0);
        let c = (120, 120, 150);
        let put = |buf: &mut String, x: i32, y: i32, col: (u8, u8, u8), s: &str| {
            let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m{}", y, x, col.0, col.1, col.2, s);
        };
        put(&mut buf, ox.max(1), oy, c, "\u{2554}");
        for _ in 0..bw - 2 {
            buf.push('\u{2550}');
        }
        buf.push('\u{2557}');
        for i in 1..bh - 1 {
            put(&mut buf, ox.max(1), oy + i, c, "\u{2551}");
            let _ = write!(buf, "\x1b[{};{}H\u{2551}", oy + i, ox + bw - 1);
        }
        put(&mut buf, ox.max(1), oy + bh - 1, c, "\u{255a}");
        for _ in 0..bw - 2 {
            buf.push('\u{2550}');
        }
        buf.push('\u{255d}');

        put(&mut buf, ox + 3, oy + 1, (255, 225, 130), "OPTIONS  —  jeu en pause");
        put(&mut buf, ox + 3, oy + 2, c, &"\u{2500}".repeat((bw - 6) as usize));

        let labels = [
            "Volume SFX",
            "Volume musique",
            "Style musique",
            "Musique active",
            "Son (sourdine)",
            "Twitch (relance)",
            "Votes style",
            "Votes marchand",
            "Votes vitesse",
            "Fenetre de vote",
            "Pathfinder",
        ];
        let preset = (cfg.music_preset.rem_euclid(audio::MUSIC_PRESETS.len() as i32)) as usize;
        let values = [
            format!("{} {:>4.1}", bar(cfg.master_volume), cfg.master_volume),
            format!("{} {:>4.1}", bar(cfg.ambient_volume), cfg.ambient_volume),
            audio::MUSIC_PRESETS[preset].to_string(),
            if cfg.ambient_enabled { "Oui".into() } else { "Non".into() },
            if cfg.sound_enabled { "Actif".into() } else { "Coupe".into() },
            if cfg.twitch_enabled { "Oui".into() } else { "Non".into() },
            if cfg.allow_style_vote { "Oui".into() } else { "Non".into() },
            if cfg.allow_merchant_vote { "Oui".into() } else { "Non".into() },
            if cfg.allow_speed_vote { "Oui".into() } else { "Non".into() },
            format!("{:.0} s", cfg.vote_window_secs),
            ai::Pathfinder::from_index(cfg.pathfinder).label().to_string(),
        ];
        for r in 0..nrows as usize {
            let y = oy + 3 + r as i32;
            let arrow = if sel == r as i32 { "\u{25b6} " } else { "  " };
            let lab_col = if sel == r as i32 { (255, 230, 140) } else { (170, 170, 185) };
            put(&mut buf, ox + 3, y, lab_col, &format!("{}{:<18}", arrow, labels[r]));
            put(&mut buf, ox + 26, y, (210, 220, 235), &values[r]);
        }
        put(&mut buf, ox + 3, oy + bh - 4, c, &"\u{2500}".repeat((bw - 6) as usize));
        put(&mut buf, ox + 3, oy + bh - 3, (160, 200, 230), "Canal Twitch : modifiable dans abyssal.config.json");
        put(&mut buf, ox + 3, oy + bh - 2, (150, 200, 150), "fleches: ajuster    o/echap/entree: fermer");

        buf.push_str("\x1b[0m");
        let _ = stdout.write_all(buf.as_bytes());
        let _ = stdout.flush();

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                let dir = match k.code {
                    KeyCode::Left => -1,
                    KeyCode::Right => 1,
                    _ => 0,
                };
                match k.code {
                    KeyCode::Char('o') | KeyCode::Esc | KeyCode::Enter => return Ok(()),
                    KeyCode::Up => sel = (sel + nrows - 1) % nrows,
                    KeyCode::Down => sel = (sel + 1) % nrows,
                    KeyCode::Left | KeyCode::Right => {
                        match sel {
                            0 => cfg.master_volume = (cfg.master_volume + 0.1 * dir as f32).clamp(0.0, 2.0),
                            1 => cfg.ambient_volume = (cfg.ambient_volume + 0.1 * dir as f32).clamp(0.0, 2.0),
                            2 => {
                                let n = audio::MUSIC_PRESETS.len() as i32;
                                cfg.music_preset = (cfg.music_preset + dir).rem_euclid(n);
                            }
                            3 => cfg.ambient_enabled = !cfg.ambient_enabled,
                            4 => cfg.sound_enabled = !cfg.sound_enabled,
                            5 => cfg.twitch_enabled = !cfg.twitch_enabled,
                            6 => cfg.allow_style_vote = !cfg.allow_style_vote,
                            7 => cfg.allow_merchant_vote = !cfg.allow_merchant_vote,
                            8 => cfg.allow_speed_vote = !cfg.allow_speed_vote,
                            9 => cfg.vote_window_secs = (cfg.vote_window_secs + dir as f32).clamp(2.0, 60.0),
                            _ => {
                                let n = ai::Pathfinder::ALL.len() as i32;
                                cfg.pathfinder = (cfg.pathfinder + dir).rem_euclid(n);
                            }
                        }
                        apply(cfg, audio);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn dims(cols: u16, rows: u16) -> (i32, i32, i32, i32) {
    let cols = cols as i32;
    let rows = rows as i32;
    let map_w = (cols - PANEL_W).clamp(24, cols.max(24));
    let map_h = (rows - 2).max(10);
    (cols, rows, map_w, map_h)
}

fn build_game(map_w: i32, map_h: i32, setup: &Option<Setup>, meta: (i32, i32, i32, bool, i32)) -> Game {
    match setup {
        Some(s) => Game::new_with(map_w, map_h, seed(), s.class, s.style, s.diff_mult, s.diff_label.clone(), s.boon, meta, s.boss_rush, s.mutator_pref, s.start_pet),
        None => Game::new(map_w, map_h, seed()),
    }
}

fn run(stdout: &mut io::Stdout) -> io::Result<()> {
    let (raw_c, raw_r) = terminal::size()?;
    let (mut cols, mut rows, mut map_w, mut map_h) = dims(raw_c, raw_r);

    let has_save = std::path::Path::new(game::SAVE_PATH).exists();
    let mut profile = profile::Profile::load();
    let mut setup: Option<Setup> = None;
    let mut game = match menu(stdout, cols, rows, has_save, &profile)? {
        MenuResult::Quit => return Ok(()),
        MenuResult::Continue => Game::load().unwrap_or_else(|| Game::new(map_w, map_h, seed())),
        MenuResult::Start(s) => {
            let g = Game::new_with(map_w, map_h, seed(), s.class, s.style, s.diff_mult, s.diff_label.clone(), s.boon, profile.meta(), s.boss_rush, s.mutator_pref, s.start_pet);
            setup = Some(s);
            g
        }
    };
    let _ = stdout.execute(Clear(ClearType::All));

    let mut cfg = Config::load_or_create();
    let mut audio = audio::Audio::new(cfg.ambient_enabled, cfg.master_volume, cfg.ambient_volume, cfg.music_preset);
    audio.muted = !cfg.sound_enabled || setup.as_ref().map_or(false, |s| s.muted);
    let votes = if cfg.twitch_active() {
        Some(twitch::connect(&cfg.twitch_channel))
    } else {
        None
    };
    let mut style_votes: HashMap<Playstyle, u32> = HashMap::new();
    let mut merch_votes: HashMap<MerchantPick, u32> = HashMap::new();
    let mut voter_counts: HashMap<String, u32> = HashMap::new();
    let mut speed_votes: i32 = 0;
    let mut vote_clock = 0.0f32;

    let mut speed = setup.as_ref().map_or(1, |s| s.speed_idx.min(SPEEDS.len() - 1));
    let mut paused = false;
    let mut sprite_mode = setup.as_ref().map_or(false, |s| s.sprite);
    let sprite_zooms = [2i32, 3, 4, 6, 8, 12];
    let mut zoom_idx = 2usize;
    let mut accumulator = 0.0f32;
    let mut last = Instant::now();
    let mut heartbeat_acc = 0.0f32;
    let mut shop_window = 0.0f32;
    let mut prev_merchant = false;
    let mut was_dead = matches!(game.phase, game::Phase::Dead(_));

    loop {
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        game.save();
                        return Ok(());
                    }
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        game.save();
                        return Ok(());
                    }
                    KeyCode::Char('d') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        game.debug = !game.debug;
                        let _ = stdout.execute(Clear(ClearType::All));
                    }
                    KeyCode::Char('s') => game.save(),
                    KeyCode::Char('l') => {
                        if let Some(g) = Game::load() {
                            game = g;
                            let _ = stdout.execute(Clear(ClearType::All));
                        }
                    }
                    KeyCode::Char('n') => {
                        game = build_game(map_w, map_h, &setup, profile.meta());
                        let _ = stdout.execute(Clear(ClearType::All));
                    }
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Up => {
                        speed = (speed + 1).min(SPEEDS.len() - 1);
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Down => {
                        speed = speed.saturating_sub(1);
                    }
                    KeyCode::Char('a') => {
                        audio.toggle_mute();
                        game.push_log(if audio.muted { "Son coupe." } else { "Son active." }.into(), (130, 235, 240));
                    }
                    KeyCode::Char('o') => {
                        config_menu(stdout, cols, rows, &mut cfg, &mut audio)?;
                        cfg.save();
                        let _ = stdout.execute(Clear(ClearType::All));
                    }
                    KeyCode::Char('k') => {
                        game.show_codex = !game.show_codex;
                        let _ = stdout.execute(Clear(ClearType::All));
                    }
                    KeyCode::Char('m') => game.cycle_style(),
                    KeyCode::Char('g') => {
                        sprite_mode = !sprite_mode;
                        let _ = stdout.execute(Clear(ClearType::All));
                    }
                    KeyCode::Char('z') => {
                        zoom_idx = (zoom_idx + 1) % sprite_zooms.len();
                        let _ = stdout.execute(Clear(ClearType::All));
                    }
                    KeyCode::Char('b') => game.spawn_test_merchant(),
                    KeyCode::Char('1') => game.set_style(Playstyle::Completionist),
                    KeyCode::Char('2') => game.set_style(Playstyle::Combatant),
                    KeyCode::Char('3') => game.set_style(Playstyle::Rusher),
                    _ => {}
                },
                Event::Resize(c, r) => {
                    let d = dims(c, r);
                    cols = d.0;
                    rows = d.1;
                    map_w = d.2;
                    map_h = d.3;
                    game = build_game(map_w, map_h, &setup, profile.meta());
                    let _ = stdout.execute(Clear(ClearType::All));
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = (now - last).as_secs_f32().min(0.25);
        last = now;

        let merchant_here = game.merchant.is_some();
        if cfg.twitch_active() {
            if merchant_here && !prev_merchant {
                shop_window = 60.0;
            }
            if !merchant_here {
                shop_window = 0.0;
            }
            if shop_window > 0.0 {
                shop_window -= dt;
            }
            game.shop_preview = merchant_here && shop_window > 0.0;
            game.shop_vote_secs = if merchant_here { shop_window.max(0.0) } else { 0.0 };
        }
        prev_merchant = merchant_here;

        if let Some(rx) = &votes {
            while let Ok((user, cmd)) = rx.try_recv() {
                let (counted, action) = match cmd {
                    ViewerCmd::Style(s) if cfg.allow_style_vote => {
                        *style_votes.entry(s).or_insert(0) += 1;
                        (true, format!("{} vote {}", user, s.label()))
                    }
                    ViewerCmd::Speed(d) if cfg.allow_speed_vote => {
                        speed_votes += d;
                        (true, format!("{} vote {}", user, if d > 0 { "+vite" } else { "-vite" }))
                    }
                    ViewerCmd::Merchant(p) if cfg.allow_merchant_vote => {
                        *merch_votes.entry(p).or_insert(0) += 1;
                        (true, format!("{} achete {}", user, p.label()))
                    }
                    _ => (false, String::new()),
                };
                if counted {
                    *voter_counts.entry(user.clone()).or_insert(0) += 1;
                    game.push_feed(action);
                    game.tag_monster(&user);
                }
            }
            let mut ranked: Vec<(String, u32)> = voter_counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1));
            ranked.truncate(5);
            game.top_voters = ranked;
            game.twitch_channel = cfg.twitch_channel.trim().trim_start_matches('#').to_string();
            game.style_tally = [
                style_votes.get(&Playstyle::Completionist).copied().unwrap_or(0),
                style_votes.get(&Playstyle::Combatant).copied().unwrap_or(0),
                style_votes.get(&Playstyle::Rusher).copied().unwrap_or(0),
            ];

            if game.merchant.is_some() && cfg.allow_merchant_vote {
                if let Some(pick) = leader(&merch_votes) {
                    game.forced_purchase = Some(pick);
                }
                let mut arr = [0u32; 7];
                for (k, n) in &merch_votes {
                    arr[k.index()] = *n;
                }
                game.merchant_votes = arr;
            } else {
                merch_votes.clear();
                game.forced_purchase = None;
                game.merchant_votes = [0; 7];
            }

            vote_clock += dt;
            if vote_clock >= cfg.vote_window_secs {
                vote_clock = 0.0;
                if let Some(s) = leader(&style_votes) {
                    game.set_style(s);
                }
                if speed_votes > 0 {
                    speed = (speed + 1).min(SPEEDS.len() - 1);
                } else if speed_votes < 0 {
                    speed = speed.saturating_sub(1);
                }
                style_votes.clear();
                speed_votes = 0;
            }

            game.hud_note = twitch_hud(&cfg, &style_votes, &merch_votes, game.merchant.is_some());
        }

        let tps = SPEEDS[speed].1;
        let mut struck = false;
        if !paused {
            if game.hitstop > 0 {
                game.cosmetic_tick();
                accumulator = 0.0;
            } else {
                accumulator += dt;
                let interval = 1.0 / tps;
                let mut steps = 0;
                while accumulator >= interval && steps < 64 {
                    game.update();
                    struck |= game.hero_struck;
                    accumulator -= interval;
                    steps += 1;
                    if game.hitstop > 0 {
                        break;
                    }
                }
            }
        }

        let mut played: Vec<audio::Sound> = Vec::new();
        for s in game.sfx.drain(..) {
            if !played.contains(&s) {
                audio.play(s);
                played.push(s);
            }
        }
        if struck {
            audio.play(audio::Sound::Hurt);
        }
        let frac = game.hp_fraction();
        if game.is_alive() && frac < 0.30 {
            let t = (frac / 0.30).clamp(0.0, 1.0);
            let interval = 0.45 + 0.45 * t;
            heartbeat_acc += dt;
            if heartbeat_acc >= interval {
                heartbeat_acc = 0.0;
                audio.play(audio::Sound::Heartbeat);
                game.low_hp_pulse = 1.0;
            }
        } else {
            heartbeat_acc = 0.0;
        }
        game.low_hp_pulse *= 0.85;
        game.pathfinder = ai::Pathfinder::from_index(cfg.pathfinder);
        audio.set_biome(game.biome.music_id());
        audio.set_intensity(game.music_intensity());
        audio.set_music_mode(game.music_mode());
        audio.tick();
        game.anim_t = game.anim_t.wrapping_add(1);
        if game.lunge.2 > 0 {
            game.lunge.2 -= 1;
        }
        let dead_now = matches!(game.phase, game::Phase::Dead(_));
        if dead_now && !was_dead {
            profile.record_death(game.floor, game.last_score, game.hero.kills, game.hero.gold);
        }
        was_dead = dead_now;

        render::draw(&game, cols, rows, paused, SPEEDS[speed].0, sprite_mode, sprite_zooms[zoom_idx], stdout);

        std::thread::sleep(Duration::from_millis(12));
        if struck {
            std::thread::sleep(Duration::from_millis(70));
        }
    }
}
