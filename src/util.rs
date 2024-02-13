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

/// Utility struct for handling LD_LIBRARY_PATH
pub struct LDPath {
    pub paths: Vec<PathBuf>,
}

impl LDPath {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    pub fn add(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub fn to_string(&self) -> String {
        self.paths
            .iter()
            .map(|p| p.to_str().unwrap())
            .collect::<Vec<&str>>()
            .join(":")
    }

    pub fn from_env() -> Self {
        let paths = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let paths = paths
            .split(":")
            .map(|p| PathBuf::from(p))
            .collect::<Vec<PathBuf>>();
        Self { paths }
    }
}
