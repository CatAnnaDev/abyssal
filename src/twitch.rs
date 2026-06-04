use crate::game::{MerchantPick, Playstyle};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub enum ViewerCmd {
    Style(Playstyle),
    Speed(i32),
    Merchant(MerchantPick),
    Bless,
    Curse,
    Name(String),
    Bet(i32),
    Join,
    Hype,
    Cheer(String),
    Chat,
}

pub fn connect(channel_name: &str) -> Receiver<(String, ViewerCmd)> {
    let (tx, rx) = channel();
    let chan = channel_name.trim().trim_start_matches('#').to_lowercase();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u32 % 100000)
        .unwrap_or(12345);

    std::thread::spawn(move || loop {
        if let Ok(stream) = TcpStream::connect("irc.chat.twitch.tv:6667") {
            if let Ok(mut writer) = stream.try_clone() {
                let _ = writer.write_all(format!("NICK justinfan{}\r\n", nonce).as_bytes());
                let _ = writer.write_all(format!("JOIN #{}\r\n", chan).as_bytes());
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    let Ok(line) = line else { break };
                    if line.starts_with("PING") {
                        let _ = writer.write_all(b"PONG :tmi.twitch.tv\r\n");
                        continue;
                    }
                    if let Some(msg) = privmsg_body(&line) {
                        let user = sender(&line).unwrap_or("anon").to_string();
                        let cmd = parse_cmd(msg).unwrap_or(ViewerCmd::Chat);
                        if tx.send((user, cmd)).is_err() {
                            return;
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_secs(5));
    });
    rx
}

pub fn sender(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(':')?;
    let bang = rest.find('!')?;
    Some(&rest[..bang])
}

pub fn privmsg_body(line: &str) -> Option<&str> {
    let idx = line.find("PRIVMSG")?;
    let rest = &line[idx..];
    let colon = rest.find(" :")?;
    Some(rest[colon + 2..].trim_end())
}

pub fn parse_cmd(msg: &str) -> Option<ViewerCmd> {
    let raw = msg.trim().trim_start_matches('!');
    let mut parts = raw.splitn(2, char::is_whitespace);
    let head = parts.next()?.to_lowercase();
    let arg = parts.next().unwrap_or("").trim();
    match head.as_str() {
        "1" | "comp" | "completionniste" | "explore" => Some(ViewerCmd::Style(Playstyle::Completionist)),
        "2" | "combat" | "combattant" | "fight" => Some(ViewerCmd::Style(Playstyle::Combatant)),
        "3" | "rush" | "rusher" | "stairs" => Some(ViewerCmd::Style(Playstyle::Rusher)),
        "faster" | "speed+" | "+" => Some(ViewerCmd::Speed(1)),
        "slower" | "speed-" | "-" => Some(ViewerCmd::Speed(-1)),
        "weapon" | "arme" | "sword" => Some(ViewerCmd::Merchant(MerchantPick::Weapon)),
        "armor" | "armure" | "shield" => Some(ViewerCmd::Merchant(MerchantPick::Armor)),
        "potion" | "pot" => Some(ViewerCmd::Merchant(MerchantPick::Potion)),
        "heal" | "soin" | "hp" => Some(ViewerCmd::Merchant(MerchantPick::Heal)),
        "reroll" | "roll" => Some(ViewerCmd::Merchant(MerchantPick::Reroll)),
        "purge" | "cleanse" | "clean" => Some(ViewerCmd::Merchant(MerchantPick::Cleanse)),
        "skip" | "rien" | "pass" => Some(ViewerCmd::Merchant(MerchantPick::Skip)),
        "bless" | "benir" | "benediction" | "buff" => Some(ViewerCmd::Bless),
        "curse" | "malediction" | "maudire" | "debuff" => Some(ViewerCmd::Curse),
        "name" | "nom" | "rename" => {
            if arg.is_empty() {
                None
            } else {
                Some(ViewerCmd::Name(arg.to_string()))
            }
        }
        "bet" | "pari" | "mise" | "predict" => arg.parse::<i32>().ok().map(|n| ViewerCmd::Bet(n.clamp(1, 300))),
        "join" | "rejoindre" | "combattre" => Some(ViewerCmd::Join),
        "hype" | "go" | "gg" | "letsgo" | "pog" | "poggers" => Some(ViewerCmd::Hype),
        "cheer" | "emote" | "love" | "coeur" | "<3" | "salut" | "hello" | "hi" => Some(ViewerCmd::Cheer(arg.chars().take(8).collect())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_message_body() {
        let line = ":bob!bob@bob.tmi.twitch.tv PRIVMSG #room :!rush go go";
        assert_eq!(privmsg_body(line), Some("!rush go go"));
    }

    #[test]
    fn parses_commands() {
        assert!(matches!(parse_cmd("!rush"), Some(ViewerCmd::Style(Playstyle::Rusher))));
        assert!(matches!(parse_cmd("2"), Some(ViewerCmd::Style(Playstyle::Combatant))));
        assert!(matches!(parse_cmd("!ARME"), Some(ViewerCmd::Merchant(MerchantPick::Weapon))));
        assert!(matches!(parse_cmd("!heal"), Some(ViewerCmd::Merchant(MerchantPick::Heal))));
        assert!(matches!(parse_cmd("!bet 15"), Some(ViewerCmd::Bet(15))));
        assert!(matches!(parse_cmd("!name Lyra"), Some(ViewerCmd::Name(_))));
        assert!(matches!(parse_cmd("!bless"), Some(ViewerCmd::Bless)));
        assert!(matches!(parse_cmd("!join"), Some(ViewerCmd::Join)));
        assert!(matches!(parse_cmd("!hype"), Some(ViewerCmd::Hype)));
        assert!(matches!(parse_cmd("hello"), Some(ViewerCmd::Cheer(_))));
        assert!(parse_cmd("!bet xyz").is_none());
        assert!(parse_cmd("blabla random").is_none());
    }
}
