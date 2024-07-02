use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
// use tracing_subscriber::;
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: &str = "info";
#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: &str = "debug";
const APPDIR: &str = "resources/app";

fn electron_path() -> String {
    // get envar

    if let Ok(cmd) = std::env::var("ELECTRON_PATH") {
        cmd
    } else {
        "electron".to_string()
    }
}

fn get_game_path(path: &Path) -> PathBuf {
    // remove file name from path
    if path.is_file() {
        path.parent().unwrap().canonicalize().unwrap().to_path_buf()
    } else {
        path.canonicalize().unwrap().to_path_buf()
    }
}

/// Specify a relative path to the game executable to load the Electron files from
/// defaults to `resources/app`, as Cookie Clicker puts its files there
fn app_path() -> String {
    if let Ok(path) = std::env::var("BOSON_LOAD_PATH") {
        path
    } else {
        APPDIR.to_string()
    }
}
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
            let electron = electron_path();

            let mut args = vec![];

            let app_path_str = app_path();
            args.push(app_path_str.as_str());

            let gpath = get_game_path(&game_path);

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
            println!("{}", get_game_path(&path).display());
            Ok(())
        }
    }

    // just

    // let cmd = Command::new("electron")
    //     .current_dir(TESTDIR)
    //     .arg("./resources/app")
    //     .arg("--in-process-gpu")
    //     .spawn()?
    //     .wait();

    // println!("{:?}", cmd);
}
