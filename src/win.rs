use crate::audio;
use crate::config::Config;
use crate::game::{self, Game, MerchantPick, Playstyle};
use crate::profile;
use crate::render;
use crate::twitch::{self, ViewerCmd};
use fontdue::{Font, Metrics};
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static FONT: OnceLock<Font> = OnceLock::new();

fn font() -> &'static Font {
    FONT.get_or_init(|| Font::from_bytes(include_bytes!("../assets/font.ttf").as_ref(), fontdue::FontSettings::default()).expect("font load"))
}

thread_local! {
    static GCACHE: RefCell<HashMap<char, (Metrics, Vec<u8>)>> = RefCell::new(HashMap::new());
}

fn fallback_bitmap(c: char) -> Option<[u8; 8]> {
    use font8x8::UnicodeFonts;
    font8x8::BASIC_FONTS
        .get(c)
        .or_else(|| font8x8::LATIN_FONTS.get(c))
        .or_else(|| font8x8::BOX_FONTS.get(c))
        .or_else(|| font8x8::BLOCK_FONTS.get(c))
        .or_else(|| font8x8::GREEK_FONTS.get(c))
        .or_else(|| font8x8::MISC_FONTS.get(c))
}

const SPEEDS: [(&str, f32); 5] = [("lent", 4.0), ("normal", 7.0), ("rapide", 12.0), ("turbo", 20.0), ("ultra", 36.0)];
const PANEL_W: i32 = 50;
const MAP_W: i32 = 170;
const MAP_H: i32 = 62;

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

static SS_VAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn ss() -> usize {
    match SS_VAL.load(std::sync::atomic::Ordering::Relaxed) {
        0 => 2,
        n => n,
    }
}

fn cw() -> usize {
    8 * ss()
}
fn cell_h() -> usize {
    16 * ss()
}
fn glyph_pad() -> usize {
    4 * ss()
}
fn font_px() -> f32 {
    13.0 * ss() as f32
}
fn baseline() -> i32 {
    12 * ss() as i32
}

fn blend(dst: u32, fg: (u8, u8, u8), cov: u8) -> u32 {
    if cov == 0 {
        return dst;
    }
    if cov == 255 {
        return rgb(fg);
    }
    let a = cov as u32;
    let ia = 255 - a;
    let dr = (dst >> 16) & 0xff;
    let dg = (dst >> 8) & 0xff;
    let db = dst & 0xff;
    let r = (fg.0 as u32 * a + dr * ia) / 255;
    let g = (fg.1 as u32 * a + dg * ia) / 255;
    let b = (fg.2 as u32 * a + db * ia) / 255;
    (r << 16) | (g << 8) | b
}

fn put_glyph(fb: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, ch: char, fg: (u8, u8, u8), bg: Option<(u8, u8, u8)>) {
    if let Some(b) = bg {
        let bp = rgb(b);
        for ry in 0..cell_h() {
            let py = cy * cell_h() + ry;
            if py >= h {
                break;
            }
            for col in 0..cw() {
                let px = cx * cw() + col;
                if px < w {
                    fb[py * w + px] = bp;
                }
            }
        }
    }
    if ch == ' ' {
        return;
    }
    if font().lookup_glyph_index(ch) == 0 {
        if let Some(g) = fallback_bitmap(ch) {
            let fgp = rgb(fg);
            for (ry, row) in g.iter().enumerate() {
                for col in 0..8 {
                    if row & (1 << col) == 0 {
                        continue;
                    }
                    for dy in 0..ss() {
                        for dx in 0..ss() {
                            let px = cx * cw() + col * ss() + dx;
                            let py = cy * cell_h() + glyph_pad() + ry * ss() + dy;
                            if px < w && py < h {
                                fb[py * w + px] = fgp;
                            }
                        }
                    }
                }
            }
        }
        return;
    }
    GCACHE.with(|c| {
        let mut c = c.borrow_mut();
        let (m, bmp) = c.entry(ch).or_insert_with(|| font().rasterize(ch, font_px()));
        let cell_x = (cx * cw()) as i32;
        let cell_y = (cy * cell_h()) as i32;
        let gx0 = cell_x + m.xmin;
        let gy0 = cell_y + baseline() - m.height as i32 - m.ymin;
        for j in 0..m.height {
            let py = gy0 + j as i32;
            if py < 0 || py >= h as i32 {
                continue;
            }
            for i in 0..m.width {
                let cov = bmp[j * m.width + i];
                if cov == 0 {
                    continue;
                }
                let px = gx0 + i as i32;
                if px < 0 || px >= w as i32 {
                    continue;
                }
                let idx = py as usize * w + px as usize;
                fb[idx] = blend(fb[idx], fg, cov);
            }
        }
    });
}

