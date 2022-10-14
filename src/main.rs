extern crate anyhow;
extern crate async_trait;
extern crate chrono;
extern crate clap;
extern crate config;
extern crate reqwest;
extern crate rust_decimal;
extern crate serde_json;
extern crate tokio;

use std::process::exit;

use clap::{arg, command};
use env_logger::Env;
use log::{debug, error, info};

use crate::kplc::KPLCBillQuery;
use crate::settings::Settings;

mod channels;
mod client;
mod kplc;
mod settings;

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(arg!(-c --config <FILE> "Config file to use").required(true))
        .arg(
            arg!(--"log-level" <LEVEL> "Level of logging")
                .required(false)
                .value_parser(["error", "warn", "info", "debug", "trace"])
                .default_value("info"),
        )
        .arg(
            arg!(--"log-style" <STYLE> "Logging style")
                .required(false)
                .value_parser(["auto", "always", "none"])
                .default_value("auto"),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let log_level = matches.get_one::<String>("log-level").unwrap();
    let log_style = matches.get_one::<String>("log-style").unwrap();

    // setup logging
    let env = Env::default()
        .filter_or("KPLC_LOG_LEVEL", log_level)
        .write_style_or("KPLC_LOG_STYLE", log_style);

    env_logger::init_from_env(env);

    debug!("fetching settings from file {}", config_path);
    let settings = match Settings::new(config_path) {
        Ok(settings) => settings,
        Err(err) => {
            error!("error loading settings: {}", err);
            exit(1);
        }
    };

    let kplc_settings = settings.kplc.clone();
    let kplc_query = KPLCBillQuery::new(kplc_settings);
    info!("fetching bill from KPLC");
    let bill = match kplc_query.get_bill().await {
        Ok(bill) => {
            info!("done fetching bill from KPLC");
            bill
        }
        Err(err) => {
            error!("error fetching bill from KPLC: {}", err);
            exit(1);
        }
    };

    if bill.data.balance.is_sign_negative() {
        info!("balance present... sending alert to enabled channels");
        for channel in channels::get_channels(&settings) {
            let channel_name = channel.name();

            if channel.is_enabled() {
                info!("sending alert to {}", channel_name);
                match channel.send_alert(&bill).await {
                    Ok(_) => info!("sent alert to {}", channel_name),
                    Err(err) => {
                        error!("error sending alert to {}: {}", channel_name, err);
                        exit(1);
                    }
                };
            }
        }
    } else {
        info!("no balance present");
    }

    info!("done!");
}
