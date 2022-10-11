use anyhow::Result;
use config::{Config, ConfigError};
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
            .build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::Settings;

    use std::fs::File;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn test_settings_from_file_with_all_fields() {
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("config.toml");

        let mut config_file = File::create(&file_path).unwrap();
        let conf = r###"
account_number = "123456"
basic_auth = "Basic asdasldkasdlasd"

[pushover]
enabled = true
token = "asdasdasdqe123"
user_key = "asd13414nkj1k2j412"
"###;
        config_file.write_all(conf.as_bytes()).unwrap();

        let result = Settings::new(file_path.as_path().to_str().unwrap());
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.account_number, "123456");
        assert_eq!(settings.basic_auth, "Basic asdasldkasdlasd");
        assert_eq!(settings.pushover.enabled, true);
        assert_eq!(settings.pushover.token, "asdasdasdqe123");
        assert_eq!(settings.pushover.user_key, "asd13414nkj1k2j412");

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_settings_from_file_missing_field() {
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("config.toml");

        let mut config_file = File::create(&file_path).unwrap();
        let conf = r###"
account_number = "123456"
basic_auth = "Basic asdasldkasdlasd"

[pushover]
enabled = true
user_key = "asd13414nkj1k2j412"
"###;
        config_file.write_all(conf.as_bytes()).unwrap();

        let result = Settings::new(file_path.as_path().to_str().unwrap());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string().as_str(), "missing field `token`");

        tmp_dir.close().unwrap();
    }
}
