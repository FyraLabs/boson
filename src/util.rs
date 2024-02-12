use std::path::{Path, PathBuf};

const APPDIR: &str = "resources/app";

pub fn electron_path() -> String {
    // get envar

    if let Ok(cmd) = std::env::var("ELECTRON_PATH") {
        cmd
    } else {
        "electron".to_string()
    }
}

pub fn get_game_path(path: &Path) -> PathBuf {
    // remove file name from path
    if path.is_file() {
        path.parent().unwrap().canonicalize().unwrap().to_path_buf()
    } else {
        path.canonicalize().unwrap().to_path_buf()
    }
}

/// Specify a relative path to the game executable to load the Electron files from
/// defaults to `resources/app`, as Cookie Clicker puts its files there
pub fn app_path() -> String {
    if let Ok(path) = std::env::var("BOSON_LOAD_PATH") {
        path
    } else {
        APPDIR.to_string()
    }
}