use crate::{kplc::KPLCBillResp, settings::Settings};
use anyhow::Result;
use async_trait::async_trait;

pub mod pushover;

#[async_trait]
pub trait Channel {
    fn new(settings: &Settings) -> Self
    where
        Self: Sized;

    fn name(&self) -> &str;

    fn is_enabled(&self) -> bool {
        false
    }

    async fn send_alert(&self, message: &KPLCBillResp) -> Result<()>;
}

pub fn get_channels(settings: &Settings) -> Vec<Box<dyn Channel>> {
    vec![Box::new(pushover::Pushover::new(settings))]
}
