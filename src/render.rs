use crate::entity::{Affix, Color, Element, FeatureKind, ScrollKind};
use crate::game::{Biome, FloorEvent, Game, Objective, Phase};
use crate::map::Tile;
use std::fmt::Write as _;
use std::io::Write;

const MROW: i32 = 2;
const MCOL: i32 = 2;
const FRAME: Color = (95, 95, 120);

pub fn draw(game: &Game, cols: i32, rows: i32, paused: bool, speed_label: &str, sprite: bool, zoom: i32, out: &mut impl Write) {
    let mut buf = String::with_capacity((cols * rows) as usize * 7);
    buf.push_str("\x1b[H");

    let mw = game.map.width;
    let mh = game.map.height;
    let tint = if game.room_kind == crate::game::RoomKind::Rift {
        (1.15, 0.7, 1.25)
    } else if game.event == FloorEvent::Inferno {
        (1.22, 0.82, 0.66)
    } else {
        game.biome.tint()
    };
    let sdx = game.fx.shake_offset();
    let lights: Vec<(f32, f32, Color)> = game.fx.projectiles.iter().map(|p| (p.x, p.y, p.color)).collect();
    let vignette = if matches!(game.phase, Phase::Playing) {
        let frac = game.hp_fraction();
        if frac < 0.30 {
            ((0.30 - frac) / 0.30) * (0.55 + 0.45 * game.low_hp_pulse.clamp(0.0, 1.0))
        } else {
            0.0
        }
    } else {
        0.0
    };

    if sprite {
        draw_sprite_map(game, mw, mh, sdx, tint, vignette, zoom, &mut buf);
        draw_fx_sprite(game, mw, mh, sdx, zoom, &mut buf);
    } else {
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
                if vignette > 0.0 {
                    bg = vignette_add(bg, x, y, mw, mh, vignette);
                }
                if last != Some((fg, bg)) {
                    let _ = write!(buf, "\x1b[38;2;{};{};{};48;2;{};{};{}m", fg.0, fg.1, fg.2, bg.0, bg.1, bg.2);
                    last = Some((fg, bg));
                }
                buf.push(ch);
            }
        }
        draw_fx(game, mw, mh, sdx, &mut buf);
    }
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
    if game.show_codex {
        draw_codex(game, cols, rows, &mut buf);
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
    let muts = if game.mutators.is_empty() {
        String::new()
    } else {
        let names: Vec<&str> = game.mutators.iter().map(|m| m.label()).collect();
        format!("  ·  \u{2622} {}", names.join("+"))
    };
    let asc = if game.ascension > 0 { format!("  ·  Asc.{}", game.ascension) } else { String::new() };
    let title = format!(
        " ABYSSAL  ·  etage {} {}  ·  {}  ·  run #{}  ·  {}{}{}{} ",
        game.floor,
        game.biome.label(),
        game.class.label(),
        game.runs,
        if paused { "PAUSE" } else { speed_label },
        diff,
        boon,
        evt
    );
    let title = format!("{}{}{}", title.trim_end(), asc, muts);
    let title = format!("{} ", title);
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
    let hint = " espace:pause  +/-:vitesse  m:mindset  a:son  g:sprite  z:zoom  k:bestiaire  b:marchand  s/l/n  q:quitter ";
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
    let hh = 10.min(ph);
    let eq = 14.min((ph - hh).max(0));
    let jh = (ph - hh - eq).max(0);
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
    put(buf, ix, r, (200, 170, 90), &fit(&format!("{} · {}%", game.style.label(), game.map.discovery_percent()), iw));
    r += 1;
    put(buf, ix, r, (210, 150, 120), &fit(&status_line(game), iw));
    r += 1;
    if game.objective != Objective::None && r < 2 + hh - 1 {
        let (col, mark) = if game.objective_done { ((120, 230, 160), "[OK]") } else { ((230, 210, 130), "") };
        put(buf, ix, r, col, &fit(&format!("Obj: {} {}", game.objective.desc(game.objective_target), mark), iw));
    }

    let eqy = 2 + hh;
    if eq >= 5 {
        draw_box(buf, px, eqy, pin, eq, "EQUIPEMENT", FRAME);
        let we = h.weapon_element();
        let wsuffix = if we != Element::Physical { format!(" [{}]", we.label()) } else { String::new() };
        let last = eqy + eq - 1;
        let mut er = eqy + 1;
        let line = |buf: &mut String, er: &mut i32, col: Color, s: String| {
            if *er < last {
                put(buf, ix, *er, col, &fit(&s, iw));
                *er += 1;
            }
        };
        line(buf, &mut er, (205, 222, 232), format!("\u{2694} {}{}", slot_txt(&h.weapon, h.weapon_bonus, h.weapon_affix), wsuffix));
        line(buf, &mut er, (195, 205, 238), format!("\u{25c8} {}", slot_txt(&h.armor, h.armor_bonus, h.armor_affix)));
        let ring = if h.ring != Affix::None { format!("anneau +{} {}", h.ring_bonus, h.ring.label()) } else { "anneau: -".to_string() };
        let amu = if h.amulet != Affix::None { format!("amulette +{} {}", h.amulet_bonus, h.amulet.label()) } else { "amulette: -".to_string() };
        line(buf, &mut er, (210, 200, 160), format!("\u{2218} {}", ring));
        line(buf, &mut er, (210, 200, 160), format!("\u{2666} {}", amu));
        if let Some(a) = h.set_affix() {
            line(buf, &mut er, (255, 215, 120), format!("SET {} \u{00d7}{}", a.label(), h.set_bonus()));
        }
        if h.armor_element() != Element::Physical {
            line(buf, &mut er, (170, 210, 190), format!("resiste : {}", h.armor_element().label()));
        }
        er += 1;
        line(buf, &mut er, (180, 215, 160), format!("sac: pot {}", h.potions));
        line(buf, &mut er, (180, 215, 160), scroll_breakdown(&h.scrolls));
        if h.talents.is_empty() {
            line(buf, &mut er, (150, 160, 175), "talents: -".to_string());
        } else {
            let names: Vec<&str> = h.talents.iter().map(|t| t.label().split(' ').next().unwrap_or("")).collect();
            let joined = format!("\u{2605} {}", names.join(", "));
            for w in wrap_text(&joined, iw as usize) {
                line(buf, &mut er, (185, 200, 235), w);
            }
        }
        if !h.relics.is_empty() {
            let names: Vec<&str> = h.relics.iter().map(|r| r.short()).collect();
            for w in wrap_text(&format!("\u{2726} {}", names.join(", ")), iw as usize) {
                line(buf, &mut er, (255, 210, 120), w);
            }
        }
        if let Some(p) = game.pet.as_ref() {
            line(buf, &mut er, (140, 230, 185), format!("\u{2726} {} niv{} {}/{}", p.name, p.level, p.hp, p.max_hp));
        }
        for a in game.allies.iter().filter(|a| a.companion) {
            line(buf, &mut er, (255, 224, 150), format!("\u{2665} {} ({}) n{} {}/{}", a.name, crate::entity::ally_role_label(a.role), a.level, a.hp, a.max_hp));
        }
    }

    let jy = 2 + hh + eq;
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

fn draw_codex(game: &Game, cols: i32, rows: i32, buf: &mut String) {
    let entries = crate::entity::bestiary();
    let bw = 48.min((cols - 4).max(20));
    let bh = ((entries.len() as i32) + 4).min((rows - 2).max(8));
    let ox = (cols - bw) / 2;
    let oy = (rows - bh) / 2;
    let blank: String = " ".repeat(bw as usize);
    for y in oy..oy + bh {
        put(buf, ox, y, (0, 0, 0), &blank);
    }
    draw_box(buf, ox, oy, bw, bh, "BESTIAIRE  (k: fermer)", (150, 150, 185));
    let found_n = entries.iter().filter(|e| game.discovered.iter().any(|n| n == e.2)).count();
    put(buf, ox + 2, oy + 1, (190, 190, 210), &fit(&format!("decouverts : {}/{}", found_n, entries.len()), bw - 4));
    let mut r = oy + 2;
    for (glyph, color, name, elem, minf, beh) in entries {
        if r >= oy + bh - 1 {
            break;
        }
        if game.discovered.iter().any(|n| n == name) {
            put(buf, ox + 2, r, color, &fit(&format!("{} {:<11} {:<7} {:<10} e{}+", glyph, name, elem, beh, minf), bw - 4));
        } else {
            put(buf, ox + 2, r, (95, 95, 105), &fit(&format!("\u{2592} ??? (etage {}+)", minf), bw - 4));
        }
        r += 1;
    }
}

fn slot_txt(name: &str, bonus: i32, affix: Affix) -> String {
    let aff = if affix != Affix::None { format!(" {}", affix.label()) } else { String::new() };
    if bonus > 0 {
        format!("{} +{}{}", name, bonus, aff)
    } else {
        format!("{}{}", name, aff)
    }
}

fn scroll_breakdown(scrolls: &[ScrollKind]) -> String {
    if scrolls.is_empty() {
        return "parch -".to_string();
    }
    let f = scrolls.iter().filter(|s| matches!(s, ScrollKind::Fireball)).count();
    let g = scrolls.iter().filter(|s| matches!(s, ScrollKind::Freeze)).count();
    let t = scrolls.iter().filter(|s| matches!(s, ScrollKind::Teleport)).count();
    let l = scrolls.iter().filter(|s| matches!(s, ScrollKind::Lightning)).count();
    let mut parts: Vec<String> = Vec::new();
    if f > 0 {
        parts.push(format!("feu{}", f));
    }
    if g > 0 {
        parts.push(format!("gel{}", g));
    }
    if t > 0 {
        parts.push(format!("tp{}", t));
    }
    if l > 0 {
        parts.push(format!("ecl{}", l));
    }
    format!("parch {}", parts.join(" "))
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
    if h.rage > 0 {
        s.push_str(&format!("\u{25b2}{} ", h.rage));
    }
    if h.bolt_cd > 0 {
        s.push_str(&format!("\u{26a1}{} ", h.bolt_cd));
    }
    if h.ability_cd > 0 {
        s.push_str(&format!("\u{2694}{} ", h.ability_cd));
    }
    if s.is_empty() {
        s.push_str("(en forme)");
    }
    if !h.scrolls.is_empty() {
        s.push_str(&format!("\u{00a7}{} ", h.scrolls.len()));
    }
    if !h.talents.is_empty() {
        s.push_str(&format!("\u{2605}{} ", h.talents.len()));
    }
    let set = h.set_bonus();
    if set > 0 {
        s.push_str(&format!("SET\u{00d7}{}", set));
    }
    s
}

fn fit(text: &str, width: i32) -> String {
    text.chars().take(width.max(0) as usize).collect()
}

fn biome_palette(biome: Biome) -> ((Color, Color), (Color, Color)) {
    biome.palette()
}

fn px_dark(c: Color, f: f32) -> Color {
    ((c.0 as f32 * f) as u8, (c.1 as f32 * f) as u8, (c.2 as f32 * f) as u8)
}

fn px_lite(c: Color, f: f32) -> Color {
    (
        (c.0 as f32 + (255.0 - c.0 as f32) * f) as u8,
        (c.1 as f32 + (255.0 - c.1 as f32) * f) as u8,
        (c.2 as f32 + (255.0 - c.2 as f32) * f) as u8,
    )
}

fn px_blend(base: Color, over: Color, a: f32) -> Color {
    (
        (base.0 as f32 * (1.0 - a) + over.0 as f32 * a) as u8,
        (base.1 as f32 * (1.0 - a) + over.1 as f32 * a) as u8,
        (base.2 as f32 * (1.0 - a) + over.2 as f32 * a) as u8,
    )
}

fn overlay_sprite(cell: &mut [[Color; 4]; 4], pat: &[&str; 4], color: Color, ox: i32, oy: i32) {
    for (sy, row) in pat.iter().enumerate() {
        for (sx, ch) in row.chars().enumerate() {
            if sx >= 4 {
                break;
            }
            let c = match ch {
                'X' => color,
                '.' => px_dark(color, 0.65),
                '*' => px_lite(color, 0.45),
                'o' => (24, 20, 26),
                'v' => (235, 235, 235),
                _ => continue,
            };
            let tx = sx as i32 + ox;
            let ty = sy as i32 + oy;
            if (0..4).contains(&tx) && (0..4).contains(&ty) {
                cell[ty as usize][tx as usize] = c;
            }
        }
    }
}

fn decor_glyph(kind: u8) -> (char, Color) {
    match kind {
        1 => ('"', (90, 140, 80)),
        2 => ('\u{2248}', (80, 120, 160)),
        3 => (',', (170, 160, 140)),
        4 => ('\u{2237}', (90, 90, 100)),
        5 => ('\u{2727}', (150, 200, 230)),
        6 => ('\u{00b0}', (200, 215, 230)),
        7 => ('\u{2219}', (210, 120, 60)),
        8 => ('\u{00b7}', (160, 110, 190)),
        _ => (' ', (0, 0, 0)),
    }
}

fn feature_color(kind: FeatureKind) -> Color {
    match kind {
        FeatureKind::Shrine => (205, 175, 255),
        FeatureKind::Fountain => (110, 205, 235),
        FeatureKind::Chest => (255, 210, 90),
        FeatureKind::Altar => (215, 110, 235),
        FeatureKind::Familiar => (120, 230, 180),
        FeatureKind::Trap => (210, 95, 75),
        FeatureKind::Forge => (255, 170, 70),
        FeatureKind::Gamble => (235, 200, 120),
        FeatureKind::Lost => (255, 224, 150),
    }
}

const SPR_HERO: [&str; 4] = [" ** ", " XX ", "XXXX", "X  X"];
const SPR_CREATURE: [&str; 4] = [" XX ", "XXXX", "XooX", "X  X"];
const SPR_BOSS: [&str; 4] = ["XXXX", "XooX", "XXXX", " XX "];
const SPR_ITEM: [&str; 4] = [" X  ", "XXX ", " X  ", "    "];
const SPR_MERCHANT: [&str; 4] = [" XX ", "XXXX", "X  X", "X  X"];
const SPR_VERMIN: [&str; 4] = ["    ", "oXXo", "XXXX", "X  X"];
const SPR_ARCHER: [&str; 4] = [" XX*", "XXX*", "Xoo*", "X X*"];
const SPR_CASTER: [&str; 4] = [" *  ", " ** ", "XXXX", "XooX"];
const SPR_BRUTE: [&str; 4] = ["XXXX", "XooX", "XXXX", "XXXX"];
const SPR_DEMON: [&str; 4] = ["*XX*", "XXXX", "XooX", "X  X"];
const SPR_DRAGON: [&str; 4] = ["*XX*", "XXXX", "XooX", "*XX*"];
const SPR_MIMIC: [&str; 4] = ["XXXX", "XvvX", "XXXX", "X  X"];
const SPR_FINAL: [&str; 4] = ["*vv*", "vXXv", "XooX", "vXXv"];
const SPR_BOMB: [&str; 4] = [" *  ", "XXXX", "XXXX", " XX "];
const SPR_BAT: [&str; 4] = ["*  *", "XvvX", " XX ", "    "];
const SPR_BLOB: [&str; 4] = [" .. ", "XooX", "XXXX", " XX "];
const SPR_FLAME: [&str; 4] = ["  * ", " ** ", " XX ", "XXXX"];
const SPR_WINGED: [&str; 4] = ["*XX*", "vXXv", "XooX", " XX "];
const SPR_COIN: [&str; 4] = [" XX ", "X*vX", "Xv*X", " XX "];
const SPR_POTION: [&str; 4] = [" .. ", " XX ", "X*XX", "XXXX"];
const SPR_BLADE: [&str; 4] = ["  .v", " .X ", ".X. ", "*.  "];
const SPR_ARMOR: [&str; 4] = [" XX ", "X..X", "X..X", "XXXX"];
const SPR_RING: [&str; 4] = ["    ", " vv ", "v  v", " vv "];
const SPR_AMULET: [&str; 4] = [" v  ", "v X ", " X v", "  v "];
const SPR_SCROLL: [&str; 4] = ["XXXX", "X..X", "XXXX", " .. "];
const SPR_CHEST: [&str; 4] = ["XXXX", "X**X", "XvvX", "XXXX"];
const SPR_TRAP: [&str; 4] = ["v v ", "XXXX", "v v ", "    "];

const SPR_EYE: [&str; 4] = [" XX ", "X**X", "X**X", " XX "];

fn item_sprite(glyph: char) -> &'static [&'static str; 4] {
    match glyph {
        '$' => &SPR_COIN,
        '!' => &SPR_POTION,
        '/' => &SPR_BLADE,
        '[' => &SPR_ARMOR,
        '\u{2218}' => &SPR_RING,
        '\u{2666}' => &SPR_AMULET,
        '?' => &SPR_SCROLL,
        '\u{2609}' => &SPR_EYE,
        _ => &SPR_ITEM,
    }
}

