use crate::audio;
use crate::config::Config;
use crate::game::{self, Game, MerchantPick, Playstyle};
use crate::profile;
use crate::render;
use crate::twitch::{self, ViewerCmd};
use font8x8::UnicodeFonts;
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SPEEDS: [(&str, f32); 5] = [("lent", 4.0), ("normal", 7.0), ("rapide", 12.0), ("turbo", 20.0), ("ultra", 36.0)];
const PANEL_W: i32 = 42;
const MAP_W: i32 = 80;
const MAP_H: i32 = 40;

fn seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E37_79B9_7F4A_7C15)
}

fn daily_seed() -> (u64, u64, String) {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let day = secs / 86_400;
    let s = day.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xABCD_1234_5678_9EF0;
    (s | 1, day, format!("#{}", day))
}

fn leader<K: Copy>(votes: &HashMap<K, u32>) -> Option<K> {
    votes.iter().max_by_key(|(_, n)| **n).map(|(k, _)| *k)
}

fn glyph(c: char) -> [u8; 8] {
    font8x8::BASIC_FONTS
        .get(c)
        .or_else(|| font8x8::LATIN_FONTS.get(c))
        .or_else(|| font8x8::BOX_FONTS.get(c))
        .or_else(|| font8x8::BLOCK_FONTS.get(c))
        .or_else(|| font8x8::GREEK_FONTS.get(c))
        .or_else(|| font8x8::MISC_FONTS.get(c))
        .unwrap_or([0; 8])
}

fn rgb(c: (u8, u8, u8)) -> u32 {
    ((c.0 as u32) << 16) | ((c.1 as u32) << 8) | c.2 as u32
}

fn sprite_px(ch: char, color: (u8, u8, u8)) -> Option<(u8, u8, u8)> {
    match ch {
        'X' => Some(color),
        '.' => Some(((color.0 as f32 * 0.6) as u8, (color.1 as f32 * 0.6) as u8, (color.2 as f32 * 0.6) as u8)),
        ':' => Some(((color.0 as f32 * 0.8) as u8, (color.1 as f32 * 0.8) as u8, (color.2 as f32 * 0.8) as u8)),
        '*' => Some((
            (color.0 as f32 + (255.0 - color.0 as f32) * 0.4) as u8,
            (color.1 as f32 + (255.0 - color.1 as f32) * 0.4) as u8,
            (color.2 as f32 + (255.0 - color.2 as f32) * 0.4) as u8,
        )),
        'o' => Some((20, 17, 24)),
        'v' => Some((245, 245, 250)),
        _ => None,
    }
}

struct Cell {
    ch: char,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
}

