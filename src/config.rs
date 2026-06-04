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
    #[serde(default = "yes")]
    pub sound_enabled: bool,
    #[serde(default = "yes")]
    pub ambient_enabled: bool,
    #[serde(default = "default_volume")]
    pub master_volume: f32,
    #[serde(default = "default_ambient_volume")]
    pub ambient_volume: f32,
    #[serde(default)]
    pub music_preset: i32,
    #[serde(default)]
    pub pathfinder: i32,
    #[serde(default = "yes")]
    pub allow_chaos_vote: bool,
    #[serde(default = "yes")]
    pub allow_bet_vote: bool,
    #[serde(default)]
    pub obs_overlay: bool,
    #[serde(default)]
    pub window_scale: i32,
    #[serde(default = "three")]
    pub render_scale: i32,
}

fn yes() -> bool {
    true
}

fn three() -> i32 {
    3
}

fn default_volume() -> f32 {
    0.5
}

fn default_ambient_volume() -> f32 {
    0.5
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
            sound_enabled: true,
            ambient_enabled: true,
            master_volume: 0.5,
            ambient_volume: 0.5,
            music_preset: 0,
            pathfinder: 0,
            allow_chaos_vote: true,
            allow_bet_vote: true,
            obs_overlay: false,
            window_scale: 0,
            render_scale: 3,
        }
    }
}

impl Config {
    pub fn load_or_create() -> Config {
        let cfg = std::fs::read_to_string(CONFIG_PATH)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default();
        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(CONFIG_PATH, json);
        }
        cfg
    }

    pub fn twitch_active(&self) -> bool {
        self.twitch_enabled && !self.twitch_channel.trim().is_empty()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(CONFIG_PATH, json);
        }
    }
}
