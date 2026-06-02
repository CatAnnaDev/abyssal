mod ai;
mod audio;
mod config;
mod entity;
mod fx;
mod game;
mod map;
mod render;
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
}

enum MenuResult {
    Start(Setup),
    Continue,
    Quit,
}

const M_CLASSES: [(&str, Option<HeroClass>); 4] = [
    ("Aleatoire", None),
    ("Guerrier", Some(HeroClass::Warrior)),
    ("Voleur", Some(HeroClass::Rogue)),
    ("Mage", Some(HeroClass::Mage)),
];
const M_MODES: [(&str, Playstyle); 3] = [
    ("Completionniste", Playstyle::Completionist),
    ("Combattant", Playstyle::Combatant),
    ("Rusher", Playstyle::Rusher),
];
const M_DIFFS: [(&str, f32); 4] = [("Facile", 0.7), ("Normal", 1.0), ("Difficile", 1.4), ("Cauchemar", 1.85)];
const M_BOONS: [(&str, Boon); 4] = [
    ("Aucun", Boon::None),
    ("Robuste", Boon::Tough),
    ("Affute", Boon::Sharp),
    ("Riche", Boon::Rich),
];

fn menu(stdout: &mut io::Stdout, cols: i32, rows: i32, has_save: bool) -> io::Result<MenuResult> {
    use std::fmt::Write as _;
    use std::io::Write as _;
    let mut sel = 0i32;
    let mut idx = [0usize, 1, 1, 0];
    let labels = ["Classe", "Mode de jeu", "Difficulte", "Trait de depart"];
    loop {
        let mut buf = String::new();
        buf.push_str("\x1b[2J\x1b[H");
        let bw = 52i32;
        let bh = 16i32;
        let ox = (cols - bw) / 2;
        let oy = (rows - bh) / 2;
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

        put(&mut buf, ox + 3, oy + 1, (255, 225, 130), "ROGUE  —  nouvelle exploration");
        put(&mut buf, ox + 3, oy + 2, c, &"\u{2500}".repeat((bw - 6) as usize));

        let values = [
            M_CLASSES[idx[0]].0,
            M_MODES[idx[1]].0,
            M_DIFFS[idx[2]].0,
            M_BOONS[idx[3]].0,
        ];
        for r in 0..4usize {
            let y = oy + 4 + r as i32 * 2;
            let arrow = if sel == r as i32 { "\u{25b6} " } else { "  " };
            let lab_col = if sel == r as i32 { (255, 230, 140) } else { (170, 170, 185) };
            put(&mut buf, ox + 3, y, lab_col, &format!("{}{:<16}", arrow, labels[r]));
            put(&mut buf, ox + 24, y, (210, 220, 235), &format!("\u{2039} {:^14} \u{203a}", values[r]));
        }

        put(&mut buf, ox + 3, oy + bh - 4, c, &"\u{2500}".repeat((bw - 6) as usize));
        put(&mut buf, ox + 3, oy + bh - 3, (150, 200, 150), "Entree: lancer    fleches: choisir/changer");
        let cont = if has_save { "c: continuer la sauvegarde    q: quitter" } else { "q: quitter" };
        put(&mut buf, ox + 3, oy + bh - 2, (150, 150, 170), cont);

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
                    KeyCode::Up => sel = (sel + 3) % 4,
                    KeyCode::Down => sel = (sel + 1) % 4,
                    KeyCode::Left => {
                        let len = [M_CLASSES.len(), M_MODES.len(), M_DIFFS.len(), M_BOONS.len()][sel as usize];
                        idx[sel as usize] = (idx[sel as usize] + len - 1) % len;
                    }
                    KeyCode::Right => {
                        let len = [M_CLASSES.len(), M_MODES.len(), M_DIFFS.len(), M_BOONS.len()][sel as usize];
                        idx[sel as usize] = (idx[sel as usize] + 1) % len;
                    }
                    KeyCode::Enter => {
                        return Ok(MenuResult::Start(Setup {
                            class: M_CLASSES[idx[0]].1,
                            style: M_MODES[idx[1]].1,
                            diff_mult: M_DIFFS[idx[2]].1,
                            diff_label: M_DIFFS[idx[2]].0.to_string(),
                            boon: M_BOONS[idx[3]].1,
                        }));
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

fn build_game(map_w: i32, map_h: i32, setup: &Option<Setup>) -> Game {
    match setup {
        Some(s) => Game::new_with(map_w, map_h, seed(), s.class, s.style, s.diff_mult, s.diff_label.clone(), s.boon),
        None => Game::new(map_w, map_h, seed()),
    }
}

fn run(stdout: &mut io::Stdout) -> io::Result<()> {
    let (raw_c, raw_r) = terminal::size()?;
    let (mut cols, mut rows, mut map_w, mut map_h) = dims(raw_c, raw_r);

    let has_save = std::path::Path::new(game::SAVE_PATH).exists();
    let mut setup: Option<Setup> = None;
    let mut game = match menu(stdout, cols, rows, has_save)? {
        MenuResult::Quit => return Ok(()),
        MenuResult::Continue => Game::load().unwrap_or_else(|| Game::new(map_w, map_h, seed())),
        MenuResult::Start(s) => {
            let g = Game::new_with(map_w, map_h, seed(), s.class, s.style, s.diff_mult, s.diff_label.clone(), s.boon);
            setup = Some(s);
            g
        }
    };
    let _ = stdout.execute(Clear(ClearType::All));

    let cfg = Config::load_or_create();
    let mut audio = audio::Audio::new(cfg.ambient_enabled, cfg.master_volume, cfg.ambient_volume);
    audio.muted = !cfg.sound_enabled;
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

    let mut speed = 1usize;
    let mut paused = false;
    let mut accumulator = 0.0f32;
    let mut last = Instant::now();
    let mut shop_window = 0.0f32;
    let mut prev_merchant = false;

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
                    KeyCode::Char('s') => game.save(),
                    KeyCode::Char('l') => {
                        if let Some(g) = Game::load() {
                            game = g;
                            let _ = stdout.execute(Clear(ClearType::All));
                        }
                    }
                    KeyCode::Char('n') => {
                        game = build_game(map_w, map_h, &setup);
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
                    KeyCode::Char('m') => game.cycle_style(),
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
                    game = build_game(map_w, map_h, &setup);
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
        }
        prev_merchant = merchant_here;

        if let Some(rx) = &votes {
            while let Ok((user, cmd)) = rx.try_recv() {
                let counted = match cmd {
                    ViewerCmd::Style(s) if cfg.allow_style_vote => {
                        *style_votes.entry(s).or_insert(0) += 1;
                        true
                    }
                    ViewerCmd::Speed(d) if cfg.allow_speed_vote => {
                        speed_votes += d;
                        true
                    }
                    ViewerCmd::Merchant(p) if cfg.allow_merchant_vote => {
                        *merch_votes.entry(p).or_insert(0) += 1;
                        true
                    }
                    _ => false,
                };
                if counted {
                    *voter_counts.entry(user).or_insert(0) += 1;
                }
            }
            let mut ranked: Vec<(String, u32)> = voter_counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1));
            ranked.truncate(5);
            game.top_voters = ranked;

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
            accumulator += dt;
            let interval = 1.0 / tps;
            let mut steps = 0;
            while accumulator >= interval && steps < 64 {
                game.update();
                struck |= game.hero_struck;
                accumulator -= interval;
                steps += 1;
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

        render::draw(&game, cols, rows, paused, SPEEDS[speed].0, stdout);

        std::thread::sleep(Duration::from_millis(12));
        if struck {
            std::thread::sleep(Duration::from_millis(70));
        }
    }
}
