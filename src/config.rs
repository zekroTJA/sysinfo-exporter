use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub loglevel: Option<String>,
    pub interval_seconds: Option<u64>,
    pub endpoint: String,
    pub auth: Option<Auth>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Auth {
    Basic { username: String, password: String },
    Bearer { token: String },
}

impl Config {
    pub fn parse<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        dotenv::dotenv().ok();

        let mut figment = Figment::new();

        if let Some(path) = path {
            figment = figment.merge(Toml::file(path));
        }

        figment = figment.merge(Env::prefixed("SYSEXP_").split('_'));

        Ok(figment.extract()?)
    }
}
