use anyhow::Result;
use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Serialize};

use crate::channels::pushover::PushoverSettings;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub account_number: String,
    pub basic_auth: String,

    // pushover config
    pub pushover: PushoverSettings,
}

impl Settings {
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name(config_path))
            .add_source(Environment::with_prefix("kplc"))
            .build()?;

        s.try_deserialize()
    }
}