fn parse_grid(s: &str, cols: usize, rows: usize) -> Vec<Cell> {
    let mut grid: Vec<Cell> = (0..cols * rows).map(|_| Cell { ch: ' ', fg: (220, 220, 220), bg: (0, 0, 0) }).collect();
    let mut fg = (220u8, 220u8, 220u8);
    let mut bg = (0u8, 0u8, 0u8);
    let (mut cr, mut cc) = (0usize, 0usize);
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' {
            if it.peek() != Some(&'[') {
                continue;
            }
            it.next();
            let mut params = String::new();
            let mut fin = ' ';
            for n in it.by_ref() {
                if n.is_ascii_alphabetic() {
                    fin = n;
                    break;
                }
                params.push(n);
            }
            match fin {
                'H' => {
                    let mut p = params.split(';');
                    let r: usize = p.next().and_then(|v| v.parse().ok()).unwrap_or(1);
                    let cn: usize = p.next().and_then(|v| v.parse().ok()).unwrap_or(1);
                    cr = r.saturating_sub(1);
                    cc = cn.saturating_sub(1);
                }
                'm' => {
                    let nums: Vec<i64> = params.split(';').map(|v| v.parse().unwrap_or(0)).collect();
                    let mut i = 0;
                    if nums.is_empty() || (nums.len() == 1 && nums[0] == 0) {
                        fg = (220, 220, 220);
                        bg = (0, 0, 0);
                    }
                    while i < nums.len() {
                        match nums[i] {
                            0 => {
                                fg = (220, 220, 220);
                                bg = (0, 0, 0);
                                i += 1;
                            }
                            38 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                                fg = (nums[i + 2] as u8, nums[i + 3] as u8, nums[i + 4] as u8);
                                i += 5;
                            }
                            48 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                                bg = (nums[i + 2] as u8, nums[i + 3] as u8, nums[i + 4] as u8);
                                i += 5;
                            }
                            _ => i += 1,
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        if c == '\n' {
            cr += 1;
            cc = 0;
            continue;
        }
        if c == '\r' {
            cc = 0;
            continue;
        }
        if cr < rows && cc < cols {
            grid[cr * cols + cc] = Cell { ch: c, fg, bg };
        }
        cc += 1;
    }
    grid
}

fn put_glyph(fb: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, ch: char, fg: (u8, u8, u8), bg: Option<(u8, u8, u8)>) {
    let g = glyph(ch);
    let fgp = rgb(fg);
    for (ry, row) in g.iter().enumerate() {
        let py = cy * 8 + ry;
        if py >= h {
            break;
        }
        for col in 0..8 {
            let px = cx * 8 + col;
            if px >= w {
                break;
            }
            let on = row & (1 << col) != 0;
            if on {
                fb[py * w + px] = fgp;
            } else if let Some(b) = bg {
                fb[py * w + px] = rgb(b);
            }
        }
    }
}

fn tile_pixels(tile: crate::map::Tile, fg: (u8, u8, u8), bg: (u8, u8, u8), cell: &mut [(u8, u8, u8); 64]) {
    use crate::map::Tile;
    for p in cell.iter_mut() {
        *p = bg;
    }
    match tile {
        Tile::Wall => {
            let mortar = (((bg.0 as u16 + fg.0 as u16) / 2) as u8, ((bg.1 as u16 + fg.1 as u16) / 2) as u8, ((bg.2 as u16 + fg.2 as u16) / 2) as u8);
            for y in 0..8 {
                for x in 0..8 {
                    let brick = if (y / 4) % 2 == 0 { x } else { (x + 4) % 8 };
                    let edge = y % 4 == 0 || brick == 0;
                    cell[y * 8 + x] = if edge { mortar } else { fg };
                }
            }
        }
        Tile::StairsDown => {
            for y in 0..8 {
                for x in 0..8 {
                    if x >= y {
                        cell[y * 8 + x] = fg;
                    }
                }
            }
        }
        Tile::Floor => {}
    }
}

fn draw_pixel_world(game: &Game, fb: &mut [u32], w: usize, h: usize, cols: i32, rows: i32) {
    let tint = render::frame_tint(game);
    let mw = game.map.width;
    let mh = game.map.height;
    let sdx = game.fx.shake_offset();
    for wy in 0..mh {
        for wx in 0..mw {
            let cx = (render::MCOL + wx + sdx) as usize;
            let cy = (render::MROW + wy) as usize;
            if cx + 1 > cols as usize || cy + 1 > rows as usize {
                continue;
            }
            let (_, fg, bg) = render::cell_render(game, wx, wy, tint);
            let mut cell = [(0u8, 0u8, 0u8); 64];
            tile_pixels(game.map.tile(wx, wy), fg, bg, &mut cell);
            for sy in 0..8 {
                for sx in 0..8 {
                    let px = cx * 8 + sx;
                    let py = cy * 8 + sy;
                    if px < w && py < h {
                        fb[py * w + px] = rgb(cell[sy * 8 + sx]);
                    }
                }
            }
            if let Some((pat, color)) = render::world_sprite(game, wx, wy) {
                for (sy, line) in pat.iter().enumerate() {
                    for (sx, ch) in line.chars().enumerate() {
                        if sx >= 8 {
                            break;
                        }
                        if let Some(p) = sprite_px(ch, color) {
                            let px = cx * 8 + sx;
                            let py = cy * 8 + sy;
                            if px < w && py < h {
                                fb[py * w + px] = rgb(p);
                            }
                        }
                    }
                }
            }
        }
    }
    for p in &game.fx.particles {
        let px = ((render::MCOL + sdx) as f32 * 8.0 + (p.x + 0.5) * 8.0) as i32;
        let py = (render::MROW as f32 * 8.0 + (p.y + 0.5) * 8.0) as i32;
        for dy in 0..2 {
            for dx in 0..2 {
                let (x, y) = (px + dx, py + dy);
                if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                    fb[y as usize * w + x as usize] = rgb(p.color);
                }
            }
        }
    }
    for p in &game.fx.projectiles {
        let px = ((render::MCOL + sdx) as f32 * 8.0 + (p.x + 0.5) * 8.0) as i32;
        let py = (render::MROW as f32 * 8.0 + (p.y + 0.5) * 8.0) as i32;
        for dy in -1..3 {
            for dx in -1..3 {
                let (x, y) = (px + dx, py + dy);
                if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                    fb[y as usize * w + x as usize] = rgb(p.color);
                }
            }
        }
    }
}

