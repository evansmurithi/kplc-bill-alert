use anyhow::{anyhow, Result};
use chrono::prelude::{DateTime, Utc};
use reqwest::{header, Client};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

use crate::client;

#[derive(Deserialize, Debug, Clone)]
pub struct KPLCSettings {
    pub basic_auth: String,
    pub token_url: String,
    pub bill_url: String,
    pub token_grant_type: String,
    pub token_scope: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct KPLCBill {
    pub data: KPLCBillData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct KPLCBillData {
    pub account_reference: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: Decimal,
    pub meter_list: Vec<KPLCBillMeterList>,
    pub full_name: String,
    pub col_bills: Vec<KPLCBillColBills>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct KPLCBillMeterList {
    pub serial_num: String,
    pub latest_usage_list: Vec<KPLCBillLatestUsage>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct KPLCBillLatestUsage {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub reading_date: DateTime<Utc>,
    pub reading_value: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct KPLCBillColBills {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub due_date: DateTime<Utc>,
    #[serde(with = "rust_decimal::serde::float")]
    pub bill_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub bill_pend_amount: Decimal,
    pub billing_period: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub to_date: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub from_date: DateTime<Utc>,
    pub bill_number: String,
}

#[derive(Deserialize, Debug)]
struct KPLCToken {
    access_token: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum KPLCTokenResponse {
    Error {
        error_description: String,
        error: String,
    },
    Success(KPLCToken),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct KPLCBillError {
    http_status: usize,
    code: String,
    msg_user: String,
    help_link: String,
    msg_developer: String,
    error_sequence: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged, rename_all = "camelCase")]
enum KPLCBillResponse {
    Success(KPLCBill),
    Error(KPLCBillError),
}

pub struct KPLCBillQuery {
    settings: KPLCSettings,
    http_client: Client,
}

impl KPLCBillQuery {
    pub fn new(settings: KPLCSettings) -> KPLCBillQuery {
        let http_client = client::get_http_client().unwrap();

        KPLCBillQuery {
            settings,
            http_client,
        }
    }

    async fn get_authorization_token(&self, basic_auth: &str) -> Result<String> {
        let mut auth_value = header::HeaderValue::from_str(basic_auth)?;
        auth_value.set_sensitive(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, auth_value);

        let query_params = &[
            ("grant_type", self.settings.token_grant_type.as_str()),
            ("scope", self.settings.token_scope.as_str()),
        ];

        let response = self
            .http_client
            .post(self.settings.token_url.as_str())
            .headers(headers)
            .query(&query_params)
            .send()
            .await?
            .json::<KPLCTokenResponse>()
            .await?;

        match response {
            KPLCTokenResponse::Success(kplc_token) => Ok(kplc_token.access_token),
            KPLCTokenResponse::Error {
                error_description,
                error,
            } => Err(anyhow!(
                "failed to get access token: code: {} message: {}",
                error,
                error_description
            )),
        }
    }

    pub async fn get_bill(&self, account_number: &str) -> Result<KPLCBill> {
        let auth_token = self
            .get_authorization_token(self.settings.basic_auth.as_str())
            .await?;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let mut query_params = HashMap::new();
        query_params.insert("accountReference", account_number);

        let response = self
            .http_client
            .get(self.settings.bill_url.as_str())
            .headers(headers)
            .bearer_auth(auth_token)
            .query(&query_params)
            .send()
            .await?
            .json::<KPLCBillResponse>()
            .await?;

        match response {
            KPLCBillResponse::Success(kplc_bill) => Ok(kplc_bill),
            KPLCBillResponse::Error(err) => Err(anyhow!("failed to get bill: {}", err.msg_user)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use mockito::mock;
    use pretty_assertions::assert_eq;
    use reqwest::Client;

    use super::{KPLCBillQuery, KPLCSettings};

    fn make_kplc() -> KPLCBillQuery {
        let settings = KPLCSettings {
            basic_auth: "Basic 123qwdqwqwe".to_string(),
            token_url: format!("{}/token", mockito::server_url()),
            bill_url: format!("{}/bill", mockito::server_url()),
            token_grant_type: "client_credentials".to_string(),
            token_scope: "public_read".to_string(),
        };
        let http_client = Client::new();

        KPLCBillQuery {
            settings,
            http_client,
        }
    }

    fn get_body(filename: &str) -> String {
        let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let filepath = format!("{base_dir}/resources/test/{filename}");

        fs::read_to_string(filepath).unwrap()
    }

    #[tokio::test]
    async fn test_get_bill_successfully() {
        let kplc = make_kplc();

        let token_body_response = get_body("kplc_token.json");

        let m1 = mock(
            "POST",
            "/token?grant_type=client_credentials&scope=public_read",
        )
        .match_header("authorization", "Basic 123qwdqwqwe")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(token_body_response.as_str())
        .create();

        let bill_body_response = get_body("kplc_bill_balance.json");

        let m2 = mock("GET", "/bill?accountReference=12345")
            .match_header("content-type", "application/json")
            .match_header("authorization", "Bearer 00cfdd3d35103c264f5cab9440aa6c2e")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(bill_body_response.as_str())
            .create();

        let result = kplc.get_bill("12345").await;

        m1.assert();
        m2.assert();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_bill_fail_getting_token() {
        let kplc = make_kplc();

        let m1 = mock(
            "POST",
            "/token?grant_type=client_credentials&scope=public_read",
        )
        .match_header("authorization", "Basic 123qwdqwqwe")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"error_description": "Client Authentication failed.", "error": "invalid_client"}"#,
        )
        .create();

        let bill_body_response = get_body("kplc_bill_balance.json");

        let m2 = mock("GET", "/bill?accountReference=12345")
            .match_header("content-type", "application/json")
            .match_header("authorization", "Bearer 00cfdd3d35103c264f5cab9440aa6c2e")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(bill_body_response.as_str())
            .create();

        let result = kplc.get_bill("12345").await;

        m1.assert();
        assert_eq!(m2.matched(), false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string().as_str(),
            "failed to get access token: code: invalid_client message: Client Authentication failed."
        );
    }

    #[tokio::test]
    async fn test_get_bill_fail_getting_bill() {
        let kplc = make_kplc();

        let token_body_response = get_body("kplc_token.json");

        let m1 = mock(
            "POST",
            "/token?grant_type=client_credentials&scope=public_read",
        )
        .match_header("authorization", "Basic 123qwdqwqwe")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(token_body_response)
        .create();

        let m2 = mock("GET", "/bill?accountReference=12345")
            .match_header("content-type", "application/json")
            .match_header("authorization", "Bearer 00cfdd3d35103c264f5cab9440aa6c2e")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"httpStatus":422,"code":"SS000106","msgUser":"The account number 12345 doesn´t exist.","helpLink":"SS000106","msgDeveloper":"The account number doesn't exist.","errorSequence":"570120b7:183d09c9ad7:23b9"}"#)
            .create();

        let result = kplc.get_bill("12345").await;

        m1.assert();
        m2.assert();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string().as_str(),
            "failed to get bill: The account number 12345 doesn´t exist."
        );
    }
}
