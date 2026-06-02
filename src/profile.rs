use serde::{Deserialize, Serialize};

pub const PROFILE_PATH: &str = "abyssal.profile.json";

#[derive(Serialize, Deserialize, Default)]
pub struct Profile {
    pub runs: u32,
    pub deaths: u32,
    pub best_floor: i32,
    pub best_score: i32,
    pub total_kills: u64,
    pub total_gold: u64,
}

impl Profile {
    pub fn load() -> Profile {
        std::fs::read_to_string(PROFILE_PATH)
            .ok()
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(PROFILE_PATH, json);
        }
    }

    pub fn record_death(&mut self, floor: i32, score: i32, kills: i32, gold: i32) {
        self.runs += 1;
        self.deaths += 1;
        self.best_floor = self.best_floor.max(floor);
        self.best_score = self.best_score.max(score);
        self.total_kills += kills.max(0) as u64;
        self.total_gold += gold.max(0) as u64;
        self.save();
    }
}