fn feature_sprite(kind: FeatureKind) -> &'static [&'static str; 4] {
    match kind {
        FeatureKind::Chest => &SPR_CHEST,
        FeatureKind::Trap => &SPR_TRAP,
        FeatureKind::Fountain => &SPR_POTION,
        FeatureKind::Lost => &SPR_HERO,
        _ => &SPR_ITEM,
    }
}

fn monster_sprite(m: &crate::entity::Monster) -> &'static [&'static str; 4] {
    if m.boss {
        return if m.glyph == '\u{2638}' { &SPR_FINAL } else { &SPR_BOSS };
    }
    match m.glyph {
        'r' | 'v' | 'p' | 'L' => &SPR_VERMIN,
        'b' | 'u' => &SPR_BAT,
        'a' => &SPR_ARCHER,
        'w' | 'S' | 'c' | 'N' | 'f' => &SPR_CASTER,
        'O' | 'T' | 'P' | 'B' | 'C' => &SPR_BRUTE,
        'D' | 'i' => &SPR_DEMON,
        'Y' | 'A' | 'Q' | 'H' => &SPR_DRAGON,
        'z' => &SPR_BOMB,
        'j' => &SPR_BLOB,
        'e' => &SPR_FLAME,
        'M' | 'x' | 'n' | 'G' | 'R' => &SPR_WINGED,
        '\u{25a4}' => &SPR_MIMIC,
        _ => &SPR_CREATURE,
    }
}

