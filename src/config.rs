use serde::{Deserialize, Serialize};

pub const CONFIG_PATH: &str = "abyssal.config.json";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub twitch_enabled: bool,
    pub twitch_channel: String,
    pub vote_window_secs: f32,
    pub allow_style_vote: bool,
    pub allow_speed_vote: bool,
    pub allow_merchant_vote: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            twitch_enabled: true,
            twitch_channel: "pholecia".into(),
            vote_window_secs: 8.0,
            allow_style_vote: true,
            allow_speed_vote: false,
            allow_merchant_vote: true,
        }
    }
}

impl Config {
    pub fn load_or_create() -> Config {
        if let Ok(data) = std::fs::read_to_string(CONFIG_PATH) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
        let cfg = Config::default();
        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(CONFIG_PATH, json);
        }
        cfg
    }

    pub fn twitch_active(&self) -> bool {
        self.twitch_enabled && !self.twitch_channel.trim().is_empty()
    }
}
