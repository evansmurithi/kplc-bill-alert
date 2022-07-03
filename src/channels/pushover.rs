use std::collections::HashMap;

use crate::{client, kplc::KPLCBillResp, settings::Settings};
use anyhow::{anyhow, Ok, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::Channel;

const PUSHOVER_MESSAGES_URL: &str = "https://api.pushover.net/1/messages.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushoverSettings {
    pub enabled: bool,
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

        let mut params = HashMap::new();
        params.insert("token", self.settings.token.as_str());
        params.insert("user", self.settings.user_key.as_str());
        params.insert("title", title.as_str());
        params.insert("message", message.as_str());

        let resp = self
            .http_client
            .post(PUSHOVER_MESSAGES_URL)
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
