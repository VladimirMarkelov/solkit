use std::env::current_exe;
use std::fs;
use std::path::{Path, PathBuf};

const CONF_FILE: &str = "config.toml";
const STAT_FILE: &str = "stats.toml";
const DEV_NAME: &str = "rionnag";
const APP_NAME: &str = "solkit";

// Returns the directory where the game binary is
fn exe_path() -> PathBuf {
    match current_exe() {
        Ok(mut p) => {
            p.pop();
            p
        }
        Err(_) => unreachable!(),
    }
}

// Returns current user's directory for configuration files
//    For Windows it is %USER%/Appdata/Roaming
//    For Linux it is ~/.config
fn user_config_path() -> PathBuf {
    match dirs::config_dir() {
        Some(p) => p,
        None => unreachable!(),
    }
}

// Returns the directory where the application save to/loads from all its configs/hiscores etc
// In normal mode:
//    For Windows it is %USER%/Appdata/Roaming/DEV_NAME/GAME_NAME/
//    For Linux it is ~/.config/DEV_NAME/GAME_NAME/
// In portable mode:
//    For all OSes it is directory where the application binary is
fn base_path() -> PathBuf {
    if is_portable() {
        exe_path()
    } else {
        let mut path = user_config_path();
        path.push(DEV_NAME);
        path.push(APP_NAME);
        ensure_path_exists(&path);
        path
    }
}

// Returns if the application works in portable mode.
// If there is CONF_FILE file in the directory where the application binary, it means the
// portable mode is on
fn is_portable() -> bool {
    let mut p = exe_path();
    p.push(CONF_FILE);
    p.exists()
}

// Creates all path's intermediate directories to make sure that the `p` exists.
// Returns false if it failed to create required directories (may happen, e.g, on read-only media
pub fn ensure_path_exists(p: &Path) -> bool {
    if p.exists() {
        return true;
    }
    fs::create_dir_all(p).is_ok()
}

// Returns path to the file with statistics
pub fn stats_path() -> PathBuf {
    let mut p = base_path();
    p.push(STAT_FILE);
    p
}

// Returns path to the file with user config
pub fn user_conf_path() -> PathBuf {
    let mut p = base_path();
    p.push(CONF_FILE);
    p
}
