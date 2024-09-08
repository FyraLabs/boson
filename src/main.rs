use clap::{Parser, Subcommand};
use color_eyre::{eyre::OptionExt, Result};
use path_search::get_asar_path;
use std::{path::PathBuf, process::Command};
mod path_search;
// use tracing_subscriber::;
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: &str = "info";
#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: &str = "trace";

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(allow_hyphen_values = true)]
pub struct Boson {
    #[command(subcommand)]
    cmd: Commands,
}
#[derive(Subcommand)]
pub enum Commands {
    Run {
        game_path: PathBuf,
        // do not parse any further, treat all further arguments here as just vec of strings
        // e.g unknown args get added here
        #[arg(trailing_var_arg = true)]
        #[arg(allow_hyphen_values = true)]
        additional_args: Vec<String>,
    },

    Path {
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(DEFAULT_LOG_LEVEL)
        .init();

    let appname = env!("CARGO_PKG_NAME");
    let appversion = env!("CARGO_PKG_VERSION");
    // print the args
    tracing::info!("{appname} {appversion} starting up, logging at {DEFAULT_LOG_LEVEL} level.");
    tracing::info!(
        "Launched with args: {:?}",
        std::env::args().collect::<Vec<String>>()
    );
    let args = Boson::parse();
    let exec_path = std::env::current_exe()?;

    // get the folder of the executable
    let exec_dir = exec_path.parent().unwrap();

    tracing::info!("Executable path: {:?}", exec_path);
    tracing::info!("Executable directory: {:?}", exec_dir);

    // Create path for hook
    let hook_path = exec_dir.join("register-hook.js");

    match args.cmd {
        Commands::Run {
            game_path,
            additional_args,
        } => {
            let electron = path_search::env_electron_path();

            let mut args = vec!["--no-sandbox"];

            let gpath = path_search::get_game_path(&game_path);
            // Actually get the game executable path here
            let app_path_str = get_asar_path(&game_path).ok_or_eyre(
                "Could not find ASAR file in game directory. Make sure you're running this from the game directory.",
            )?;

            // todo: path to boson hook
            let load_hook_arg = vec!["--require", hook_path.to_str().unwrap()];

            // Add the args before the app path
            args.extend(load_hook_arg.iter());
            args.extend(additional_args.iter().map(|s| s.as_str()));
            args.push(app_path_str.to_str().unwrap());

            tracing::info!(?gpath);

            tracing::debug!(?args);

            // Remove steam overlay from LD_PRELOAD

            let ld_preload = std::env::var("LD_PRELOAD").unwrap_or_default();
            // shadow the variable
            //
            // filter out the gameoverlayrenderer
            let ld_preload = std::env::split_paths(&ld_preload)
                .filter(|x| {
                    x.to_str()
                        .map(|x| !x.contains("gameoverlayrenderer"))
                        .unwrap_or(true)
                })
                .collect::<Vec<_>>();

            let ld_preload = std::env::join_paths(ld_preload).unwrap();

            let ld_library_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
            let mut ld_library_path = std::env::split_paths(&ld_library_path).collect::<Vec<_>>();

            // add the exec_dir/lib to the LD_LIBRARY_PATH

            ld_library_path.push(exec_dir.join("lib"));

            let ld_library_path = std::env::join_paths(ld_library_path).unwrap();

            let mut cmd = Command::new(electron);
            cmd.current_dir(&gpath)
                .env("LD_LIBRARY_PATH", ld_library_path)
                // Do not preload any libraries, hack to fix Steam overlay
                .env("LD_PRELOAD", ld_preload)
                .args(args);

            let c = cmd.spawn()?.wait();

            if let Err(e) = c {
                return Err(color_eyre::eyre::eyre!(e));
            };
            Ok(())
        }
        Commands::Path { path } => {
            println!("{}", path_search::get_game_path(&path).display());
            Ok(())
        }
    }
}
