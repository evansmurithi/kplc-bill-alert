use anyhow::Result;
use reqwest::{header, Client};

// user agent to use to make requests
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub fn get_http_client() -> Result<Client> {
    let mut default_headers = header::HeaderMap::new();
    default_headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
    default_headers.insert(
        header::CONNECTION,
        header::HeaderValue::from_static("keep-alive"),
    );

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .default_headers(default_headers)
        .https_only(true)
        .build()?;
    Ok(client)
}
