use anyhow::Result;
use config::{Config, ConfigError};
use serde::Deserialize;

use crate::{channels::pushover::PushoverSettings, kplc::KPLCSettings};

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub kplc: KPLCSettings,

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

    use pretty_assertions::assert_eq;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_settings_from_file_with_all_fields() {
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("config.toml");

        let mut config_file = File::create(&file_path).unwrap();
        let conf = r###"
[kplc]
basic_auth = "Basic asdasldkasdlasd"
token_url = "https://selfservice.kplc.co.ke/api/token"
bill_url = "https://selfservice.kplc.co.ke/api/publicData/2.0.1/"
token_grant_type = "client_credentials"
token_scope = "token_public"

[pushover]
enabled = true
token = "asdasdasdqe123"
user_key = "asd13414nkj1k2j412"
api_url = "https://api.pushover.net/1/messages.json"
"###;
        config_file.write_all(conf.as_bytes()).unwrap();

        let result = Settings::new(file_path.as_path().to_str().unwrap());
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.kplc.basic_auth, "Basic asdasldkasdlasd");
        assert_eq!(settings.kplc.token_url, "https://selfservice.kplc.co.ke/api/token");
        assert_eq!(settings.kplc.bill_url, "https://selfservice.kplc.co.ke/api/publicData/2.0.1/");
        assert_eq!(settings.kplc.token_grant_type, "client_credentials");
        assert_eq!(settings.kplc.token_scope, "token_public");

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
[kplc]
basic_auth = "Basic asdasldkasdlasd"
token_url = "https://selfservice.kplc.co.ke/api/token"
bill_url = "https://selfservice.kplc.co.ke/api/publicData/2.0.1/"
token_grant_type = "client_credentials"
token_scope = "token_public"

[pushover]
enabled = true
user_key = "asd13414nkj1k2j412"
api_url = "https://api.pushover.net/1/messages.json"
"###;
        config_file.write_all(conf.as_bytes()).unwrap();

        let result = Settings::new(file_path.as_path().to_str().unwrap());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string().as_str(),
            "missing field `token`"
        );

        tmp_dir.close().unwrap();
    }
}
