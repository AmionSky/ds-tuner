mod bpf;
mod cli;
mod conf;
mod device;
mod input;
mod instance;
mod service;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use instance::SingleInstance;
use log::LevelFilter;
use std::path::PathBuf;

const NAME: &str = "DS Tuner";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let cli = Cli::parse();
    init_logger(&cli).expect("Failed to initialize logger!");

    match cli.command {
        Commands::Start { config } => start(config),
    }
}

fn start(config_path: PathBuf) {
    // Check if the service is already running
    let instance = SingleInstance::new();
    if !instance.single() {
        log::info!("{NAME} is already running!");
        return;
    }

    log::info!("{NAME} v{VERSION} started!");

    // Start the service
    if let Err(error) = service::start(config_path) {
        log::error!("Fatal error: {error}");
        panic!("{error}");
    }
}

fn init_logger(options: &Cli) -> Result<()> {
    let level = match options.verbose {
        true => LevelFilter::Trace,
        false => {
            if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            }
        }
    };

    #[cfg(feature = "systemd")]
    if options.journal {
        return init_journal(level);
    }

    init_termlogger(level)
}

fn init_termlogger(level: LevelFilter) -> Result<()> {
    use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

    TermLogger::init(
        level,
        ConfigBuilder::new()
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    Ok(())
}

#[cfg(feature = "systemd")]
fn init_journal(level: LevelFilter) -> Result<()> {
    use systemd_journal_logger::JournalLog;

    JournalLog::new()?.install()?;
    log::set_max_level(level);

    Ok(())
}
