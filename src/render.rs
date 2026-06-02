use crate::entity::{Color, Element, FeatureKind};
use crate::game::{FloorEvent, Game, Objective, Phase};
use crate::map::Tile;
use std::fmt::Write as _;
use std::io::Write;

const MROW: i32 = 2;
const MCOL: i32 = 2;
const FRAME: Color = (95, 95, 120);

pub fn draw(game: &Game, cols: i32, rows: i32, paused: bool, speed_label: &str, out: &mut impl Write) {
    let mut buf = String::with_capacity((cols * rows) as usize * 7);
    buf.push_str("\x1b[H");

    let mw = game.map.width;
    let mh = game.map.height;
    let tint = if game.event == FloorEvent::Inferno {
        (1.22, 0.82, 0.66)
    } else {
        floor_tint(game.floor)
    };
    let sdx = game.fx.shake_offset();
    let lights: Vec<(f32, f32, Color)> = game.fx.projectiles.iter().map(|p| (p.x, p.y, p.color)).collect();

    for y in 0..mh {
        let _ = write!(buf, "\x1b[{};{}H\x1b[0m", MROW + y, MCOL);
        for _ in 0..sdx {
            buf.push(' ');
        }
        let mut last: Option<(Color, Color)> = None;
        for x in 0..mw {
            let (ch, fg, mut bg) = cell_render(game, x, y, tint);
            if !lights.is_empty() {
                bg = light_add(bg, &lights, x, y);
            }
            if last != Some((fg, bg)) {
                let _ = write!(buf, "\x1b[38;2;{};{};{};48;2;{};{};{}m", fg.0, fg.1, fg.2, bg.0, bg.1, bg.2);
                last = Some((fg, bg));
            }
            buf.push(ch);
        }
    }

    draw_fx(game, mw, mh, sdx, &mut buf);
    buf.push_str("\x1b[0m");
    draw_frame(game, cols, rows, mw, paused, speed_label, &mut buf);
    draw_panel(game, cols, rows, mw, &mut buf);

    if let Some(b) = game.monsters.iter().find(|m| m.boss) {
        draw_boss_bar(mw, &b.name, b.hp, b.max_hp, b.color, &mut buf);
    }
    if game.merchant.is_some() && (game.shop_timer > 0 || game.shop_preview) {
        draw_shop(game, mw, &mut buf);
    }
    if !game.top_voters.is_empty() {
        draw_top_voters(game, mh, &mut buf);
    }
    if game.fx.combo >= 3 {
        let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;255;200;70mCOMBO x{}\x1b[0m", MROW, MCOL + 1, game.fx.combo);
    }
    if game.fx.transition > 0 {
        draw_transition(game.fx.transition_floor, mw, mh, &mut buf);
    }
    if let Phase::Dead(_) = game.phase {
        draw_death(game, mw, mh, &mut buf);
    }

    buf.push_str("\x1b[0m");
    let _ = out.write_all(buf.as_bytes());
    let _ = out.flush();
}

fn put(buf: &mut String, x: i32, y: i32, color: Color, text: &str) {
    let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m{}", y, x, color.0, color.1, color.2, text);
}

fn draw_frame(game: &Game, cols: i32, rows: i32, mw: i32, paused: bool, speed_label: &str, buf: &mut String) {
    let evt = if game.event == FloorEvent::Calm {
        String::new()
    } else {
        format!("  ·  {}", game.event.label())
    };
    let diff = if game.diff_label == "Normal" {
        String::new()
    } else {
        format!("  ·  {}", game.diff_label)
    };
    let boon = match game.boon {
        crate::game::Boon::None => String::new(),
        b => format!("  ·  {}", b.label()),
    };
    let title = format!(
        " ROGUE  ·  etage {}  ·  {}  ·  run #{}  ·  {}{}{}{} ",
        game.floor,
        game.class.label(),
        game.runs,
        if paused { "PAUSE" } else { speed_label },
        diff,
        boon,
        evt
    );
    let avail = (cols - 2).max(0) as usize;
    let t: String = title.chars().take(avail).collect();
    let fill = avail - t.chars().count();
    let _ = write!(buf, "\x1b[1;1H\x1b[38;2;{};{};{}m\u{2554}", FRAME.0, FRAME.1, FRAME.2);
    let _ = write!(buf, "\x1b[38;2;255;225;130m{}", t);
    let _ = write!(buf, "\x1b[38;2;{};{};{}m", FRAME.0, FRAME.1, FRAME.2);
    for _ in 0..fill {
        buf.push('\u{2550}');
    }
    buf.push('\u{2557}');

    let _ = write!(buf, "\x1b[{};1H\u{255a}", rows);
    for _ in 0..(cols - 2) {
        buf.push('\u{2550}');
    }
    buf.push('\u{255d}');
    let hint = " espace:pause  +/-:vitesse  m:mindset  a:son  b:marchand  s/l/n  q:quitter ";
    let h: String = hint.chars().take((cols - 4).max(0) as usize).collect();
    let _ = write!(buf, "\x1b[{};3H\x1b[38;2;130;130;150m{}", rows, h);

    let sep = mw + 2;
    for y in 2..rows {
        let _ = write!(buf, "\x1b[{};1H\u{2551}", y);
        let _ = write!(buf, "\x1b[{};{}H\u{2503}", y, sep);
        let _ = write!(buf, "\x1b[{};{}H\u{2551}", y, cols);
    }
}

fn draw_box(buf: &mut String, x: i32, y: i32, w: i32, h: i32, title: &str, color: Color) {
    if w < 4 || h < 2 {
        return;
    }
    let inner = (w - 2) as usize;
    let t: String = format!(" {} ", title).chars().take(inner).collect();
    let fill = inner - t.chars().count();
    let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m\u{250c}", y, x, color.0, color.1, color.2);
    let _ = write!(buf, "\x1b[38;2;200;200;215m{}", t);
    let _ = write!(buf, "\x1b[38;2;{};{};{}m", color.0, color.1, color.2);
    for _ in 0..fill {
        buf.push('\u{2500}');
    }
    buf.push('\u{2510}');
    for i in 1..h - 1 {
        let _ = write!(buf, "\x1b[{};{}H\u{2502}", y + i, x);
        let _ = write!(buf, "\x1b[{};{}H\u{2502}", y + i, x + w - 1);
    }
    let _ = write!(buf, "\x1b[{};{}H\u{2514}", y + h - 1, x);
    for _ in 0..inner {
        buf.push('\u{2500}');
    }
    buf.push('\u{2518}');
}

fn bar(buf: &mut String, x: i32, y: i32, w: i32, cur: i32, max: i32, fg: Color, label: &str) {
    let lab = format!("{} ", label);
    put(buf, x, y, (160, 160, 175), &lab);
    let bx = x + lab.chars().count() as i32;
    let suffix = format!(" {}/{}", cur.max(0), max);
    let inner = (w - lab.chars().count() as i32 - suffix.chars().count() as i32).max(3);
    let filled = if max > 0 { (cur.max(0) as i64 * inner as i64 / max as i64) as i32 } else { 0 };
    let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m", y, bx, fg.0, fg.1, fg.2);
    for _ in 0..filled {
        buf.push('\u{2588}');
    }
    let _ = write!(buf, "\x1b[38;2;46;46;56m");
    for _ in filled..inner {
        buf.push('\u{2588}');
    }
    put(buf, bx + inner, y, (200, 200, 210), &suffix);
}

fn draw_panel(game: &Game, cols: i32, rows: i32, mw: i32, buf: &mut String) {
    let px = mw + 3;
    let pin = (cols - mw - 3).max(8);
    if pin < 8 {
        return;
    }
    let blank: String = " ".repeat(pin as usize);
    for y in 2..rows {
        put(buf, px, y, (0, 0, 0), &blank);
    }

    let ph = rows - 2;
    let hh = 13.min(ph);
    let cm = 9.min((ph - hh).max(0));
    let jh = (ph - hh - cm).max(0);
    let ix = px + 1;
    let iw = pin - 2;
    let h = &game.hero;

    draw_box(buf, px, 2, pin, hh, "HEROS", FRAME);
    let mut r = 3;
    put(buf, ix, r, (235, 210, 140), &fit(&format!("{} · {}", game.class.label(), titre(h.level)), iw));
    r += 1;
    put(buf, ix, r, (180, 180, 195), &format!("niveau {}", h.level));
    r += 1;
    bar(buf, ix, r, iw, h.hp, h.max_hp, (90, 210, 110), "PV");
    r += 1;
    bar(buf, ix, r, iw, h.xp, h.xp_next, (110, 160, 240), "XP");
    r += 1;
    put(buf, ix, r, (170, 170, 185), &format!("ATQ {:<4} DEF {:<4} or {}", h.atk(), h.def(), h.gold));
    r += 1;
    let we = h.weapon_element();
    let wtxt = if we != Element::Physical {
        format!("\u{2694} {} [{}]", h.weapon, we.label())
    } else {
        format!("\u{2694} {}", h.weapon)
    };
    put(buf, ix, r, (200, 220, 230), &fit(&wtxt, iw));
    r += 1;
    put(buf, ix, r, (190, 200, 235), &fit(&format!("\u{25c8} {}", h.armor), iw));
    r += 1;
    put(buf, ix, r, (200, 170, 90), &fit(&format!("{} · pot {} · {}%", game.style.label(), h.potions, game.map.discovery_percent()), iw));
    r += 1;
    put(buf, ix, r, (210, 150, 120), &fit(&status_line(game), iw));
    r += 1;
    if game.objective != Objective::None && r < 2 + hh - 1 {
        let (col, mark) = if game.objective_done { ((120, 230, 160), "[OK]") } else { ((230, 210, 130), "") };
        put(buf, ix, r, col, &fit(&format!("Obj: {} {}", game.objective.desc(game.objective_target), mark), iw));
    }

    let cy = 2 + hh;
    if cm >= 4 {
        draw_box(buf, px, cy, pin, cm, "CARTE", FRAME);
        draw_minimap(game, ix, cy + 1, iw, cm - 2, buf);
    }

    let jy = 2 + hh + cm;
    if jh >= 3 {
        draw_box(buf, px, jy, pin, jh, "JOURNAL", FRAME);
        let rows_avail = (jh - 2) as usize;
        let owned: Vec<(String, Color)> = game
            .log
            .iter()
            .rev()
            .take(rows_avail + 4)
            .rev()
            .flat_map(|e| wrap_text(&e.text, iw as usize).into_iter().map(move |s| (s, e.color)))
            .collect();
        let start = owned.len().saturating_sub(rows_avail);
        for (k, (text, color)) in owned[start..].iter().enumerate() {
            put(buf, ix, jy + 1 + k as i32, *color, text);
        }
    }
}

fn status_line(game: &Game) -> String {
    let h = &game.hero;
    let mut s = String::new();
    if h.poison > 0 {
        s.push_str(&format!("\u{2620}{} ", h.poison));
    }
    if h.burn > 0 {
        s.push_str(&format!("\u{2668}{} ", h.burn));
    }
    if h.shield > 0 {
        s.push_str(&format!("\u{26e8}{} ", h.shield));
    }
    if h.regen > 0 {
        s.push_str(&format!("\u{2726}{} ", h.regen));
    }
    if h.bolt_cd > 0 {
        s.push_str(&format!("\u{26a1}{} ", h.bolt_cd));
    }
    if s.is_empty() {
        s.push_str("(en forme)");
    }
    if !h.scrolls.is_empty() {
        s.push_str(&format!("\u{00a7}{} ", h.scrolls.len()));
    }
    if !h.talents.is_empty() {
        s.push_str(&format!("\u{2605}{}", h.talents.len()));
    }
    s
}

fn draw_minimap(game: &Game, x: i32, y: i32, w: i32, h: i32, buf: &mut String) {
    if w < 1 || h < 1 {
        return;
    }
    let mw = game.map.width;
    let mh = game.map.height;
    let sx = ((mw + w - 1) / w).max(1);
    let sy = ((mh + h - 1) / h).max(1);
    for j in 0..h {
        let _ = write!(buf, "\x1b[{};{}H", y + j, x);
        let mut last: Option<Color> = None;
        for i in 0..w {
            let (ch, col) = minimap_cell(game, i * sx, j * sy, sx, sy);
            if last != Some(col) {
                let _ = write!(buf, "\x1b[38;2;{};{};{}m", col.0, col.1, col.2);
                last = Some(col);
            }
            buf.push(ch);
        }
    }
}

fn minimap_cell(game: &Game, x0: i32, y0: i32, sx: i32, sy: i32) -> (char, Color) {
    let mut hero = false;
    let mut boss = false;
    let mut mob = false;
    let mut stairs = false;
    let mut floor_vis = false;
    let mut floor_exp = false;
    let mut wall_exp = false;
    for yy in y0..(y0 + sy).min(game.map.height) {
        for xx in x0..(x0 + sx).min(game.map.width) {
            if game.hero.x == xx && game.hero.y == yy {
                hero = true;
            }
            if !game.map.is_explored(xx, yy) {
                continue;
            }
            let vis = game.map.is_visible(xx, yy);
            match game.map.tile(xx, yy) {
                Tile::Wall => wall_exp = true,
                Tile::Floor => {
                    if vis {
                        floor_vis = true;
                    } else {
                        floor_exp = true;
                    }
                }
                Tile::StairsDown => stairs = true,
            }
            if vis {
                if let Some(i) = game.monster_at(xx, yy) {
                    if game.monsters[i].boss {
                        boss = true;
                    } else {
                        mob = true;
                    }
                }
            }
        }
    }
    if hero {
        ('@', (255, 245, 150))
    } else if boss {
        ('\u{2588}', (235, 70, 70))
    } else if mob {
        ('\u{2588}', (210, 110, 90))
    } else if stairs {
        ('>', (245, 235, 120))
    } else if floor_vis {
        ('\u{2588}', (90, 95, 115))
    } else if floor_exp {
        ('\u{2588}', (48, 50, 62))
    } else if wall_exp {
        ('\u{2588}', (28, 26, 30))
    } else {
        (' ', (0, 0, 0))
    }
}

fn fit(text: &str, width: i32) -> String {
    text.chars().take(width.max(0) as usize).collect()
}

fn floor_tint(floor: i32) -> (f32, f32, f32) {
    const THEMES: [(f32, f32, f32); 6] = [
        (1.00, 1.00, 1.00),
        (0.84, 1.06, 0.90),
        (1.12, 0.92, 0.84),
        (0.85, 0.96, 1.18),
        (1.06, 0.86, 1.12),
        (1.12, 1.00, 0.78),
    ];
    THEMES[((floor - 1).max(0) as usize) % THEMES.len()]
}

fn light_add(base: Color, lights: &[(f32, f32, Color)], x: i32, y: i32) -> Color {
    let mut r = base.0 as f32;
    let mut g = base.1 as f32;
    let mut b = base.2 as f32;
    for &(lx, ly, col) in lights {
        let dx = lx - x as f32;
        let dy = ly - y as f32;
        let d2 = dx * dx + dy * dy;
        if d2 < 9.0 {
            let mut inten = 1.0 - d2.sqrt() / 3.0;
            inten = (inten * inten * 0.8).max(0.0);
            r += col.0 as f32 * inten;
            g += col.1 as f32 * inten;
            b += col.2 as f32 * inten;
        }
    }
    (r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
}

fn shade(c: Color, light: f32, tint: (f32, f32, f32)) -> Color {
    let r = (c.0 as f32 * light * tint.0).clamp(0.0, 255.0) as u8;
    let g = (c.1 as f32 * light * tint.1).clamp(0.0, 255.0) as u8;
    let b = (c.2 as f32 * light * tint.2).clamp(0.0, 255.0) as u8;
    (r, g, b)
}

fn cell_render(game: &Game, x: i32, y: i32, tint: (f32, f32, f32)) -> (char, Color, Color) {
    let visible = game.map.is_visible(x, y);
    let explored = game.map.is_explored(x, y);
    if !visible && !explored {
        return (' ', (0, 0, 0), (0, 0, 0));
    }
    let tile = game.map.tile(x, y);

    if !visible {
        let (fg, bg, ch) = match tile {
            Tile::Wall => ((44, 40, 36), (22, 20, 17), ' '),
            Tile::Floor => ((30, 29, 35), (11, 11, 14), '\u{00b7}'),
            Tile::StairsDown => ((120, 110, 64), (18, 17, 13), '>'),
        };
        return (ch, fg, bg);
    }

    let dx = (x - game.hero.x) as f32;
    let dy = (y - game.hero.y) as f32;
    let light = (1.2 - (dx * dx + dy * dy).sqrt() * 0.085).clamp(0.34, 1.0);

    let (terrain_fg, terrain_bg, terrain_ch) = match tile {
        Tile::Wall => ((150, 134, 112), (78, 67, 54), ' '),
        Tile::Floor => ((64, 61, 74), (26, 25, 32), '\u{00b7}'),
        Tile::StairsDown => ((255, 240, 140), (46, 42, 30), '>'),
    };
    let bg_lit = shade(terrain_bg, light, tint);
    let mut result = (terrain_ch, shade(terrain_fg, light, tint), bg_lit);

    if game.hero.x == x && game.hero.y == y {
        result = ('@', (255, 246, 150), bg_lit);
    } else if game.pet.as_ref().is_some_and(|p| p.x == x && p.y == y) {
        result = ('d', (120, 230, 180), bg_lit);
    } else if let Some(i) = game.monster_at(x, y) {
        let m = &game.monsters[i];
        result = (m.glyph, shade(m.color, light.max(0.8), (1.0, 1.0, 1.0)), bg_lit);
    } else if game.merchant.as_ref().is_some_and(|m| m.x == x && m.y == y) {
        result = ('M', (130, 235, 240), bg_lit);
    } else if let Some(it) = game.items.iter().find(|it| it.x == x && it.y == y) {
        result = (it.glyph, shade(it.color, light.max(0.82), (1.0, 1.0, 1.0)), bg_lit);
    } else if let Some(f) = game.features.iter().find(|f| f.x == x && f.y == y) {
        let (ch, col) = match f.kind {
            FeatureKind::Shrine => ('\u{2691}', (205, 175, 255)),
            FeatureKind::Fountain => ('\u{2248}', (110, 205, 235)),
            FeatureKind::Chest => ('\u{25a4}', (255, 210, 90)),
            FeatureKind::Altar => ('\u{2628}', (215, 110, 235)),
            FeatureKind::Familiar => ('d', (120, 230, 180)),
            FeatureKind::Trap => ('^', (210, 95, 75)),
        };
        result = (ch, shade(col, light.max(0.8), (1.0, 1.0, 1.0)), bg_lit);
    }

    let dcol = if game.danger.iter().any(|&(dx, dy)| dx == x && dy == y) {
        Some(game.danger_color)
    } else if game.cast_danger.iter().any(|&(dx, dy)| dx == x && dy == y) {
        Some((235, 140, 60))
    } else {
        None
    };
    if let Some(d) = dcol {
        result.2 = (
            ((d.0 as u16 + result.2.0 as u16) / 2) as u8,
            ((d.1 as u16 + result.2.1 as u16) / 2) as u8,
            ((d.2 as u16 + result.2.2 as u16) / 2) as u8,
        );
    }
    if let Some(f) = game.flashes.iter().find(|f| f.0 == x && f.1 == y) {
        result.2 = f.2;
    }
    result
}

fn draw_fx(game: &Game, mw: i32, mh: i32, sdx: i32, buf: &mut String) {
    for p in &game.fx.particles {
        let x = p.x.round() as i32 + sdx;
        let y = p.y.round() as i32;
        if x >= 0 && x < mw && y >= 0 && y < mh {
            put(buf, MCOL + x, MROW + y, p.color, &p.glyph.to_string());
        }
    }
    for p in &game.fx.projectiles {
        let x = p.x.round() as i32 + sdx;
        let y = p.y.round() as i32;
        if x >= 0 && x < mw && y >= 0 && y < mh {
            put(buf, MCOL + x, MROW + y, p.color, &p.glyph.to_string());
        }
    }
    for f in &game.fx.floats {
        let x = f.x.round() as i32 + sdx;
        let y = f.y.round() as i32;
        if y >= 0 && y < mh && x >= 0 && x < mw {
            let text: String = f.text.chars().take((mw - x) as usize).collect();
            put(buf, MCOL + x, MROW + y, f.color, &text);
        }
    }
}

fn draw_transition(floor: i32, mw: i32, mh: i32, buf: &mut String) {
    let text = format!("\u{2726}  ETAGE {}  \u{2726}", floor);
    let w = text.chars().count() as i32 + 4;
    let ox = ((mw - w) / 2).max(0);
    let oy = mh / 2 - 1;
    let c: Color = (200, 160, 245);
    put(buf, MCOL + ox, MROW + oy, c, "\u{2554}");
    for _ in 0..w {
        buf.push('\u{2550}');
    }
    buf.push('\u{2557}');
    let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m\u{2551}  \x1b[38;2;255;240;180m{}\x1b[38;2;{};{};{}m  \u{2551}", MROW + oy + 1, MCOL + ox, c.0, c.1, c.2, text, c.0, c.1, c.2);
    put(buf, MCOL + ox, MROW + oy + 2, c, "\u{255a}");
    for _ in 0..w {
        buf.push('\u{2550}');
    }
    buf.push('\u{255d}');
    buf.push_str("\x1b[0m");
}

fn draw_top_voters(game: &Game, mh: i32, buf: &mut String) {
    let mut lines: Vec<String> = vec!["TOP CHAT".to_string()];
    for (i, (user, n)) in game.top_voters.iter().enumerate() {
        let u: String = user.chars().take(14).collect();
        lines.push(format!("{}. {} ({})", i + 1, u, n));
    }
    let inner = lines.iter().map(|l| l.chars().count()).max().unwrap_or(8) as i32 + 1;
    let bh = lines.len() as i32 + 2;
    let oy = (mh - bh).max(0);
    let border: Color = (180, 130, 235);
    put(buf, MCOL, MROW + oy, border, "\u{250c}");
    for _ in 0..inner {
        buf.push('\u{2500}');
    }
    buf.push('\u{2510}');
    for (i, l) in lines.iter().enumerate() {
        let color = if i == 0 { (200, 170, 90) } else { (210, 210, 225) };
        put(buf, MCOL, MROW + oy + 1 + i as i32, border, "\u{2502}");
        let mut content: String = l.chars().take(inner as usize).collect();
        while (content.chars().count() as i32) < inner {
            content.push(' ');
        }
        let _ = write!(buf, "\x1b[38;2;{};{};{}m{}", color.0, color.1, color.2, content);
        let _ = write!(buf, "\x1b[38;2;{};{};{}m\u{2502}", border.0, border.1, border.2);
    }
    put(buf, MCOL, MROW + oy + 1 + lines.len() as i32, border, "\u{2514}");
    for _ in 0..inner {
        buf.push('\u{2500}');
    }
    buf.push('\u{2518}');
    buf.push_str("\x1b[0m");
}

fn draw_shop(game: &Game, mw: i32, buf: &mut String) {
    let Some(m) = game.merchant.as_ref() else { return };
    let v = &game.merchant_votes;
    let mut lines: Vec<String> = Vec::new();
    match &m.weapon {
        Some((name, bonus, price)) => lines.push(format!("!arme    {:<16} ATQ+{:<2} {:>4}or [{}]", name, bonus, price, v[0])),
        None => lines.push(format!("!arme    {:<26} [{}]", "(rien)", v[0])),
    }
    match &m.armor {
        Some((name, bonus, price)) => lines.push(format!("!armure  {:<16} DEF+{:<2} {:>4}or [{}]", name, bonus, price, v[1])),
        None => lines.push(format!("!armure  {:<26} [{}]", "(rien)", v[1])),
    }
    lines.push(format!("!potion  {:<23} {:>4}or [{}]", "potion de soin", m.potion_price, v[2]));
    lines.push(format!("!soin    {:<23} {:>4}or [{}]", "soin complet", m.heal_price, v[3]));
    lines.push(format!("!reroll  {:<28} [{}]", "nouveau stock", v[4]));
    lines.push(format!("!purge   {:<28} [{}]", "retirer les maux", v[5]));
    lines.push(format!("!skip    {:<28} [{}]", "passer son tour", v[6]));
    lines.push(match game.forced_purchase {
        Some(p) => format!("le chat achete: {}", p.label()),
        None => "marchandage en cours...".to_string(),
    });

    let inner = lines.iter().map(|l| l.chars().count()).max().unwrap_or(20) as i32 + 2;
    let bw = inner + 2;
    let ox = ((mw - bw) / 2).max(0);
    let oy = 3;
    let border: Color = (130, 235, 240);
    put(buf, MCOL + ox, MROW + oy, border, "\u{2554}");
    let title = " MARCHAND ";
    let tpad = (inner - title.chars().count() as i32).max(0);
    for _ in 0..tpad / 2 {
        buf.push('\u{2550}');
    }
    buf.push_str(title);
    for _ in 0..(tpad - tpad / 2) {
        buf.push('\u{2550}');
    }
    buf.push('\u{2557}');
    for (i, l) in lines.iter().enumerate() {
        let color = if i + 1 == lines.len() { (255, 225, 120) } else { (220, 220, 230) };
        put(buf, MCOL + ox, MROW + oy + 1 + i as i32, border, "\u{2551} ");
        let mut content: String = l.chars().take(inner as usize - 1).collect();
        while (content.chars().count() as i32) < inner - 1 {
            content.push(' ');
        }
        let _ = write!(buf, "\x1b[38;2;{};{};{}m{}", color.0, color.1, color.2, content);
        let _ = write!(buf, "\x1b[38;2;{};{};{}m\u{2551}", border.0, border.1, border.2);
    }
    put(buf, MCOL + ox, MROW + oy + 1 + lines.len() as i32, border, "\u{255a}");
    for _ in 0..inner {
        buf.push('\u{2550}');
    }
    buf.push('\u{255d}');
    buf.push_str("\x1b[0m");
}

fn draw_boss_bar(mw: i32, name: &str, hp: i32, max_hp: i32, color: Color, buf: &mut String) {
    let width = (mw - 6).clamp(16, 54);
    let ox = (mw - width) / 2;
    let title = format!("\u{2620} BOSS — {}  {}/{}", name, hp.max(0), max_hp);
    let tx = MCOL + ox + (width - title.chars().count() as i32).max(0) / 2;
    put(buf, tx.max(MCOL), MROW, color, &title);
    let filled = if max_hp > 0 { (hp.max(0) as i64 * width as i64 / max_hp as i64) as i32 } else { 0 };
    let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m", MROW + 1, MCOL + ox, color.0, color.1, color.2);
    for _ in 0..filled {
        buf.push('\u{2588}');
    }
    let _ = write!(buf, "\x1b[38;2;60;28;30m");
    for _ in filled..width {
        buf.push('\u{2588}');
    }
    buf.push_str("\x1b[0m");
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split(' ') {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn titre(level: i32) -> &'static str {
    match level {
        1 => "Novice",
        2 | 3 => "Aventurier",
        4 | 5 => "Mercenaire",
        6 | 7 => "Veteran",
        8 | 9 => "Champion",
        10..=12 => "Heros",
        _ => "Legende",
    }
}

fn top_scores(game: &Game) -> String {
    if game.high_scores.is_empty() {
        return "-".to_string();
    }
    game.high_scores
        .iter()
        .take(3)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("  ")
}

fn draw_death(game: &Game, mw: i32, mh: i32, buf: &mut String) {
    let lines = [
        "  V O U S   E T E S   M O R T  ".to_string(),
        String::new(),
        format!("  {} tue par {}", game.class.label(), game.last_cause),
        format!("  etage {}  ·  niveau {}", game.floor, game.hero.level),
        format!("  {} or  ·  {} elimines", game.hero.gold, game.hero.kills),
        format!("  record: etage {}  ·  {} tues", game.best_floor, game.total_kills),
        format!("  SCORE {}", game.last_score),
        format!("  top: {}", top_scores(game)),
        String::new(),
        format!("  \" {} \"", game.death_quip),
        String::new(),
        "  une nouvelle ame se prepare...".to_string(),
    ];
    let bw = lines.iter().map(|l| l.chars().count()).max().unwrap_or(20) as i32 + 4;
    let bh = lines.len() as i32 + 2;
    let ox = (mw - bw) / 2;
    let oy = (mh - bh) / 2;
    let border: Color = (235, 90, 80);
    let fill: Color = (235, 220, 200);

    for i in 0..bh {
        let yy = oy + i;
        if yy < 0 || yy >= mh {
            continue;
        }
        put(buf, MCOL + ox.max(0), MROW + yy, border, "");
        if i == 0 || i == bh - 1 {
            buf.push('+');
            for _ in 0..bw - 2 {
                buf.push('\u{2550}');
            }
            buf.push('+');
        } else {
            buf.push('\u{2551}');
            let text = lines.get((i - 1) as usize).map(|s| s.as_str()).unwrap_or("");
            let _ = write!(buf, "\x1b[38;2;{};{};{}m", fill.0, fill.1, fill.2);
            let mut content: String = text.chars().collect();
            while (content.chars().count() as i32) < bw - 2 {
                content.push(' ');
            }
            buf.push_str(&content);
            let _ = write!(buf, "\x1b[38;2;{};{};{}m\u{2551}", border.0, border.1, border.2);
        }
    }
}
