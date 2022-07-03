use anyhow::Result;
use chrono::prelude::{DateTime, Utc};
use reqwest::{header, Client};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

use crate::client;

const KPLC_TOKEN_URL: &str = "https://selfservice.kplc.co.ke/api/token";
const KPLC_BILL_URL: &str = "https://selfservice.kplc.co.ke/api/publicData/2.0.1/";

const KPLC_TOKEN_GRANT_TYPE: &str = "client_credentials";
const KPLC_TOKEN_SCOPE: &str = "token_public accounts_public attributes_public customers_public documents_public listData_public rccs_public sectorSupplies_public selfReads_public serviceRequests_public services_public streets_public supplies_public users_public workRequests_public publicData_public juaforsure_public calculator_public sscalculator_public token_private accounts_private accounts_public attributes_public attributes_private customers_public customers_private documents_private documents_public listData_public rccs_private rccs_public sectorSupplies_private sectorSupplies_public selfReads_private selfReads_public serviceRequests_private serviceRequests_public services_private services_public streets_public supplies_private supplies_public users_private users_public workRequests_private workRequests_public notification_private outage_private juaforsure_private juaforsure_public prepayment_private pdfbill_private publicData_public selfReadsPeriod_private corporateAccount_private calculator_public sscalculator_public register_public ssaccounts_public addaccount_public summaryLetter_public whtcertificate_public selfService_public";

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
    http_client: Client,
}

impl KPLCBillQuery {
    pub fn new() -> KPLCBillQuery {
        let http_client = client::get_http_client().unwrap();

        KPLCBillQuery { http_client }
    }

    async fn get_authorization_token(&self, basic_auth: &str) -> Result<String> {
        let mut auth_value = header::HeaderValue::from_str(basic_auth)?;
        auth_value.set_sensitive(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, auth_value);

        let mut query_params = HashMap::new();
        query_params.insert("grant_type", KPLC_TOKEN_GRANT_TYPE);
        query_params.insert("scope", KPLC_TOKEN_SCOPE);

        let kplc_token = self
            .http_client
            .post(KPLC_TOKEN_URL)
            .headers(headers)
            .query(&query_params)
            .send()
            .await?
            .json::<KPLCToken>()
            .await?;

        Ok(kplc_token.access_token)
    }

    pub async fn get_bill(&self, basic_auth: &str, account_number: &str) -> Result<KPLCBillResp> {
        let auth_token = self.get_authorization_token(basic_auth).await?;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let mut query_params = HashMap::new();
        query_params.insert("accountReference", account_number);

        let kplc_bill = self
            .http_client
            .get(KPLC_BILL_URL)
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
