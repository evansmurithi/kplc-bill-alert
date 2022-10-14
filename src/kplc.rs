use anyhow::Result;
use chrono::prelude::{DateTime, Utc};
use reqwest::{header, Client};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

use crate::client;

#[derive(Deserialize, Debug, Clone)]
pub struct KPLCSettings {
    pub account_number: String,
    pub basic_auth: String,
    pub token_url: String,
    pub bill_url: String,
    pub token_grant_type: String,
    pub token_scope: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct KPLCBillResp {
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

        let mut query_params = HashMap::new();
        query_params.insert("grant_type", self.settings.token_grant_type.as_str());
        query_params.insert("scope", self.settings.token_scope.as_str());

        let kplc_token = self
            .http_client
            .post(self.settings.token_url.as_str())
            .headers(headers)
            .query(&query_params)
            .send()
            .await?
            .json::<KPLCToken>()
            .await?;

        Ok(kplc_token.access_token)
    }

    pub async fn get_bill(&self) -> Result<KPLCBillResp> {
        let auth_token = self
            .get_authorization_token(self.settings.basic_auth.as_str())
            .await?;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let mut query_params = HashMap::new();
        query_params.insert("accountReference", self.settings.account_number.as_str());

        let kplc_bill = self
            .http_client
            .get(self.settings.bill_url.as_str())
            .headers(headers)
            .bearer_auth(auth_token)
            .query(&query_params)
            .send()
            .await?
            .json::<KPLCBillResp>()
            .await?;

        Ok(kplc_bill)
    }
}
