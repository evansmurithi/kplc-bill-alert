use crate::{client, kplc::KPLCBillResp, settings::Settings};
use anyhow::{anyhow, Ok, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::Channel;

#[derive(Deserialize, Debug, Clone)]
pub struct PushoverSettings {
    pub enabled: bool,
    pub api_url: String,
    pub token: String,
    pub user_key: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct PushoverResponse {
    status: usize,
    request: String,
    errors: Option<Vec<String>>,
}

pub struct Pushover {
    settings: PushoverSettings,
    http_client: Client,
}

#[async_trait]
impl Channel for Pushover {
    fn new(settings: &Settings) -> Pushover {
        let http_client = client::get_http_client().unwrap();

        Pushover {
            settings: settings.pushover.clone(),
            http_client,
        }
    }

    fn name(&self) -> &str {
        "Pushover"
    }

    fn is_enabled(&self) -> bool {
        self.settings.enabled
    }

    async fn send_alert(&self, bill: &KPLCBillResp) -> Result<(), anyhow::Error> {
        let title = self.get_title(bill);
        let message = self.get_message(bill);

        let params = &[
            ("token", self.settings.token.as_str()),
            ("user", self.settings.user_key.as_str()),
            ("title", title.as_str()),
            ("message", message.as_str()),
        ];

        let resp = self
            .http_client
            .post(self.settings.api_url.as_str())
            .form(&params)
            .send()
            .await?
            .json::<PushoverResponse>()
            .await?;

        // status code of `1` means the request was successfull
        if resp.status == 1 {
            Ok(())
        } else {
            Err(anyhow!(
                "failed sending alert to Pushover: {:?}",
                resp.errors.unwrap()
            ))
        }
    }
}

impl Pushover {
    fn get_title(&self, bill: &KPLCBillResp) -> String {
        let account_ref = bill.data.account_reference.as_str();
        let billing_period = bill.data.col_bills[0].billing_period.as_str();

        format!("KPLC Bill (#{account_ref}): {billing_period}")
    }

    fn get_message(&self, bill: &KPLCBillResp) -> String {
        let balance = bill.data.balance;
        let due_date = bill.data.col_bills[0].due_date.format("%d %B, %Y");

        format!("Balance of KES {balance} is due on {due_date}!")
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs::File, path::Path};

    use crate::{channels::Channel, kplc::KPLCBillResp};
    use mockito::mock;
    use pretty_assertions::assert_eq;
    use reqwest::Client;

    use super::{Pushover, PushoverSettings};

    fn make_pushover() -> Pushover {
        let settings = PushoverSettings {
            enabled: true,
            api_url: mockito::server_url(),
            token: "asdasd".to_string(),
            user_key: "a1213qd".to_string(),
        };
        let http_client = Client::new();

        Pushover {
            settings,
            http_client,
        }
    }

    fn get_kplc_bill_resp(filename: &str) -> KPLCBillResp {
        let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let filepath = format!("{base_dir}/resources/test/{filename}");
        let path = Path::new(filepath.as_str());
        let file = File::open(path).unwrap();

        serde_json::from_reader(file).unwrap()
    }

    #[tokio::test]
    async fn test_send_alert_successfully() {
        let p = make_pushover();
        let bill = get_kplc_bill_resp("kplc_bill_balance.json");
        let body = form_urlencoded::Serializer::new(String::new())
            .append_pair("token", "asdasd")
            .append_pair("user", "a1213qd")
            .append_pair("title", "KPLC Bill (#1234567): 10 - October 2022")
            .append_pair(
                "message",
                "Balance of KES -3592.34 is due on 25 October, 2022!",
            )
            .finish();

        let _m = mock("POST", "/")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body(body.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"status\":1,\"request\":\"647d2300-702c-4b38-8b2f-d56326ae460b\"}")
            .create();

        let result = p.send_alert(&bill).await;
        assert!(result.is_ok());
        _m.assert();
    }

    #[tokio::test]
    async fn test_send_alert_error() {
        let p = make_pushover();
        let bill = get_kplc_bill_resp("kplc_bill_balance.json");

        let _m = mock("POST", "/")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body("{\"user\":\"invalid\",\"errors\":[\"user identifier is invalid\"],\"status\":0,\"request\":\"5042853c-402d-4a18-abcb-168734a801de\"}")
            .create();

        let result = p.send_alert(&bill).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string().as_str(),
            "failed sending alert to Pushover: [\"user identifier is invalid\"]"
        );
    }
}
