use crate::input::StickOptions;
use anyhow::Result;
use serde::Deserialize;

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
    let toml_str = std::fs::read_to_string("dsmod.toml")?;
    Ok(toml::from_str(&toml_str)?)
}