pub fn render_frame(game: &Game, cols: i32, rows: i32, paused: bool, speed_label: &str) -> Vec<u32> {
    let overlay = game.show_codex || game.show_hall || matches!(game.phase, game::Phase::Dead(_));
    let mut out: Vec<u8> = Vec::new();
    render::draw(game, cols, rows, paused, speed_label, false, 4, !overlay, &mut out);
    let text = String::from_utf8_lossy(&out);
    let grid = parse_grid(&text, cols as usize, rows as usize);

    let w = cols as usize * 8;
    let h = rows as usize * 8;
    let mut fb = vec![0u32; w * h];

    if !overlay {
        draw_pixel_world(game, &mut fb, w, h, cols, rows);
    }

    for cy in 0..rows as usize {
        for cx in 0..cols as usize {
            let cell = &grid[cy * cols as usize + cx];
            if !overlay {
                if cell.ch == ' ' {
                    continue;
                }
                put_glyph(&mut fb, w, h, cx, cy, cell.ch, cell.fg, None);
            } else {
                put_glyph(&mut fb, w, h, cx, cy, cell.ch, cell.fg, Some(cell.bg));
            }
        }
    }
    fb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_buffer_is_drawn() {
        let cols = MAP_W + PANEL_W;
        let rows = MAP_H + 2;
        let mut g = Game::new(MAP_W, MAP_H, 0xBEEF);
        for _ in 0..200 {
            g.update();
        }
        let fb = render_frame(&g, cols, rows, false, "1x");
        assert_eq!(fb.len(), (cols as usize * 8) * (rows as usize * 8));
        assert!(fb.iter().any(|&p| p != 0), "framebuffer should have lit pixels");
    }

    #[test]
    fn parse_grid_reads_cursor_and_color() {
        let s = "\x1b[1;1H\x1b[38;2;10;20;30mA\x1b[2;3H\x1b[0mB";
        let g = parse_grid(s, 5, 3);
        assert_eq!(g[0].ch, 'A');
        assert_eq!(g[0].fg, (10, 20, 30));
        assert_eq!(g[1 * 5 + 2].ch, 'B');
    }
}

