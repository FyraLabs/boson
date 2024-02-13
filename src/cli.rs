use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::{os::unix::process::ExitStatusExt, path::PathBuf, process::ExitStatus};

use crate::{
    runtime::{self, Runtime},
    util,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(allow_hyphen_values = true)]
pub struct Boson {
    #[command(subcommand)]
    cmd: Commands,
}
impl Boson {
    pub fn run() -> Result<ExitStatus> {
        let args = Self::parse();
        match args.cmd {
            Commands::Run {
                game_path,
                additional_args,
            } => {
                let electron = util::electron_path();

                // right now let's do this funny hack, use `config::join_game_path` to get the asar path
                // should improve ootb compat for most games

                // let app_path_str = util::app_path();
                // let gamepath = util::get_game_path(&game_path);
                let gamepath = crate::config::join_game_path(util::get_game_path(&game_path), None);

                tracing::info!(?gamepath);
                tracing::debug!(?additional_args);

                let runtime = runtime::CustomRuntime::new(&electron);

                runtime.run(&gamepath, &util::get_game_path(&game_path), additional_args)
            }
            Commands::Path { path } => {
                let gpath = util::get_game_path(&path);
                println!("{}", gpath.display());
                Ok(ExitStatus::from_raw(0))
            }
        }
    }
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