fn sprite_cam(game: &Game, mw: i32, mh: i32, zoom: i32) -> (i32, i32, i32) {
    let t = zoom.clamp(2, 8);
    let camw = (mw / t).max(1);
    let camh = (mh * 2 / t).max(1);
    let cx0 = (game.hero.x - camw / 2).clamp(0, (game.map.width - camw).max(0));
    let cy0 = (game.hero.y - camh / 2).clamp(0, (game.map.height - camh).max(0));
    (cx0, cy0, t)
}

fn draw_fx_sprite(game: &Game, mw: i32, mh: i32, sdx: i32, zoom: i32, buf: &mut String) {
    let (cx0, cy0, t) = sprite_cam(game, mw, mh, zoom);
    let map_x = |wx: f32| MCOL + sdx + ((wx - cx0 as f32) * t as f32 + t as f32 * 0.5) as i32;
    let map_y = |wy: f32| MROW + ((wy - cy0 as f32) * (t as f32 * 0.5) + t as f32 * 0.25) as i32;
    for p in &game.fx.particles {
        let x = map_x(p.x);
        let y = map_y(p.y);
        if x >= MCOL && x < MCOL + mw && y >= MROW && y < MROW + mh {
            put(buf, x, y, p.color, &p.glyph.to_string());
        }
    }
    for p in &game.fx.projectiles {
        let x = map_x(p.x);
        let y = map_y(p.y);
        if x >= MCOL && x < MCOL + mw && y >= MROW && y < MROW + mh {
            put(buf, x, y, p.color, &p.glyph.to_string());
        }
    }
    for f in &game.fx.floats {
        let x = map_x(f.x);
        let y = map_y(f.y);
        if x >= MCOL && x < MCOL + mw && y >= MROW && y < MROW + mh {
            let room = (MCOL + mw - x).max(0) as usize;
            let text: String = f.text.chars().take(room).collect();
            put(buf, x, y, f.color, &text);
        }
    }
}