pub fn run() {
    let cols = MAP_W + PANEL_W;
    let rows = MAP_H + 2;
    let w = cols as usize * 8;
    let h = rows as usize * 8;

    let mut profile = profile::Profile::load();
    let mut game = if let Some(g) = Game::load() { g } else { Game::new(MAP_W, MAP_H, seed()) };
    game.seed_lore(profile.graveyard.clone(), profile.nemeses.clone(), profile.feats.clone(), profile.dailies.clone());

    let cfg = Config::load_or_create();
    let mut audio = audio::Audio::new(cfg.ambient_enabled, cfg.master_volume, cfg.ambient_volume, cfg.music_preset);
    audio.muted = !cfg.sound_enabled;
    let votes = if cfg.twitch_active() { Some(twitch::connect(&cfg.twitch_channel)) } else { None };

    let scale = match cfg.window_scale {
        2 => Scale::X2,
        4 => Scale::X4,
        _ => Scale::X1,
    };
    let mut window = match Window::new(
        "Abyssal",
        w,
        h,
        WindowOptions { scale, scale_mode: ScaleMode::AspectRatioStretch, resize: true, ..WindowOptions::default() },
    ) {
        Ok(win) => win,
        Err(e) => {
            eprintln!("Impossible d'ouvrir la fenetre : {}", e);
            return;
        }
    };
    window.set_target_fps(60);

    let mut style_votes: HashMap<Playstyle, u32> = HashMap::new();
    let mut merch_votes: HashMap<MerchantPick, u32> = HashMap::new();
    let mut voter_counts: HashMap<String, u32> = HashMap::new();
    let mut speed_votes: i32 = 0;
    let mut last_chaos = Instant::now();
    let mut last_name = Instant::now();
    let mut last_obs = Instant::now();
    let mut bets: HashMap<String, i32> = HashMap::new();

    let mut speed = 1usize;
    let mut paused = false;
    let mut accumulator = 0.0f32;
    let mut last = Instant::now();
    let mut heartbeat_acc = 0.0f32;
    let mut shop_window = 0.0f32;
    let mut prev_merchant = false;
    let mut was_dead = matches!(game.phase, game::Phase::Dead(_));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for key in window.get_keys_pressed(KeyRepeat::No) {
            match key {
                Key::Q => {
                    game.save();
                    return;
                }
                Key::S => game.save(),
                Key::L => {
                    if let Some(g) = Game::load() {
                        game = g;
                        game.seed_lore(profile.graveyard.clone(), profile.nemeses.clone(), profile.feats.clone(), profile.dailies.clone());
                    }
                }
                Key::N => {
                    game = Game::new(MAP_W, MAP_H, seed());
                    game.seed_lore(profile.graveyard.clone(), profile.nemeses.clone(), profile.feats.clone(), profile.dailies.clone());
                    bets.clear();
                }
                Key::D => {
                    let (ds, day, code) = daily_seed();
                    game = Game::new(MAP_W, MAP_H, ds);
                    game.daily = true;
                    game.daily_day = day;
                    game.daily_code = code;
                    game.seed_lore(profile.graveyard.clone(), profile.nemeses.clone(), profile.feats.clone(), profile.dailies.clone());
                    bets.clear();
                }
                Key::Space => paused = !paused,
                Key::Equal | Key::Up => speed = (speed + 1).min(SPEEDS.len() - 1),
                Key::Minus | Key::Down => speed = speed.saturating_sub(1),
                Key::A => {
                    audio.toggle_mute();
                    game.push_log(if audio.muted { "Son coupe." } else { "Son active." }.into(), (130, 235, 240));
                }
                Key::K => game.show_codex = !game.show_codex,
                Key::H => game.show_hall = !game.show_hall,
                Key::M => game.cycle_style(),
                Key::B => game.spawn_test_merchant(),
                Key::Key1 => game.set_style(Playstyle::Completionist),
                Key::Key2 => game.set_style(Playstyle::Combatant),
                Key::Key3 => game.set_style(Playstyle::Rusher),
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
                    ViewerCmd::Bless if cfg.allow_chaos_vote => {
                        if last_chaos.elapsed() >= Duration::from_secs(15) {
                            game.twitch_bless(&user);
                            last_chaos = Instant::now();
                        }
                        (false, String::new())
                    }
                    ViewerCmd::Curse if cfg.allow_chaos_vote => {
                        if last_chaos.elapsed() >= Duration::from_secs(15) {
                            game.twitch_curse(&user);
                            last_chaos = Instant::now();
                        }
                        (false, String::new())
                    }
                    ViewerCmd::Name(n) if cfg.allow_chaos_vote => {
                        if last_name.elapsed() >= Duration::from_secs(30) {
                            game.twitch_rename(&user, &n);
                            last_name = Instant::now();
                        }
                        (false, String::new())
                    }
                    ViewerCmd::Bet(f) if cfg.allow_bet_vote => {
                        bets.insert(user.clone(), f);
                        (true, format!("{} parie etage {}", user, f))
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
            game.bet_pool = bets.len() as u32;
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
            }
            if cfg.allow_speed_vote && speed_votes.abs() >= 3 {
                if speed_votes > 0 {
                    speed = (speed + 1).min(SPEEDS.len() - 1);
                } else {
                    speed = speed.saturating_sub(1);
                }
                speed_votes = 0;
            }
        }

        let tps = SPEEDS[speed].1;
        let mut struck = false;
        if !paused && !game.show_codex && !game.show_hall {
            accumulator += dt;
            let interval = 1.0 / tps;
            let mut steps = 0;
            while accumulator >= interval && steps < 64 {
                game.update();
                struck |= game.hero_struck;
                if !game.nemesis_add.is_empty() {
                    for nem in game.nemesis_add.drain(..) {
                        profile.add_nemesis(nem);
                    }
                }
                if !game.nemesis_defeated.is_empty() {
                    for name in game.nemesis_defeated.drain(..) {
                        profile.retire_nemesis(&name);
                    }
                }
                if !game.feats_pending.is_empty() {
                    let pend: Vec<String> = game.feats_pending.drain(..).collect();
                    profile.record_feats(&pend);
                }
                accumulator -= interval;
                steps += 1;
                if game.hitstop > 0 {
                    break;
                }
            }
        }

        for s in game.sfx.drain(..).collect::<Vec<_>>() {
            audio.play(s);
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
        game.pathfinder = crate::ai::Pathfinder::from_index(cfg.pathfinder);
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
            if let Some(name) = game.nemesis_promoted.take() {
                profile.promote_nemesis(&name);
            }
            if game.daily {
                profile.record_daily(game.daily_day, &game.daily_code, game.floor, game.last_score, &game.identity.name, game.class.label());
                game.seed_lore(profile.graveyard.clone(), profile.nemeses.clone(), profile.feats.clone(), profile.dailies.clone());
            }
            if !bets.is_empty() {
                let actual = game.floor + game.boss_wave;
                let best = bets.values().map(|b| (*b - actual).abs()).min().unwrap_or(0);
                let winners: Vec<String> = bets.iter().filter(|(_, v)| (**v - actual).abs() == best).map(|(u, _)| (*u).clone()).collect();
                let shown: String = winners.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
                game.bet_result = if best == 0 {
                    format!("Pronostic gagne pile (etage {}) : {}", actual, shown)
                } else {
                    format!("Pronostic (etage {}) : {} a {} d'ecart", actual, shown, best)
                };
                game.push_feed(format!("pari gagne: {}", shown));
            }
            profile.record_death(game.floor, game.last_score, game.hero.kills, game.hero.gold);
        }
        if was_dead && !dead_now {
            bets.clear();
            game.bet_result.clear();
            game.bet_pool = 0;
        }
        was_dead = dead_now;

        if cfg.obs_overlay && last_obs.elapsed() >= Duration::from_millis(250) {
            crate::write_obs(&game);
            last_obs = Instant::now();
        }

        let fb = render_frame(&game, cols, rows, paused, SPEEDS[speed].0);
        let _ = window.update_with_buffer(&fb, w, h);
    }
    game.save();
}
