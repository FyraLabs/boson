//! TOML config module
//!
//! This module is responsible for reading and parsing the TOML configuration file.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
/// Default path to the app.asar file.
const DEFAULT_ASAR_PATH: &str = "resources/app.asar";
/// Default path to the extracted app.asar.
/// Preferred over `DEFAULT_ASAR_PATH` if exists
const PREFERRED_ASAR_PATH: &str = "resources/app";

/// Try to select the best path to the asar data
pub fn join_game_path(path: PathBuf, asar_path: Option<PathBuf>) -> PathBuf {
    if let Some(asar_path) = asar_path {
        asar_path
    } else {
        tracing::warn!("No asar_path provided, using default path");
        if path.join(PREFERRED_ASAR_PATH).is_dir() {
            path.join(PREFERRED_ASAR_PATH)
        } else {
            tracing::warn!("No extracted app.asar found or valid at `app/`, using default path");
            path.join(DEFAULT_ASAR_PATH)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GameConfig {
    /// Path to the game's app.asar file
    /// Can also be a plain directory
    /// default is `resources/app.asar`
    /// Boson should however, prefer the extracted app.asar (extracted as `app/` in the same directory as the `app.asar`)
    /// if no `asar_path` is provided
    #[serde(default)]
    pub asar_path: Option<PathBuf>,
    /// Replace all Electron flags with provided flags?
    ///
    /// Useful for some games that has some flags provided by default, but
    /// not compatible with Boson's runtime
    #[serde(default)]
    pub override_electron_flags: bool,

    /// Flags to pass to the Electron runtime
    ///
    /// Extra flags to pass to the Electron runtime.
    /// If `override_electron_flags` is set to `false`, these flags will be appended to the default flags
    /// instead of replacing them.
    #[serde(default)]
    pub electron_flags: Vec<String>,

    /// Path to the game's main folder or executable
    ///
    /// This is the path to the game's main folder or executable.
    /// Uses `util::get_game_path()` to resolve the path, so if a path is a file, it will be resolved to the parent directory.
    /// If a path is a directory, it will be resolved to the directory itself.
    /// If not provided, Boson will resolve the path automatically from CLI arguments (The ones provided to from Steam)
    #[serde(default)]
    pub game_path: Option<PathBuf>,

    // todo: figure out how to handle game patches. do we apply in runtime or before runtime? or both?
    // todo: also figure out a database of configs/patches for games
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    /// Game configurations
    ///
    /// Should be a map of either folder names, or steam IDs to `GameConfig`
    /// 
    /// Example:
    /// ```toml
    /// [games."123456"]
    /// asar_path = "resources/app"
    /// override_electron_flags = false
    /// 
    /// [games."Cookie Clicker"]
    /// asar_path = "resources/app"
    /// override_electron_flags = true
    /// electron_flags = ["--no-sandbox"]
    /// ```
    #[serde(default)]
    pub games: std::collections::HashMap<String, GameConfig>,
}
