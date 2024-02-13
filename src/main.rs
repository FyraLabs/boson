use color_eyre::Result;

use crate::cli::Boson;
mod cli;
mod util;
mod runtime;
mod config;
// use tracing_subscriber::;
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: &str = "info";
#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: &str = "debug";

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
    tracing::debug!(
        "envars: {:#?}",
        std::env::vars().collect::<Vec<(String, String)>>()
    );

    // Run main entrypoint
    Boson::run()?;
    Ok(())
}
