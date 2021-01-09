use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

use crate::config::stats_path;

const DAY_SEC: u64 = 60 * 60 * 24;
const HOUR_SEC: u64 = 60 * 60;
const MIN_SEC: u64 = 60;

pub(crate) fn duration_to_human(dur: Option<Duration>) -> String {
    let d = match dur {
        None => 0,
        Some(tm) => tm.as_secs(),
    };
    if d == 0 {
        return String::new();
    }
    if d >= 98 * DAY_SEC {
        return format!("{}d", (d - 1 + DAY_SEC / 2) / DAY_SEC);
    } else if d >= DAY_SEC {
        let days = d / DAY_SEC;
        let hours = (d - days * DAY_SEC + HOUR_SEC / 2 - 1) / HOUR_SEC;
        return format!("{}d{}h", days, hours);
    } else if d >= HOUR_SEC {
        let hours = d / HOUR_SEC;
        let mins = (d - hours * HOUR_SEC + MIN_SEC / 2 - 1) / MIN_SEC;
        return format!("{}h{}m", hours, mins);
    } else {
        let mins = d / MIN_SEC;
        let secs = d - mins * MIN_SEC;
        return format!("{}m{}s", mins, secs);
    }
}

// solitaire statistics
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct GameStat {
    pub(crate) played: u64,
    pub(crate) won: u64,
    pub(crate) spent: Option<Duration>,
}

impl Default for GameStat {
    fn default() -> GameStat {
        GameStat { played: 0, won: 0, spent: Some(Duration::new(0, 0)) }
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

    pub(crate) fn update_stat(&mut self, name: &str, won: bool, spent: Duration) {
        let stat = self.games.entry(name.to_string()).or_insert_with(Default::default);
        stat.played += 1;
        let old = match stat.spent {
            None => Duration::new(0, 0),
            Some(d) => d,
        };
        stat.spent = Some(old + spent);
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
        let tml = toml::to_string(&self).expect("failed to serialize statistics");
        let path = stats_path();
        if let Err(e) = write(path, tml) {
            eprintln!("Failed to save statistics: {:?}", e);
        }
    }
}

#[cfg(test)]
mod stats_test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn human_duration() {
        struct Dt {
            d: u64,
            s: &'static str,
        }
        let day_short = DAY_SEC + 20;
        let day_long = DAY_SEC * 2 + HOUR_SEC * 3 + 150;
        let days_only = DAY_SEC * 101;

        let data: Vec<Dt> = vec![
            Dt { d: 0, s: "" },
            Dt { d: 17, s: "0m17s" },
            Dt { d: 60, s: "1m0s" },
            Dt { d: 130, s: "2m10s" },
            Dt { d: 3600, s: "1h0m" },
            Dt { d: 3620, s: "1h0m" },
            Dt { d: 3640, s: "1h1m" },
            Dt { d: 7203, s: "2h0m" },
            Dt { d: 7383, s: "2h3m" },
            Dt { d: day_short, s: "1d0h" },
            Dt { d: day_long, s: "2d3h" },
            Dt { d: days_only, s: "101d" },
        ];
        for d in data.iter() {
            let dur = Some(Duration::new(d.d, 0));
            let st = duration_to_human(dur);
            assert_eq!(d.s, &st);
        }
    }
}
