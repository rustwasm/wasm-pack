use crate::install::Tool;
use anyhow::Result;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct Krate {
    pub max_version: String,
}

#[derive(Debug, Deserialize)]
pub struct KrateResponse {
    #[serde(rename = "crate")]
    pub krate: Krate,
}

impl Krate {
    pub fn new(name: &Tool) -> Result<Krate> {
        let krate_address = format!("https://crates.io/api/v1/crates/{}", name);
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(&krate_address)
            .header(
                USER_AGENT,
                format!("wasm-pack/{}", VERSION.unwrap_or("unknown")),
            )
            .send()?;

        let kr: KrateResponse = serde_json::from_str(&res.text()?)?;
        Ok(kr.krate)
    }
}
