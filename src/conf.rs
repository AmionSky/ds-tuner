use crate::input::StickOptions;
use anyhow::Result;
use serde::Deserialize;

#[cfg(debug_assertions)]
const PATH: &str = "./dsmod.toml";
#[cfg(not(debug_assertions))]
const PATH: &str = "/etc/dsmod.toml";

#[derive(Debug, Deserialize)]
pub struct Sticks {
    pub left: StickOptions,
    pub right: StickOptions,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub sticks: Sticks,
}

pub fn load() -> Result<Config> {
    let toml_str = std::fs::read_to_string(PATH)?;
    Ok(toml::from_str(&toml_str)?)
}
