//! Configuration module for Boson Steam compat layer.
//!
//! We want Boson to be more of a "native if possible" compat layer to override some Steam games that run poorly
//! under Wine, or have native Linux source port/runtimes available already, i.e. Electron or LOVE games.
//!
//! This module handles configuration options for Boson, including environment variables and command-line arguments.
//!
//! Note: Please refer to https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/main/docs/steam-compat-tool-interface.md
//! for details on the Steam compatibility tool interface and environment variables.

use clap::Parser;
use serde::{Deserialize, Serialize};
use stable_eyre::Result;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::path_search;

// Custom parser for reading 0/1 (and true/false) into a bool
fn parse_bool(s: &str) -> std::result::Result<bool, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        other => Err(format!(
            "invalid boolean '{}', expected 0/1 or true/false",
            other
        )),
    }
}

/// Wrapper for Steam compatibility options
#[derive(Parser, Default, Debug, Clone)]
#[clap(allow_missing_positional = true, hide = true)]
pub struct SteamCompatConfig {
    #[clap(long, hide = true, env = "STEAM_COMPAT_DATA_PATH")]
    pub app_id: Option<String>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_CLIENT_INSTALL_PATH")]
    pub client_install_path: Option<PathBuf>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_DATA_PATH")]
    pub data_path: Option<PathBuf>,
    #[clap(
        long,
        hide = true,
        env = "STEAM_COMPAT_FLAGS",
        use_value_delimiter = true,
        value_delimiter = ','
    )]
    pub flags: Vec<String>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_INSTALL_PATH")]
    pub install_path: Option<PathBuf>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_LAUNCHER_SERVICE")]
    pub launcher_service: Option<String>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_LIBRARY_PATHS")]
    pub library_paths: Option<String>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_MOUNTS")]
    pub mounts: Option<String>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_RUNTIME_SDL2")]
    pub runtime_sdl2: Option<PathBuf>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_RUNTIME_SDL3")]
    pub runtime_sdl3: Option<PathBuf>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_SESSION_ID")]
    pub session_id: Option<String>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_SHADER_PATH")]
    pub shader_path: Option<PathBuf>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_TOOL_PATHS")]
    pub tool_paths: Option<String>,
    #[clap(long, hide = true, env = "STEAM_COMPAT_TRACING")]
    pub tracing: Option<String>,
    #[clap(long, hide = true, env = "SteamAppId")]
    pub steam_app_id: Option<u32>,
    /// Whether Steam Deck mode is enabled for this game (if applicable)
    /// i.e Running dedicated Big Picture shell, or on SteamOS 3
    #[clap(long, hide = true, env = "SteamDeck", value_parser = parse_bool)]
    pub steam_deck: Option<bool>,
}

impl SteamCompatConfig {
    /// Extract the Steam app ID from the available sources
    /// First tries steam_app_id, then falls back to parsing app_id if it's numeric
    pub fn get_app_id(&self) -> Option<u32> {
        // First try the direct steam_app_id field
        if let Some(id) = self.steam_app_id {
            return Some(id);
        }

        // Fall back to parsing app_id as a number if available
        if let Some(app_id_str) = &self.app_id {
            if let Ok(id) = app_id_str.parse::<u32>() {
                return Some(id);
            }
        }

        None
    }
}

/// How Boson should handle compatibility modes for different games
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub enum CompatType {
    #[default]
    /// Defer to another Steam compatibility tool, e.g., Proton
    DeferProton,
    /// Attempt native execution
    /// Might be useful for .NET applications or cross-platform binaries
    ForceNative,

    /// Electron-based games, Wrap the game with Boson's Electron runtime
    Electron,
    /// LOVE2D games, wrap the game with `love` runtime
    /// Requires `love` to be installed on the system
    Love,
}

