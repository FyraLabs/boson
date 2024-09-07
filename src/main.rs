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

    match args.cmd {
        Commands::Run {
            game_path,
            additional_args,
        } => {
            let electron = path_search::env_electron_path();

            let mut args = vec![];

            let gpath = path_search::get_game_path(&game_path);
            let app_path_str = get_asar_path(&gpath).ok_or_eyre(
                "Could not find ASAR file in game directory. Make sure you're running this from the game directory.",
            )?;
            args.push(app_path_str.to_str().unwrap());

            tracing::info!(?gpath);

            args.extend(additional_args.iter().map(|s| s.as_str()));

            tracing::debug!(?args);

            let mut cmd = Command::new(electron);
            cmd.current_dir(&gpath)
                // Do not preload any libraries, hack to fix Steam overlay
                .env_remove("LD_PRELOAD")
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
