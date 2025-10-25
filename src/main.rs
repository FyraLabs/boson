use clap::{Parser, Subcommand};
use stable_eyre::Result;
use std::path::PathBuf;

use crate::config::BosonConfig;
pub mod config;
mod path_search;
mod runtime;
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
    #[clap(subcommand)]
    cmd: Commands,

    #[clap(flatten)]
    pub steam_opts: config::SteamCompatConfig,
}
#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "waitforexitandrun")]
    /// Launch the game with Boson, injecting hooks
    Run {
        game_path: PathBuf,
        // do not parse any further, treat all further arguments here as just vec of strings
        // e.g unknown args get added here
        #[clap(trailing_var_arg = true)]
        #[clap(allow_hyphen_values = true)]
        additional_args: Vec<String>,
    },

    /// Get the game path for a given executable
    Path { path: PathBuf },
}

fn main() -> Result<()> {
    stable_eyre::install()?;

    tracing_subscriber::fmt()
        .with_env_filter(DEFAULT_LOG_LEVEL)
        .without_time()
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
    tracing::trace_span!("env").in_scope(|| {
        tracing::trace!(
            "{:#?}",
            std::env::vars().collect::<std::collections::HashMap<_, _>>()
        );

        tracing::trace!("{:#?}", args.steam_opts)
    });
    let bosoncfg = BosonConfig::load()?;
    let app_id = args.steam_opts.get_app_id().unwrap_or_default();
    match args.cmd {
        Commands::Run {
            game_path,
            additional_args,
        } => {
            // todo: Move this to another function
            tracing::info!("Running game at path: {:?}", game_path);

            let gamecfg = bosoncfg.get_game_config(app_id);

            tracing::info!(
                "Using app ID: {}, compat type: {:?}, disable_steam_overlay: {}",
                app_id,
                gamecfg.compat_type,
                gamecfg.disable_steam_overlay
            );

            tracing::debug!("Determining runtime for game");
            let runtime = runtime::Runtime::new(args.steam_opts, gamecfg, game_path);

            runtime.launch_game(additional_args)?;
            Ok(())
        }
        Commands::Path { path } => {
            let game_path = path_search::get_game_path(&path);
            println!("{}", game_path.display());
            Ok(())
        }
    }
}
