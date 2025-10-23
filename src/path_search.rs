//! Path Searching module
//!
//! This module is a helper to quickly find the path to the Electron app's ASAR file by looking for them in common locations.
//!
//! It also supports checking the environment variable `BOSON_LOAD_PATH` for a custom path.
use std::{
    path::{Path, PathBuf},
    process,
};

pub fn global_config_path() -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let cfg = config_dir.join("boson.toml");
        if cfg.is_file() {
            Some(cfg)
        } else {
            None
        }
    } else {
        None
    }
}

/// Returns a list of load paths, from the factory Boson database
/// + any additional user configured paths
///
/// Load priority:
/// - ~/.config/boson.d/<gameid>.toml
/// - ~/<steam>/compatibilitytools.d/boson/data/<gameid>.toml
pub fn config_load_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let userconfig_dir = dirs::config_dir().and_then(|f| Some(f.join("boson.d")));
    if let Some(dir) = userconfig_dir {
        paths.push(dir);
    }

    let exec_path = std::env::current_exe().unwrap_or_else(|e| {
        tracing::error!("Failed to get current executable path: {:?}", e);
        process::exit(1);
    });

    let steam_compat_path = exec_path.parent().map(|p| p.join("data"));

    tracing::debug!("Steam compatibility data path: {:?}", steam_compat_path);

    if let Some(dir) = steam_compat_path {
        paths.push(dir);
    }

    tracing::debug!("Config load paths: {:?}", paths);

    paths
}

pub fn env_boson_load_path() -> Option<String> {
    std::env::var("BOSON_LOAD_PATH").ok()
}

pub fn get_game_path(path: &Path) -> PathBuf {
    // remove file name from path
    if path.is_file() {
        path.parent().unwrap().canonicalize().unwrap().to_path_buf()
    } else {
        path.canonicalize().unwrap().to_path_buf()
    }
}

pub fn env_electron_path() -> String {
    std::env::var("ELECTRON_PATH").unwrap_or_else(|_| "electron".to_string())
}

// This function scans for a package.json file in the game directory
// Does nothing for now except logging

fn package_json_scan(path: &Path) {
    use serde_json::Value;
    use std::fs::File;
    use std::io::Read;

    // if is file
    if path.is_file() {
        tracing::info!("Path is a file, ignoring.");
        return;
    }

    let package_json = path.join("package.json");
    if package_json.exists() {
        tracing::info!("Found package.json at {:?}", package_json);

        // Try to open the file
        let mut file = match File::open(&package_json) {
            Ok(file) => file,
            Err(e) => {
                tracing::warn!("Failed to open package.json: {:?}", e);
                return;
            }
        };

        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            tracing::warn!("Failed to read package.json: {:?}", e);
            return;
        }

        let v: Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to parse package.json: {:?}", e);
                return;
            }
        };

        // Check if it's a valid Electron package
        if v.get("main").is_none() {
            tracing::warn!("package.json does not specify a main script, it may not be a valid Electron package.");
        } else {
            tracing::info!("Validated package.json as an Electron package");
        }
    } else {
        tracing::warn!(
            "Could not find package.json in game directory. This may not be the game directory."
        );
    }
}

fn _compat_data_path() -> Option<String> {
    std::env::var("STEAM_COMPAT_DATA_PATH")
        .ok()
        .map(|s| PathBuf::from(s).join("boson"))
        .map(|p| p.to_str().unwrap().to_string())
}

/// Get ASAR path
///
/// Accepts a game root directory, usually from `get_game_path()`
/// and returns the path to the ASAR
#[tracing::instrument]
pub fn get_asar_path(game_exec_path: &Path) -> Option<PathBuf> {
    let game_path = {
        if let Ok(path) = std::env::var("STEAM_COMPAT_INSTALL_PATH") {
            tracing::info!("STEAM_COMPAT_INSTALL_PATH found: {:?}", path);
            path.into()
        }
        // If the game path is not provided, use the game executable path
        else {
            get_game_path(game_exec_path)
        }
    };

    tracing::trace!("Game path: {:?}", game_path);
    // First check if there's an override in the environment
    if let Some(path) = env_boson_load_path() {
        return Some(game_path.join(path));
    }

    // ASAR paths priority
    //
    // Unpacked folders are prioritized over ASAR files
    // as games may be unpacked for development or modding.
    //
    // todo: walk through every directory in the actual game path (except node_modules) and find a package.json with "main" pointing to the JS file
    // if found, return that directory as the game path
    // else find the ASAR archive

    const ASAR_PATHS: [&str; 4] = [
        "app.asar",
        "resources/app.asar.unpacked",
        "resources/app.asar",
        "resources/app",
    ];

    // Funny guard clause

    // If the current game_path is actually one of (ends with) the actual ASAR path here, return it directly
    if ASAR_PATHS.iter().any(|path| game_path.ends_with(path)) {
        // find package.json
        package_json_scan(&game_path);
        tracing::info!("Found ASAR at {:?}", game_path);
        return Some(game_path);
    }

    for path in ASAR_PATHS.iter() {
        let asar_path = game_path.join(path);
        tracing::trace!("Checking path: {:?}", asar_path);
        if asar_path.exists() {
            if asar_path.is_dir() {
                tracing::info!("Found unpacked ASAR at {:?}", asar_path);
            } else {
                tracing::info!("Found ASAR at {:?}", asar_path);
            }
            package_json_scan(&asar_path);
            return Some(asar_path);
        }
    }

    None
}