impl CompatType {
    /// Get runtime-specific default configuration for this compatibility type
    /// These are the base defaults that apply to all games using this runtime
    pub fn runtime_defaults(&self) -> GameConfig {
        match self {
            CompatType::DeferProton => GameConfig {
                compat_type: CompatType::DeferProton,
                wrapper_command: None,
                wrapper_args: vec![],
                env_vars: BTreeMap::new(),
                append_args: vec![],
                extra_preloads: vec![],
                compat_tool_dir: default_compat_tool_dir(),
                ..Default::default()
            },
            CompatType::ForceNative => GameConfig::default(),
            CompatType::Electron => GameConfig {
                compat_type: CompatType::Electron,
                wrapper_command: None,
                wrapper_args: vec![],
                env_vars: BTreeMap::new(),
                append_args: vec![],
                extra_preloads: vec![],
                disable_steam_overlay: true,
                ..Default::default()
            },
            CompatType::Love => GameConfig {
                compat_type: CompatType::Love,
                wrapper_command: None,
                wrapper_args: vec![],
                env_vars: BTreeMap::new(),
                append_args: vec![],
                extra_preloads: vec![],
                disable_steam_overlay: false,
                ..Default::default()
            },
        }
    }

    /// Get executable path and default args for this compatibility type
    ///
    /// returns:
    /// - Option<PathBuf>: Path to the default wrapper executable, if any
    /// - Vec<String>: Default arguments to pass to the wrapper executable
    pub fn executable(&self) -> Result<(Option<String>, Vec<String>)> {
        let exec_path = std::env::current_exe()?;
        match self {
            // DeferProton is handled specially in runtime - returns empty defaults here
            CompatType::DeferProton => Ok((None, vec![])),
            CompatType::ForceNative => Ok((None, vec![])),
            CompatType::Electron => {
                let exec_dir = exec_path.parent().unwrap();
                let electron = path_search::env_electron_path();
                let mut args = vec!["--no-sandbox"];
                // todo: Probably better way to hook into Electron apps?
                let hook_path = exec_dir.join("register-hook.js");
                // Use Boson's built-in Electron runtime
                let load_hook_arg = vec!["--require", hook_path.to_str().unwrap()];
                args.extend(load_hook_arg);
                Ok((Some(electron), args.iter().map(|s| s.to_string()).collect()))
            }
            CompatType::Love => {
                // Use system LOVE2D runtime

                Ok((Some("love".to_string()), vec![]))
            }
        }
    }
}

/// Embedded default configs for some known games,
/// don't expect this to be exhaustive, just the most commonly known ones
pub fn default_game_configs() -> Vec<(u32, GameConfig)> {
    vec![
        // Balatro: Use LOVE2D runtime
        // consider setting up LD_PRELOAD with liblovely.so
        // if you use SMODS or mod
        (
            2379780,
            GameConfig {
                compat_type: CompatType::Love,
                ..Default::default()
            },
        ),
        // Cookie Clicker: use Electron
        (
            1454400,
            GameConfig {
                compat_type: CompatType::Electron,
                disable_steam_overlay: true,
                ..Default::default()
            },
        ),
    ]
}

#[derive(Serialize, Deserialize)]
pub struct BosonConfig {
    pub default_compat_config: GameConfig,
    pub game_overrides: Vec<(u32, GameConfig)>,
}

impl Default for BosonConfig {
    fn default() -> Self {
        Self {
            default_compat_config: GameConfig::default(),
            game_overrides: default_game_configs(),
        }
    }
}

impl BosonConfig {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let mut config = BosonConfig::default();
        let config_paths = path_search::config_load_paths();

        tracing::debug!("Loading configuration from paths: {:?}", config_paths);

