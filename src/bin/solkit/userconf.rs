use std::fs::{read_to_string, write};

use serde_derive::{Deserialize, Serialize};

use crate::config::user_conf_path;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct UserConf {
    pub(crate) last_played: String,
}

impl Default for UserConf {
    fn default() -> UserConf {
        UserConf { last_played: String::new() }
    }
}

impl UserConf {
    pub(crate) fn load() -> Self {
        let path = user_conf_path();
        if !path.exists() {
            return UserConf::default();
        }
        let data = match read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to load user configuration from: {:?}", e);
                return UserConf::default();
            }
        };
        let uconf: UserConf = match toml::from_str(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read TOML user configuration: {:?}", e);
                UserConf::default()
            }
        };
        uconf
    }

    pub(crate) fn save(&self) {
        let tml = toml::to_string(&self).expect("failed to serialize user configuration");
        let path = user_conf_path();
        if let Err(e) = write(path, tml) {
            eprintln!("Failed to save user configuration: {:?}", e);
        }
    }
}