fn overlay_world(game: &Game, fb: &mut [u32], w: usize, h: usize, cols: i32, rows: i32) {
    let mw = game.map.width;
    let mh = game.map.height;
    let sdx = game.fx.shake_offset();
    let tint = render::frame_tint(game);
    let blit = |fb: &mut [u32], px: i32, py: i32, c: (u8, u8, u8)| {
        if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
            fb[py as usize * w + px as usize] = rgb(c);
        }
    };
    for wy in 0..mh {
        for wx in 0..mw {
            let Some((pat, color)) = render::world_sprite(game, wx, wy) else {
                continue;
            };
            let cx = (render::MCOL + wx + sdx) as usize;
            let cy = (render::MROW + wy) as usize;
            if cx >= cols as usize || cy >= rows as usize {
                continue;
            }
            let (_, _, bg) = render::cell_render(game, wx, wy, tint);
            let bgp = rgb(bg);
            for ry in 0..cell_h() {
                let py = cy * cell_h() + ry;
                if py >= h {
                    break;
                }
                for col in 0..cw() {
                    let px = cx * cw() + col;
                    if px < w {
                        fb[py * w + px] = bgp;
                    }
                }
            }
            for (sy, line) in pat.iter().enumerate() {
                for (sx, ch) in line.chars().enumerate() {
                    if sx >= 8 {
                        break;
                    }
                    if let Some(p) = sprite_px(ch, color) {
                        for dy in 0..ss() {
                            for dx in 0..ss() {
                                blit(fb, (cx * cw() + sx * ss() + dx) as i32, (cy * cell_h() + glyph_pad() + sy * ss() + dy) as i32, p);
                            }
                        }
                    }
                }
            }
        }
    }
    let bx = (render::MCOL + sdx) as f32 * cw() as f32;
    let by = render::MROW as f32 * cell_h() as f32;
    let dot = ss() as i32;
    for p in &game.fx.particles {
        let px = (bx + (p.x + 0.5) * cw() as f32) as i32;
        let py = (by + (p.y + 0.5) * cell_h() as f32) as i32;
        for dy in 0..dot {
            for dx in 0..dot {
                blit(fb, px + dx, py + dy, p.color);
            }
        }
    }
    for p in &game.fx.projectiles {
        let px = (bx + (p.x + 0.5) * cw() as f32) as i32;
        let py = (by + (p.y + 0.5) * cell_h() as f32) as i32;
        for dy in -dot..dot {
            for dx in -dot..dot {
                blit(fb, px + dx, py + dy, p.color);
            }
        }
    }
}

