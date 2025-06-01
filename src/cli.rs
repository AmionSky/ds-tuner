use crate::{NAME, VERSION};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Parser)]
#[command(
    name = NAME,
    version = VERSION,
    about = format!("{NAME} v{VERSION} - {PKG_DESCRIPTION}"),
    long_about = None,
)]
pub struct Cli {
    /// Turns verbose logging on
    #[arg(short, long)]
    pub verbose: bool,

    /// Log to journal
    #[cfg(feature = "systemd")]
    #[arg(short, long)]
    pub journal: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the DS Tuner service
    Start {
        /// Path to the config file
        #[arg(short, long, value_name = "FILE", default_value = "./ds-tuner.toml")]
        config: PathBuf,
    },
}