        // Iterate through each config directory
        for config_dir in config_paths {
            if !config_dir.exists() {
                tracing::debug!("Config directory does not exist: {:?}", config_dir);
                continue;
            }

            // Use jwalk to find all .toml files in the directory
            for entry in jwalk::WalkDir::new(&config_dir)
                .max_depth(1) // Only look in the immediate directory
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|entry| {
                    entry.file_type().is_file()
                        && entry.path().extension().map_or(false, |ext| ext == "toml")
                })
            {
                let file_path = entry.path();
                tracing::debug!("Found TOML file: {:?}", file_path);

                match Self::load_config_file(&file_path) {
                    Ok(game_config_file) => {
                        tracing::info!("Loaded config from: {:?}", file_path);

                        // Merge the loaded overrides into our config
                        for (app_id, game_config) in game_config_file.overrides {
                            // Check if we already have an override for this app_id
                            let existing_index = config
                                .game_overrides
                                .iter()
                                .position(|(id, _)| *id == app_id);

                            if let Some(index) = existing_index {
                                // Merge the configs, with the loaded one taking priority
                                let mut existing_config = config.game_overrides[index].1.clone();
                                Self::merge_config_static(&mut existing_config, &game_config);
                                config.game_overrides[index].1 = existing_config;
                            } else {
                                // Add new override
                                config.game_overrides.push((app_id, game_config));
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load config file {:?}: {}", file_path, e);
                    }
                }
            }
        }

        tracing::info!("Loaded {} game overrides", config.game_overrides.len());
        Ok(config)
    }

    /// Load a single config file
    fn load_config_file(path: &Path) -> Result<GameConfigFile> {
        let contents = fs::read_to_string(path)?;
        let config_file: GameConfigFile = toml::from_str(&contents)?;
        Ok(config_file)
    }

    /// Get the game configuration for the given app ID, merging 3 layers of config:
    /// 1. Runtime defaults (based on CompatType)
    /// 2. Global defaults (default_compat_config)
    /// 3. User-defined overrides (game_overrides)
    ///
    /// Later layers override earlier ones, with Vec fields being extended rather than replaced.
    ///
    /// # Example
    /// ```
    /// let config = BosonConfig::default();
    /// let balatro_config = config.get_game_config(2379780); // Balatro
    /// // Gets Love runtime defaults + global defaults + Love compat type override
    /// ```
    pub fn get_game_config(&self, app_id: u32) -> GameConfig {
        // Find the user override if it exists
        let user_override = self
            .game_overrides
            .iter()
            .find(|(id, _)| *id == app_id)
            .map(|(_, config)| config);

        // Determine the compat type from user override or default
        let compat_type = user_override
            .map(|config| &config.compat_type)
            .unwrap_or(&self.default_compat_config.compat_type);

        // Start with runtime defaults for this compat type
        let mut merged = compat_type.runtime_defaults();

        // Layer 2: Merge with global defaults
        self.merge_config(&mut merged, &self.default_compat_config);

        // Layer 3: Merge with user overrides if they exist
        if let Some(override_config) = user_override {
            self.merge_config(&mut merged, override_config);
        }

        merged
    }

    /// Helper function to merge one config into another
    /// Vec fields are extended, other fields are overwritten if present
    fn merge_config(&self, base: &mut GameConfig, overlay: &GameConfig) {
        Self::merge_config_static(base, overlay);
    }

