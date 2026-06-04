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
mod win;

use game::Game;

fn main() {
    win::run();
}

pub(crate) const OBS_PATH: &str = "abyssal.obs.html";

pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub(crate) fn write_obs(game: &Game) {
    let h = &game.hero;
    let hp_pct = (h.hp.max(0) * 100 / h.max_hp.max(1)).clamp(0, 100);
    let dead = matches!(game.phase, game::Phase::Dead(_));
    let thought = game.thoughts.last().map(|s| s.as_str()).unwrap_or("");
    let mut events = String::new();
    for line in game.log.iter().rev().take(4) {
        events.push_str(&format!("<div class=ev>{}</div>", esc(&line.text)));
    }
    let banner = if dead {
        format!("<div class=dead>MORTE — {}</div><div class=ob>{}</div>", esc(&game.last_cause), esc(&game.obituary))
    } else {
        String::new()
    };
    let mut tw = String::new();
    if !game.twitch_channel.is_empty() {
        tw.push_str(&format!("<div class=tw>#{}", esc(&game.twitch_channel)));
        if game.bet_pool > 0 {
            tw.push_str(&format!(" · {} paris", game.bet_pool));
        }
        if let Some((u, n)) = game.top_voters.first() {
            tw.push_str(&format!(" · top {} ({})", esc(u), n));
        }
        tw.push_str("</div>");
    }
    if !game.bet_result.is_empty() {
        tw.push_str(&format!("<div class=tw>{}</div>", esc(&game.bet_result)));
    }
    let html = format!(
        "<!doctype html><html><head><meta charset=utf-8><meta http-equiv=refresh content=1>\
<style>body{{margin:0;background:transparent;font-family:'Menlo','Consolas',monospace;color:#eee}}\
.card{{display:inline-block;background:rgba(8,8,14,.74);border:2px solid #46b6ff;border-radius:12px;padding:16px 20px}}\
.name{{font-size:32px;font-weight:bold;color:#ffd76e}}\
.sub{{font-size:18px;color:#9fb2c8;margin-bottom:6px}}\
.row{{font-size:20px;margin-top:6px}}\
.hpbar{{height:14px;width:340px;background:#2a2a33;border-radius:7px;overflow:hidden;margin-top:8px}}\
.hpfill{{height:100%;background:linear-gradient(90deg,#5ed27a,#9be88c)}}\
.thought{{font-style:italic;color:#a6d8ff;font-size:21px;margin-top:10px;max-width:380px}}\
.feed{{margin-top:8px;color:#8a93a6;font-size:15px}}\
.dead{{color:#ff5a5a;font-size:24px;font-weight:bold;margin-top:8px}}\
.ob{{color:#e8c9a0;font-size:16px;max-width:400px;margin-top:4px}}\
.tw{{color:#c79bff;font-size:16px;margin-top:6px}}</style></head><body><div class=card>\
<div class=name>{name}</div>\
<div class=sub>{class} · {trait_} · {origin}</div>\
<div class=row>Étage {floor} · {biome} · niv {level}</div>\
<div class=hpbar><div class=hpfill style=\"width:{hp_pct}%\"></div></div>\
<div class=row>♥ {hp}/{maxhp} · ⚔ {atk} · ◈ {def} · {gold} or · {kills} kills · corr {corr}%</div>\
{banner}\
<div class=thought>{thought}</div>\
{tw}\
<div class=feed>{events}</div>\
</div></body></html>",
        name = esc(&game.identity.name),
        class = esc(game.class.label()),
        trait_ = esc(game.identity.trait_kind.label()),
        origin = esc(&game.identity.origin),
        floor = game.floor,
        biome = esc(game.biome.label()),
        level = h.level,
        hp_pct = hp_pct,
        hp = h.hp.max(0),
        maxhp = h.max_hp,
        atk = h.atk(),
        def = h.def(),
        gold = h.gold,
        kills = h.kills,
        corr = game.corruption,
        banner = banner,
        thought = esc(thought),
        tw = tw,
        events = events,
    );
    let _ = std::fs::write(OBS_PATH, html);
}
