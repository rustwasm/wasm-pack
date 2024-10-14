use crate::install::Tool;
use anyhow::Result;
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
        let res = ureq::builder()
            .try_proxy_from_env(true)
            .build()
            .get(&krate_address)
            .set(
                "user-agent",
                &format!("wasm-pack/{}", VERSION.unwrap_or("unknown")),
            )
            .call()?;

        let kr: KrateResponse = res.into_json()?;
        Ok(kr.krate)
    }
}