    /// Static version of merge_config for use without self reference
    fn merge_config_static(base: &mut GameConfig, overlay: &GameConfig) {
        // Only override compat_type if overlay has a different one
        if !matches!(
            (&overlay.compat_type, &base.compat_type),
            (CompatType::Love, CompatType::Love)
        ) {
            base.compat_type = overlay.compat_type.clone();
        }

        // Only override wrapper_command if overlay explicitly sets it
        if overlay.wrapper_command.is_some() {
            base.wrapper_command = overlay.wrapper_command.clone();
        }

        // Always extend wrapper_args additively - don't replace if base has args
        base.wrapper_args.extend(overlay.wrapper_args.clone());

        // Extend vec fields rather than replace
        base.env_vars.extend(overlay.env_vars.clone());
        base.append_args.extend(overlay.append_args.clone());
        base.extra_preloads.extend(overlay.extra_preloads.clone());

        // Only override compat_tool_dir if overlay explicitly sets it to Some value
        // This preserves runtime defaults when user config doesn't specify compat_tool_dir
        if overlay.compat_tool_dir.is_some() {
            base.compat_tool_dir = overlay.compat_tool_dir.clone();
        }

        // Boolean fields are overridden if explicitly set
        base.disable_steam_overlay = overlay.disable_steam_overlay;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_defaults() {
        // Test Electron runtime defaults
        let electron_defaults = CompatType::Electron.runtime_defaults();
        assert_eq!(electron_defaults.compat_type, CompatType::Electron);
        assert_eq!(electron_defaults.disable_steam_overlay, true);
        assert!(electron_defaults.env_vars.is_empty());

        // Test Love runtime defaults
        let love_defaults = CompatType::Love.runtime_defaults();
        assert_eq!(love_defaults.compat_type, CompatType::Love);
        assert_eq!(love_defaults.disable_steam_overlay, false);
    }

    #[test]
    fn test_3_layer_config_merging() {
        let config = BosonConfig::default();

        // Test Balatro (has user override for Love)
        let balatro_config = config.get_game_config(2379780);
        assert_eq!(balatro_config.compat_type, CompatType::Love);
        assert_eq!(balatro_config.disable_steam_overlay, false); // Love runtime default

        // Test unknown app ID (uses default compat type which is Love)
        let unknown_config = config.get_game_config(999999);
        assert_eq!(unknown_config.compat_type, CompatType::DeferProton);
        assert_eq!(unknown_config.disable_steam_overlay, false);
    }

    #[test]
    fn test_electron_game_config() {
        let mut config = BosonConfig::default();

        // Add an Electron game override
        config.game_overrides.push((
            123456,
            GameConfig {
                compat_type: CompatType::Electron,
                wrapper_command: None,
                wrapper_args: vec![],
                env_vars: [("CUSTOM_VAR".to_string(), "value".to_string())].into(),
                append_args: vec!["--custom-arg".to_string()],
                extra_preloads: vec![],
                disable_steam_overlay: false, // User wants to override runtime default
                ..Default::default()
            },
        ));

        let electron_config = config.get_game_config(123456);
        assert_eq!(electron_config.compat_type, CompatType::Electron);
        // User override should win for disable_steam_overlay
        assert_eq!(electron_config.disable_steam_overlay, false);
        // Should have custom env var from user override
        assert!(electron_config.env_vars.contains_key("CUSTOM_VAR"));
        assert!(electron_config
            .append_args
            .contains(&"--custom-arg".to_string()));
    }

    #[test]
    fn test_comprehensive_3_layer_merging() {
        // Create a custom config with global defaults
        let mut config = BosonConfig {
            default_compat_config: GameConfig {
                compat_type: CompatType::ForceNative,
                wrapper_command: None,
                wrapper_args: vec!["--global-arg".to_string()],
                env_vars: [("GLOBAL_VAR".to_string(), "global_value".to_string())].into(),
                append_args: vec!["--global-append".to_string()],
                extra_preloads: vec!["libglobal.so".to_string()],
                disable_steam_overlay: false,
                ..Default::default()
            },
            game_overrides: vec![],
        };

        // Add an Electron game with user overrides
        config.game_overrides.push((
            555555,
            GameConfig {
                compat_type: CompatType::Electron,
                wrapper_command: Some("custom-electron".to_string()),
                wrapper_args: vec!["--user-arg".to_string()],
                env_vars: [("USER_VAR".to_string(), "user_value".to_string())].into(),
                append_args: vec!["--user-append".to_string()],
                extra_preloads: vec!["libuser.so".to_string()],
                disable_steam_overlay: false, // Override Electron runtime default
                ..Default::default()
            },
        ));

        let final_config = config.get_game_config(555555);

        // Verify the final merged config
        assert_eq!(final_config.compat_type, CompatType::Electron);
        assert_eq!(
            final_config.wrapper_command,
            Some("custom-electron".to_string())
        );

        // wrapper_args should be merged: global + user (since user is non-empty, it extends)
        assert_eq!(
            final_config.wrapper_args,
            vec!["--global-arg".to_string(), "--user-arg".to_string()]
        );

        // env_vars should be merged: runtime + global + user
        assert_eq!(
            final_config.env_vars.get("GLOBAL_VAR"),
            Some(&"global_value".to_string())
        );
        assert_eq!(
            final_config.env_vars.get("USER_VAR"),
            Some(&"user_value".to_string())
        );

        // append_args should be merged: runtime + global + user
        assert!(final_config
            .append_args
            .contains(&"--global-append".to_string()));
        assert!(final_config
            .append_args
            .contains(&"--user-append".to_string()));

        // extra_preloads should be merged: runtime + global + user
        assert!(final_config
            .extra_preloads
            .contains(&"libglobal.so".to_string()));
        assert!(final_config
            .extra_preloads
            .contains(&"libuser.so".to_string()));

        // disable_steam_overlay should use user override (false) not Electron runtime default (true)
        assert_eq!(final_config.disable_steam_overlay, false);
    }

    #[test]
    fn test_defer_proton_config_merging() {
        // Test that DeferProton configs merge additively and use runtime defaults
        let mut config = BosonConfig {
            default_compat_config: GameConfig {
                compat_type: CompatType::ForceNative,
                wrapper_args: vec!["--global-arg".to_string()],
                compat_tool_dir: Some("GlobalProton".to_string()),
                ..Default::default()
            },
            game_overrides: vec![],
        };

        // Add a DeferProton game with user overrides that don't specify compat_tool_dir
        config.game_overrides.push((
            666666,
            GameConfig {
                compat_type: CompatType::DeferProton,
                wrapper_command: None,
                wrapper_args: vec!["--user-arg".to_string()],
                env_vars: BTreeMap::new(),
                append_args: vec![],
                extra_preloads: vec![],
                disable_steam_overlay: false,
                // Don't specify compat_tool_dir - should get runtime default
                compat_tool_dir: None,
            },
        ));

        let final_config = config.get_game_config(666666);

        // Verify the final merged config
        assert_eq!(final_config.compat_type, CompatType::DeferProton);

        // Should have global default compat_tool_dir since that's how merging works
        // Runtime defaults get overridden by global defaults
        assert_eq!(
            final_config.compat_tool_dir,
            Some("GlobalProton".to_string())
        );

        // wrapper_args should be merged: global + user
        assert!(final_config
            .wrapper_args
            .contains(&"--global-arg".to_string()));
        assert!(final_config
            .wrapper_args
            .contains(&"--user-arg".to_string()));
    }

    #[test]
    fn test_defer_proton_explicit_override() {
        // Test that DeferProton explicit overrides work correctly
        let mut config = BosonConfig {
            default_compat_config: GameConfig::default(),
            game_overrides: vec![],
        };

        // Add a DeferProton game with explicit compat_tool_dir override
        config.game_overrides.push((
            777777,
            GameConfig {
                compat_type: CompatType::DeferProton,
                wrapper_args: vec!["--custom-arg".to_string()],
                compat_tool_dir: Some("Proton-GE-8-32".to_string()),
                ..Default::default()
            },
        ));

        let final_config = config.get_game_config(777777);

        // Should use the explicitly set compat_tool_dir, not runtime default
        assert_eq!(
            final_config.compat_tool_dir,
            Some("Proton-GE-8-32".to_string())
        );
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct GameConfig {
    pub compat_type: CompatType,
    /// Optional command to wrap the game execution, e.g., `love` for LOVE2D games
    /// If set, this will override the default executable used for each `CompatType`
    pub wrapper_command: Option<String>,

    /// Additional arguments to pass to the wrapper command
    pub wrapper_args: Vec<String>,

    /// Additional environment variables to set when launching the game
    pub env_vars: BTreeMap<String, String>,

    /// Additional arguments to pass to the game executable
    pub append_args: Vec<String>,

    /// Additional libraries to preload via LD_PRELOAD
    pub extra_preloads: Vec<String>,

    /// Disable Steam Overlay for this game
    /// May fix some compatibility issues
    pub disable_steam_overlay: bool,

    /// Specify the directory of the compatibility tool to use
    /// This is for deferring to another compatibility tool like Proton
    ///
    /// If not set, this will default to "Proton - Experimental"
    ///
    /// This looks up the compatibility tool in Steam's compatibilitytools.d directory
    /// or the Steam library paths
    #[serde(default = "default_compat_tool_dir")]
    pub compat_tool_dir: Option<String>,
}

fn default_compat_tool_dir() -> Option<String> {
    Some("Proton - Experimental".to_string())
}

/// Game config file on disk, used for storing factory defaults and user overrides
/// This is a simple wrapper around GameConfig for potential future expansion
///
/// ```toml
/// [override.123456] # Game ID
/// compat_type = "Electron"
/// disable_steam_overlay = true
/// wrapper_command = "/custom/path/to/electron"
#[derive(Serialize, Deserialize)]
pub struct GameConfigFile {
    #[serde(rename = "override")]
    pub overrides: BTreeMap<u32, GameConfig>,
}

impl GameConfigFile {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_string(&self) -> Result<String> {
        let toml_str = toml::to_string(self)?;
        Ok(toml_str)
    }
}

#[cfg(test)]
mod game_config_file_tests {
    use super::*;
    #[test]
    fn test_game_config_file_serialization() {
        let mut overrides = BTreeMap::new();
        overrides.insert(
            123456,
            GameConfig {
                compat_type: CompatType::Electron,
                wrapper_command: Some("/custom/path/to/electron".to_string()),
                wrapper_args: vec!["--arg1".to_string(), "--arg2".to_string()],
                env_vars: [("VAR1".to_string(), "value1".to_string())].into(),
                append_args: vec!["--game-arg".to_string()],
                extra_preloads: vec!["libcustom.so".to_string()],
                disable_steam_overlay: true,
                ..Default::default()
            },
        );

        let game_config_file = GameConfigFile { overrides };
        let toml_str = game_config_file.to_string().unwrap();

        let _expected_toml = r#"[override.123456]
        compat_type = "Electron"
        wrapper_command = "/custom/path/to/electron"
        wrapper_args = ["--arg1", "--arg2"]
        env_vars = { VAR1 = "value1" }
        append_args = ["--game-arg"]
        extra_preloads = ["libcustom.so"]
        disable_steam_overlay = true"#;

        println!("Serialized TOML:\n{}", toml_str);
        assert!(toml_str.contains("[override.123456]"));
        assert!(toml_str.contains("compat_type = \"Electron\""));
        assert!(toml_str.contains("wrapper_command = \"/custom/path/to/electron\""));
    }

    #[test]
    fn test_config_loading() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("boson.d");
        fs::create_dir_all(&config_dir).unwrap();

        // Create a test config file
        let test_config = r#"
[override.123456]
compat_type = "Electron"
wrapper_command = "/custom/electron"
env_vars = { TEST_VAR = "test_value", ANOTHER_VAR = "another_value" }
append_args = ["--test-arg"]
disable_steam_overlay = true

[override.789012]
compat_type = "Love"
wrapper_args = ["--love-arg"]
"#;

        let config_file_path = config_dir.join("test_games.toml");
        fs::write(&config_file_path, test_config).unwrap();

        // Mock the config_load_paths function by temporarily setting up the path
        // Since we can't easily mock the function, we'll test the load_config_file method directly
        let loaded_config = BosonConfig::load_config_file(&config_file_path).unwrap();

        // Verify the loaded config
        assert_eq!(loaded_config.overrides.len(), 2);

        let electron_config = &loaded_config.overrides[&123456];
        assert_eq!(electron_config.compat_type, CompatType::Electron);
        assert_eq!(
            electron_config.wrapper_command,
            Some("/custom/electron".to_string())
        );
        assert_eq!(
            electron_config.env_vars.get("TEST_VAR"),
            Some(&"test_value".to_string())
        );
        assert_eq!(
            electron_config.env_vars.get("ANOTHER_VAR"),
            Some(&"another_value".to_string())
        );
        assert!(electron_config
            .append_args
            .contains(&"--test-arg".to_string()));
        assert_eq!(electron_config.disable_steam_overlay, true);

        let love_config = &loaded_config.overrides[&789012];
        assert_eq!(love_config.compat_type, CompatType::Love);
        assert!(love_config.wrapper_args.contains(&"--love-arg".to_string()));
    }
}