pub fn render_frame(game: &Game, cols: i32, rows: i32, paused: bool, speed_label: &str) -> Vec<u32> {
    let mut out: Vec<u8> = Vec::new();
    render::draw(game, cols, rows, paused, speed_label, false, 4, false, &mut out);
    let text = String::from_utf8_lossy(&out);
    let grid = parse_grid(&text, cols as usize, rows as usize);

    let w = cols as usize * cw();
    let h = rows as usize * cell_h();
    let mut fb = vec![0u32; w * h];

    for cy in 0..rows as usize {
        for cx in 0..cols as usize {
            let cell = &grid[cy * cols as usize + cx];
            put_glyph(&mut fb, w, h, cx, cy, cell.ch, cell.fg, Some(cell.bg));
        }
    }

    let overlay = game.show_codex || game.show_hall || matches!(game.phase, game::Phase::Dead(_));
    if !overlay {
        overlay_world(game, &mut fb, w, h, cols, rows);
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
        assert_eq!(fb.len(), (cols as usize * cw()) * (rows as usize * cell_h()));
        assert!(fb.iter().any(|&p| p != 0), "framebuffer should have lit pixels");
    }

    #[test]
    fn pregame_menu_renders() {
        let cols = MAP_W + PANEL_W;
        let w = cols as usize * cw();
        let h = (MAP_H + 2) as usize * cell_h();
        let classes = [("Aleatoire", None), ("Mage", None)];
        let idx = [1usize, 1, 1, 0, 0, 0, 1, 1, 0];
        let prof = profile::Profile::default();
        let fb = pregame_fb(w, h, cols, &classes, &idx, 2, true, &prof);
        assert_eq!(fb.len(), w * h);
        assert!(fb.iter().any(|&p| p != 0));
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

fn present(window: &mut Window, fb: &[u32], w: usize, h: usize) {
    let _ = window.update_with_buffer(fb, w, h);
}

fn draw_text(fb: &mut [u32], w: usize, h: usize, col: usize, row: usize, s: &str, fg: (u8, u8, u8)) {
    for (i, ch) in s.chars().enumerate() {
        put_glyph(fb, w, h, col + i, row, ch, fg, None);
    }
}

const MENU_MODES: [(&str, Playstyle); 6] = [
    ("Completionniste", Playstyle::Completionist),
    ("Combattant", Playstyle::Combatant),
    ("Rusher", Playstyle::Rusher),
    ("Pilleur", Playstyle::Looter),
    ("Prudent", Playstyle::Cautious),
    ("Traqueur", Playstyle::Hunter),
];
const MENU_BOONS: [(&str, game::Boon); 4] = [("Aucun", game::Boon::None), ("Robuste", game::Boon::Tough), ("Affute", game::Boon::Sharp), ("Riche", game::Boon::Rich)];
const MENU_VARIANTS: [&str; 2] = ["Normal", "Boss Rush"];
const MENU_MUTATORS: [&str; 3] = ["Aleatoire", "Aucun", "Garanti"];
const MENU_FAMILIAR: [&str; 2] = ["Aucun", "Au depart"];
const MENU_SOUND: [&str; 2] = ["Active", "Coupe"];

struct Started {
    game: Game,
    speed: usize,
    muted: bool,
}

fn pregame_menu(window: &mut Window, profile: &profile::Profile, w: usize, h: usize, cols: i32) -> Option<Started> {
    let mut classes: Vec<(&str, Option<crate::entity::HeroClass>)> = vec![("Aleatoire", None)];
    for c in crate::entity::CLASSES {
        classes.push((c.label, Some(c.class)));
    }
    let lens = [classes.len(), MENU_MODES.len(), game::DIFFICULTIES.len(), MENU_BOONS.len(), MENU_VARIANTS.len(), MENU_MUTATORS.len(), MENU_FAMILIAR.len(), SPEEDS.len(), MENU_SOUND.len()];
    let mut idx = [0usize, 1, 1, 0, 0, 0, 0, 1, 0];
    let mut sel = 0usize;
    let has_save = Game::load().is_some();
    let n = 9usize;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let fb = pregame_fb(w, h, cols, &classes, &idx, sel, has_save, profile);
        present(window, &fb, w, h);
        for key in window.get_keys_pressed(KeyRepeat::No) {
            match key {
                Key::Up => sel = (sel + n - 1) % n,
                Key::Down => sel = (sel + 1) % n,
                Key::Left => idx[sel] = (idx[sel] + lens[sel] - 1) % lens[sel],
                Key::Right => idx[sel] = (idx[sel] + 1) % lens[sel],
                Key::Enter => {
                    let (dl, dm) = game::DIFFICULTIES[idx[2]];
                    let g = Game::new_with(
                        MAP_W,
                        MAP_H,
                        seed(),
                        classes[idx[0]].1,
                        MENU_MODES[idx[1]].1,
                        dm,
                        dl.to_string(),
                        MENU_BOONS[idx[3]].1,
                        profile.meta(),
                        idx[4] == 1,
                        idx[5] as i32,
                        idx[6] == 1,
                    );
                    return Some(Started { game: g, speed: idx[7], muted: idx[8] == 1 });
                }
                Key::C if has_save => {
                    if let Some(g) = Game::load() {
                        return Some(Started { game: g, speed: 1, muted: false });
                    }
                }
                Key::Q => return None,
                _ => {}
            }
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn pregame_fb(w: usize, h: usize, cols: i32, classes: &[(&str, Option<crate::entity::HeroClass>)], idx: &[usize; 9], sel: usize, has_save: bool, profile: &profile::Profile) -> Vec<u32> {
    let diffs = game::DIFFICULTIES;
    let labels = ["Classe", "Etat d'esprit", "Difficulte", "Trait de depart", "Variante", "Mutateurs", "Familier", "Vitesse", "Son"];
    let mut fb = vec![0u32; w * h];
    let ox = (cols as usize) / 2 - 16;
    let mut r = 3usize;
    draw_text(&mut fb, w, h, ox, r, "A B Y S S A L", (255, 225, 130));
    r += 2;
    draw_text(&mut fb, w, h, ox, r, "fleches: choisir   entree: commencer", (150, 150, 165));
    r += 1;
    if has_save {
        draw_text(&mut fb, w, h, ox, r, "c: continuer la sauvegarde   q: quitter", (150, 150, 165));
    } else {
        draw_text(&mut fb, w, h, ox, r, "q: quitter", (150, 150, 165));
    }
    r += 2;
    let values: [String; 9] = [
        classes[idx[0]].0.to_string(),
        MENU_MODES[idx[1]].0.to_string(),
        format!("{} (x{})", diffs[idx[2]].0, diffs[idx[2]].1),
        MENU_BOONS[idx[3]].0.to_string(),
        MENU_VARIANTS[idx[4]].to_string(),
        MENU_MUTATORS[idx[5]].to_string(),
        MENU_FAMILIAR[idx[6]].to_string(),
        SPEEDS[idx[7]].0.to_string(),
        MENU_SOUND[idx[8]].to_string(),
    ];
    for i in 0..labels.len() {
        let y = r + i * 2;
        let arrow = if i == sel { ">" } else { " " };
        let col = if i == sel { (255, 235, 150) } else { (180, 180, 195) };
        draw_text(&mut fb, w, h, ox, y, &format!("{} {:<16}", arrow, labels[i]), col);
        draw_text(&mut fb, w, h, ox + 20, y, &format!("< {} >", values[i]), if i == sel { (235, 215, 140) } else { (150, 195, 210) });
    }
    let dy = r + 9 * 2 + 1;
    let desc = if sel == 0 {
        match idx[0] {
            0 => "Classe tiree au hasard a chaque lancement.".to_string(),
            n => crate::entity::CLASSES[n - 1].describe(),
        }
    } else {
        String::new()
    };
    if !desc.is_empty() {
        draw_text(&mut fb, w, h, ox, dy, &desc, (170, 200, 175));
    }
    let perks = profile.perk_labels();
    if !perks.is_empty() {
        draw_text(&mut fb, w, h, ox, dy + 2, &format!("Bonus debloques: {}", perks.join(", ")), (235, 205, 130));
    }
    fb
}

fn options_menu(window: &mut Window, cfg: &mut Config, audio: &mut audio::Audio, w: usize, h: usize, cols: i32) {
    let n = 14usize;
    let mut sel = 0usize;
    let onoff = |b: bool| if b { "oui" } else { "non" };
    loop {
        if !window.is_open() {
            break;
        }
        audio.muted = !cfg.sound_enabled;
        audio.set_levels(cfg.master_volume, if cfg.ambient_enabled { cfg.ambient_volume } else { 0.0 });
        audio.set_preset(cfg.music_preset);

        let presets = audio::MUSIC_PRESETS;
        let rows_txt: [(String, String); 14] = [
            ("Son".into(), onoff(cfg.sound_enabled).into()),
            ("Ambiance".into(), onoff(cfg.ambient_enabled).into()),
            ("Volume SFX".into(), format!("{:.1}", cfg.master_volume)),
            ("Volume musique".into(), format!("{:.1}", cfg.ambient_volume)),
            ("Preset musique".into(), presets[cfg.music_preset.rem_euclid(6) as usize].into()),
            ("Pathfinder".into(), crate::ai::Pathfinder::from_index(cfg.pathfinder).label().into()),
            ("Echelle fenetre".into(), format!("{} (au relancement)", if cfg.window_scale == 2 { "2x" } else if cfg.window_scale == 4 { "4x" } else { "Auto (16:9)" })),
            ("Twitch".into(), onoff(cfg.twitch_enabled).into()),
            ("Vote mindset".into(), onoff(cfg.allow_style_vote).into()),
            ("Vote marchand".into(), onoff(cfg.allow_merchant_vote).into()),
            ("Vote chaos".into(), onoff(cfg.allow_chaos_vote).into()),
            ("Vote paris".into(), onoff(cfg.allow_bet_vote).into()),
            ("Overlay OBS".into(), onoff(cfg.obs_overlay).into()),
            ("Finesse (resolution)".into(), format!("x{} (au relancement)", cfg.render_scale.clamp(1, 4))),
        ];
        let mut fb = vec![0u32; w * h];
        let ox = (cols as usize) / 2 - 18;
        draw_text(&mut fb, w, h, ox, 2, "O P T I O N S", (255, 225, 130));
        draw_text(&mut fb, w, h, ox, 4, "fleches: regler   o/entree/echap: fermer", (150, 150, 165));
        for i in 0..n {
            let y = 6 + i * 2;
            let arrow = if i == sel { ">" } else { " " };
            let col = if i == sel { (255, 235, 150) } else { (180, 180, 195) };
            draw_text(&mut fb, w, h, ox, y, &format!("{} {:<16}", arrow, rows_txt[i].0), col);
            draw_text(&mut fb, w, h, ox + 20, y, &format!("< {} >", rows_txt[i].1), if i == sel { (235, 215, 140) } else { (150, 195, 210) });
        }
        present(window, &fb, w, h);

        let mut close = false;
        for key in window.get_keys_pressed(KeyRepeat::No) {
            match key {
                Key::Up => sel = (sel + n - 1) % n,
                Key::Down => sel = (sel + 1) % n,
                Key::O | Key::Enter | Key::Escape => close = true,
                Key::Left | Key::Right => {
                    let dir = if key == Key::Right { 1 } else { -1 };
                    match sel {
                        0 => cfg.sound_enabled = !cfg.sound_enabled,
                        1 => cfg.ambient_enabled = !cfg.ambient_enabled,
                        2 => cfg.master_volume = (cfg.master_volume + dir as f32 * 0.1).clamp(0.0, 2.0),
                        3 => cfg.ambient_volume = (cfg.ambient_volume + dir as f32 * 0.1).clamp(0.0, 2.0),
                        4 => cfg.music_preset = (cfg.music_preset + dir).rem_euclid(6),
                        5 => cfg.pathfinder = (cfg.pathfinder + dir).rem_euclid(crate::ai::Pathfinder::ALL.len() as i32),
                        6 => {
                            cfg.window_scale = match (cfg.window_scale, dir > 0) {
                                (2, true) => 4,
                                (2, false) => 0,
                                (4, true) => 0,
                                (4, false) => 2,
                                (_, true) => 2,
                                (_, false) => 4,
                            }
                        }
                        7 => cfg.twitch_enabled = !cfg.twitch_enabled,
                        8 => cfg.allow_style_vote = !cfg.allow_style_vote,
                        9 => cfg.allow_merchant_vote = !cfg.allow_merchant_vote,
                        10 => cfg.allow_chaos_vote = !cfg.allow_chaos_vote,
                        11 => cfg.allow_bet_vote = !cfg.allow_bet_vote,
                        12 => cfg.obs_overlay = !cfg.obs_overlay,
                        _ => cfg.render_scale = (cfg.render_scale - 1 + dir).rem_euclid(4) + 1,
                    }
                }
                _ => {}
            }
        }
        if close {
            break;
        }
    }
    cfg.save();
}

pub fn run() {
    let cols = MAP_W + PANEL_W;
    let rows = MAP_H + 2;

    let mut profile = profile::Profile::load();

    let mut cfg = Config::load_or_create();
    SS_VAL.store(cfg.render_scale.clamp(1, 4) as usize, std::sync::atomic::Ordering::Relaxed);
    let w = cols as usize * cw();
    let h = rows as usize * cell_h();
    let mut audio = audio::Audio::new(cfg.ambient_enabled, cfg.master_volume, cfg.ambient_volume, cfg.music_preset);
    audio.muted = !cfg.sound_enabled;
    let votes = if cfg.twitch_active() { Some(twitch::connect(&cfg.twitch_channel)) } else { None };

    let scale = match cfg.window_scale {
        2 => Scale::X2,
        4 => Scale::X4,
        0 => Scale::FitScreen,
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

    let started = match pregame_menu(&mut window, &profile, w, h, cols) {
        Some(s) => s,
        None => return,
    };
    let mut game = started.game;
    game.seed_lore(profile.graveyard.clone(), profile.nemeses.clone(), profile.feats.clone(), profile.dailies.clone());
    audio.muted = !cfg.sound_enabled || started.muted;

    let mut style_votes: HashMap<Playstyle, u32> = HashMap::new();
    let mut merch_votes: HashMap<MerchantPick, u32> = HashMap::new();
    let mut voter_counts: HashMap<String, u32> = HashMap::new();
    let mut speed_votes: i32 = 0;
    let mut last_chaos = Instant::now();
    let mut last_name = Instant::now();
    let mut last_obs = Instant::now();
    let mut bets: HashMap<String, i32> = HashMap::new();

    let mut speed = started.speed.min(SPEEDS.len() - 1);
    let mut paused = false;
    let mut accumulator = 0.0f32;
    let mut last = Instant::now();
    let mut heartbeat_acc = 0.0f32;
    let mut shop_window = 0.0f32;
    let mut prev_merchant = false;
    let mut was_dead = matches!(game.phase, game::Phase::Dead(_));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let ctrl = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
        for key in window.get_keys_pressed(KeyRepeat::No) {
            match key {
                Key::Q => {
                    game.save();
                    return;
                }
                Key::D if ctrl => game.debug = !game.debug,
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
                Key::O => options_menu(&mut window, &mut cfg, &mut audio, w, h, cols),
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
        game.cosmetic_tick();
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
            profile.record_ghost(game.make_ghost());
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
        present(&mut window, &fb, w, h);
    }
    game.save();
}
