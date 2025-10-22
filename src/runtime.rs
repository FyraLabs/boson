//! Runtime code for Boson
//!
//! This actually does the actual calling logic to we can actually run the games and stuff
//!
//! should replace the messy spaghetti in main.rs
use crate::{
    config::{GameConfig, SteamCompatConfig},
    path_search::get_asar_path,
};
use stable_eyre::Result;

pub struct Runtime {
    pub steam_opts: SteamCompatConfig,
    pub game_config: GameConfig,
    pub exec_path: std::path::PathBuf,
}

impl Runtime {
    pub fn new(
        steam_opts: SteamCompatConfig,
        game_config: GameConfig,
        exec_path: std::path::PathBuf,
    ) -> Self {
        Self {
            steam_opts,
            game_config,
            exec_path,
        }
    }

    pub fn launch_game(&self, additional_args: Vec<String>) -> Result<()> {
        let executable_path = match &self.game_config.compat_type {
            &crate::config::CompatType::Electron => {
                // find the ASAR path
                get_asar_path(&self.exec_path).ok_or_else(|| {
                    stable_eyre::eyre::eyre!("Could not find ASAR path for Electron game")
                })?
            }
            _ => self.exec_path.clone(),
        };

        let (wrapper_default, wrapper_extras_default) =
            self.game_config.compat_type.executable()?;

        let wrapper = self
            .game_config
            .wrapper_command
            .as_ref()
            .or(wrapper_default.as_ref());

        let wrapper_args = {
            let mut args = self.game_config.wrapper_args.clone();
            args.extend(wrapper_extras_default);
            args
        };

        tracing::debug!(?wrapper, "Wrapper executable");
        tracing::debug!(?wrapper_args, "Wrapper arguments");

        let ld_preload = {
            let mut preloads = vec![];
            let mut extra_preloads = self.game_config.extra_preloads.clone();
            if let Ok(existing) = std::env::var("LD_PRELOAD") {
                tracing::debug!(?existing, "Existing LD_PRELOAD found");
                // Split the existing LD_PRELOAD by colons and add each path separately
                let filtered_preloads = std::env::split_paths(&existing)
                    .filter_map(|path| path.to_str().map(|s| s.to_string()))
                    .filter(|s| !s.is_empty())
                    .filter(|s| {
                        // Filter out Steam overlay if disabled in config
                        if self.game_config.disable_steam_overlay {
                            !s.contains("gameoverlayrenderer")
                        } else {
                            true
                        }
                    });
                preloads.extend(filtered_preloads);
            }

            tracing::trace!(?preloads, "Processed preloads");

            preloads.extend(extra_preloads.iter().cloned());
            preloads.join(":")
        };
        tracing::debug!(?ld_preload, "LD_PRELOAD");

        tracing::debug!(?executable_path, "Game executable path");
        tracing::debug!(?additional_args, "Additional arguments");

        let ld_library_path = {
            let mut paths = vec![];
            if let Ok(existing) = std::env::var("LD_LIBRARY_PATH") {
                tracing::debug!(?existing, "Existing LD_LIBRARY_PATH found");
                let existing_paths = std::env::split_paths(&existing)
                    .filter_map(|path| path.to_str().map(|s| s.to_string()))
                    .filter(|s| !s.is_empty());
                paths.extend(existing_paths);
            }
            // Add boson lib directory if it exists
            let exec_path = std::env::current_exe()?;
            if let Some(exec_dir) = exec_path.parent() {
                let lib_dir = exec_dir.join("lib");
                if lib_dir.exists() {
                    paths.push(lib_dir.display().to_string());
                }
            }

            if self.game_config.disable_steam_overlay {
                // filter out gameoverlayrenderer
                paths = paths
                    .into_iter()
                    .filter(|s| !s.contains("gameoverlayrenderer"))
                    .collect();
            }
            paths.join(":")
        };

        let mut cmd = if let Some(wrapper) = wrapper {
            let mut cmd = std::process::Command::new(wrapper);
            cmd.args(&wrapper_args);
            cmd.args(&additional_args);
            cmd.arg(&executable_path);
            cmd
        } else {
            let mut cmd = std::process::Command::new(&executable_path);
            cmd.args(&additional_args);
            cmd
        };

        cmd.env("LD_PRELOAD", ld_preload)
            .env("LD_LIBRARY_PATH", ld_library_path);

        // Add extra envars
        for (key, value) in &self.game_config.env_vars {
            cmd.env(key, value);
        }

        tracing::info!("Launching game with command: {:?}", cmd);


        let mut child = cmd.spawn()?;
        let status = child.wait()?;
        if !status.success() {
            return Err(stable_eyre::eyre::eyre!(
                "Game exited with non-zero status: {}",
                status
            ));
        }
        Ok(())  
    }
}
