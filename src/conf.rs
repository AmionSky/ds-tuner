use crate::Event;
use crate::input::{StickOptions, TriggerOptions};
use anyhow::Result;
use serde::Deserialize;
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(not(feature = "systemd"))]
const PATH: &str = "./ds-tuner.toml";
#[cfg(feature = "systemd")]
const PATH: &str = "/etc/ds-tuner.toml";

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Sticks {
    pub left: StickOptions,
    pub right: StickOptions,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Triggers {
    pub left: TriggerOptions,
    pub right: TriggerOptions,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub stick: Sticks,
    pub trigger: Triggers,
}

pub struct ConfigWatcher {
    config: Arc<Mutex<Config>>,
}

impl ConfigWatcher {
    pub fn init(tx: SyncSender<Event>) -> Self {
        let config = Arc::new(Mutex::new(Config::default()));
        spawn_watcher(tx, config.clone());
        if !try_load(&config) {
            log::info!("Using default config")
        }
        Self { config }
    }

    pub fn config(&self) -> MutexGuard<'_, Config> {
        self.config.lock().expect("Config mutex is invalid!")
    }
}

/// Return true if the config changed
fn try_load(mutex: &Mutex<Config>) -> bool {
    log::debug!("Reloading config from {PATH}");
    match load() {
        Ok(config) => {
            let mut lock = mutex.lock().expect("Config mutex is invalid!");
            if *lock != config {
                log::debug!("Updated config: {:#?}", config);
                *lock = config;
                return true;
            }
        }
        Err(error) => log::error!("Failed to load configuration file: {error}"),
    }
    false
}

fn load() -> Result<Config> {
    let toml_str = std::fs::read_to_string(PATH)?;
    Ok(toml::from_str(&toml_str)?)
}

fn spawn_watcher(tx: SyncSender<Event>, config: Arc<Mutex<Config>>) {
    std::thread::Builder::new()
        .name("config_watcher".into())
        .spawn(move || {
            if let Err(error) = watcher(tx, &config) {
                log::error!("Config watcher stopped: {error}");
            }
        })
        .expect("Failed to spawn config watcher thread!");
}

fn watcher(tx: SyncSender<Event>, config: &Mutex<Config>) -> Result<()> {
    use inotify::{Inotify, WatchMask};

    let mut inotify = Inotify::init()?;
    inotify.watches().add(PATH, WatchMask::CLOSE_WRITE)?;

    let mut buffer = [0u8; 4096];
    loop {
        // Just block until any event is received then reload the config
        inotify.read_events_blocking(&mut buffer)?;
        if try_load(config) {
            tx.send(Event::ConfigChanged)?;
        }
    }
}
