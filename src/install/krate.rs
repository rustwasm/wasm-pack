use install::Tool;
use serde::Deserialize;

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
    pub fn new(name: &Tool) -> Result<Krate, failure::Error> {
        let krate_address = format!("https://crates.io/api/v1/crates/{}", name);
        let client = reqwest::blocking::Client::new();
        let res = client.get(&krate_address).send()?;
        let text = res.text()?;
        let kr: KrateResponse = serde_json::from_str(&text)?;
        Ok(kr.krate)
    }
}
