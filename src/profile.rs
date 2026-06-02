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

    pub fn meta(&self) -> (i32, i32, i32, bool) {
        let mut hp = 0;
        let mut might = 0;
        let mut pot = 0;
        if self.best_floor >= 4 {
            pot += 1;
        }
        if self.best_floor >= 14 {
            pot += 1;
        }
        if self.total_kills >= 75 {
            might += 2;
        }
        if self.best_score >= 8000 {
            might += 2;
        }
        if self.best_floor >= 8 {
            hp += 12;
        }
        if self.best_floor >= 15 {
            hp += 12;
        }
        let talent = self.total_kills >= 250;
        (hp, might, pot, talent)
    }

    pub fn perk_labels(&self) -> Vec<String> {
        let (hp, might, pot, talent) = self.meta();
        let mut v = Vec::new();
        if might > 0 {
            v.push(format!("+{} ATQ", might));
        }
        if hp > 0 {
            v.push(format!("+{} PV", hp));
        }
        if pot > 0 {
            v.push(format!("+{} potion", pot));
        }
        if talent {
            v.push("talent de depart".to_string());
        }
        v
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
