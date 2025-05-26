use crate::input::StickOptions;
use anyhow::Result;
use serde::Deserialize;
use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(debug_assertions)]
const PATH: &str = "./dsmod.toml";
#[cfg(not(debug_assertions))]
const PATH: &str = "/etc/dsmod.toml";

#[derive(Debug, Default, Deserialize)]
pub struct Sticks {
    pub left: StickOptions,
    pub right: StickOptions,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub sticks: Sticks,
}

pub struct ConfigWatcher {
    config: Arc<Mutex<Config>>,
}

impl ConfigWatcher {
    pub fn init() -> Self {
        let config = Arc::new(Mutex::new(Config::default()));
        spawn_watcher(config.clone());
        try_load(&config);
        Self { config }
    }

    pub fn config(&self) -> MutexGuard<'_, Config> {
        self.config.lock().expect("Config mutex is invalid!")
    }
}

fn try_load(mutex: &Mutex<Config>) {
    log::debug!("Reloading config from {PATH}");
    match load() {
        Ok(config) => *mutex.lock().expect("Config mutex is invalid!") = config,
        Err(error) => log::error!("Failed to load configuration file: {error}"),
    }
}

fn load() -> Result<Config> {
    let toml_str = std::fs::read_to_string(PATH)?;
    Ok(toml::from_str(&toml_str)?)
}

fn spawn_watcher(config: Arc<Mutex<Config>>) {
    std::thread::Builder::new()
        .name("config_watcher".into())
        .spawn(move || {
            if let Err(error) = watcher(&config) {
                log::error!("Config watcher stopped: {error}");
            }
        })
        .expect("Failed to spawn config watcher thread!");
}

fn watcher(config: &Mutex<Config>) -> Result<()> {
    use inotify::{Inotify, WatchMask};

    let mut inotify = Inotify::init()?;
    inotify.watches().add(PATH, WatchMask::CLOSE_WRITE)?;

    let mut buffer = [0u8; 4096];
    loop {
        // Just block until any event is received then reload the config
        inotify.read_events_blocking(&mut buffer)?;
        try_load(config);
    }
}
