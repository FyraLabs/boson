//! Runtime code for Boson
//!
//! This actually does the actual calling logic to we can actually run the games and stuff
//!
//! should replace the messy spaghetti in main.rs
use std::path::{Path, PathBuf};

use crate::{
    config::{GameConfig, SteamCompatConfig},
    path_search::get_asar_path,
};
use stable_eyre::Result;
#[derive(Debug)]
pub struct Runtime {
    pub steam_opts: SteamCompatConfig,
    pub game_config: GameConfig,
    pub exec_path: std::path::PathBuf,
}
#[derive(Debug, serde::Deserialize)]
pub struct ToolManifestVdf {
    pub manifest: ToolManifest,
}

#[derive(Debug, serde::Deserialize)]
pub struct ToolManifest {
    pub commandline: String,
    pub commandline_waitforexitandrun: Option<String>,
}

fn lookup_compat_tool(
    path: &str,
    steam_compat_config: &SteamCompatConfig,
) -> Result<Option<PathBuf>> {
    // Build the list of library paths to search. The STEAM_LIBRARY_PATHS env/config is required.
    let mut lib_paths = std::env::split_paths(
        steam_compat_config
            .library_paths
            .as_ref()
            .ok_or_else(|| stable_eyre::eyre::eyre!("STEAM_LIBRARY_PATHS is not set"))?,
    )
    .map(|p| p.join("common"))
    .collect::<Vec<_>>();

    // If a client install path is configured, add its compatibilitytools.d folder to the search list.
    if let Some(client_install) = &steam_compat_config.client_install_path {
        lib_paths.push(client_install.join("compatibilitytools.d"));
    }

    tracing::trace!(
        ?lib_paths,
        "Library paths to search for compatibility tools"
    );

    // Search each library path for a directory matching `path`.
    for base in lib_paths {
        let candidate = base.join(path);
        tracing::trace!(?candidate, "Checking for compatibility tool candidate");
        if candidate.exists() && candidate.is_dir() {
            tracing::debug!(?candidate, "Found compatibility tool");
            return Ok(Some(candidate));
        }
    }

    tracing::debug!(
        path = path,
        "Compatibility tool not found in configured library paths"
    );
    Ok(None)
}

/// Takes in a string path and returns the path to the compatibility tool wrapper if it exists
/// i.e defer to Proton
fn get_compat_tool_wrapper(path: &Path) -> Result<Option<(std::path::PathBuf, Vec<String>)>> {
    let toolmanifest_file = path.join("toolmanifest.vdf");
    // manifest.
    let toolmanifest_file = std::fs::File::open(toolmanifest_file)?;
    let toolmanifest: ToolManifest = keyvalues_serde::from_reader(toolmanifest_file)?;
    let cmdline = toolmanifest
        .commandline
        .strip_prefix('/')
        .unwrap_or(&toolmanifest.commandline);

    let mut iter = cmdline.split_whitespace();
    if let Some(first) = iter.next() {
        let expanded_path = shellexpand_full_no_errors(first).to_string();
        let wrapper_path = if Path::new(&expanded_path).is_relative() {
            // If path is relative, resolve it relative to the compat tool directory
            path.join(expanded_path)
        } else {
            PathBuf::from(expanded_path)
        };
        let wrapper_args = iter
            .map(|s| shellexpand_full_no_errors(s).to_string())
            .collect::<Vec<String>>();
        return Ok(Some((wrapper_path, wrapper_args)));
    }
    Ok(None)
    // todo!()
}

fn shellexpand_full_no_errors(s: &str) -> std::borrow::Cow<'_, str> {
    let home: String = dirs::home_dir()
        .and_then(|p| p.to_str().map(|s| s.to_owned()))
        .unwrap_or_default();
    shellexpand::full_with_context_no_errors::<str, _, _, String, _>(
        s,
        || Some(home.clone()),
        |var| std::env::var(var).ok(),
    )
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
        tracing::trace!(?self, ?additional_args, "Launching game");
        let executable_path = match &self.game_config.compat_type {
            &crate::config::CompatType::Electron => {
                // find the ASAR path
                get_asar_path(&self.exec_path).ok_or_else(|| {
                    stable_eyre::eyre::eyre!("Could not find ASAR path for Electron game")
                })?
            }
            _ => self.exec_path.clone(),
        };

        // Handle DeferProton case - dynamically get wrapper from compat tool
        let (wrapper, wrapper_args) = match &self.game_config.compat_type {
            crate::config::CompatType::DeferProton => {
                if let Some(tool_dir) = self.game_config.compat_tool_dir.as_deref() {
                    let found =
                        lookup_compat_tool(tool_dir, &self.steam_opts)?.ok_or_else(|| {
                            stable_eyre::eyre::eyre!(
                                "Compatibility tool '{}' not found in configured library paths",
                                tool_dir
                            )
                        })?;

                    tracing::debug!(?found, "Using compatibility tool from config");

                    // Get wrapper command and args from the compat tool
                    if let Some((wrapper_path, mut tool_wrapper_args)) =
                        get_compat_tool_wrapper(&found)?
                    {
                        // Combine tool wrapper args with user config wrapper args
                        tool_wrapper_args.extend(self.game_config.wrapper_args.clone());

                        tool_wrapper_args
                            .iter_mut()
                            .for_each(|arg| *arg = arg.replace("%verb%", "run"));

                        let wrapper_cmd = wrapper_path.to_string_lossy().to_string();
                        tracing::debug!(
                            ?wrapper_cmd,
                            ?tool_wrapper_args,
                            "DeferProton wrapper resolved"
                        );

                        (Some(wrapper_cmd), tool_wrapper_args)
                    } else {
                        tracing::warn!(
                            "Could not parse compatibility tool wrapper, falling back to none"
                        );
                        (None, self.game_config.wrapper_args.clone())
                    }
                } else {
                    tracing::warn!(
                        "DeferProton set but no compat_tool_dir configured, falling back to none"
                    );
                    (None, self.game_config.wrapper_args.clone())
                }
            }
            _ => {
                // Standard behavior for other compat types
                let (wrapper_default, wrapper_extras_default) =
                    self.game_config.compat_type.executable()?;

                let wrapper = self
                    .game_config
                    .wrapper_command
                    .as_ref()
                    .or(wrapper_default.as_ref())
                    .cloned();

                let wrapper_args = {
                    let mut args = self.game_config.wrapper_args.clone();
                    args.extend(wrapper_extras_default);
                    args
                };

                (wrapper, wrapper_args)
            }
        };

        tracing::debug!(?wrapper, "Wrapper executable");
        tracing::debug!(?wrapper_args, "Wrapper arguments");

        let ld_preload = {
            let mut preloads = vec![];
            let extra_preloads = self.game_config.extra_preloads.clone();
            if let Ok(existing) = std::env::var("LD_PRELOAD") {
                tracing::debug!(?existing, "Existing LD_PRELOAD found");
                // Split the existing LD_PRELOAD by colons and add each path separately
                let filtered_preloads = std::env::split_paths(&existing)
                    .filter_map(|path| {
                        path.to_str()
                            .map(|s| shellexpand_full_no_errors(s).to_string())
                    })
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
                    .filter_map(|path| {
                        path.to_str()
                            .map(|s| shellexpand_full_no_errors(s).to_string())
                    })
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
            cmd.env(key, shellexpand_full_no_errors(value).to_string());
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
