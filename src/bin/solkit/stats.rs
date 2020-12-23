use std::collections::HashMap;
use std::fs::{read_to_string, write};

use serde_derive::{Deserialize, Serialize};

use crate::config::stats_path;

// solitaire statistics
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct GameStat {
    pub(crate) played: u64,
    pub(crate) won: u64,
}

impl Default for GameStat {
    fn default() -> GameStat {
        GameStat { played: 0, won: 0 }
    }
}

// all solitaires statistics
#[derive(Serialize, Deserialize)]
pub(crate) struct Stats {
    pub(crate) games: HashMap<String, GameStat>,
}

impl Stats {
    fn new() -> Self {
        Stats { games: HashMap::new() }
    }

    pub(crate) fn load() -> Self {
        let path = stats_path();
        if !path.exists() {
            return Stats::new();
        }
        let data = match read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to load statistics: {:?}", e);
                return Stats::new();
            }
        };
        let stats: Stats = match toml::from_str(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read TOML statistics: {:?}", e);
                Stats::new()
            }
        };
        stats
    }

    pub(crate) fn update_stat(&mut self, name: &str, won: bool) {
        let stat = self.games.entry(name.to_string()).or_insert_with(Default::default);
        stat.played += 1;
        if won {
            stat.won += 1;
        }
    }

    pub(crate) fn game_stat(&self, name: &str) -> GameStat {
        match self.games.get(name) {
            None => GameStat::default(),
            Some(st) => st.clone(),
        }
    }

    pub(crate) fn save(&self) {
        let tml = toml::to_string(&self).unwrap(); // TODO:
        let path = stats_path();
        if let Err(e) = write(path, tml) {
            eprintln!("Failed to save statistics: {:?}", e);
        }
    }
}