fn draw_sprite_map(game: &Game, mw: i32, mh: i32, sdx: i32, tint: (f32, f32, f32), vignette: f32, zoom: i32, buf: &mut String) {
    let pw = mw;
    let ph = mh * 2;
    let (cx0, cy0, t) = sprite_cam(game, mw, mh, zoom);
    let camw = (pw / t).max(1);
    let camh = (ph / t).max(1);
    let bgfill: Color = (6, 6, 9);
    let mut fb = vec![bgfill; (pw * ph) as usize];
    let ((wall_fg, wall_bg), (floor_fg, floor_bg)) = biome_palette(game.biome);

    for ty in 0..camh {
        for tx in 0..camw {
            let wx = cx0 + tx;
            let wy = cy0 + ty;
            if !game.map.in_bounds(wx, wy) {
                continue;
            }
            let visible = game.map.is_visible(wx, wy);
            let explored = game.map.is_explored(wx, wy);
            if !visible && !explored {
                continue;
            }
            let light = if visible {
                let dx = (wx - game.hero.x) as f32;
                let dy = (wy - game.hero.y) as f32;
                (1.2 - (dx * dx + dy * dy).sqrt() * 0.085).clamp(0.34, 1.0)
            } else {
                0.42
            };
            let tile = game.map.tile(wx, wy);
            let mut cell = [[bgfill; 4]; 4];
            for sy in 0..4usize {
                for sx in 0..4usize {
                    let base = match tile {
                        Tile::Wall => {
                            if sy == 0 {
                                wall_fg
                            } else if sy == 3 {
                                px_dark(wall_bg, 0.78)
                            } else {
                                wall_bg
                            }
                        }
                        Tile::Floor => {
                            if (wx * 7 + wy * 13 + sx as i32 * 3 + sy as i32 * 5) & 7 == 0 {
                                floor_fg
                            } else {
                                floor_bg
                            }
                        }
                        Tile::StairsDown => floor_bg,
                    };
                    cell[sy][sx] = shade(base, light, tint);
                }
            }
            if tile == Tile::StairsDown {
                for &(ax, ay) in &[(1usize, 0usize), (2, 1), (1, 2), (0, 1), (1, 1)] {
                    cell[ay][ax] = shade((255, 240, 140), light.max(0.8), (1.0, 1.0, 1.0));
                }
            }
            if game.danger.iter().any(|&(a, b)| a == wx && b == wy) {
                for row in cell.iter_mut() {
                    for c in row.iter_mut() {
                        *c = px_blend(*c, game.danger_color, 0.5);
                    }
                }
            } else if game.hazard.iter().any(|&(a, b, _)| a == wx && b == wy) {
                for row in cell.iter_mut() {
                    for c in row.iter_mut() {
                        *c = px_blend(*c, (235, 110, 40), 0.55);
                    }
                }
            } else if game.cast_danger.iter().any(|&(a, b)| a == wx && b == wy) {
                for row in cell.iter_mut() {
                    for c in row.iter_mut() {
                        *c = px_blend(*c, (235, 140, 60), 0.5);
                    }
                }
            }
            if visible {
                let bob = ((game.anim_t / 18 + (wx + wy) as u32) % 2) as i32;
                if game.hero.x == wx && game.hero.y == wy {
                    let (lx, ly) = if game.lunge.2 > 0 { (game.lunge.0, game.lunge.1) } else { (0, 0) };
                    overlay_sprite(&mut cell, &SPR_HERO, (255, 246, 150), lx, -bob + ly);
                } else if game.pet.as_ref().is_some_and(|p| p.x == wx && p.y == wy) {
                    overlay_sprite(&mut cell, &SPR_CREATURE, (120, 230, 180), 0, -bob);
                } else if let Some(a) = game.allies.iter().find(|a| a.x == wx && a.y == wy) {
                    overlay_sprite(&mut cell, if a.companion { &SPR_HERO } else { &SPR_CREATURE }, a.color, 0, -bob);
                } else if let Some(i) = game.monster_at(wx, wy) {
                    let m = &game.monsters[i];
                    overlay_sprite(&mut cell, monster_sprite(m), m.color, 0, -bob);
                } else if game.merchant.as_ref().is_some_and(|mm| mm.x == wx && mm.y == wy) {
                    overlay_sprite(&mut cell, &SPR_MERCHANT, (130, 235, 240), 0, 0);
                } else if let Some(it) = game.items.iter().find(|it| it.x == wx && it.y == wy) {
                    overlay_sprite(&mut cell, item_sprite(it.glyph), it.color, 0, 0);
                } else if let Some(f) = game.features.iter().find(|f| f.x == wx && f.y == wy) {
                    overlay_sprite(&mut cell, feature_sprite(f.kind), feature_color(f.kind), 0, 0);
                }
            }
            if let Some(f) = game.flashes.iter().find(|f| f.0 == wx && f.1 == wy) {
                for row in cell.iter_mut() {
                    for c in row.iter_mut() {
                        *c = px_blend(*c, f.2, 0.6);
                    }
                }
            }
            let ox = tx * t;
            let oy = ty * t;
            for py in 0..t {
                for px in 0..t {
                    let fx = ox + px;
                    let fy = oy + py;
                    if fx < pw && fy < ph {
                        let sx = (px * 4 / t).min(3);
                        let sy = (py * 4 / t).min(3);
                        fb[(fy * pw + fx) as usize] = cell[sy as usize][sx as usize];
                    }
                }
            }
        }
    }

    for cy in 0..mh {
        let _ = write!(buf, "\x1b[{};{}H\x1b[0m", MROW + cy, MCOL);
        for _ in 0..sdx {
            buf.push(' ');
        }
        let mut last: Option<(Color, Color)> = None;
        for cx in 0..mw {
            let mut top = fb[((2 * cy) * pw + cx) as usize];
            let mut bot = fb[((2 * cy + 1) * pw + cx) as usize];
            if vignette > 0.0 {
                top = vignette_add(top, cx, cy, mw, mh, vignette);
                bot = vignette_add(bot, cx, cy, mw, mh, vignette);
            }
            if last != Some((top, bot)) {
                let _ = write!(buf, "\x1b[38;2;{};{};{};48;2;{};{};{}m", top.0, top.1, top.2, bot.0, bot.1, bot.2);
                last = Some((top, bot));
            }
            buf.push('\u{2580}');
        }
    }
}

