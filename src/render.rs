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
        for m in &game.monsters {
            if m.owner.is_empty() || m.x < 0 || m.y < 1 || m.x >= mw || m.y >= mh || !game.map.is_visible(m.x, m.y) {
                continue;
            }
            let tag: String = m.owner.chars().take(12).collect();
            let half = tag.chars().count() as i32 / 2;
            let tx = (MCOL + sdx + m.x - half).clamp(MCOL + sdx, MCOL + sdx + mw - tag.chars().count() as i32);
            let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;235;130;205m{}\x1b[0m", MROW + m.y - 1, tx, tag);
        }
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
    if !game.top_voters.is_empty() || !game.twitch_channel.is_empty() {
        draw_top_voters(game, mh, &mut buf);
    }
    if game.fx.combo >= 3 {
        let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;255;200;70mCOMBO x{}\x1b[0m", MROW, MCOL + 1, game.fx.combo);
    }
    if game.debug {
        draw_debug(game, mw, mh, sdx, sprite, &mut buf);
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
    if game.show_hall {
        draw_hall(game, cols, rows, &mut buf);
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
    let rush = if game.boss_rush {
        if game.floor >= 10 { format!("  ·  BOSS RUSH vague {}", game.boss_wave + 1) } else { "  ·  Boss Rush(10)".to_string() }
    } else {
        String::new()
    };
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
    let title = format!("{}{}{}{}", title.trim_end(), asc, rush, muts);
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
    let (bottom, bcol) = match game.thoughts.last() {
        Some(t) => (format!(" \u{201c}{}\u{201d}  (o:options) ", t), (150, 200, 225)),
        None => (
            " espace:pause  +/-:vitesse  m:mindset  a:son  g:sprite  z:zoom  k:bestiaire  h:hall  b:marchand  s/l/n  q:quitter ".to_string(),
            (130, 130, 150),
        ),
    };
    let h: String = bottom.chars().take((cols - 4).max(0) as usize).collect();
    let _ = write!(buf, "\x1b[{};3H\x1b[38;2;{};{};{}m{}", rows, bcol.0, bcol.1, bcol.2, h);

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

    draw_box(buf, px, 2, pin, hh, "HEROINE", FRAME);
    let mut r = 3;
    put(buf, ix, r, (235, 210, 140), &fit(&format!("{} · {}", game.identity.name, game.class.label()), iw));
    r += 1;
    put(buf, ix, r, (180, 180, 195), &fit(&format!("niv {} {} · {}", h.level, titre(h.level), game.identity.trait_kind.label()), iw));
    r += 1;
    bar(buf, ix, r, iw, h.hp, h.max_hp, (90, 210, 110), "PV");
    r += 1;
    bar(buf, ix, r, iw, h.xp, h.xp_next, (110, 160, 240), "XP");
    r += 1;
    put(buf, ix, r, (170, 170, 185), &format!("ATQ {:<4} DEF {:<4} or {}", h.atk(), h.def(), h.gold));
    r += 1;
    let corr_col = if game.corruption >= 70 { (220, 90, 200) } else if game.corruption >= 40 { (210, 140, 200) } else { (200, 170, 90) };
    put(buf, ix, r, corr_col, &fit(&format!("{} · {}% · corr {}%", game.style.label(), game.map.discovery_percent(), game.corruption), iw));
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

fn draw_hall(game: &Game, cols: i32, rows: i32, buf: &mut String) {
    let ghosts = game.graveyard();
    let nemeses = game.known_nemeses();
    let feats = crate::lore::FEATS;
    let bw = 56.min((cols - 4).max(24));
    let body = ghosts.len() as i32 + nemeses.len() as i32 + feats.len() as i32 + 9;
    let bh = body.clamp(12, (rows - 2).max(12));
    let ox = (cols - bw) / 2;
    let oy = (rows - bh) / 2;
    let blank: String = " ".repeat(bw as usize);
    for y in oy..oy + bh {
        put(buf, ox, y, (0, 0, 0), &blank);
    }
    draw_box(buf, ox, oy, bw, bh, "HALL DES AMES  (h: fermer)", (165, 160, 190));
    let mut r = oy + 1;
    let last = oy + bh - 1;
    put(buf, ox + 2, r, (200, 195, 215), &fit(&format!("\u{271d} Cimetiere ({} ames)", ghosts.len()), bw - 4));
    r += 1;
    if ghosts.is_empty() {
        put(buf, ox + 2, r, (120, 120, 130), &fit("aucune ame tombee... pour l'instant.", bw - 4));
        r += 1;
    } else {
        for g in ghosts.iter().rev() {
            if r >= last - 2 {
                break;
            }
            put(buf, ox + 2, r, (190, 195, 210), &fit(&format!("{} {} · {} · etage {} · {} or", g.name, g.origin, g.class, g.floor, g.gold), bw - 4));
            r += 1;
        }
    }
    if r < last - 1 {
        r += 1;
        put(buf, ox + 2, r, (225, 130, 200), &fit(&format!("\u{2620} Nemesis actives ({})", nemeses.len()), bw - 4));
        r += 1;
        if nemeses.is_empty() {
            put(buf, ox + 2, r, (120, 120, 130), &fit("aucune rancune en cours.", bw - 4));
            r += 1;
        } else {
            for n in nemeses {
                if r >= last - 2 {
                    break;
                }
                put(buf, ox + 2, r, (225, 140, 205), &fit(&format!("{} (rang {}, {} morts inflige)", n.name, n.rank, n.hero_kills), bw - 4));
                r += 1;
            }
        }
    }
    let earned = game.feats();
    if r < last - 1 {
        r += 1;
        put(buf, ox + 2, r, (255, 215, 120), &fit(&format!("\u{2605} Hauts faits ({}/{})", earned.len(), feats.len()), bw - 4));
        r += 1;
        for (id, name, desc) in feats {
            if r >= last {
                break;
            }
            if earned.iter().any(|f| f == id) {
                put(buf, ox + 2, r, (255, 215, 120), &fit(&format!("\u{2605} {} — {}", name, desc), bw - 4));
            } else {
                put(buf, ox + 2, r, (110, 110, 120), &fit(&format!("\u{2606} ??? — {}", desc), bw - 4));
            }
            r += 1;
        }
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

fn overlay_sprite(cell: &mut [[Color; 8]; 8], pat: &[&str; 8], color: Color, ox: i32, oy: i32) {
    for (sy, row) in pat.iter().enumerate() {
        for (sx, ch) in row.chars().enumerate() {
            if sx >= 8 {
                break;
            }
            let c = match ch {
                'X' => color,
                '.' => px_dark(color, 0.6),
                ':' => px_dark(color, 0.8),
                '*' => px_lite(color, 0.4),
                'o' => (20, 17, 24),
                'v' => (245, 245, 250),
                _ => continue,
            };
            let tx = sx as i32 + ox;
            let ty = sy as i32 + oy;
            if (0..8).contains(&tx) && (0..8).contains(&ty) {
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
        FeatureKind::Grave => (170, 175, 190),
    }
}

const SPR_HERO: [&str; 8] = ["  o**o  ", " oX**Xo ", " oXvvXo ", " oXXXXo ", "oXXXXXXo", "o:XXXX:o", "  X  X  ", "  o  o  "];
const SPR_CREATURE: [&str; 8] = [" o    o ", "oXo  oXo", "oXXXXXXo", "XvXXXXvX", "oXXXXXXo", "oXX..XXo", " oX  Xo ", "  o  o  "];
const SPR_BOSS: [&str; 8] = ["o  XX  o", "oXXXXXXo", "XvX..XvX", "oXXXXXXo", "oXX..XXo", "oXXXXXXo", "oXo  oXo", " o    o "];
const SPR_ITEM: [&str; 8] = ["        ", "   vv   ", "  *XX*  ", " *XXXX* ", " *XXXX* ", "  *XX*  ", "   ::   ", "        "];
const SPR_MERCHANT: [&str; 8] = ["  oooo  ", " o*XX*o ", " oXvvXo ", " oXXXXo ", "oX*XX*Xo", "oXXXXXXo", " oX  Xo ", " o:  :o "];
const SPR_VERMIN: [&str; 8] = ["        ", "o      o", "oXo  oXo", "oXXXXXXo", "XvXXXXvX", "oXX..XXo", " oXXXXo ", "o o  o o"];
const SPR_ARCHER: [&str; 8] = ["  o*o  v", " oXvXo v", " oXXXo v", "oXXXXo v", "oXXXXo v", " XX XX v", " o   o v", " o   o  "];
const SPR_CASTER: [&str; 8] = ["   v    ", "  *X*   ", " o**o   ", " oXvXo  ", "oXXXXo  ", "oX**Xo  ", "oXXXXXo ", " oXXXo  "];
const SPR_BRUTE: [&str; 8] = ["oXXXXXXo", "XvXXXXvX", "XXX..XXX", "oXXXXXXo", "oXXXXXXo", "oXX..XXo", "oXo  oXo", "o o  o o"];
const SPR_DEMON: [&str; 8] = ["v      v", "oXo  oXo", "oXXXXXXo", "XvX..XvX", "oXXXXXXo", "oX*..*Xo", " oXXXXo ", " o:  :o "];
const SPR_DRAGON: [&str; 8] = ["*o    o*", "*Xo  oX*", "*XXXXXX*", "vXX..XXv", "oXXXXXXo", "*XXXXXX*", "*oXXXXo*", " *o  o* "];
const SPR_MIMIC: [&str; 8] = ["oXXXXXXo", "X*XXXX*X", "oXXXXXXo", "ovvvvvvo", "oXvXvXvo", "oXXXXXXo", "oXo  oXo", "o      o"];
const SPR_FINAL: [&str; 8] = ["v o  o v", "oXXXXXXo", "Xv.XX.vX", "oXXXXXXo", "o*XXXX*o", "oXX..XXo", "voXXXXov", " o o o  "];
const SPR_BOMB: [&str; 8] = ["     v  ", "    *   ", "  oooo  ", " oXXXXo ", "oXX**XXo", "oXXXXXXo", " oXXXXo ", "  oooo  "];
const SPR_BAT: [&str; 8] = ["        ", "o      o", "oXo  oXo", "oXXvvXXo", " XXXXXX ", "  oXXo  ", "   oo   ", "        "];
const SPR_BLOB: [&str; 8] = ["        ", "  oooo  ", " o*XX*o ", "oXvXXvXo", "oXXXXXXo", "oXX..XXo", " oXXXXo ", "o oooo o"];
const SPR_FLAME: [&str; 8] = ["   v    ", "  *v*   ", "  *X*   ", " *XXX*  ", " *XXX*  ", "*XXXXX* ", "*XX:XX* ", " *XXX*  "];
const SPR_WINGED: [&str; 8] = ["* o  o *", "*oXXXXo*", "vXX..XXv", "oXXXXXXo", "*oXXXXo*", " *XXXX* ", "  *  *  ", "  o  o  "];
const SPR_COIN: [&str; 8] = ["        ", "  oooo  ", " o*vv*o ", "oXvXXvXo", "oXvXXvXo", " o*vv*o ", "  oooo  ", "        "];
const SPR_POTION: [&str; 8] = ["   oo   ", "   ::   ", "  o::o  ", "  oXXo  ", " oX**Xo ", " oXvvXo ", " oXXXXo ", "  oooo  "];
const SPR_BLADE: [&str; 8] = ["      ov", "     oXv", "    oXo ", "   oXo  ", "  oXo   ", " *Xo    ", "*Xoo    ", "oo      "];
const SPR_ARMOR: [&str; 8] = [" oooooo ", "oX*vv*Xo", "oXvXXvXo", "oXXXXXXo", "oX:..:Xo", "oXXXXXXo", " oXXXXo ", "  oooo  "];
const SPR_RING: [&str; 8] = ["        ", "  *vv*  ", " vo  ov ", " vo  ov ", " vo  ov ", "  *vv*  ", "        ", "        "];
const SPR_AMULET: [&str; 8] = ["  *vv*  ", " v    v ", "  v  v  ", "   vv   ", "  oXXo  ", " oXvvXo ", "  oXXo  ", "   oo   "];
const SPR_SCROLL: [&str; 8] = [" oooooo ", "oX::::Xo", "oX....Xo", "oX::::Xo", "oX....Xo", "oX::::Xo", " oooooo ", "        "];
const SPR_CHEST: [&str; 8] = [" oooooo ", "oX****Xo", "oXvvvvXo", "oXXXXXXo", "oX*vv*Xo", "oXXXXXXo", "oXX..XXo", " oooooo "];
const SPR_TRAP: [&str; 8] = ["        ", " v v v v", " X X X X", "oXoXoXoX", "oXXXXXXo", "oooooooo", "        ", "        "];

const SPR_EYE: [&str; 8] = ["        ", "  oooo  ", " o****o ", "o*XvvX*o", "o*vXXv*o", " o****o ", "  oooo  ", "        "];

fn item_sprite(glyph: char) -> &'static [&'static str; 8] {
    match glyph {
        '$' => &SPR_COIN,
        '!' => &SPR_POTION,
        '/' => &SPR_BLADE,
        '[' => &SPR_ARMOR,
        '\u{2218}' => &SPR_RING,
        '\u{2666}' => &SPR_AMULET,
        '?' => &SPR_SCROLL,
        '\u{2609}' => &SPR_EYE,
        '\u{2624}' => &SPR_POTION,
        _ => &SPR_ITEM,
    }
}

fn feature_sprite(kind: FeatureKind) -> &'static [&'static str; 8] {
    match kind {
        FeatureKind::Chest => &SPR_CHEST,
        FeatureKind::Trap => &SPR_TRAP,
        FeatureKind::Fountain => &SPR_POTION,
        FeatureKind::Lost => &SPR_HERO,
        _ => &SPR_ITEM,
    }
}

fn monster_sprite(m: &crate::entity::Monster) -> &'static [&'static str; 8] {
    if m.boss {
        return if m.glyph == '\u{2638}' { &SPR_FINAL } else { &SPR_BOSS };
    }
    match m.glyph {
        'r' | 'v' | 'p' | 'L' | 't' => &SPR_VERMIN,
        'b' | 'u' => &SPR_BAT,
        'a' => &SPR_ARCHER,
        'w' | 'S' | 'c' | 'N' | 'f' | 'q' | 'U' => &SPR_CASTER,
        'O' | 'T' | 'P' | 'B' | 'C' => &SPR_BRUTE,
        'D' | 'i' | 'V' => &SPR_DEMON,
        'F' => &SPR_CASTER,
        'Y' | 'A' | 'Q' | 'H' | 'y' => &SPR_DRAGON,
        'z' => &SPR_BOMB,
        'j' => &SPR_BLOB,
        'e' => &SPR_FLAME,
        'M' | 'x' | 'n' | 'G' | 'R' => &SPR_WINGED,
        '\u{25a4}' => &SPR_MIMIC,
        _ => &SPR_CREATURE,
    }
}

fn sprite_cam(game: &Game, mw: i32, mh: i32, zoom: i32) -> (i32, i32, i32) {
    let t = zoom.clamp(2, 16);
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
            let mut cell = [[bgfill; 8]; 8];
            for sy in 0..8usize {
                for sx in 0..8usize {
                    let base = match tile {
                        Tile::Wall => {
                            if sy == 0 {
                                wall_fg
                            } else if sy == 1 {
                                px_lite(wall_fg, 0.12)
                            } else if sy >= 6 {
                                px_dark(wall_bg, 0.74)
                            } else if (wx * 5 + sx as i32) % 4 == 0 && sy == 3 {
                                px_dark(wall_bg, 0.88)
                            } else {
                                wall_bg
                            }
                        }
                        Tile::Floor => {
                            if (wx * 7 + wy * 13 + sx as i32 * 3 + sy as i32 * 5) % 23 == 0 {
                                floor_fg
                            } else if (wx * 11 + wy * 5 + sx as i32 + sy as i32 * 2) % 17 == 0 {
                                px_lite(floor_bg, 0.1)
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
                for sy in 0..8usize {
                    for sx in 0..8usize {
                        let edge = sx == 0 || sx == 7 || sy == 0 || sy == 7;
                        let step = (sy / 2 + 1) as i32;
                        let on = !edge && (sx as i32) >= 4 - step && (sx as i32) <= 3 + step;
                        if on {
                            cell[sy][sx] = shade((255, 238, 140), light.max(0.8), (1.0, 1.0, 1.0));
                        }
                    }
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
                        let sx = (px * 8 / t).min(7);
                        let sy = (py * 8 / t).min(7);
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
            FeatureKind::Grave => ('\u{271d}', (185, 190, 205)),
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

fn arrow_glyph(dx: i32, dy: i32) -> char {
    match (dx.signum(), dy.signum()) {
        (1, 0) => '\u{2192}',
        (-1, 0) => '\u{2190}',
        (0, 1) => '\u{2193}',
        (0, -1) => '\u{2191}',
        (1, 1) => '\u{2198}',
        (1, -1) => '\u{2197}',
        (-1, 1) => '\u{2199}',
        (-1, -1) => '\u{2196}',
        _ => '\u{00b7}',
    }
}

fn draw_debug(game: &Game, mw: i32, mh: i32, sdx: i32, sprite: bool, buf: &mut String) {
    use std::fmt::Write as _;
    let h = &game.hero;
    let field = game.debug_field();
    if !sprite {
        let mark = |buf: &mut String, x: i32, y: i32, col: Color, ch: char| {
            if x >= 0 && y >= 0 && x < mw && y < mh {
                let _ = write!(buf, "\x1b[{};{}H\x1b[38;2;{};{};{}m{}", MROW + y, MCOL + sdx + x, col.0, col.1, col.2, ch);
            }
        };
        for &(x, y) in &game.danger {
            mark(buf, x, y, (210, 70, 70), '\u{2592}');
        }
        for &(x, y) in &game.cast_danger {
            mark(buf, x, y, (235, 120, 60), '\u{2592}');
        }
        for &(x, y, _) in &game.hazard {
            mark(buf, x, y, (235, 110, 40), '\u{2592}');
        }
        let path = game.debug_path();
        let mut prev = (game.hero.x, game.hero.y);
        for &(x, y) in &path {
            let arrow = arrow_glyph(x - prev.0, y - prev.1);
            mark(buf, x, y, (90, 235, 130), arrow);
            prev = (x, y);
        }
        if let Some((gx, gy)) = game.debug_goal() {
            mark(buf, gx, gy, (130, 255, 150), 'G');
        }
        for m in &game.monsters {
            if !game.map.is_explored(m.x, m.y) {
                continue;
            }
            let (ch, col) = if m.boss {
                ('B', (255, 120, 120))
            } else if m.cast_wind > 0 {
                ('c', (255, 180, 80))
            } else if m.aggro {
                ('!', (255, 90, 90))
            } else if m.flees {
                ('f', (120, 200, 255))
            } else {
                ('z', (150, 150, 160))
            };
            mark(buf, m.x, m.y, col, ch);
        }
    }
    let goalpos = game.debug_goal();
    let goal = goalpos.map(|(x, y)| format!("{},{}", x, y)).unwrap_or_else(|| "-".into());
    let reach = field.iter().filter(|&&d| d >= 0).count();
    let gdist = goalpos.map(|(x, y)| if game.map.in_bounds(x, y) { field[game.map.idx(x, y)] } else { -1 }).unwrap_or(-1);
    let bossc = game.monsters.iter().filter(|m| m.boss).count();
    let aggro = game.monsters.iter().filter(|m| m.aggro).count();
    let mode = match game.music_mode() {
        crate::audio::MusicMode::Calm => "calm",
        crate::audio::MusicMode::Combat => "combat",
        crate::audio::MusicMode::Boss => "boss",
    };
    let evt = if game.event.label().is_empty() { "calme" } else { game.event.label() };
    let muts: Vec<&str> = game.mutators.iter().map(|m| m.label()).collect();
    let relics: Vec<&str> = h.relics.iter().map(|r| r.short()).collect();
    let tals: Vec<&str> = h.talents.iter().map(|t| t.label().split(' ').next().unwrap_or("")).collect();
    let near = game
        .monsters
        .iter()
        .filter(|m| game.map.is_visible(m.x, m.y))
        .min_by_key(|m| (m.x - h.x).abs() + (m.y - h.y).abs());
    let nstr = match near {
        Some(m) => format!("{} hp{}/{} a{} d{} {} d{}", m.name, m.hp, m.max_hp, m.atk, m.def, m.element.label(), (m.x - h.x).abs().max((m.y - h.y).abs())),
        None => "-".into(),
    };
    let pet = game.pet.as_ref().map(|p| format!("{} l{} {}/{}", p.name, p.level, p.hp, p.max_hp)).unwrap_or_else(|| "-".into());
    let obj = if game.objective == crate::game::Objective::None { "-".to_string() } else { game.objective.desc(game.objective_target) };
    let we = h.weapon_element();
    let lines = vec![
        "== DEBUG (ctrl+d) ==".to_string(),
        format!("act:{}  goal:{}", game.last_action, goal),
        format!("pf:{} reach:{} gdist:{} steps:{}", game.pathfinder.label(), reach, gdist, game.debug_path().len()),
        "legende: G=but, fleches=chemin, []=danger".to_string(),
        format!("style:{}  rush:{} w{}", game.style.label(), game.boss_rush, game.boss_wave),
        format!("music:{} int:{:.2} boss:{}", mode, game.music_intensity(), bossc),
        format!("floor:{} {} [{}]", game.floor, game.biome.label(), game.room_kind.label()),
        format!("event:{} diffx{:.2} asc:{}", evt, game.diff_mult, game.ascension),
        format!("obj:{}", obj),
        format!("mut:{}", if muts.is_empty() { "-".into() } else { muts.join(",") }),
        format!("mob:{} aggro:{} ally:{}", game.monsters.len(), aggro, game.allies.len()),
        format!("item:{} feat:{} dng:{}/{}/{}", game.items.len(), game.features.len(), game.danger.len(), game.cast_danger.len(), game.hazard.len()),
        format!("hp:{}/{} lvl{} xp{}/{}", h.hp, h.max_hp, h.level, h.xp, h.xp_next),
        format!("atk:{} def:{} crit:{}%", h.atk(), h.def(), (game.class.crit_chance() * 100.0) as i32),
        format!("wpn:{} +{} {} [{}]", fit(&h.weapon, 8), h.weapon_bonus, h.weapon_affix.label(), we.label()),
        format!("arm:{} +{} {}", fit(&h.armor, 8), h.armor_bonus, h.armor_affix.label()),
        format!("set:{} ring:{} amu:{}", h.set_bonus(), h.ring.label(), h.amulet.label()),
        format!("relics:{}", if relics.is_empty() { "-".into() } else { relics.join(",") }),
        format!("tal:{}", if tals.is_empty() { "-".into() } else { tals.join(",") }),
        format!("st b{} p{} ra{} sh{} rg{}", h.burn, h.poison, h.rage, h.shield, h.regen),
        format!("cd bolt{} abil{} pot{} scr{}", h.bolt_cd, h.ability_cd, h.potions, h.scrolls.len()),
        format!("pet:{}", pet),
        format!("near:{}", nstr),
        format!("bwind:{} bpend:{} hstop:{}", game.boss_wind, game.boss_pending, game.hitstop),
        format!("pos:{},{} ft:{}", h.x, h.y, game.floor_turns),
        format!("kills:{} runs:{} disc:{}%", game.total_kills, game.runs, game.map.discovery_percent()),
    ];
    let maxw = (mw - 2).clamp(8, 38) as usize;
    for (i, l) in lines.iter().enumerate() {
        if i as i32 + 1 >= mh {
            break;
        }
        let txt: String = l.chars().take(maxw).collect();
        let _ = write!(buf, "\x1b[{};{}H\x1b[48;2;10;12;18m\x1b[38;2;120;240;160m {:<width$}\x1b[0m", MROW + 1 + i as i32, MCOL + sdx + 1, txt, width = maxw);
    }
    let stats = game.debug_pf_stats();
    if !stats.is_empty() && mw > 30 {
        let cw = 24i32;
        let cx = MCOL + sdx + mw - cw - 1;
        let mut srow = vec!["-- pathfinder cout/temps --".to_string()];
        for (pf, nodes, ns) in stats {
            let cur = if pf == game.pathfinder { '\u{25b6}' } else { ' ' };
            srow.push(format!("{}{:<9}{:>4}n {:>6.1}us", cur, pf.label(), nodes, ns as f64 / 1000.0));
        }
        for (i, l) in srow.iter().enumerate() {
            if i as i32 + 1 >= mh {
                break;
            }
            let txt: String = l.chars().take(cw as usize).collect();
            let _ = write!(buf, "\x1b[{};{}H\x1b[48;2;10;12;18m\x1b[38;2;150;210;255m {:<width$}\x1b[0m", MROW + 1 + i as i32, cx, txt, width = cw as usize);
        }
    }
}

fn draw_top_voters(game: &Game, mh: i32, buf: &mut String) {
    let title = if game.twitch_channel.is_empty() {
        "TWITCH".to_string()
    } else {
        format!("TWITCH #{}", game.twitch_channel)
    };
    let mut lines: Vec<String> = vec![title];
    let st = game.style_tally;
    if st[0] + st[1] + st[2] > 0 {
        let bar = |n: u32| -> String {
            let k = n.min(8) as usize;
            "\u{2588}".repeat(k)
        };
        lines.push("votes etat d'esprit:".to_string());
        lines.push(format!("!1 complet {:<8} {}", bar(st[0]), st[0]));
        lines.push(format!("!2 combat  {:<8} {}", bar(st[1]), st[1]));
        lines.push(format!("!3 rush    {:<8} {}", bar(st[2]), st[2]));
    }
    if !game.top_voters.is_empty() {
        lines.push("top chat:".to_string());
    }
    for (i, (user, n)) in game.top_voters.iter().enumerate() {
        let u: String = user.chars().take(14).collect();
        lines.push(format!("{}. {} ({})", i + 1, u, n));
    }
    if !game.twitch_feed.is_empty() {
        lines.push("actions:".to_string());
        for a in game.twitch_feed.iter() {
            lines.push(format!("\u{2022} {}", a.chars().take(26).collect::<String>()));
        }
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
    let sep = "\u{2500}".repeat(40);
    let afford = |p: i32| if game.hero.gold >= p { "" } else { "  (trop cher)" };
    let mut lines: Vec<String> = Vec::new();
    if game.shop_vote_secs > 0.0 {
        let blink = (game.anim_t / 8) % 2 == 0;
        let arrows = if blink { "\u{25b6}\u{25b6}\u{25b6}" } else { "   " };
        lines.push(format!("{} CHAT, VOTEZ MAINTENANT — {:>2}s restantes {}", arrows, game.shop_vote_secs.ceil() as i32, arrows));
        let total = 60.0f32;
        let filled = ((game.shop_vote_secs / total) * 36.0).round().clamp(0.0, 36.0) as usize;
        lines.push(format!("[{}{}]", "\u{2588}".repeat(filled), "\u{2591}".repeat(36 - filled)));
    } else {
        lines.push("Le marchand etale sa camelote.".to_string());
    }
    lines.push(format!("Votre or : {}", game.hero.gold));
    lines.push(sep.clone());
    match &m.weapon {
        Some((name, bonus, price)) => lines.push(format!("!arme    {:<18} ATQ+{:<2} {:>4}or [{}]{}", name, bonus, price, v[0], afford(*price))),
        None => lines.push(format!("!arme    {:<28} [{}]", "(rien en stock)", v[0])),
    }
    match &m.armor {
        Some((name, bonus, price)) => lines.push(format!("!armure  {:<18} DEF+{:<2} {:>4}or [{}]{}", name, bonus, price, v[1], afford(*price))),
        None => lines.push(format!("!armure  {:<28} [{}]", "(rien en stock)", v[1])),
    }
    lines.push(format!("!potion  {:<25} {:>4}or [{}]{}", "potion de soin", m.potion_price, v[2], afford(m.potion_price)));
    lines.push(format!("!soin    {:<25} {:>4}or [{}]{}", "soin complet (PV max)", m.heal_price, v[3], afford(m.heal_price)));
    lines.push(format!("!reroll  {:<30} [{}]", "renouveler le stock", v[4]));
    lines.push(format!("!purge   {:<30} [{}]", "retirer poison/brulure", v[5]));
    lines.push(format!("!skip    {:<30} [{}]", "passer son tour", v[6]));
    lines.push(sep.clone());
    lines.push(match game.forced_purchase {
        Some(p) => format!("le chat a vote : {}", p.label()),
        None => "marchandage en cours... (votez avec !arme, !potion, ...)".to_string(),
    });

    let inner = (lines.iter().map(|l| l.chars().count()).max().unwrap_or(20) as i32 + 2).max(50);
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
    let alert = game.shop_vote_secs > 0.0;
    for (i, l) in lines.iter().enumerate() {
        let color = if alert && i <= 1 {
            (255, 120, 90)
        } else if i + 1 == lines.len() {
            (255, 225, 120)
        } else {
            (220, 220, 230)
        };
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
        2 | 3 => "Aventuriere",
        4 | 5 => "Mercenaire",
        6 | 7 => "Veterane",
        8 | 9 => "Championne",
        10..=12 => "Heroine",
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
    let mut lines = vec![
        "  V O U S   E T E S   M O R T E  ".to_string(),
        String::new(),
        format!("  {}", game.identity.title()),
        format!("  {} · {} ({})", game.class.label(), game.identity.trait_kind.label(), game.last_cause),
        format!("  etage {}  ·  niveau {}", game.floor, game.hero.level),
        format!("  {} or  ·  {} elimines", game.hero.gold, game.hero.kills),
        format!("  record: etage {}  ·  {} tues", game.best_floor, game.total_kills),
        format!("  SCORE {}", game.last_score),
        format!("  top: {}", top_scores(game)),
        String::new(),
    ];
    if !game.obituary.is_empty() {
        for w in wrap_text(&game.obituary, 50) {
            lines.push(format!("  {}", w));
        }
    } else {
        lines.push(format!("  \" {} \"", game.death_quip));
    }
    lines.push(String::new());
    lines.push("  une nouvelle ame se prepare...".to_string());
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
