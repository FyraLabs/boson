//! Path Searching module
//! 
//! This module is a helper to quickly find the path to the Electron app's ASAR file by looking for them in common locations.
//! 
//! It also supports checking the environment variable `BOSON_LOAD_PATH` for a custom path.
use std::path::{Path, PathBuf};

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

/// Get ASAR path
/// 
/// Accepts a game root directory, usually from `get_game_path()`
pub fn get_asar_path(game_root: &Path) -> Option<PathBuf> {
    // First check if there's an override in the environment
    if let Some(path) = env_boson_load_path() {
        return Some(game_root.join(path));
    }

    // ASAR paths priority
    // 
    // Unpacked folders are prioritized over ASAR files
    // as games may be unpacked for development or modding.
    
    const ASAR_PATHS: [&str; 5] = [
        "app.asar/",
        "app.asar",
        "resources/app.asar/",
        "resources/app.asar",
        "resources/app/",
    ];
    
    for path in ASAR_PATHS.iter() {
        let asar_path = game_root.join(path);
        tracing::trace!("Checking path: {:?}", asar_path);
        if asar_path.exists() {
            if asar_path.is_dir() {
                tracing::info!("Found unpacked ASAR at {:?}", asar_path);
            } else {
                tracing::info!("Found ASAR at {:?}", asar_path);
            }
            return Some(asar_path);
        }
    }
    
    None
    
    
    
    
    
}