fn vignette_add(base: Color, x: i32, y: i32, mw: i32, mh: i32, strength: f32) -> Color {
    if strength <= 0.0 {
        return base;
    }
    let d = x.min(mw - 1 - x).min(y).min(mh - 1 - y) as f32;
    let band = (mw.min(mh) as f32 * 0.45).max(1.0);
    let edge = (1.0 - d / band).clamp(0.0, 1.0);
    let a = (strength * edge * edge).clamp(0.0, 0.85);
    let target = (185.0, 25.0, 25.0);
    (
        (base.0 as f32 * (1.0 - a) + target.0 * a) as u8,
        (base.1 as f32 * (1.0 - a) + target.1 * a) as u8,
        (base.2 as f32 * (1.0 - a) + target.2 * a) as u8,
    )
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

    let ((wall_fg, wall_bg), (floor_fg, floor_bg)) = biome_palette(game.biome);
    let (mut terrain_fg, terrain_bg, mut terrain_ch) = match tile {
        Tile::Wall => (wall_fg, wall_bg, ' '),
        Tile::Floor => (floor_fg, floor_bg, '\u{00b7}'),
        Tile::StairsDown => ((255, 240, 140), (46, 42, 30), '>'),
    };
    if tile == Tile::Floor {
        let (dch, dfg) = decor_glyph(game.map.decor_at(x, y));
        if dch != ' ' {
            terrain_ch = dch;
            terrain_fg = dfg;
        }
    }
    let bg_lit = shade(terrain_bg, light, tint);
    let mut result = (terrain_ch, shade(terrain_fg, light, tint), bg_lit);

    if game.hero.x == x && game.hero.y == y {
        result = ('@', (255, 246, 150), bg_lit);
    } else if game.pet.as_ref().is_some_and(|p| p.x == x && p.y == y) {
        result = ('d', (120, 230, 180), bg_lit);
    } else if let Some(a) = game.allies.iter().find(|a| a.x == x && a.y == y) {
        result = (a.glyph, shade(a.color, light.max(0.8), (1.0, 1.0, 1.0)), bg_lit);
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
            FeatureKind::Forge => ('\u{2692}', (255, 170, 70)),
            FeatureKind::Gamble => ('\u{2684}', (235, 200, 120)),
            FeatureKind::Lost => ('\u{263a}', (255, 224, 150)),
        };
        result = (ch, shade(col, light.max(0.8), (1.0, 1.0, 1.0)), bg_lit);
    }

    let dcol = if game.danger.iter().any(|&(dx, dy)| dx == x && dy == y) {
        Some(game.danger_color)
    } else if game.hazard.iter().any(|&(dx, dy, _)| dx == x && dy == y) {
        Some((235, 110, 40))
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
